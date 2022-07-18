use crate::Error;
use crate::sexp::SexpParser;
use super::sexp;
use super::sexp::{Get, Sexp, Test, get, get_unit, get_property, get_pins};
use crate::shape::{Shape, Transform};

use ndarray::{Array1, Array2};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}
impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) && (self.y == other.y)
    }
}
impl Eq for Point {}
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            std::mem::transmute::<f64, u64>(self.x).hash(state);
            std::mem::transmute::<f64, u64>(self.y).hash(state);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NetlistItem {
    identifier: Option<String>,
    netlist_type: String,
    coords: Point,
}

impl NetlistItem {
    fn new(
        identifier: Option<String>,
        netlist_type: String,
        coords: Point,
    ) -> NetlistItem {
        NetlistItem {
            identifier,
            netlist_type,
            coords,
        }
    }
}

fn libraries(sexp_parser: &SexpParser) -> Result<std::collections::HashMap<String, Sexp>, Error> {
   let mut libraries: std::collections::HashMap<String, Sexp> = std::collections::HashMap::new();
   for element in sexp_parser.values() {
       if let Sexp::Node(name, values) = element {
           if name == "lib_symbols" {
               for value in values {
                   let name: String = value.get(0).unwrap();
                   libraries.insert(String::from(name), value.clone());
               }
           }
       }
   }
   Ok(libraries)
}

/// The Netlist struct
///
/// Create a netlist as a graph.
///
pub struct Netlist {
    index: u8,
    libraries: std::collections::HashMap<String, Sexp>,
    symbols: HashMap<String, Vec<Sexp>>,
    netlists: Vec<NetlistItem>,
    nodes: HashMap<Point, usize>,
}

/* impl SexpConsumer for Netlist {
    fn visit(&mut self, node: SexpNode) -> Result<(), Error> {
        if self.index == 1 && node.name == "symbol" {
            self.libraries.insert(get!(node, 0), node.clone());
        } else if self.index == 0 && node.name == "symbol" {
            let lib_id: String = get!(&node, "lib_id", 0);
            let unit: usize = node.unit().unwrap();
            let library = self.libraries.get(&lib_id).unwrap();
            let identifier: Option<String> = if library.contains("power") {
                node.property("Value")
            } else { None };
            for _unit in &library.nodes("symbol").unwrap() {
                let unit_number = _unit.unit().unwrap();
                if unit_number == 0 || unit_number == unit {
                    for graph in &_unit.values {
                        match graph {
                            SexpType::ChildSexpNode(pin) => {
                                if &pin.name == "pin" {
                                    let pin_pos: Array1<f64> = get!(pin, "at");
                                    let pts = node.transform(&pin_pos);
                                    let p0 = Point::new(pts[0], pts[1]);
                                    if self.nodes.contains_key(&p0) {
                                        let id = if let Some(_) = &identifier {
                                            identifier.clone()
                                        } else {
                                            let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                                            self.netlists[nl].identifier.clone()

                                        };
                                        let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                                        self.netlists[nl].netlist_type = get!(pin, 0);
                                        self.netlists[nl].identifier = id;
                                    } else {
                                        self.netlists.push(NetlistItem::new(
                                            identifier.clone(),
                                            get!(pin, 0),
                                            p0,
                                        ));
                                        self.nodes.insert(p0, self.netlists.len() - 1);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            let mut reference: Option<String> = None;
            for prop in &node.nodes("property").unwrap() {
                let key: String = get!(prop, 0);
                if key == "Reference" {
                    let r: String = get!(prop, 1);
                    reference = Option::from(r);
                }
            }
            match reference {
                Some(r) => {
                    if self.symbols.contains_key(&r) {
                        self.symbols.get_mut(&r).unwrap().push(node.clone());
                    } else {
                        self.symbols.insert(r, Vec::from([node.clone()]));
                    }
                }
                __ => {
                    println!("no reference in {:?}", node)
                }
            }
        } else if self.index == 0 && node.name == "wire" {
            let pts: Array2<f64> = get!(node, "pts");
            let p0 = Point::new(pts.row(0)[0], pts.row(0)[1]);
            let p1 = Point::new(pts.row(1)[0], pts.row(1)[1]);
            println!("search points: {:?} {:?} ", p0, p1);
            if self.nodes.contains_key(&p0) && self.nodes.contains_key(&p1) {
                println!("both ends exist");
            } else if self.nodes.contains_key(&p0) {
                let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                self.nodes.insert(p1, nl);
            } else if self.nodes.contains_key(&p1) {
                let nl: usize = *self.nodes.get_mut(&p1).unwrap();
                self.nodes.insert(p0, nl);
            } else {
                self.netlists.push(NetlistItem::new(
                    None,
                    "".to_string(),
                    p0,
                ));
                self.nodes.insert(p0, self.netlists.len() - 1);
                self.nodes.insert(p1, self.netlists.len() - 1);
            }
        } else if self.index == 0 && node.name == "label" {
            let pts: Array1<f64> = get!(node, "at");
            let p0 = Point::new(pts[0], pts[1]);
            if self.nodes.contains_key(&p0) {
                let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                let id: String = get!(&node, 0);
                self.netlists[nl].identifier = Option::from(id);
            } else {
                 let id: String = get!(&node, 0);
                self.netlists.push(NetlistItem::new(
                    Option::from(id),
                    "".to_string(),
                    p0,
                ));
                self.nodes.insert(p0, self.netlists.len() - 1);
            }
        } else if self.index == 0 && node.name == "global_label" {
            let pts: Array1<f64> = get!(node, "at");
            let p0 = Point::new(pts[0], pts[1]);
            if self.nodes.contains_key(&p0) {
                let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                let id: String = get!(&node, 0);
                self.netlists[nl].identifier = Option::from(id);
            } else {
                let id: String = get!(&node, 0);
                self.netlists.push(NetlistItem::new(
                    Option::from(id),
                    "".to_string(),
                    p0,
                ));
                self.nodes.insert(p0, self.netlists.len() - 1);
            }
        } else if self.index == 0 && node.name == "no_connect" {
            let pts: Array1<f64> = get!(node, "at");
            let p0 = Point::new(pts[0], pts[1]);
            if self.nodes.contains_key(&p0) {
                let nl: usize = *self.nodes.get_mut(&p0).unwrap();
                self.netlists[nl].identifier = Option::from("NC".to_string());
                self.netlists[nl].netlist_type = "no_connect".to_string();
            } else {
                self.netlists.push(NetlistItem::new(
                    Option::from("NC".to_string()),
                    "no_connect".to_string(),
                    p0,
                ));
                self.nodes.insert(p0, self.netlists.len() - 1);
            }
        }
        Ok(())
    }
    fn start_library_symbols(&mut self) -> Result<(), Error> {
        self.index += 1;
        Ok(())
    }
    fn end_library_symbols(&mut self) -> Result<(), Error> {
        self.index -= 1;
        Ok(())
    }
    fn start(&mut self, _: &String, _: &String) -> Result<(), Error> { Ok(()) }
    fn start_sheet_instances(&mut self) -> Result<(), Error> { Ok(()) }
    fn end_sheet_instances(&mut self) -> Result<(), Error> { Ok(()) }
    fn start_symbol_instances(&mut self) -> Result<(), Error> { Ok(()) }
    fn end_symbol_instances(&mut self) -> Result<(), Error> { Ok(()) }
    fn end(&mut self) -> Result<(), Error> { Ok(()) }
} */

