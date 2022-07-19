use crate::Error;
use crate::libraries::Libraries;
use crate::shape::{Shape, Transform, Bounds};
use crate::sexp::{
    Sexp, Get, Test, get, get_pin, get_unit, get_pins, get_property, parser::SexpParser
};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::io::Write;
use std::fs::File;

use ndarray::{arr1, arr2, Array1};
use uuid::Uuid;
use crate::sexp::elements::{node, uuid, pos, stroke, effects, pts, property, junction, label, wire, symbol, symbol_instance, sheet};

#[pyclass]
pub struct Draw {
    version: String,
    generator: String,
    uuid: Sexp,
    paper: Sexp,
    title_block: Sexp,
    pub elements: Vec<Sexp>,
    libraries: Vec<Sexp>,
    sheet_instance: Vec<Sexp>,
    symbol_instance: Vec<Sexp>,
    libs: Libraries,
}

#[pymethods]
impl Draw {
    #[new]
    pub fn new(library_path: Vec<String>) -> Self {
        let p = vec![Sexp::Value(String::from("A4"))];
        let uuid = Uuid::new_v4();
        let u = vec![Sexp::Value(uuid.to_string())];
        Self {
            version: String::from("20211123"),
            generator: String::from("elektron"),
            uuid: Sexp::Node(String::from("uuid"), u),
            paper: Sexp::Node(String::from("paper"), p),
            title_block: Sexp::Node(String::from("title_block"), vec![Sexp::Value("".to_string())]),
            elements: Vec::<Sexp>::new(),
            libraries: Vec::<Sexp>::new(),
            sheet_instance: vec![sheet!("/", "1")],
            symbol_instance: Vec::<Sexp>::new(),
            libs: Libraries::new(library_path),
        }
    }

