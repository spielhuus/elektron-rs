use crate::sexp::model::{LibrarySymbol, Pin, SchemaElement, Symbol};
use crate::sexp::{Shape, Transform};
use crate::error::Error;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::circuit::Circuit;

fn pins<'a>(
    lib_id: &str,
    unit: i32,
    libraries: &'a HashMap<String, LibrarySymbol>,
) -> Vec<&'a Pin> {
    let mut items: Vec<&Pin> = Vec::new();
    let library = libraries.get(lib_id).unwrap();
    for _unit in &library.symbols {
        if unit == 0 || _unit.unit == 0 || _unit.unit == unit {
            for pin in &_unit.pin {
                items.push(pin);
            }
        }
    }
    items
}
fn pin_names(
    lib_id: &str,
    libraries: &HashMap<String, LibrarySymbol>,
) -> Result<HashMap<String, (Pin, i32)>, Error> {
    let mut pins = HashMap::new();
    if libraries.contains_key(lib_id) {
        let lib = libraries.get(lib_id).unwrap();
        for symbol in &lib.symbols {
            //search the pins
            for pin in &symbol.pin {
                pins.insert(pin.number.0.clone(), (pin.clone(), symbol.unit));
            }
        }
    }
    Ok(pins)
}

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
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
            /* std::mem::transmute::<f64, u64>(self.x).hash(state);
            std::mem::transmute::<f64, u64>(self.y).hash(state); */
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetlistItem {
    identifier: Option<String>,
    netlist_type: String,
    coords: Point,
}

impl NetlistItem {
    fn new(identifier: Option<String>, netlist_type: String, coords: Point) -> NetlistItem {
        NetlistItem {
            identifier,
            netlist_type,
            coords,
        }
    }
}

/// The Netlist struct
///
/// Create a netlist as a graph.
///
pub struct Netlist {
    libraries: std::collections::HashMap<String, LibrarySymbol>,
    symbols: HashMap<String, Vec<Symbol>>,
    pub netlists: Vec<NetlistItem>,
    pub nodes: HashMap<Point, usize>,
}

impl Netlist {
    pub fn from<T: Iterator<Item = SchemaElement>>(document: &mut T) -> Result<Self, Error> {
        let mut libraries: HashMap<String, LibrarySymbol> = HashMap::new();
        let mut nodes: HashMap<Point, usize> = std::collections::HashMap::new();
        let mut netlists: Vec<NetlistItem> = Vec::new();
        let mut symbols: HashMap<String, Vec<Symbol>> = HashMap::new();

        for node in document {
            if let SchemaElement::LibrarySymbols(symbols) = node {
                libraries = symbols;
            } else if let SchemaElement::Symbol(symbol) = node {
                pins(&symbol.lib_id, symbol.unit, &libraries)
                    .iter()
                    .for_each(|el| {
                        let library = libraries.get(&symbol.lib_id).unwrap();
                        let identifier: Option<String> = if library.power {
                            symbol.get_property("Value")
                        } else {
                            None
                        };
                        let pts = Shape::transform(&symbol, &el.at);
                        let p0 = Point::new(pts[0], pts[1]);
                         if let std::collections::hash_map::Entry::Vacant(e) = nodes.entry(p0) {
                             netlists.push(NetlistItem::new(identifier, el.pin_type.clone(), p0));
                             e.insert(netlists.len() - 1);
                         } else {
                             let id = if identifier.is_some() {
                                 identifier
                             } else {
                                 let nl: usize = *nodes.get_mut(&p0).unwrap();
                                 netlists[nl].identifier.clone()
                             };
                             let nl: usize = *nodes.get_mut(&p0).unwrap();
                             netlists[nl].netlist_type = el.pin_type.clone();
                             netlists[nl].identifier = id;
                         }
                    });
                let reference: String = symbol.get_property("Reference").unwrap();
                symbols
                    .entry(reference)
                    .or_insert(Vec::new())
                    .push(symbol.clone());
            } else if let SchemaElement::Wire(wire) = node {
                let p0 = Point::new(wire.pts.row(0)[0], wire.pts.row(0)[1]);
                let p1 = Point::new(wire.pts.row(1)[0], wire.pts.row(1)[1]);
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
                    netlists.push(NetlistItem::new(None, "".to_string(), p0));
                    nodes.insert(p0, netlists.len() - 1);
                    nodes.insert(p1, netlists.len() - 1);
                }
            } else if let SchemaElement::Label(label) = node {
                let p0 = Point::new(label.at[0], label.at[1]);
                 if let std::collections::hash_map::Entry::Vacant(e) = nodes.entry(p0) {
                     netlists.push(NetlistItem::new(
                         Option::from(label.text),
                         "".to_string(),
                         p0,
                     ));
                     e.insert(netlists.len() - 1);
                 } else {
                     let nl: usize = *nodes.get_mut(&p0).unwrap();
                     netlists[nl].identifier = Option::from(label.text);
                 }
            } else if let SchemaElement::GlobalLabel(label) = node {
                let p0 = Point::new(label.at[0], label.at[1]);
                 if let std::collections::hash_map::Entry::Vacant(e) = nodes.entry(p0) {
                     netlists.push(NetlistItem::new(
                         Option::from(label.text),
                         "global_label".to_string(),
                         p0,
                     ));
                     e.insert(netlists.len() - 1);
                 } else {
                     let nl: usize = *nodes.get(&p0).unwrap();
                     netlists[nl].identifier = Option::from(label.text);
                 }
            } else if let SchemaElement::NoConnect(nc) = node {
                let p0 = Point::new(nc.at[0], nc.at[1]);
                 if let std::collections::hash_map::Entry::Vacant(e) = nodes.entry(p0) {
                     netlists.push(NetlistItem::new(
                         Option::from("NC".to_string()),
                         "no_connect".to_string(),
                         p0,
                     ));
                     e.insert(netlists.len() - 1);
                 } else {
                     let nl: usize = *nodes.get_mut(&p0).unwrap();
                     netlists[nl].identifier = Option::from("NC".to_string());
                     netlists[nl].netlist_type = "no_connect".to_string();
                 }
            }
        }

