//! Run the DRC schecks for the board.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::drc::drc;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let results = drc(&schema).unwrap();
use rand::Rng;
use std::env::temp_dir;
use std::io::prelude::*;
use std::process::Command;
use std::{
    fmt,
    fs::{self, File},
};

use crate::Error;

#[derive(Clone, Debug)]
pub enum ErrType {
    Unconnected,
    Violation,
    Parity,
}

impl fmt::Display for ErrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrType::Unconnected => write!(f, "Unconnected"),
            ErrType::Violation => write!(f, "Violation"),
            ErrType::Parity => write!(f, "Parity"),
        }
    }
}

#[derive(Debug)]
/// DRC error types.
pub struct DrcResult {
    pub coordinate_units: String,
    pub date: String,
    pub kicad_version: String,
    pub source: String,
    pub errors: Vec<DrcItem>,
}

impl DrcResult {
    pub fn new() -> Self {
        Self {
            coordinate_units: String::new(),
            date: String::new(),
            kicad_version: String::new(),
            source: String::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for DrcResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
/// DRC error types.
pub struct DrcItem {
    pub error_type: ErrType,
    pub drc_type: String,
    pub description: String,
    pub severity: String,
    pub items: Vec<DrcItems>,
}

#[derive(Debug)]
pub struct DrcItems {
    pub description: String,
    pub pos: (f64, f64),
    pub uuid: String,    
}

impl fmt::Display for DrcItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}, {})", self.description, self.pos.0, self.pos.1)
    }
}

/// Run the DRC schecks for the board.
///
/// # Arguments
///
/// * `document` - A PCB filename.
/// * `return`   - DrcResult with the errors.
///
pub fn drc(document: String) -> Result<DrcResult, Error> {
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    let output = String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".json";

    let result = if cfg!(target_os = "windows") {
        todo!("implement command for windows");
    } else {
        Command::new("kicad-cli")
            .arg("pcb")
            .arg("drc")
            .arg("--severity-all")
            .arg("--format")
            .arg("json")
            .arg("--output")
            .arg(output.clone())
            .arg(&document)
            .output()
            .expect("failed to execute process")
    };

    if result.status.code() != Some(0) {
        return Err(Error::IoError(output.to_string(), format!("failed to generate drc report from {}", document)));
    }

    let mut file = match File::open(output.clone()) {
        Ok(file) => file,
        Err(err) => return Err(Error::NetlistFileError(output.to_string(), err.to_string())),
    };
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Unable to read string");

    let js = json::parse(&data);
    let mut drc_result = DrcResult::new();
    if let Ok(js) = js {
        match js {
            json::JsonValue::Null => todo!(),
            json::JsonValue::Short(_) => todo!(),
            json::JsonValue::String(_) => todo!(),
            json::JsonValue::Number(_) => todo!(),
            json::JsonValue::Boolean(_) => todo!(),
            json::JsonValue::Object(obj) => {
                drc_result = DrcResult {
                    coordinate_units: obj.get("coordinate_units").map_or(String::from("NONE"), |m| m.to_string()),
                    date: obj.get("date").map_or(String::from("NONE"), |m| m.to_string()),
                    kicad_version: obj.get("kicad_version").map_or(String::from("NONE"), |m| m.to_string()),
                    source: obj.get("source").map_or(String::from("NONE"), |m| m.to_string()),
                    errors: Vec::new(),
                };
                if let Some(unconnected_items) = obj.get("unconnected_items") {
                    match unconnected_items {
                        json::JsonValue::Array(arr) => {
                            get_items(ErrType::Unconnected, arr, &mut drc_result.errors);
                        },
                        _ => todo!(),
                    }
                }
                if let Some(violations) = obj.get("violations") {
                    match violations {
                        json::JsonValue::Array(arr) => {
                            get_items(ErrType::Violation, arr, &mut drc_result.errors);
                        },
                        _ => todo!(),
                    }
                }
                if let Some(parity) = obj.get("schematic_parity") {
                    match parity {
                        json::JsonValue::Array(arr) => {
                            get_items(ErrType::Parity, arr, &mut drc_result.errors);
                        },
                        _ => todo!(),
                    }
                }
            },
            json::JsonValue::Array(_) => todo!(),
        }     
    }

    fs::remove_file(output).unwrap();

    Ok(drc_result)
}

fn get_items(error_type: ErrType,values: &Vec<json::JsonValue>, result: &mut Vec<DrcItem>) {
    for obj in values {
        match obj {
            json::JsonValue::Object(obj) => {
                let mut items: Vec<DrcItems> = Vec::new();
                for item in obj.get("items").unwrap().members() {
                    if let json::JsonValue::Object(item) = item {
                        items.push(DrcItems {
                            description: item.get("description").unwrap().to_string(),
                            pos: if let json::JsonValue::Object(pos) = item.get("pos").unwrap() {
                                (pos.get("x").unwrap().as_f64().unwrap(),
                                 pos.get("y").unwrap().as_f64().unwrap())
                            } else { (0.0, 0.0) },
                            uuid: item.get("uuid").unwrap().to_string(),
                        });
                    }
                }
                result.push(DrcItem {
                    error_type: error_type.clone(),
                    drc_type: obj.get("type").map_or(String::from("NONE"), |m| m.to_string()),
                    description: obj.get("description").map_or(String::from("NONE"), |m| m.to_string()),
                    severity: obj.get("severity").map_or(String::from("NONE"), |m| m.to_string()),
                    items,
                });
            },
            _ => todo!(),
        }
    }
}
