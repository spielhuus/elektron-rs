use std::fs::File;
use std::io::Write;

use pyo3::prelude::*;

pub mod cairo_plotter;
pub mod libraries;
pub mod plot;
pub mod reports;
pub mod draw;
pub mod sexp;
pub mod themes;
pub mod netlist;
pub mod shape;
pub mod circuit;
pub mod ngspice;
pub mod error;

use crate::error::Error;
use crate::sexp::parser::SexpParser;
use crate::reports::bom;
use crate::cairo_plotter::CairoPlotter;
use crate::themes::Style;
use crate::plot::plot;
use crate::libraries::{Libraries, SearchItem};
use crate::circuit::{Circuit, Simulation};

use self::cairo_plotter::ImageType;


#[pyfunction]
fn get_bom(input: &str, output: Option<String>, group: bool) -> PyResult<()> {
    let parser = SexpParser::load(input)?;
    
    if let Some(filename) = &output {
        let path = std::path::Path::new(&filename);
        let parent = path.parent();
        if let Some(parent) = parent {
            if parent.to_str().unwrap() != "" && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
    }
    bom(output, &parser, group).unwrap();
    Ok(())
}

#[pyfunction]
fn schema_plot(
    filename: &str,
    output: Option<&str>,
    border: bool,
    scale: f64,
) -> Result<(), Error> {
    let mut cairo = CairoPlotter::new();
    let style = Style::new();
    let parser = SexpParser::load(filename).unwrap();


    if let Some(filename) = output {
        let path = std::path::Path::new(filename);
        let parent = path.parent();
        if let Some(parent) = parent {
            if parent.to_str().unwrap() != "" && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let out: Box<dyn Write> = Box::new(File::create(filename).unwrap());
        plot(&mut cairo, out, &parser, border, scale, style, ImageType::Svg).unwrap();
    } else {
        let out: Box<dyn Write> = Box::new(std::io::stdout());
        plot(&mut cairo, out, &parser, border, scale, style, ImageType::Svg).unwrap();
    };
    Ok(())
}

#[pyfunction]
fn search(term: &str, path: Vec<String>) -> PyResult<Vec<SearchItem>> {
    let mut libs: Libraries = Libraries::new(path);
    let res = libs.search(term)?;
    Ok(res)
}

#[pyfunction]
fn schema_netlist(input: &str, output: Option<String>) -> PyResult<()> {
    let out: Box<dyn Write> = if let Some(filename) = output {
        Box::new(File::create(filename).unwrap())
    } else {
        Box::new(std::io::stdout())
    };
    let parser = SexpParser::load(input).unwrap();
    let mut netlist = netlist::Netlist::from(&parser);
    let mut circuit = Circuit::new(input.to_string(), Vec::new());
    netlist.dump(&mut circuit)?;
    //TODO println!("{}", circuit);
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_bom, m)?)?;
    m.add_function(wrap_pyfunction!(schema_plot, m)?)?;
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(schema_netlist, m)?)?;
    m.add_class::<draw::Draw>()?;
    m.add_class::<libraries::SearchItem>()?;
    m.add_class::<circuit::Circuit>()?;
    m.add_class::<circuit::Simulation>()?;
    Ok(())
}