impl Netlist {
    pub fn new() -> Self {
        Netlist {
            index: 0,
            libraries: std::collections::HashMap::new(),
            symbols: std::collections::HashMap::new(),
            netlists: Vec::new(),
            nodes: std::collections::HashMap::new(),
        }
    }
    pub fn from(doc: &SexpParser) -> Self {

        let libraries = libraries(doc).unwrap();
        let mut nodes: HashMap<Point, usize> =  std::collections::HashMap::new();
        let mut netlists: Vec<NetlistItem> = Vec::new();
        let mut symbols: HashMap<String, Vec<Sexp>> = HashMap::new();

        for node in doc.values() {
            if let Sexp::Node(name, values) = node {
                if name == "symbol" {
                    let lib_id: String = get!(node, "lib_id", 0);
                    let unit: usize = get_unit(node).unwrap();
                    let library = libraries.get(&lib_id).unwrap();
                    let identifier: Option<String> = if library.contains("power") {
                        Option::from(get_property(node, "Value").unwrap())
                    } else { None };
                    let syms: Vec<&Sexp> = library.get("symbol").unwrap();
                    for _unit in syms {
                        let unit_number = get_unit(_unit).unwrap();
                        if unit_number == 0 || unit_number == unit {
                            if let Sexp::Node(name, values) = _unit {
                                for el in values {
                                    if let Sexp::Node(name, values) = el {
                                        if name == "pin" {
                                            let pin_pos: Array1<f64> = get!(el, "at").unwrap();
                                            let pts = Shape::transform(node, &pin_pos);
                                            let p0 = Point::new(pts[0], pts[1]);
                                            if nodes.contains_key(&p0) {
                                                let id = if let Some(_) = &identifier {
                                                    identifier.clone()
                                                } else {
                                                    let nl: usize = *nodes.get_mut(&p0).unwrap();
                                                    netlists[nl].identifier.clone()

                                                };
                                                let nl: usize = *nodes.get_mut(&p0).unwrap();
                                                netlists[nl].netlist_type = get!(el, 0).unwrap();
                                                netlists[nl].identifier = id;
                                            } else {
                                                netlists.push(NetlistItem::new(
                                                    identifier.clone(),
                                                    get!(el, 0).unwrap(),
                                                    p0,
                                                ));
                                                nodes.insert(p0, netlists.len() - 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let mut reference: Option<String> = None;
                    let props: Vec<&Sexp> = node.get("property").unwrap();
                    for prop in props {
                        let key: String = get!(prop, 0).unwrap();
                        if key == "Reference" {
                            let r: String = get!(prop, 1).unwrap();
                            reference = Option::from(r);
                        }
                    }
                    match reference {
                        Some(r) => {
                            if symbols.contains_key(&r) {
                                symbols.get_mut(&r).unwrap().push(node.clone());
                            } else {
                                symbols.insert(r, Vec::from([node.clone()]));
                            }
                        }
                        __ => {
                            println!("no reference in {:?}", node)
                        }
                    }
                } else if name == "wire" {
                    let pts: Array2<f64> = get!(node, "pts").unwrap();
                    let p0 = Point::new(pts.row(0)[0], pts.row(0)[1]);
                    let p1 = Point::new(pts.row(1)[0], pts.row(1)[1]);
                    println!("search points: {:?} {:?} ", p0, p1);
                    if nodes.contains_key(&p0) && nodes.contains_key(&p1) {
                        println!("both ends exist");
                    } else if nodes.contains_key(&p0) {
                        let nl: usize = *nodes.get_mut(&p0).unwrap();
                        nodes.insert(p1, nl);
                    } else if nodes.contains_key(&p1) {
                        let nl: usize = *nodes.get_mut(&p1).unwrap();
                        nodes.insert(p0, nl);
                    } else {
                        netlists.push(NetlistItem::new(
                            None,
                            "".to_string(),
                            p0,
                        ));
                        nodes.insert(p0, netlists.len() - 1);
                        nodes.insert(p1, netlists.len() - 1);
                    }
                } else if name == "label" {
                    let pts: Array1<f64> = get!(node, "at").unwrap();
                    let p0 = Point::new(pts[0], pts[1]);
                    if nodes.contains_key(&p0) {
                        let nl: usize = *nodes.get_mut(&p0).unwrap();
                        let id: String = get!(&node, 0).unwrap();
                        netlists[nl].identifier = Option::from(id);
                    } else {
                         let id: String = get!(&node, 0).unwrap();
                        netlists.push(NetlistItem::new(
                            Option::from(id),
                            "".to_string(),
                            p0,
                        ));
                        nodes.insert(p0, netlists.len() - 1);
                    }
                } else if name == "global_label" {
                    let pts: Array1<f64> = get!(node, "at").unwrap();
                    let p0 = Point::new(pts[0], pts[1]);
                    if nodes.contains_key(&p0) {
                        let nl: usize = *nodes.get_mut(&p0).unwrap();
                        let id: String = get!(&node, 0).unwrap();
                        netlists[nl].identifier = Option::from(id);
                    } else {
                        let id: String = get!(&node, 0).unwrap();
                        netlists.push(NetlistItem::new(
                            Option::from(id),
                            "".to_string(),
                            p0,
                        ));
                        nodes.insert(p0, netlists.len() - 1);
                    }
                } else if name == "no_connect" {
                    let pts: Array1<f64> = get!(node, "at").unwrap();
                    let p0 = Point::new(pts[0], pts[1]);
                    if nodes.contains_key(&p0) {
                        let nl: usize = *nodes.get_mut(&p0).unwrap();
                        netlists[nl].identifier = Option::from("NC".to_string());
                        netlists[nl].netlist_type = "no_connect".to_string();
                    } else {
                        netlists.push(NetlistItem::new(
                            Option::from("NC".to_string()),
                            "no_connect".to_string(),
                            p0,
                        ));
                        nodes.insert(p0, netlists.len() - 1);
                    }
                }
            }
        }

        Netlist {
            index: 0,
            libraries: libraries,
            symbols: symbols,
            netlists: netlists,
            nodes: nodes,
        }
    }
    pub fn pins(&self, lib_name: &str) -> Result<HashMap<String, (Sexp, usize)>, Error> {
        let mut pins = HashMap::new();
        if self.libraries.contains_key(lib_name) {
            let lib: &Sexp = self.libraries.get(lib_name).unwrap();
            let symbols: Vec<&Sexp> = lib.get("symbol").unwrap();
            for symbol in symbols {
                //get the symbol unit number
                let name: String = get!(&symbol, 0).unwrap();
                let unit = if let Some(line) = sexp::RE.captures_iter(&name).next() {
                    line[1].parse::<usize>().unwrap()
                } else {
                    //TODO return Result<(), ParseError>
                    println!("unit name not found in: {:?} ", symbol);
                    0
                };
                //search the pins
                if symbol.contains("pin") {
                    let _pins: Vec<&Sexp> = symbol.get("pin").unwrap();
                    for pin in _pins {
                        let number: String = get!(pin, "number", 0);
                        pins.insert(number, (pin.clone(), unit));
                    }
                }
            }
        } //TODO generatea error when library is not foud
        Ok(pins)
    }
    /*
    fn property(&self, key: &str, node: &SexpNode) -> Option<String> {
        for prop in node.nodes("property").unwrap() {
            let my_key: String = get!(prop, 0);
            if my_key == key {
                let r: String = get!(prop, 1);
                return Option::from(r);
            }
        }
        None
    } */
    pub fn dump(&mut self) -> Result<(), Error> {


        //TODO should go to <end>
        //create a numeric netname for the unnamed nets in the netlist
        let mut _id = 1;
        for mut net in self.netlists.iter_mut() {
            match net.identifier {
                None => {
                    net.identifier = Option::from(_id.to_string());
                    _id += 1;
                }
                _ => {}
            }
        }

        //Create a spice entry for each referenca
        for reference in self.symbols.keys() {
            //but not for the power symbols
            if reference.starts_with('#') {
                continue;
            }

            let symbols = &self.symbols.get(reference).unwrap();
            let first_symbol: &Sexp = &symbols[0];

            //skip symbol when Netlist_Enabled is 'N'
            let netlist_enabled = get_property(first_symbol, "Spice_Netlist_Enabled").unwrap();
            if netlist_enabled == "N" {
                continue;
            }

            //create the pin order
            let lib_id: String = get!(first_symbol, "lib_id", 0);
            let my_pins = self.pins(&lib_id).unwrap();
            let mut pin_sequence: Vec<usize> = (0..my_pins.len()).collect();

            //when Node_Sequence is defined, use it
            let netlist_sequence = get_property(first_symbol, "Spice_Node_Sequence").unwrap();
            //if let Some(seq) = &netlist_sequence {
                pin_sequence.clear();
                let splits: Vec<&str> = netlist_sequence.split(' ').collect();
                for s in splits {
                    pin_sequence.push(s.parse::<usize>().unwrap());
                }
            //}

            //write the spice netlist item
            let spice_primitive = get_property(first_symbol, "Spice_Primitive");
            let spice_model = get_property(first_symbol, "Spice_Model");
            let spice_value = get_property(first_symbol, "Value");
            match spice_primitive {
                Ok(primitive) => {
                    if primitive == "X" {
                        let mut seq_string = String::new();
                        for seq in pin_sequence {
                            let real_pin = (&seq + 1).to_string();
                            let pin = my_pins.get(&real_pin).unwrap();
                            let pin_pos: Array1<f64> = get!(pin.0, "at").unwrap();
                            //get the symbol from the unit number
                            for s in symbols.iter() {
                                let unit: usize = get!(s, "unit", 0);
                                if unit == pin.1 {
                                    let pts = Shape::transform(s, &pin_pos);
                                    let p0 = Point::new(pts[0], pts[1]);
                                    if let Some(n) = self.nodes.get(&p0) {
                                        let id = if let Some(id) = &self.netlists[*n].identifier {
                                            id.clone()
                                        } else {
                                            "should not happen".to_string()
                                        };
                                        seq_string += &id;
                                    } else {
                                        seq_string += "NaN"
                                    }
                                    seq_string += " ";
                                }
                            }
                        }
                        println!(
                            "{}{} - {}- {}",
                            primitive,
                            reference,
                            seq_string,
                            spice_model.unwrap()
                        );
                    } else {
                        println!("{}{} - - {}", primitive, reference, spice_value.unwrap());
                    }
                },
                Err(Error::PropertyNotFound(_)) => {
                    let mut seq_string = String::new();
                    for seq in pin_sequence {
                        let real_pin = (&seq + 1).to_string();
                        let pin = my_pins.get(&real_pin).unwrap();
                        let pin_pos: Array1<f64> = get!(pin.0, "at").unwrap();
                        //get the symbol from the unit number
                        for s in symbols.iter() {
                            let unit: usize = get!(s, "unit", 0);
                            if unit == pin.1 {
                                let pts = Shape::transform(s, &pin_pos);
                                let p0 = Point::new(pts[0], pts[1]);
                                if let Some(n) = self.nodes.get(&p0) {
                                    let id = if let Some(id) = &self.netlists[*n].identifier {
                                        id.clone()
                                    } else {
                                        "should not happen".to_string()
                                    };
                                    seq_string += &id;
                                } else {
                                    seq_string += "NaN"
                                }
                                seq_string += " ";

                            }
                        }
                    }
                    println!("{} {}{}", reference, seq_string, spice_value.unwrap());
                }, 
                _ => { spice_primitive.unwrap(); }
            }
        }
        Ok(())
    }
}
