//! Run the ERC schecks for the schema.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::erc::erc;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let results = erc(&schema).unwrap();
use std::{collections::HashMap, fmt};

use itertools::Itertools;
use ndarray::{Array1, arr1};

use crate::Error;
use sexp::math::{Shape, Transform};
use simulation::{Netlist, Point};

use sexp::{el, utils, Sexp, SexpProperty, SexpTree, SexpValueQuery, SexpParser};
#[derive(Debug, Clone)]
/// ERC error types.
pub enum ErcType {
    ///Reference for Symbol is not set or '?'.
    NoReference,
    ///The values for the Symbol units are different.
    ValuesDiffer,
    ///Unable to construct the Netlist.
    Netlist,
    ///Not all unts for the Symbol are on the schema.
    NotAllParts,
    ///Pin is not connected.
    PinNotConnected,
}

impl fmt::Display for ErcType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErcType::NoReference => write!(f, "NoReference"),
            ErcType::ValuesDiffer => write!(f, "ValuesDiffer"),
            ErcType::Netlist => write!(f, "Netlist"),
            ErcType::NotAllParts => write!(f, "NotAllParts"),
            ErcType::PinNotConnected => write!(f, "PinNotConnected"),
        }
    }
}

#[derive(Debug)]
/// ERC error types.
pub struct ErcItem {
    ///Reference for Symbol is not set or '?'.
    pub id: ErcType,
    pub reference: String, 
    pub at: Array1<f64>,
}

impl ErcItem {
    pub fn from(id: ErcType, reference: &str, at: Array1<f64>) -> Self {
        Self {
            id,
            reference: reference.to_string(),
            at: at.clone(),

        }
    }
}

/// Run the ERC schecks for the schema.
///
/// # Arguments
///
/// * `input`    - Filename of the shema.
/// * `return`   - Vec<ErcItem> with the errors.
pub fn erc(input: &str) -> Result<Vec<ErcItem>, Error> {
    let mut results = Vec::new();
    let doc = match SexpParser::load(input) {
        Ok(doc) => doc,
        Err(err) => return Err(Error::IoError(input.to_string(), err.to_string())),
    };
    let document = match SexpTree::from(doc.iter()) {
        Ok(doc) => doc,
        Err(err) => return Err(Error::SexpError(err.to_string())),
    };

    let elements = symbols(&document);
    results.append(&mut references(&document, &elements));
    results.append(&mut values(&elements));
    let netlist = Netlist::from(&document);
    match netlist {
        Ok(netlist) => {
            results.append(&mut pins(&document, &elements, &netlist));
        }
        Err(netlist) => {
            results.push(ErcItem::from(ErcType::Netlist, &netlist.to_string(), arr1(&[])));
        }
    }
    Ok(results)
}

/// Run the ERC schecks for the schema.
///
/// # Arguments
///
/// * `input`    - Filename of the shema.
/// * `return`   - Vec<ErcItem> with the errors.
pub fn erc_from_tree(document: &SexpTree) -> Result<Vec<ErcItem>, Error> {
    let mut results = Vec::new();
    let elements = symbols(document);
    results.append(&mut references(document, &elements));
    results.append(&mut values(&elements));
    let netlist = Netlist::from(document);
    match netlist {
        Ok(netlist) => {
            results.append(&mut pins(document, &elements, &netlist));
        }
        Err(netlist) => {
            results.push(ErcItem::from(ErcType::Netlist, &netlist.to_string(), arr1(&[])));
        }
    }
    Ok(results)
}

//Colllect all the symbols in the schematic.
pub fn symbols(document: &SexpTree) -> HashMap<String, Vec<&Sexp>> {
    let mut elements: HashMap<String, Vec<&Sexp>> = HashMap::new();
    for item in document.root().unwrap().nodes() {
        if item.name == el::SYMBOL {
            let lib_id: String = item.value(el::LIB_ID).unwrap();
            if !lib_id.starts_with("power:") && !lib_id.starts_with("Mechanical:") {
                if let Some(reference) = item.property(el::PROPERTY_REFERENCE) {
                    elements.entry(reference).or_default().push(item);
                }
            }
        }
    }
    elements
}

