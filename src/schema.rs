use crate::cairo_plotter::CairoPlotter;
use crate::libraries::Libraries;
use crate::sexp::transform::{Transform, Bounds};
use crate::sexp::{
    Error, SexpConsumer, SexpNode, SexpText, SexpType, SexpValue,
};
use crate::sexp::get::{get, SexpGet};
use crate::sexp::set::{set, Set};
use crate::sexp::del::Del;
use crate::sexp_write::SexpWrite;
use crate::plot::Plot;
use crate::netlist::Netlist;
use pyo3::prelude::*;
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

use ndarray::{arr1, arr2, Array1};
use uuid::Uuid;
use crate::sexp::elements::{node, uuid, pos, stroke, effects, pts, property, junction, label, wire, symbol, symbol_instance, sheet};

/* macro_rules! node {
    ($key:expr, $($value:expr),*) => {
        SexpType::ChildSexpNode(SexpNode { name: $key.to_string(), values: vec![
            $(SexpType::ChildSexpValue(SexpValue { value: $value.to_string() }),)*]
        })
    }
}

macro_rules! uuid {
    () => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("uuid"),
            values: vec![SexpType::ChildSexpValue(SexpValue {
                value: Uuid::new_v4().to_string(),
            })],
        })
    };
}

macro_rules! pos {
    ($pos:expr) => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("at"),
            values: vec![
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[0].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[1].to_string(),
                }),
            ],
        })
    };
    ($pos:expr, $angle:expr) => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("at"),
            values: vec![
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[0].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[1].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $angle.to_string(),
                }),
            ],
        })
    };
}

macro_rules! stroke {
    () => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("stroke"),
            values: vec![
                node!("width", 0),
                node!("type", "default"),
                node!("color", 0, 0, 0, 0),
            ],
        })
    };
}

macro_rules! effects {
    () => {
        SexpType::ChildSexpNode(SexpNode { name: String::from("effects"), values: vec![
            SexpType::ChildSexpNode(SexpNode { name: String::from("font"), values: vec![
                SexpType::ChildSexpNode(SexpNode { name: String::from("size"), values: vec![
                    SexpType::ChildSexpValue(SexpValue { value: String::from("1.27") }),
                    SexpType::ChildSexpValue(SexpValue { value: String::from("1.27") })]
                    })]
                }),
            SexpType::ChildSexpNode(SexpNode { name: String::from("justify"), values: vec![
                SexpType::ChildSexpValue(SexpValue { value: String::from("left") }),
                SexpType::ChildSexpValue(SexpValue { value: String::from("bottom") })]
            })]
        })
    };
    ($font_width:expr, $font_height:expr, $($align:expr),+) => {
        SexpType::ChildSexpNode(SexpNode { name: String::from("effects"), values: vec![
            SexpType::ChildSexpNode(SexpNode { name: String::from("font"), values: vec![
                SexpType::ChildSexpNode(SexpNode { name: String::from("size"), values: vec![
                    SexpType::ChildSexpValue(SexpValue { value: String::from($font_width.to_string()) }),
                    SexpType::ChildSexpValue(SexpValue { value: String::from($font_width.to_string()) })]
                    })]
                }),
            SexpType::ChildSexpNode(SexpNode { name: String::from("justify"), values: vec![
                $(SexpType::ChildSexpValue(SexpValue { value: String::from($align.to_string()) }),)* ]
            })]
        })
    }
}

macro_rules! pts {
    ($($pt:expr),+) => {
        SexpType::ChildSexpNode(SexpNode {name: String::from("pts"), values: vec![
            $(SexpType::ChildSexpNode(SexpNode { name: String::from("xy"), values: vec![
                SexpType::ChildSexpValue( SexpValue {
                    value: String::from($pt[0].to_string()),
                }),
                SexpType::ChildSexpValue( SexpValue {
                    value: String::from($pt[1].to_string()),
                }),
            ]}),)*
        ]})
    }
}

macro_rules! property {
    ($pos:expr, $angle:expr, $key:expr, $value:expr, $id:expr) => {
        SexpType::ChildSexpNode(SexpNode { name: "property".to_string(), values: vec![
            SexpType::ChildSexpText(SexpText { value: $key.to_string() }),
            SexpType::ChildSexpText(SexpText { value: $value.to_string() }),
            node!("id", $id),
            pos!($pos, $angle),
            effects!(),
        ]})
    }
}

macro_rules! junction {
    ($pos:expr) => {
        SexpNode {
            name: String::from("junction"),
            values: vec![
                pos!($pos),
                node!("diameter", "0"),
                node!("color", 0, 0, 0, 0),
                uuid!(),
            ],
        }
    };
}

macro_rules! label {
    ($pos:expr, $angle:expr, $name:expr) => {
        SexpNode {
            name: String::from("label"),
            values: vec![
                SexpType::ChildSexpText(SexpText { value: $name }),
                pos!($pos, $angle),
                effects!(
                    "1.27",
                    "1.27",
                    if vec![0.0, 90.0].contains($angle) {
                        "left"
                    } else {
                        "right"
                    }
                ),
                uuid!(),
            ],
        }
    };
}

macro_rules! wire {
    ($pts:expr) => {
        SexpNode {
            name: String::from("wire"),
            values: vec![pts!($pts.row(0), $pts.row(1)), stroke!(), uuid!()],
        }
    };
}

macro_rules! symbol {
    ($pos:expr, $angle:expr, $reference:expr, $library:expr, $unit:expr, $uuid:expr) => {
        SexpNode {
            name: String::from("symbol"),
            values: vec![
                node!("lib_id", $library),
                pos!($pos, $angle),
                node!("unit", $unit),
                node!("in_bom", "yes"),
                node!("on_board", "yes"),
                node!("uuid", $uuid),
            ],
        }
    };
}

macro_rules! sheet {
    ($path:expr, $page:expr) => {
        SexpNode {
            name: String::from("path"),
            values: vec![
                SexpType::ChildSexpText(SexpText {
                    value: $path.to_string(),
                }),
                SexpType::ChildSexpNode(SexpNode {
                    name: String::from("page"),
                    values: vec![SexpType::ChildSexpText(SexpText {
                        value: $page.to_string(),
                    })],
                }),
            ],
        }
    };
}

macro_rules! symbol_instance {
    ($uuid:expr, $reference:expr, $value:expr, $unit:expr, $footprint:expr) => {
        SexpNode {
            name: String::from("path"),
            values: vec![
                SexpType::ChildSexpText(SexpText {
                    value: $uuid.to_string(),
                }),
                node!("reference", $reference),
                node!("unit", $unit),
                node!("value", $value),
                node!("footprint", $footprint.unwrap_or(String::from("~"))),
            ],
        }
    };
} */

