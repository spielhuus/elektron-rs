//! Run the DRC schecks for the board.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::drc::drc;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let results = drc(&schema).unwrap();
use std::collections::HashMap;
use std::{fs::File, io::{BufWriter, BufReader, Read}};
use std::io::prelude::*;
use itertools::Itertools;
use ndarray::Array1;
use rand::Rng;
use std::env::temp_dir;
use pyo3::{prelude::*, py_run, types::{PyList, PyBytes, PyDict}};
use regex::Regex;
use lazy_static::lazy_static;

use crate::{
    error::Error,
    sexp::{Schema, SchemaElement, Shape, Transform, Symbol},
    spice::{Netlist, Point},
};

lazy_static! {
    pub static ref DRC_TITLE_TOKEN: regex::Regex = Regex::new(r"^\[(.*)\]: (.*)$").unwrap();
    pub static ref DRC_DESC_TOKEN: regex::Regex = Regex::new(r"\s+Rule: (.*); Severity: (.*)").unwrap();
    pub static ref DRC_OVERRIDE_TOKEN: regex::Regex = Regex::new(r"\s+(.*); Severity: (.*)").unwrap();
    pub static ref DRC_POS_TOKEN: regex::Regex = Regex::new(r"\s+@\((.*) mm, (.*) mm\): (.*)").unwrap();

}

#[derive(Debug)]
/// DRC error types.
pub struct DrcItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub position: Vec<Position>,
}
#[derive(Debug)]
pub struct Position(pub String, pub String, pub String); 

impl DrcItem {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(), title: title.to_string(), description: String::new(), severity: String::new(), position: Vec::new(),
        }
    }
}

/// Run the DRC schecks for the board.
///
/// # Arguments
///
/// * `document` - A PCB struct.
/// * `return`   - Vec<DrcItem> with the errors.
///
pub fn drc(document: String, output: Option<String>) -> Result<Vec<DrcItem>, Error> {

    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    let filename = String::new()
        + temp_dir().to_str().unwrap()
        + "/"
        + &num.to_string()
        + ".txt";

    /* Python::with_gil(|py| {
        let list = PyList::new(py, &[document.to_string(), filename.to_string()]);
        println!("DRC {}->{}", document, filename);
        py_run!(py, list, r#"import pcbnew
            board = pcbnew.LoadBoard(list[0])
            pcbnew.WriteDRCReport(board, list[1], pcbnew.EDA_UNITS_MILLIMETRES, True)"#);
    }); */
    Python::with_gil(|py| {
        //pcbnew binding can not be called multiple times
        let pool = unsafe { py.new_pool() };
        let py = pool.python();
        let locals = PyDict::new(py);
        locals.set_item("document", document.to_string());
        locals.set_item("filename", filename.to_string());
        py.run(
            r#"
import pcbnew
board = pcbnew.LoadBoard(document)
pcbnew.WriteDRCReport(board, filename, pcbnew.EDA_UNITS_MILLIMETRES, True)
            "#,
            None,
            Some(locals),
        ).unwrap();

});
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    
    let mut results = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(cap) = DRC_TITLE_TOKEN.captures(&line) {
                results.push(DrcItem::new(cap.get(1).map_or("NONE", |m| m.as_str()), cap.get(2).map_or("NONE", |m| m.as_str())));
            } else if let Some(cap) = DRC_DESC_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.description = cap.get(1).map_or("NONE", |m| m.as_str()).to_string();
                    last.severity = cap.get(2).map_or("NONE", |m| m.as_str()).to_string();
                }
            } else if let Some(cap) = DRC_OVERRIDE_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.description = cap.get(1).map_or("NONE", |m| m.as_str()).to_string();
                    last.severity = cap.get(2).map_or("NONE", |m| m.as_str()).to_string();
                }
            } else if let Some(cap) = DRC_POS_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.position.push(Position(
                        cap.get(1).map_or("NONE", |m| m.as_str()).to_string(),
                        cap.get(2).map_or("NONE", |m| m.as_str()).to_string(),
                        cap.get(3).map_or("NONE", |m| m.as_str()).to_string(),
                    ))
                }
            } else if !line.starts_with("**") && !line.trim().is_empty() {
                println!("LINE: {}", line);
            }
        } else {
            panic!("can not read file");
        }
    }

    //TODO: workaround because of kicad issue
    results = results.into_iter().filter(|i| { !i.title.starts_with("The current configuration does not include the library") } ).collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools; 
    use crate::sexp::Schema;

    #[test]
    fn collect_symbols() {
    }
}

