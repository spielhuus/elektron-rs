use crate::Error;
use crate::sexp::iterator::iterate_unit_pins;
use crate::sexp::parser::SexpParser;
use super::sexp;
use super::sexp::{Sexp, get_property};
use super::sexp::get::{Get, get};
use super::sexp::test::Test;
use super::sexp::iterator::libraries;
use super::circuit::Circuit;
use crate::shape::{Shape, Transform};

use ndarray::{Array1, Array2};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug)]
pub struct Point {
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
pub struct NetlistItem {
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

/* fn libraries(sexp_parser: &SexpParser) -> Result<std::collections::HashMap<String, Sexp>, Error> {
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
} */

/// The Netlist struct
///
/// Create a netlist as a graph.
///
pub struct Netlist<'a> {
    index: u8,
    sexp_doc: &'a SexpParser,
    libraries: std::collections::HashMap<String, &'a Sexp>,
    symbols: HashMap<String, Vec<Sexp>>,
    pub netlists: Vec<NetlistItem>,
    pub nodes: HashMap<Point, usize>,
}

impl<'a> Netlist<'a> {
    /* pub fn new() -> Self {
        Netlist {
            index: 0,
            libraries: std::collections::HashMap::new(),
            symbols: std::collections::HashMap::new(),
            netlists: Vec::new(),
            nodes: std::collections::HashMap::new(),
        }
    } */
    pub fn from(doc: &'a SexpParser) -> Self {

        let libraries = libraries(doc).unwrap();
        let mut nodes: HashMap<Point, usize> =  std::collections::HashMap::new();
        let mut netlists: Vec<NetlistItem> = Vec::new();
        let mut symbols: HashMap<String, Vec<Sexp>> = HashMap::new();

        for node in doc.iter() {
            if let Sexp::Node(name, _) = node {
                if name == "symbol" {
                    iterate_unit_pins(node, &libraries).iter().for_each(|el| {
                    let lib_id: String = get!(node, "lib_id", 0);
                    // let unit: usize = get_unit(node).unwrap();
                    let library = libraries.get(&lib_id).unwrap();
                    let identifier: Option<String> = if library.contains("power") {
                        Option::from(get_property(node, "Value").unwrap())
                    } else { None };
                    /* let syms: Vec<&Sexp> = library.get("symbol").unwrap();
                    for _unit in syms {
                        let unit_number = get_unit(_unit).unwrap();
                        if unit_number == 0 || unit_number == unit {
                            if let Sexp::Node(_, values) = _unit {
                                for el in values {
                                    if let Sexp::Node(name, _) = el {
                                        if name == "pin" { */
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
                                        /*}
                                    } 
                                }
                            }
                        }*/
                    });
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
                    if nodes.contains_key(&p0) && nodes.contains_key(&p1) {
                        let n1: usize = *nodes.get_mut(&p0).unwrap();
                        let n2: usize = *nodes.get_mut(&p1).unwrap();
                        for node in &mut nodes {
                            if node.1 == &n1 {
                                *node.1 = n2;
                            }
                        }
                        
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
                        let nl: usize = *nodes.get(&p0).unwrap();
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
            sexp_doc: doc,
            libraries,
            symbols,
            netlists,
            nodes,
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
    pub fn dump(&mut self, circuit: &mut Circuit) -> Result<(), Error> {

        println!("{:?}", self.netlists);
        println!("{:?}", self.nodes);
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
            let netlist_enabled = get_property(first_symbol, "Spice_Netlist_Enabled");
            match netlist_enabled {
                Ok(enabled) => {
                    if enabled == "N" {
                       continue;
                    }
                }
                _ => {
                }
            }

            //create the pin order
            let lib_id: String = get!(first_symbol, "lib_id", 0);
            let my_pins = self.pins(&lib_id).unwrap();
            let mut pin_sequence: Vec<usize> = (0..my_pins.len()).collect();

            //when Node_Sequence is defined, use it
            let netlist_sequence = get_property(first_symbol, "Spice_Node_Sequence");
                match netlist_sequence {
                    Ok(sequence) => {
                        pin_sequence.clear();
                        let splits: Vec<&str> = sequence.split(' ').collect();
                        for s in splits {
                            pin_sequence.push(s.parse::<usize>().unwrap());
                        }
                    }
                    _ => {}
            }

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
                        if primitive == "X" {
                            let nodes = seq_string.split(" ").map(|s| s.to_string()).collect();
                            circuit.circuit(reference.to_string(), nodes, spice_model.unwrap())?;
                        } else {
                            println!(
                                "=> {}{} - {}- {}",
                                primitive,
                                reference,
                                seq_string,
                                spice_model.unwrap()
                            );
                        }
                    } else {
                        println!("-> {}{} - - {}", primitive, reference, spice_value.unwrap());
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
                    if reference.starts_with("R") {
                        let nodes: Vec<String> = seq_string.split(" ").map(|s| s.to_string()).collect();
                        circuit.resistor(reference.clone(), nodes[0].clone(), nodes[1].clone(), spice_value.unwrap());
                    } else if reference.starts_with("C") {
                        let nodes: Vec<String> = seq_string.split(" ").map(|s| s.to_string()).collect();
                        circuit.capacitor(reference.clone(), nodes[0].clone(), nodes[1].clone(), spice_value.unwrap());
                    } else {
                        println!("->> {} {}{}", reference, seq_string, spice_value.unwrap());
                    }
                }, 
                _ => { spice_primitive.unwrap(); }
            }
        }
        Ok(())
    }
}
