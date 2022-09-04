#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod circuit;
mod draw;
mod error;
mod plot;
mod reports;
pub mod sexp;
mod spice;

pub use crate::error::Error;

use rand::Rng;
use std::fs::File;
use viuer::{print_from_file, Config};

use pyo3::prelude::*;
use rust_fuzzy_search::fuzzy_compare;
use std::env::temp_dir;

use self::{
    circuit::{Circuit, Netlist},
    reports::BomItem,
    sexp::{model::LibrarySymbol, Schema, SexpParser, State},
};

pub fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn bom(filename: &str, group: bool) -> Result<Vec<BomItem>, Error> {
    let schema = Schema::load(filename)?;
    reports::bom(&schema, group)
}

pub fn netlist(
    input: &str,
    output: Option<String>,
    spice_models: Vec<String>,
) -> Result<(), Error> {
    let schema = Schema::load(input)?;
    let mut netlist = Netlist::from(&schema)?;
    let mut circuit = Circuit::new(input.to_string(), spice_models);
    netlist.dump(&mut circuit)?;
    circuit.save(output)
}

pub fn dump(input: &str, output: Option<String>) -> Result<(), Error> {
    let schema = Schema::load(input)?;
    if let Some(output) = output {
        check_directory(&output)?;
        let mut out = File::create(output)?;
        schema.write(&mut out)
    } else {
        schema.write(&mut std::io::stdout())
    }
}

pub fn get_library(key: &str, path: Vec<String>) -> Result<LibrarySymbol, Error> {
    let mut library = sexp::Library::new(path);
    library.get(key)
}

pub fn plot(
    input: &str,
    output: Option<String>,
    scale: Option<f64>,
    border: bool,
    theme: Option<String>,
) -> Result<(), Error> {
    let scale = if let Some(scale) = scale { scale } else { 1.0 };
    let theme = if let Some(theme) = theme {
        theme
    } else {
        "kicad_2000".to_string()
    };
    let schema = Schema::load(input)?;
    if let Some(filename) = &output {
        schema.plot(filename.as_str(), scale, border, theme.as_str())?;
    } else {
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen();
        let filename =
            String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".png";
        schema.plot(filename.as_str(), scale, border, theme.as_str())?;
        print_from_file(&filename, &Config::default()).expect("Image printing failed.");
    };
    Ok(())
}

pub fn search_library(key: &str, paths: Vec<String>) -> Result<Vec<(f32, String, String)>, Error> {
    let mut results: Vec<(f32, String, String)> = Vec::new();
    for path in paths {
        for entry in std::fs::read_dir(path).unwrap() {
            let dir = entry.unwrap();
            if dir.path().is_file() {
                let doc = SexpParser::load(dir.path().to_str().unwrap()).unwrap();
                let mut iter = doc.iter();

                if let Some(State::StartSymbol(name)) = &iter.next() {
                    if *name == "kicad_symbol_lib" {
                        iter.next(); //take first symbol
                        while let Some(state) = iter.next_siebling() {
                            if let State::StartSymbol(name) = state {
                                if name == "symbol" {
                                    if let Some(State::Text(id)) = iter.next() {
                                        let score: f32 = fuzzy_compare(
                                            &id.to_lowercase(),
                                            &key.to_string().to_lowercase(),
                                        );
                                        if score > 0.4 {
                                            while let Some(node) = iter.next() {
                                                if let State::StartSymbol(name) = node {
                                                    if name == "property" {
                                                        if let Some(State::Text(name)) = iter.next()
                                                        {
                                                            if name == "ki_description" {
                                                                if let Some(State::Text(desc)) =
                                                                    iter.next()
                                                                {
                                                                    results.push((
                                                                        score,
                                                                        id.to_string(),
                                                                        desc.to_string(),
                                                                    ));
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        panic!("file is not a symbol library")
                    }
                }
            }
        }
    }
    Ok(results)
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<draw::Draw>()?;
    m.add_class::<draw::model::Line>()?;
    m.add_class::<draw::model::Dot>()?;
    m.add_class::<draw::model::Label>()?;
    m.add_class::<draw::model::Element>()?;
    m.add_class::<circuit::Circuit>()?;
    m.add_class::<circuit::Simulation>()?;
    Ok(())
}