//Check if all pins are connected
fn pins(
    document: &SexpTree,
    elements: &HashMap<String, Vec<&Sexp>>,
    netlist: &Netlist,
) -> Vec<ErcItem> {
    let mut results = Vec::new();
    let alphabet: Vec<char> = ('a'..='z').collect();
    for (_, symbols) in elements.iter() {
        for symbol in symbols {
            let lib_id: String = symbol.value(el::LIB_ID).unwrap();
            let unit: usize = symbol.value(el::SYMBOL_UNIT).unwrap();
            if let Some(libsymbol) = utils::get_library(document.root().unwrap(), &lib_id) {
                if let Ok(pins) = utils::pins(libsymbol, unit) {
                    for pin in pins {
                        let at = utils::at(pin).unwrap();
                        let point: Array1<f64> = Shape::transform(*symbol, &at);
                        let number: String = pin.value(el::PIN_NUMBER).unwrap();
                        if netlist.node_name(&Point::new(point[0], point[1])).is_none() {
                            if unit > 1 {
                                results.push(
                                    ErcItem::from(
                                        ErcType::PinNotConnected,
                                        &format!(
                                            "{}{}:{}",
                                            <Sexp as SexpProperty::<String>>::property(
                                                symbol,
                                                el::PROPERTY_REFERENCE
                                            ).unwrap(),
                                            alphabet[unit - 1],
                                            number,
                                        ),
                                        point,
                                    )
                                );
                            } else {
                                results.push(ErcItem::from(
                                    ErcType::PinNotConnected,
                                    &format!(
                                        "{}:{}",
                                        <Sexp as SexpProperty::<String>>::property(
                                            symbol,
                                            el::PROPERTY_REFERENCE
                                        )
                                        .unwrap(),
                                        number
                                    ),
                                    point,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    results
}

///Check if all Values are the same for all symbols units.
fn values(elements: &HashMap<String, Vec<&Sexp>>) -> Vec<ErcItem> {
    let mut results = Vec::new();
    for (_, symbols) in elements.iter() {
        let value: String = symbols[0].property(el::PROPERTY_VALUE).unwrap();
        for symbol in symbols {
            if value
                != <Sexp as SexpProperty<String>>::property(symbol, el::PROPERTY_VALUE).unwrap()
            {
                let at = utils::at(symbols.first().unwrap()).unwrap();
                results.push(ErcItem::from(
                    ErcType::ValuesDiffer, 
                    &<Sexp as SexpProperty<String>>::property(symbol, el::PROPERTY_REFERENCE).unwrap(),
                    at.clone(),
                ));
            }
        }
    }
    results
}

///Check the symbol references
fn references(document: &SexpTree, elements: &HashMap<String, Vec<&Sexp>>) -> Vec<ErcItem> {
    let mut results = Vec::new();
    for (reference, symbols) in elements.iter() {
        let at = utils::at(symbols.first().unwrap()).unwrap();
        let lib_id: String = symbols.first().unwrap().value(el::LIB_ID).unwrap();
        if reference.contains('?') {
            results.push(ErcItem::from(
                ErcType::NoReference,
                reference,
                at.clone(),
            ));
        }
        let Some(libsymbol) = utils::get_library(document.root().unwrap(), &lib_id) else {
            //Library Symbol not found
            break;
        };
        let parts: usize = libsymbol
            .query(el::SYMBOL)
            .filter_map(|u| {
                let sub_name: String = u.get(0).unwrap();
                let unit_number = utils::unit_number(sub_name);
                if unit_number > 0 {
                    Some(unit_number)
                } else {
                    None
                }
            })
            .sorted()
            .dedup()
            .count();
        if parts != symbols.len() {
            //not all parts on schema
            results.push(ErcItem::from(
                ErcType::NotAllParts,
                reference,
                at.clone(),
            ));
        }
    }
    results
}
