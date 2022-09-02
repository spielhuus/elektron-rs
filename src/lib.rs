#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod circuit;
pub mod draw;
pub mod error;
pub mod plot;
pub mod reports;
pub mod sexp;
pub mod spice;

pub use crate::error::Error;

use std::{fs::File, io::Write};

use pyo3::prelude::*;
use rust_fuzzy_search::fuzzy_compare;

use crate::{
    plot::{CairoPlotter, ImageType, Theme},
    sexp::model::{SchemaElement, Sheet},
};

use self::{
    circuit::{Circuit, Netlist},
    reports::BomItem,
    sexp::{model::LibrarySymbol, SchemaIterator, SexpParser, State},
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
    let doc = SexpParser::load(filename)?;
    reports::bom(&mut doc.iter().node(), group)
}

pub fn netlist(
    input: &str,
    output: Option<String>,
    spice_models: Vec<String>,
) -> Result<(), Error> {
    let doc = SexpParser::load(input)?; //.iter().node();
    let mut iter = doc.iter().node();
    let mut netlist = Netlist::from(&mut iter)?;
    let mut circuit = Circuit::new(input.to_string(), spice_models);
    netlist.dump(&mut circuit)?;
    circuit.save(output)?;
    Ok(())
}

pub fn dump(input: &str, output: Option<String>) -> Result<(), Error> {
    let doc = SexpParser::load(input)?; //.iter().node();
    let iter = doc.iter().node();
    if let Some(output) = output {
        check_directory(&output)?;
        let mut out = File::create(output)?;
        sexp::write(&mut out, iter)?;
    } else {
        sexp::write(&mut std::io::stdout(), iter)?;
    }
    Ok(())
}

pub fn get_library(key: &str, path: Vec<String>) -> Result<LibrarySymbol, Error> {
    let mut library = sexp::Library::new(path);
    library.get(key)
}

pub fn plot(
    input: &str,
    output: &str,
    scale: Option<f64>,
    border: bool,
    theme: Option<String>,
) -> Result<(), Error> {
    use crate::plot::{PlotIterator, Plotter};
    let scale: f64 = if let Some(scale) = scale { scale } else { 1.0 };
    let doc = SexpParser::load(input).unwrap();

    let iter: Vec<SchemaElement> = doc.iter().node().collect();
    let sheets: Vec<&Sheet> = iter
        .iter()
        .filter_map(|n| {
            if let SchemaElement::Sheet(sheet) = n {
                Some(sheet)
            } else {
                None
            }
        })
        .collect();

    for sheet in sheets {

    }

    let image_type = if output.ends_with(".svg") {
        ImageType::Svg
    } else if output.ends_with(".png") {
        ImageType::Png
    } else {
        ImageType::Pdf
    };
    let theme = if let Some(theme) = theme {
        if theme == "kicad_2000" {
            Theme::kicad_2000() //TODO:
        } else {
            Theme::kicad_2000()
        }
    } else {
        Theme::kicad_2000()
    };

    let iter = iter.into_iter().plot(theme, border).flatten().collect();
    let mut cairo = CairoPlotter::new(&iter);

    check_directory(output)?;
    let out: Box<dyn Write> = Box::new(File::create(output)?);
    cairo.plot(out, border, scale, image_type)?;
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
    m.add_class::<circuit::Circuit>()?;
    m.add_class::<circuit::Simulation>()?;
    Ok(())
}
