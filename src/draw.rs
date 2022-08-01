use crate::Error;
use crate::circuit::{Simulation, Circuit};
use crate::libraries::Libraries;
use crate::shape::{Shape, Transform, Bounds};
use crate::sexp::{
    Sexp, get_pin, get_unit, get_pins, get_property, parser::SexpParser
};
use crate::sexp::get::{Get, get};
use crate::sexp::test::Test;
use crate::netlist::Netlist;
use crate::cairo_plotter::{CairoPlotter, ImageType};
use crate::plot::plot;
use crate::themes::Style;
use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::fs;
use std::fs::File;

use ndarray::{arr1, arr2, Array1};
use uuid::Uuid;
use crate::sexp::elements::{node, uuid, pos, stroke, effects, pts, property, junction, label, wire, symbol, symbol_instance, sheet};
use std::env::temp_dir;
use rand::Rng;

const LABEL_BORDER: f64 = 2.54;
const LABEL_MIN_BORDER: f64 = 5.06;

fn filter_properties(node: &&mut Sexp) -> bool {
    let mut show = true;
    if let Sexp::Node(name, values) = node {
        if name == "property" {
            for value in values {
                if let Sexp::Node(name, _) = value {
                    if name == "effects" {
                        if value.has("hide") {
                            show = false 
                        }
                    }
                }
            }
        }
    }
    show
}