    fn pin_pos(&mut self, reference: &str, pin: usize) -> Vec<f64> {
        let symbol = self.get_symbol(reference, 1).unwrap();
        let lib_name: String = get!(&symbol, "lib_id", 0);
        let library = self.get_library(&lib_name).unwrap();
        let libs: Vec<&Sexp> =  library.get("symbol").unwrap();
        for _unit in libs {
            let number: usize = get_unit(_unit).unwrap(); //get!(_pin, "unit", 0);
            let sym_pins: Vec<&Sexp> = _unit.get("pin").unwrap();
            if let p = sym_pins {
                for _pin in p {
                    let number_node: Vec<&Sexp> = _pin.get("number").unwrap();
                    let _pin_number: usize = number_node.get(0).unwrap().get(0).unwrap();
                    if _pin_number == pin {
                        let pin_pos: Array1<f64> = get!(_pin, "at").unwrap();
                        let _lib_instance = self.get_symbol(reference, number).unwrap();
                        let at: Array1<f64> = Shape::transform(&_lib_instance, &pin_pos);
                        return vec![at[0], at[1]];
                    }
                }
            }
        }
        panic!("pin not found {}:{}", reference, pin); //TODO return error
    }
    pub fn wire(&mut self, pts: Vec<f64>, end: Vec<f64>) {
        self.elements
            .push(wire!(arr2(&[[pts[0], pts[1]], [end[0], end[1]]])));
    }
    fn junction(&mut self, pos: Vec<f64>) {
        self.elements.push(junction!(arr1(&[pos[0], pos[1]])));
    }
    fn label(&mut self, name: &str, pos: Vec<f64>, angle: f64) {

        println!("Label: {} {:?} {}", name, pos, angle);
        self.elements
            .push(label!(arr1(&[pos[0], pos[1]]), &angle, name.to_string()));
    }
    fn symbol(
        &mut self,
        reference: &str,
        value: &str,
        library: &str,
        unit: usize,
        pos: Vec<f64>,
        pin: usize,
        angle: f64,
        mirror: String,
        end_pos: Option<f64>,
        properties: HashMap<String, String>,
    ) {
        println!(
            "Symbol: {} {} {} {:?} {} {} {}",
            reference, value, unit, pos, pin, angle, mirror
        );
        let lib_symbol = self.get_library(library).unwrap();
        let uuid = Uuid::new_v4();

        let sym_pin = get_pin(&lib_symbol, pin).unwrap();
        let pin_pos: Array1<f64> = get!(sym_pin, "at").unwrap();
        // transform pin pos
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pin_pos.dot(&rot);
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
        verts = arr1(&[pos[0], pos[1]]) - &verts;
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        if let Some(end_pos) = end_pos {
            let pins = get_pins(&lib_symbol, Option::from(unit)).unwrap();
            if pins.len() == 2 {
                for p in pins {
                    let pin_number: usize = get!(p, "number", 0);
                    if pin_number != unit {
                        let other_pos: Array1<f64> = get!(p, "at").unwrap();
                        let mut verts2: Array1<f64> = other_pos.dot(&rot);
                        verts2 = verts2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
                        verts2 = arr1(&[pos[0], pos[1]]) - &verts2;
                        let len = end_pos - &verts2;
                        let sym_len = verts[0] - verts2[0];
                        let wire_len = ((end_pos - pos[0]) - sym_len) / 2.0;
                        verts = verts + arr1(&[wire_len, 0.0]);
                        println!("the pinlen is {}, len:{}", verts[0] - verts2[0], pos[0] - end_pos);
                        self.elements
                            .push(wire!(arr2(&[[pos[0], pos[1]], [pos[0] + wire_len, pos[1]]])));
                        self.elements
                            .push(wire!(arr2(&[[pos[0] + wire_len + sym_len, pos[1]], [pos[0] + 2.0 * wire_len + sym_len, pos[1]]])));
                    }
                }
            }
        }

        let mut symbol = symbol!(verts, angle, reference, library, unit, &uuid);

        //copy the properties from the library to the symbol
        let mut footprint: Option<String> = None;
        let props: Vec<&Sexp> = lib_symbol.get("property").unwrap();
        if let Sexp::Node(_, ref mut values) = symbol {
            for prop in props {
                let name: String = get!(prop, 0).unwrap();
                //skip properties with ki_
                if name.starts_with("ki_") {
                    break;
                //set the reference
                } else if name == "Reference" {
                    values.push(property!(verts, 0.0, "Reference", reference.to_string(), "0"));
                //set the value
                } else if name == "Value" {
                    values.push(property!(verts, 0.0, "Value", value.to_string(), "1"));
                } else if name == "footprint" {
                    footprint = Option::from(name);
                    values.push(property!(verts, 0.0, "Value", value.to_string(), "1"));
                } else {
                    values.push(prop.clone());
                }
            }
            // add the extra properties
            for (k, v) in properties.iter() {
                values.push(property!(verts, 0.0, k, v, "1"));
            }
        }

        self.place_property(&mut symbol).unwrap(); //TODO
        self.elements.push(symbol);
        self.symbol_instance
            .push(symbol_instance!(uuid, reference, value, 1, footprint));
    }

