//! Run the DRC schecks for the board.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::drc::drc;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let results = drc(&schema).unwrap();
use lazy_static::lazy_static;
use pyo3::{prelude::*, types::PyDict};
use rand::Rng;
use regex::Regex;
use std::env::temp_dir;
use std::io::prelude::*;
use std::{
    fmt,
    fs::{self, File},
    io::BufReader,
};

use crate::Error;

use log::debug;

lazy_static! {
    pub static ref DRC_TITLE_TOKEN: regex::Regex = Regex::new(r"^\[(.*)\]: (.*)$").unwrap();
    pub static ref DRC_DESC_TOKEN: regex::Regex =
        Regex::new(r"\s+Rule: (.*); Severity: (.*)").unwrap();
    pub static ref DRC_OVERRIDE_TOKEN: regex::Regex =
        Regex::new(r"\s+(.*); Severity: (.*)").unwrap();
    pub static ref DRC_POS_TOKEN: regex::Regex =
        Regex::new(r"\s+@\((.*) mm, (.*) mm\): (.*)").unwrap();
}

#[derive(Debug)]
/// DRC error types.
pub struct DrcItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub position: Position,
}
#[derive(Debug)]
pub struct Position(pub String, pub String, pub String);
impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

impl DrcItem {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            title: String::new(),
            description: description.to_string(),
            severity: String::new(),
            position: Position(String::from("0.0"), String::from("0.0"), String::new()),
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
pub fn drc(document: String) -> Result<Vec<DrcItem>, Error> {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    let output = String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".txt";

    //TODO footprint dir as variable
    Python::with_gil(|py| {
        let globals = PyDict::new_bound(py);
        let locals = PyDict::new_bound(py);
        locals.set_item("document", document.clone()).unwrap();
        locals.set_item("filename", output.to_string()).unwrap();
        py.run_bound(
            r#"
import os
os.environ['KICAD8_FOOTPRINT_DIR'] = '/usr/share/kicad/footprints'

from elektron import Pcb
board = Pcb(document)
board.drc(filename)"#,
            Some(&globals),
            Some(&locals),
        )
        .map_err(|m| Error::IoError(m.to_string(), document))
    })?;

    let file = match File::open(output.clone()) {
        Ok(file) => file,
        Err(err) => return Err(Error::NetlistFileError(output.to_string(), err.to_string())),
    };
    let reader = BufReader::new(file);

    let mut results = Vec::new();
    for line in reader.lines() {
        debug!("> {:?}", line);
        if let Ok(line) = line {
            if let Some(cap) = DRC_TITLE_TOKEN.captures(&line) {
                results.push(DrcItem::new(
                    cap.get(1).map_or("NONE", |m| m.as_str()),
                    cap.get(2).map_or("NONE", |m| m.as_str()),
                ));
            } else if let Some(cap) = DRC_DESC_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.title = cap.get(1).map_or("NONE", |m| m.as_str()).to_string();
                    last.severity = cap.get(2).map_or("NONE", |m| m.as_str()).to_string();
                }
            } else if let Some(cap) = DRC_OVERRIDE_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.title = cap.get(1).map_or("NONE", |m| m.as_str()).to_string();
                    last.severity = cap.get(2).map_or("NONE", |m| m.as_str()).to_string();
                }
            } else if let Some(cap) = DRC_POS_TOKEN.captures(&line) {
                if let Some(last) = results.last_mut() {
                    last.position = Position(
                        cap.get(1).map_or("NONE", |m| m.as_str()).to_string(),
                        cap.get(2).map_or("NONE", |m| m.as_str()).to_string(),
                        cap.get(3).map_or("NONE", |m| m.as_str()).to_string(),
                    )
                }
            //this is for testing if everything is parsed
            //} else if !line.starts_with("**") && !line.trim().is_empty() {
            //    warn!("LINE: {}", line);
            }
        } else {
            return Err(Error::IoError(
                String::from("can not read temporary file"),
                output,
            ));
        }
    }

    fs::remove_file(output).unwrap();
    //TODO: workaround because of kicad issue
    results.retain(|i| {
        !i.title
            .starts_with("The current configuration does not include the library")
    });

    Ok(results)
}