#[pyclass]
pub struct Schema {
    version: String,
    generator: String,
    uuid: SexpNode,
    paper: SexpNode,
    title_block: SexpNode,
    elements: Vec<SexpNode>,
    libraries: Vec<SexpNode>,
    sheet_instance: Vec<SexpNode>,
    symbol_instance: Vec<SexpNode>,
    libs: Libraries,
}

#[pymethods]
impl Schema {
    #[new]
    pub fn new(library_path: Vec<String>) -> Schema {
        let p = vec![SexpType::ChildSexpValue(SexpValue::new(String::from("A4")))];
        let uuid = Uuid::new_v4();
        let u = vec![SexpType::ChildSexpValue(SexpValue::new(uuid.to_string()))];
        Schema {
            version: String::from("20211123"),
            generator: String::from("elektron"),
            uuid: SexpNode::from(String::from("uuid"), u),
            paper: SexpNode::from(String::from("paper"), p),
            title_block: SexpNode::new(String::from("title_block")),
            elements: Vec::<SexpNode>::new(),
            libraries: Vec::<SexpNode>::new(),
            sheet_instance: vec![sheet!("/", "1")],
            symbol_instance: Vec::<SexpNode>::new(),
            libs: Libraries::new(library_path),
        }
    }

    fn pin_pos(&mut self, reference: &str, pin: i32) -> Vec<f64> {
        let symbol = self.get_symbol(reference, 1).unwrap();
        let lib_name: String = get!(&symbol, "lib_id", 0);
        let library = self.get_library(&lib_name).unwrap();
        for _unit in &library.nodes("symbol").unwrap() {
            let number: usize = _unit.unit().unwrap(); //get!(_pin, "unit", 0);
            let sym_pins = _unit.nodes("pin");
            if let Ok(p) = sym_pins {
                for _pin in p {
                    let number_node: SexpNode = _pin.get("number").unwrap();
                    let _pin_number: i32 = number_node.get(0).unwrap();
                    if _pin_number == pin {
                        let pin_pos: Array1<f64> = get!(_pin, "at");
                        let _lib_instance = self.get_symbol(reference, number).unwrap();
                        let at = _lib_instance.transform(&pin_pos);
                        return vec![at[0], at[1]];
                    }
                }
            }
        }
        panic!("pin not found {}:{}", reference, pin); //TODO return error
    }
    fn wire(&mut self, pts: Vec<f64>, end: Vec<f64>) {
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

        let sym_pin = lib_symbol.pin(pin).unwrap();
        let pin_pos: Array1<f64> = get!(sym_pin, "at");
        // transform pin pos
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pin_pos.dot(&rot);
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
        verts = arr1(&[pos[0], pos[1]]) - &verts;
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());