    pub fn write(&mut self, filename: Option<&str>, pretty: bool) {
        let mut out: Box<dyn Write> = if let Some(filename) = filename {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        match self._write(&mut out) {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }

    /* pub fn plot(&mut self, filename: Option<String>, border: bool, scale: f64) {
        let plotter = Box::new(CairoPlotter::new());
        let plot = Plot::new(plotter, filename, border, scale);
        match self._write(Box::new(plot)) {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }
    } */
    /* pub fn netlist(&mut self, filename: Option<String>) {
        let netlist = Box::new(Netlist::new());
        match self._write(netlist) {
            Ok(_) => {
                //netlist.dump();
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    } */
}

impl Draw {

    //check if this schema has a library symbol.
    fn has_library(&mut self, name: &str) -> bool {
        for l in &self.libraries {
            let lib_name: String = get!(l, 0).unwrap();
            if name == lib_name {
                return true;
            }
        }
        false
    }

    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<Sexp, Error> {
        if !self.has_library(name) { //load the library symbol
            let mut lib_symbol = self.libs.get(name).unwrap();
            //update the name with full path XX:yy
            if let Sexp::Node(_, ref mut values) = lib_symbol {
                let sym_name: &mut Sexp = values.get_mut(0).unwrap();
                if let Sexp::Text(ref mut value) = sym_name {
                    println!("set name {}", name);
                    *value = name.to_string();
                } else {
                    println!("symbol value is not a value node");
                }
            } else {
                println!("symbol is not a node");
            }
            self.libraries.push(lib_symbol.clone());
            return Ok(lib_symbol);

        } else { //get the existing library symbol
            for l in &self.libraries {
                let lib_name: String = get!(l, 0).unwrap();
                if name == lib_name {
                    return Ok(l.clone());
                }
            }
        }
        Err(Error::LibraryNotFound(name.to_string()))
    }

    /// get the symbol by reference and unit from this schema.
    fn get_symbol(&mut self, reference: &str, unit: usize) -> Result<Sexp, Error> {
        for l in &self.elements {
            if let Sexp::Node(name, values) = l {
            if name == "symbol" {
                if let _ref = get_property(l, "Reference").unwrap() {
                    let _unit = get_unit(&l).unwrap();
                    if reference == _ref && unit == _unit {
                        return Ok(l.clone());
                    }
                }
            }
            }
        }
        Err(Error::SymbolNotFound(reference.to_string()))
    }

    fn _write(&mut self, writer: &mut dyn Write) -> Result<(), Error> {

        let mut doc = SexpParser::new();
        doc.push(Sexp::Node(String::from("version"), vec![Sexp::Value(self.version.clone())]))?;
        doc.push(Sexp::Node(String::from("generator"), vec![Sexp::Value(self.generator.clone())]))?;
        doc.push(self.uuid.clone())?;
        doc.push(self.paper.clone())?;
        doc.push(self.title_block.clone())?;
        doc.push(Sexp::Node(String::from("lib_symbols"), self.libraries.clone()))?;
        for element in &self.elements {
            doc.push(element.clone())?;
        }
        doc.push(Sexp::Node(String::from("sheet_instances"), self.sheet_instance.clone()))?;
        doc.push(Sexp::Node(String::from("symbol_instances"), self.symbol_instance.clone()))?;
        doc.save(writer)
    }

    fn place_property(&mut self, symbol: &mut Sexp) -> Result<(), Error> {
        let pos: Array1<f64> = get!(symbol, "at")?;
        let props: Vec<&Sexp> = symbol.get("property").unwrap();
        let vis_field = props.iter().filter_map(|node| {
            let effects: Vec<&Sexp> = get!(node, "effects").unwrap();
            if !effects[0].has("hide") { Option::from(node) } else { None }
        }).count();
        let angle: f64 = get!(symbol, "at", 2);
        let lib_name: String = get!(symbol, "lib_id", 0);
        let lib = self.get_library(&lib_name).unwrap();
        let _size = Shape::transform(symbol, &symbol.bounds(&lib).unwrap());
        let positions = self.pin_position(&symbol, &lib);
        let mut offset = 0.0;
        let pins = get_pins(&lib, None).unwrap().len();
        if pins == 1 { //PINS!
            if positions[0] == 1 { //west
                todo!();
                /* vis_fields[0].pos = (_size[1][0]+1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.LEFT]
                vis_fields[0].angle = 360 - symbol.angle */

            } else if positions[1] == 1 { //south
                todo!();
                /* vis_fields[0].pos = (symbol.pos[0], _size[0][1]-1.28)
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.CENTER] */

            } else if positions[2] == 1 { //east
                todo!();
                /* vis_fields[0].pos = (_size[0][0]-1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.RIGHT]
                vis_fields[0].angle = 360 - symbol.angle */

            } else if positions[3] == 1 { //south
                let top_pos = if _size[[0, 1]] > _size[[1, 1]] {
                    _size[[0, 1]] - ((vis_field as f64-1.0) * 2.0) + 0.64
                } else {
                    _size[[1, 1]] - ((vis_field as f64-1.0) * 2.0) + 0.64
                };
                //symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                let props: Vec<&Sexp> = symbol.get("property")?;
                /* props.iter_mut().for_each(|node| {
                    let effects: Vec<&Sexp> = get!(node, "effects").unwrap();
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[1] = top_pos;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let Sexp::Node(name, effects) = n {
                                if name == "effects" {
                                    let index: usize = 0;
                                    for val in effects {
                                        if let Sexp::Node(name, nodes) = val {
                                            if name == "justify" {
                                                effects.remove(index);                                           }
                                        }
                                        index += 1;
                                    }
                                }
                            }
                        }
                        //TODO set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                }); */
                //vis_fields[0].text_effects.justify = [Justify.CENTER]
                return Ok(());
            } 
        } else {
            let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                _size[[0, 1]] - ((vis_field as f64-1.0) * 2.0) - 0.64
            } else {
                _size[[1, 1]] - ((vis_field as f64-1.0) * 2.0) - 0.64
            };
            if positions[3] == 0 { //north
                /* symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                    let effects: Sexp = get!(node, "effects");
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[1] = top_pos - offset;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let Sexp::Node(_, effects) = n {
                                if effects.name == "effects" {
                                    effects.delete("justify".to_string()).unwrap();
                                }
                            }
                        }
                        //TODO set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                }); */
                return Ok(());

            } else if positions[2] == 0 { //east
                /* let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0) - 
                    ((vis_field as f64-1.0) * 2.0) / 2.0;
                symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                    let effects: Sexp = get!(node, "effects");
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[0] = _size[[0, 1]];
                        field_pos[1] = top_pos - offset;
                        field_pos = field_pos + &pos;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let Sexp::Node(effects) = n {
                                if effects.name == "effects" {
                                    effects.delete("justify".to_string()).unwrap();
                                }
                            }
                        }
                        node.values.push(effects!( "1.27", "1.27", "left"));
                        set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                }); */
                return Ok(());

            } else if positions[0] == 0 { //west
                todo!();
            } else if positions[1] == 0 { //south
                todo!();
            } else {
                todo!();
            }
        }
        Err(Error::ParseError)
    }

    fn pin_position(&self, symbol: &Sexp, lib: &Sexp) -> Vec<usize> {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_angle: f64 = get!(symbol, "at", 2);
        let symbol_shift: usize = (symbol_angle / 90.0).round() as usize;
        let mirror: String = if symbol.contains("mirror") {
            get!(symbol, "mirror", 0)
        } else { String::new() };

        for pin in get_pins(lib, Option::from(get_unit(symbol).unwrap())).unwrap() {
            let pin_angle: f64 = get!(pin, "at", 2);
            let lib_pos: usize = (pin_angle / 90.0).round() as usize;
            let pos: usize =
                /* if mirror.contains("xy") {
                    todo!(); */
                if mirror.contains('x') {
                    if lib_pos == 1 {
                        3
                    } else if lib_pos == 3 {
                        1
                    } else {
                        lib_pos
                    }
                } else if mirror.contains('y') {
                    if lib_pos == 1 {
                        2 //TODO
                    } else if lib_pos == 3 {
                       0 
                    } else {
                        lib_pos
                    }
                } else {
                    lib_pos
                };
            position[pos] += 1;
        }
        position.rotate_right(symbol_shift);
        position
    }
} 

/* #[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp_write::SexpWriter;
    use ndarray::arr2;

    #[test]
    fn macro_pts() {
        let wire = pts!(arr1(&[179.07, 34.29]), arr1(&[179.07, 49.53]));
        if let SexpType::ChildSexpNode(wire) = wire {
            assert_eq!(
                "(pts (xy 179.07 34.29) (xy 179.07 49.53))",
                wire.write(false, 0).content
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn macro_wire() {
        let wire = wire!(arr2(&[[179.07, 34.29], [179.07, 49.53]]));
        assert_eq!("(wire (pts (xy 179.07 34.29) (xy 179.07 49.53)) (stroke (width 0) (type default) (color 0 0 0 0)) (uuid 008da5b9-6f95-4113-b7d0-d93ac62efd33))",
                   wire.write(false, 0).content);
    }

    #[test]
    fn draw_line() {
        /* let content = "(SEXP (NAME VALUE))";
        let mut schema = Schema::new(); */
        // schema.push(Wire::new().length(2.0*UNIT).left());
        // schema.push(&mut Junction::new());
        // schema.push(Wire::new().length(2.0*UNIT).up());
        // schema.push(Symbol::new("Device:R").left().anchor(2));

        //println!("{:#?}", schema.draw_elements);
        //assert_eq!(&node[0].name, &String::from("NAME"));
    }
} */