        Ok(Netlist {
            libraries,
            symbols,
            netlists,
            nodes,
        })
    }
    pub fn dump(&mut self, circuit: &mut Circuit) -> Result<(), Error> {
        //create a numeric netname for the unnamed nets in the netlist
        let mut _id = 1;
        for mut net in self.netlists.iter_mut() {
            if net.identifier == None {
                net.identifier = Option::from(_id.to_string());
                _id += 1;
            }
        }

        //Create a spice entry for each referenca
        for reference in self.symbols.keys() {
            //but not for the power symbols
            if reference.starts_with('#') {
                continue;
            }

            let symbols = &self.symbols.get(reference).unwrap();
            let first_symbol = &symbols[0];

            //skip symbol when Netlist_Enabled is 'N'
            let netlist_enabled = first_symbol.get_property("Spice_Netlist_Enabled");
            if let Some(enabled) = netlist_enabled {
                if enabled == "N" {
                    continue;
                }
            }

            //create the pin order
            let my_pins = pin_names(&first_symbol.lib_id, &self.libraries).unwrap();
            let mut pin_sequence: Vec<usize> = (0..my_pins.len()).collect();

            //when Node_Sequence is defined, use it
            let netlist_sequence = first_symbol.get_property("Spice_Node_Sequence");
            if let Some(sequence) = netlist_sequence {
                pin_sequence.clear();
                let splits: Vec<&str> = sequence.split(' ').collect();
                for s in splits {
                    pin_sequence.push(s.parse::<usize>().unwrap());
                }
            }

            //write the spice netlist item
            let spice_primitive = first_symbol.get_property("Spice_Primitive");
            let spice_model = first_symbol.get_property("Spice_Model");
            let spice_value = first_symbol.get_property("Value");
            match spice_primitive {
                Some(primitive) => {
                    if primitive == "X" {
                        let mut seq_string = String::new();
                        for seq in pin_sequence {
                            let real_pin = (&seq + 1).to_string();
                            let pin = my_pins.get(&real_pin).unwrap();
                            //get the symbol from the unit number
                            for s in symbols.iter() {
                                if s.unit == pin.1 {
                                    let pts = Shape::transform(s, &pin.0.at);
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
                            let nodes = seq_string.split(' ').map(|s| s.to_string()).collect();
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
                }
                None => {
                    let mut seq_string = String::new();
                    for seq in pin_sequence {
                        let real_pin = (&seq + 1).to_string();
                        let pin = my_pins.get(&real_pin).unwrap();
                        //get the symbol from the unit number
                        for s in symbols.iter() {
                            if s.unit == pin.1 {
                                let pts = Shape::transform(s, &pin.0.at);
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
                    if reference.starts_with('R') {
                        let nodes: Vec<String> =
                            seq_string.split(' ').map(|s| s.to_string()).collect();
                        circuit.resistor(
                            reference.clone(),
                            nodes[0].clone(),
                            nodes[1].clone(),
                            spice_value.unwrap(),
                        );
                    } else if reference.starts_with('C') {
                        let nodes: Vec<String> =
                            seq_string.split(' ').map(|s| s.to_string()).collect();
                        circuit.capacitor(
                            reference.clone(),
                            nodes[0].clone(),
                            nodes[1].clone(),
                            spice_value.unwrap(),
                        );
                    } else {
                        println!("->> {} {}{}", reference, seq_string, spice_value.unwrap());
                    }
                }
                /* _ => {
                    spice_primitive.unwrap();
                } */
            }
        }
        Ok(())
    }
}