fn sort_properties(a: &&mut Sexp, b: &&mut Sexp) -> std::cmp::Ordering {
    let mut ida = 0;
    if let Sexp::Node(_, values) = a {
        for v in values {
            if let Sexp::Node(name, values) = v {
                if name.as_str() == "id" {
                    let idnode = values.get(0).unwrap();
                    if let Sexp::Value(value) = idnode {
                        ida = value.parse::<usize>().unwrap();
                    }
                }
            }
        }
    }
    let mut idb = 0;
    if let Sexp::Node(_, values) = b {
        for v in values {
            if let Sexp::Node(name, values) = v {
                if name.as_str() == "id" {
                    let idnode = values.get(0).unwrap();
                    if let Sexp::Value(value) = idnode {
                        idb = value.parse::<usize>().unwrap();
                    }
                }
            }
        }
    }
    ida.cmp(&idb)
}

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

    fn wire(&mut self, pts: Vec<f64>, end: Vec<f64>) {
        let pts: Vec<f64> = pts.iter().map(|v| format!("{:.2}", v).parse::<f64>().unwrap()).collect();
        let end: Vec<f64> = end.iter().map(|v| format!("{:.2}", v).parse::<f64>().unwrap()).collect();
        self.elements
            .push(wire!(arr2(&[[pts[0], pts[1]], [end[0], end[1]]])));
    }
    fn junction(&mut self, pos: Vec<f64>) {
        let pos: Vec<f64> = pos.iter().map(|v| format!("{:.2}", v).parse::<f64>().unwrap()).collect();
        self.elements.push(junction!(arr1(&[pos[0], pos[1]])));
    }
    fn label(&mut self, name: &str, pos: Vec<f64>, angle: f64) {
        let pos: Vec<f64> = pos.iter().map(|v| format!("{:.2}", v).parse::<f64>().unwrap()).collect();
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
        let pos: Vec<f64> = pos.iter().map(|v| format!("{:.2}", v).parse::<f64>().unwrap()).collect();
        let lib_symbol = self.get_library(library).unwrap();
        let uuid = Uuid::new_v4();

        let sym_pin = get_pin(&lib_symbol, pin).unwrap();
        let pin_pos: Array1<f64> = get!(sym_pin, "at").unwrap();
        // transform pin pos
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pin_pos.dot(&rot);
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        verts = arr1(&[pos[0], pos[1]]) - &verts;

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
                        let sym_len = verts[0] - verts2[0];
                        let wire_len = ((end_pos - pos[0]) - sym_len) / 2.0;
                        verts = verts + arr1(&[wire_len, 0.0]);
                        let mut wire1 = arr2(&[[pos[0], pos[1]], [pos[0] + wire_len, pos[1]]]);
                        wire1 = wire1.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        let mut wire2 = arr2(&[[pos[0] + wire_len + sym_len, pos[1]], [pos[0] + 2.0 * wire_len + sym_len, pos[1]]]);
                        wire2 = wire2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        self.elements.push(wire!(wire1));
                        self.elements.push(wire!(wire2));
                    }
                }
            }
        }

        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
        let on_schema = if properties.contains_key("on_schema") {
            properties.get("on_schema").unwrap().clone()
        } else {
            String::from("yes")
        };
        let mut symbol = symbol!(verts, angle, mirror, reference, library, unit, &uuid, on_schema);

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
                    values.push(property!(verts, 0.0, "Reference", reference.to_string(), "0", false));
                //set the value
                } else if name == "Value" {
                    values.push(property!(verts, 0.0, "Value", value.to_string(), "1", false));
                } else if name == "footprint" {
                    footprint = Option::from(name);
                    values.push(property!(verts, 0.0, "Value", value.to_string(), "1", false));
                } else {
                    values.push(prop.clone());
                }
            }
            // add the extra properties
            for (k, v) in properties.iter() {
                if k != "on_schema" {
                    values.push(property!(verts, 0.0, k, v, "1", true));
                }
            }
        }

        self.place_property(&mut symbol).unwrap();
        self.elements.push(symbol);
        self.symbol_instance
            .push(symbol_instance!(uuid, reference, value, unit, footprint));
    }

    pub fn write(&mut self, filename: Option<&str>) -> Result<(), Error> {
        let mut out: Box<dyn Write> = if let Some(filename) = filename {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        match self._write() {
            Ok(doc) => {doc.save(&mut out)?;}
            Err(err) => {
                println!("{:?}", err);
            }
        }
        Ok(())
    }

    pub fn plot(&mut self, filename: Option<&str>, border: bool, scale: f64, image_type: Option<&str>) -> Result<Vec<u8>, Error> {
        let mut plotter = CairoPlotter::new();
        let image_type: ImageType = match image_type {
            Some("png") => ImageType::Png,
            Some("svg") => ImageType::Svg,
            Some("pdf") => ImageType::Pdf,
            None => ImageType::Svg,
            Some(&_) => { todo!(); },
        };
        match self._write() {
            Ok(doc) => {

                if let Some(filename) = filename {
                    let out: Box<dyn Write> = Box::new(File::create(filename).unwrap());
                    plot(&mut plotter, out, &doc, border, scale, Style::new(), image_type).unwrap();
                    Ok(Vec::new())

                } else {
                    let mut rng = rand::thread_rng();
                    let num: u32 = rng.gen();
                    let filename = String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".svg";
                    let out: Box<dyn Write> = Box::new(File::create(&filename).unwrap());
                    plot(&mut plotter, out, &doc, border, scale, Style::new(), image_type).unwrap();
                    let mut f = File::open(&filename).expect("no file found");
                    let metadata = fs::metadata(&filename).expect("unable to read metadata");
                    let mut buffer = vec![0; metadata.len() as usize];
                    f.read(&mut buffer).expect("buffer overflow");
                    Ok(buffer)
                }
            }
            Err(err) => {
                Err(err)
            }
        }
    }
    pub fn circuit(&mut self, pathlist: Vec<String>) -> Circuit {
        let mut circuit: Circuit = Circuit::new("circuit from draw".to_string(), pathlist);
        match self._write() {
            Ok(doc) => {
                let mut netlist = Box::new(Netlist::from(&doc));
                netlist.dump(&mut circuit).unwrap();
                return circuit;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
        panic!();
    }
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
            if let Sexp::Node(name, _) = l {
                if name == "symbol" {
                    let _ref = get_property(l, "Reference").unwrap();
                    let _unit = get_unit(&l).unwrap();
                    if reference == _ref && unit == _unit {
                        return Ok(l.clone());
                    }
                }
            }
        }
        Err(Error::SymbolNotFound(reference.to_string()))
    }

    fn _write(&mut self) -> Result<SexpParser, Error> {

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
        Ok(doc)
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

        //get and sort the shape size
        let _size = Shape::transform(symbol, &symbol.bounds(&lib).unwrap());
        let _size = if _size[[0, 0]] > _size[[1, 0]] {
            arr2(&[[_size[[1, 0]], _size[[0, 1]]],[_size[[0, 0]], _size[[1, 1]]]])
        } else { _size };
        let _size = if _size[[0, 1]] > _size[[1, 1]] {
            arr2(&[[_size[[0, 0]], _size[[1, 1]]],[_size[[1, 0]], _size[[0, 1]]]])
        } else { _size };
        let positions = self.pin_position(&symbol, &lib);
        let mut offset = 0.0;
        let pins = get_pins(&lib, None).unwrap().len();
        if pins == 1 { //PINS!
            if positions[0] == 1 { //west
                /* vis_fields[0].pos = (_size[1][0]+1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.LEFT]
                vis_fields[0].angle = 360 - symbol.angle */

                return Ok(());
            } else if positions[3] == 1 { //south
                if let Sexp::Node(_, ref mut values) = symbol {
                    values.iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|node|{
                        if let Sexp::Node(name, ref mut values) = node {
                            if name == "property" {
                                for value in values {
                                    if let Sexp::Node(name, values) = value {
                                        if name == "effects" {
                                            for value in values {
                                                if let Sexp::Node(name, values) = value {
                                                    if name == "justify" {
                                                        values.clear();
                                                    }
                                                }
                                            }
                                        } else if name == "at" {
                                            values[0] = Sexp::Value(pos[0].to_string());
                                            values[1] = Sexp::Value((_size[[1, 1]] + LABEL_BORDER).to_string());
                                            values[2] = Sexp::Value((0.0 - angle).to_string());
                                        }
                                    }
                                }
                            }
                        }
                    });
                }

                return Ok(());
            } else if positions[2] == 1 { //east
                todo!();
                /* vis_fields[0].pos = (_size[0][0]-1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.RIGHT]
                vis_fields[0].angle = 360 - symbol.angle */

            } else if positions[1] == 1 { //south
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
                _size[[0, 1]] - ((vis_field as f64-1.0) * LABEL_BORDER) - LABEL_BORDER
            } else {
                _size[[1, 1]] - ((vis_field as f64-1.0) * LABEL_BORDER) - LABEL_BORDER
            };
            if positions[3] == 0 { //north
                if let Sexp::Node(_, ref mut values) = symbol {
                    values.iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|node|{
                        if let Sexp::Node(name, ref mut values) = node {
                            if name == "property" {
                                for value in values {
                                    if let Sexp::Node(name, values) = value {
                                        if name == "effects" {
                                            for value in values {
                                                if let Sexp::Node(name, values) = value {
                                                    if name == "justify" {
                                                        values.clear();
                                                    }
                                                }
                                            }
                                        } else if name == "at" {
                                            values[0] = Sexp::Value(pos[0].to_string());
                                            values[1] = Sexp::Value((top_pos - offset).to_string());
                                            values[2] = Sexp::Value((0.0 - angle).to_string());
                                            offset -= LABEL_BORDER;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                return Ok(());

            } else if positions[2] == 0 { //east
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0) - 
                    ((vis_field as f64-1.0) * LABEL_BORDER) / 2.0;
                if let Sexp::Node(_, ref mut values) = symbol {
                    values.iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|node|{
                        if let Sexp::Node(name, ref mut values) = node {
                            if name == "property" {
                                for value in values {
                                    if let Sexp::Node(name, values) = value {
                                        if name == "effects" {
                                            for value in values {
                                                if let Sexp::Node(name, values) = value {
                                                    if name == "justify" {
                                                        values.clear();
                                                        values.push(Sexp::Value(String::from("left")));
                                                    }
                                                }
                                            }
                                        } else if name == "at" {
                                            values[0] = Sexp::Value((_size[[1, 0]] + LABEL_BORDER / 2.0).to_string());
                                            values[1] = Sexp::Value((top_pos - offset).to_string());
                                            values[2] = Sexp::Value((360.0 - angle).to_string());
                                            offset -= LABEL_BORDER;
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
                return Ok(());

            } else if positions[0] == 0 { //west
                todo!();
            } else if positions[1] == 0 { //south
                return Ok(());
            } else {
                return Ok(()); //todo!();
            }
        }
        Err(Error::ParseError)
    }


    /// get the pin position
    /// returns an array containing the number of pins:
    ///   3
    /// 2   0
    ///   1 
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
            position[lib_pos] += 1;
        }
        position.rotate_right(symbol_shift);
        if mirror.contains("x") {
            position = vec![position[0], position[3], position[2], position[1]];
        } else if mirror.contains("y") {
            position = vec![position[2], position[1], position[0], position[3]];
        }
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
