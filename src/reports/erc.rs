//! Run the ERC schecks for the schema.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::erc::erc;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let results = erc(&schema).unwrap();
use std::collections::HashMap;

use itertools::Itertools;
use ndarray::Array1;

use crate::{
    error::Error,
    sexp::{Schema, SchemaElement, Shape, Transform, Symbol},
    spice::{Netlist, Point},
};

#[derive(Debug)]
/// ERC error types.
pub enum ErcItem {
    ///Reference for Symbol is not set or '?'.
    NoReference{reference: String, at:Array1<f64>},
    ///The values for the Symbol units are different.
    ValuesDiffer{reference: String, at:Array1<f64>},
    ///Unable to construct the Netlist.
    Netlist(String),
    ///Not all unts for the Symbol are on the schema.
    NotAllParts{reference: String, at:Array1<f64>},
    ///Pin is not connected.
    PinNotConnected{reference: String, at:Array1<f64>},
}

/// Run the ERC schecks for the schema.
///
/// # Arguments
///
/// * `document` - A Schema struct.
/// * `return`   - Vec<ErcItem> with the errors.
///
pub fn erc(document: &Schema) -> Result<Vec<ErcItem>, Error> {
    let mut results = Vec::new();
    let elements = symbols(document);
    results.append(&mut references(document, &elements));
    results.append(&mut values(&elements));
    let netlist = Netlist::from(document);
    match netlist {
        Ok(netlist) => {
            results.append(&mut pins(document, &elements, &netlist));
        },
        Err(netlist) => {
            results.push(ErcItem::Netlist(netlist.to_string()));
        }
    }
    Ok(results)
}

fn symbols(document: &Schema) -> HashMap<String, Vec<&Symbol>> {
    let mut elements: HashMap<String, Vec<&Symbol>> = HashMap::new();
    for item in document.iter_all() {
        if let SchemaElement::Symbol(symbol) = item {
            if !symbol.lib_id.starts_with("power:") && !symbol.lib_id.starts_with("Mechanical:"){
                if let Some(reference) = symbol.get_property("Reference") {
                    elements.entry(reference).or_default().push(symbol);
                }
            }
        }
    }
    elements
}

fn pins(document: &Schema, elements: &HashMap<String, Vec<&Symbol>>, netlist: &Netlist) -> Vec<ErcItem> {
    let mut results = Vec::new();
    let alphabet: Vec<char> = ('a'..='z').collect();
    for (_, symbols) in elements.iter() {
        for symbol in symbols {
            if let Some(libsymbol) = document.get_library(&symbols.first().unwrap().lib_id) {
                if let Ok(pins) = libsymbol.pins(symbol.unit) {
                    for pin in pins {
                        let point: Array1<f64> = Shape::transform(*symbol, &pin.at);
                        if netlist.node_name(&Point::new(point[0], point[1])).is_none() {
                            if libsymbol.symbols.len() > 1 {
                                results.push(ErcItem::PinNotConnected{reference: format!("{}{}:{}", symbol.get_property("Reference").unwrap(), alphabet[(symbol.unit - 1) as usize], pin.number.0), at: point});
                            } else {
                                results.push(ErcItem::PinNotConnected{reference: format!("{}:{}", symbol.get_property("Reference").unwrap(), pin.number.0), at: point});
                            }
                        }
                    }
                }
            }
        }
    }
    results
}

fn values(elements: &HashMap<String, Vec<&Symbol>>) -> Vec<ErcItem> {

    let mut results = Vec::new();
    for (_, symbols) in elements.iter() {
        let value = symbols[0].get_property("Value");
        for symbol in symbols {
            if value != symbol.get_property("Value") {
                results.push(ErcItem::ValuesDiffer{reference: symbol.get_property("Reference").unwrap(), at: symbol.at.clone()});
            }
        }
    }
    results
}

fn references(document: &Schema, elements: &HashMap<String, Vec<&Symbol>>) -> Vec<ErcItem> {
    let mut results = Vec::new();
    for (reference, symbols) in elements.iter() {
        if reference.contains('?') {
            results.push(ErcItem::NoReference{reference: reference.to_string(), at: symbols.first().unwrap().at.clone()});
        }
        let Some(libsymbol) = document.get_library(&symbols.first().unwrap().lib_id) else {
            //Library Symbol not found
            break;
        };
        let parts: usize = libsymbol.symbols.iter().filter_map(|u| {
            if u.unit > 0 {
                Some(u.unit)
            } else {
                None
            } 
        }).sorted().dedup().count();
        if parts != symbols.len() {
            //not all parts on schema
            results.push(ErcItem::NotAllParts{reference: reference.to_string(), at: symbols.first().unwrap().at.clone()});
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use itertools::Itertools; 
    use crate::sexp::Schema;
    use super::{erc, symbols};

    #[test]
    fn collect_symbols() {
        let schema = Schema::load("files/low_pass_filter_unconnected.kicad_sch").unwrap();
        let symbols = symbols(&schema);
        let mut keys: Vec<String> = Vec::new();
        for key in symbols.keys().sorted() {
            keys.push(key.to_string());
        }
        assert_eq!(vec![String::from("C1"), String::from("R1"), String::from("R?"), String::from("U1")], keys);
    }
    #[test]
    fn check_no_errors() {
        let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
        let erc = erc(&schema).unwrap();
        assert!(erc.is_empty());
    }
    #[test]
    fn check_with_mounting_holes() {
        let schema = Schema::load("files/produkt.kicad_sch").unwrap();
        let erc = erc(&schema).unwrap();
        assert!(erc.is_empty());
    }
    #[test]
    fn check_unconnected_pin() {
        let schema = Schema::load("files/low_pass_filter_unconnected.kicad_sch").unwrap();
        let erc = erc(&schema).unwrap();
        assert_eq!(10, erc.len());
    }
    #[test]
    fn all_units() {
        let schema = Schema::load("files/3280.kicad_sch").unwrap();
        let erc = erc(&schema).unwrap();
        assert_eq!(0, erc.len());
    }
}