        if let Some(end_pos) = end_pos {
            let pins = lib_symbol.pins(Option::from(unit));
            if pins.len() == 2 {
                for p in pins {
                    let pin_number: usize = get!(p, "number", 0);
                    if pin_number != unit {
                        let other_pos: Array1<f64> = get!(p, "at");
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
        for prop in &mut lib_symbol.nodes("property").unwrap() {
            if let SexpType::ChildSexpText(p) = prop.values.get(0).unwrap() {
                //skip properties with ki_
                if p.value.starts_with("ki_") {
                    break;
                //set the reference
                } else if p.value == "Reference" {
                    symbol.values.push(property!(verts, 0.0, "Reference", reference.to_string(), "0"));
                    /* if let SexpType::ChildSexpText(v) = prop.values.get_mut(1).unwrap() {
                        v.value = reference.to_string();
                    } */
                //set the value
                } else if p.value == "Value" {
                    symbol.values.push(property!(verts, 0.0, "Value", value.to_string(), "1"));
                    /* if let SexpType::ChildSexpText(v) = prop.values.get_mut(1).unwrap() {
                        v.value = value.to_string();
                    } */
                } else {
                    symbol.values.push(SexpType::ChildSexpNode(prop.clone()));
                }
            }
        }
        // add the extra properties
        for (k, v) in properties.iter() {
            symbol.values.push(property!(verts, 0.0, k, v, "1"));
        }

        self.place_property(&mut symbol).unwrap(); //TODO
        self.elements.push(symbol);

        let mut footprint: Option<String> = None;
        for prop in lib_symbol.nodes("property").unwrap() {
            if let SexpType::ChildSexpText(p) = prop.values.get(0).unwrap() {
                //set the reference
                if p.value.starts_with("footprint") {
                    if let SexpType::ChildSexpText(v) = prop.values.get(1).unwrap() {
                        footprint = Option::from(v.value.clone());
                    }
                }
            }
        }
        self.symbol_instance
            .push(symbol_instance!(uuid, reference, value, 1, footprint));
    }

    pub fn write(&mut self, filename: Option<&str>, pretty: bool) {
        let out: Box<dyn Write> = if let Some(filename) = filename {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        let writer: Box<dyn SexpConsumer> = Box::new(SexpWrite::new(out, pretty));
        match self._write(writer) {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
    pub fn plot(&mut self, filename: Option<String>, border: bool, scale: f64) {
        let plotter = Box::new(CairoPlotter::new());
        let plot = Plot::new(plotter, filename, border, scale);
        match self._write(Box::new(plot)) {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
    pub fn netlist(&mut self, filename: Option<String>) {
        let netlist = Box::new(Netlist::new());
        match self._write(netlist) {
            Ok(_) => {
                //netlist.dump();
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

impl Schema {

    //check if this schema has a library symbol.
    fn has_library(&mut self, name: &str) -> bool {
        for l in &self.libraries {
            let lib_name: String = get!(l, 0);
            if name == lib_name {
                return true;
            }
        }
        false
    }

    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<SexpNode, Error> {
        if !self.has_library(name) {
            let mut lib_symbol = self.libs.get(name).unwrap();
            let val = lib_symbol.values.get_mut(0).unwrap();
            if let SexpType::ChildSexpText(v) = val {
                v.value = name.to_string();
            } else {
                println!("other type found for {:?}", val); //TODO this is an error ERROR
            }
            self.libraries.push(lib_symbol.clone());
            return Ok(lib_symbol);
        } else {
            for l in &self.libraries {
                let lib_name: String = get!(l, 0);
                if name == lib_name {
                    return Ok(l.clone());
                }
            }
        }
        Err(Error::LibraryNotFound(name.to_string()))
    }

    /// get the symbol by reference and unit from this schema.
    fn get_symbol(&mut self, reference: &str, unit: usize) -> Result<SexpNode, Error> {
        for l in &self.elements {
            if l.name == "symbol" {
                if let Some(_ref) = l.property("Reference") {
                    let _unit = l.unit().unwrap();
                    if reference == _ref && unit == _unit {
                        return Ok(l.clone());
                    }
                }
            }
        }
        Err(Error::SymbolNotFound(reference.to_string()))
    }

    fn _write(&mut self, mut writer: Box<dyn SexpConsumer>) -> Result<(), Error> {
        //produce the content
        writer.start(&self.version, &self.generator)?;
        writer.visit(&self.uuid)?;
        writer.visit(&self.paper)?;
        writer.visit(&self.title_block)?;
        writer.start_library_symbols()?;
        for lib in &self.libraries {
            writer.visit(lib)?;
        }
        writer.end_library_symbols()?;
        for element in &self.elements {
            writer.visit(element)?;
        }
        writer.start_sheet_instances()?;
        for sheet in &self.sheet_instance {
            writer.visit(sheet)?;
        }
        writer.end_sheet_instances()?;
        writer.start_symbol_instances()?;
        for lib in &self.symbol_instance {
            writer.visit(lib)?;
        }
        writer.end_symbol_instances()?;
        writer.end()?;
        Ok(())
    }

    fn place_property(&mut self, symbol: &mut SexpNode) -> Result<(), Error> {
        let pos: Array1<f64> = get!(symbol, "at");
        let vis_field = symbol.nodes("property").unwrap().iter().filter_map(|node| {
            let effects: SexpNode = get!(node, "effects");
            if !effects.has("hide") { Option::from(node) } else { None }
        }).count();
        let angle: f64 = get!(symbol, "at", 2);
        let lib_name: String = get!(symbol, "lib_id", 0);
        let lib = self.get_library(&lib_name).unwrap();
        let _size = symbol.transform(&symbol.bounds(&lib).unwrap());
        let positions = self.pin_position(&symbol, &lib);
        let mut offset = 0.0;
        let pins = lib.pins(None).len();
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
                symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                    let effects: SexpNode = get!(node, "effects");
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[1] = top_pos;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let SexpType::ChildSexpNode(effects) = n {
                                if effects.name == "effects" {
                                    effects.delete("justify".to_string()).unwrap();
                                }
                            }
                        }
                        set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                });
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
                symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                    let effects: SexpNode = get!(node, "effects");
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[1] = top_pos - offset;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let SexpType::ChildSexpNode(effects) = n {
                                if effects.name == "effects" {
                                    effects.delete("justify".to_string()).unwrap();
                                }
                            }
                        }
                        set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                });
                return Ok(());

            } else if positions[2] == 0 { //east
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0) - 
                    ((vis_field as f64-1.0) * 2.0) / 2.0;
                symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                    let effects: SexpNode = get!(node, "effects");
                    if !effects.has("hide") { 
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[0] = _size[[0, 1]];
                        field_pos[1] = top_pos - offset;
                        field_pos = field_pos + &pos;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let SexpType::ChildSexpNode(effects) = n {
                                if effects.name == "effects" {
                                    effects.delete("justify".to_string()).unwrap();
                                }
                            }
                        }
                        node.values.push(effects!( "1.27", "1.27", "left"));
                        set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                });
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

    fn pin_position(&self, symbol: &SexpNode, lib: &SexpNode) -> Vec<usize> {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_angle: f64 = get!(symbol, "at", 2);
        let symbol_shift: usize = (symbol_angle / 90.0).round() as usize;
        let mirror: String = if symbol.contains("mirror") {
            get!(symbol, "mirror", 0)
        } else { String::new() };

        for pin in lib.pins(Option::from(symbol.unit().unwrap())) {
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

#[cfg(test)]
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
}
