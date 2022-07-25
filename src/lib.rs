use std::fs::File;
use std::io::Write;

use pyo3::exceptions::PyOSError;
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

use crate::sexp::parser::SexpParser;
use crate::reports::bom;
use crate::cairo_plotter::CairoPlotter;
use crate::themes::Style;
use crate::plot::plot;
use crate::libraries::Libraries;
use crate::circuit::Circuit;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Can not parse file.")]
    ParseError,
    #[error("can not find symbol {0}.")]
    SymbolNotFound(String),
    #[error("can not find symbol.")]
    ExpectSexpNode,
    #[error("element is not a value node.")]
    ExpectValueNode,
    #[error("Justify value error.")]
    JustifyValueError,
    #[error("LineType value error")]
    LineTypeValueError,
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("Sexp content not loaded.")]
    NotLoaded,
    #[error("Pin not found for {0}")]
    PinNotFound(usize),
    #[error("Property not found for {0}")]
    PropertyNotFound(String),
    #[error("More then one Property found for {0}")]
    MoreThenOnPropertyFound(String),
    #[error("Spice model not found: {0}")]
    SpiceModelNotFound(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

#[derive(Debug)]
#[pyclass]
pub struct SearchItem {
    lib: String,
    key: String,
    description: String,
}

#[pyfunction]
fn get_bom(input: &str, output: Option<&str>, group: bool) -> PyResult<()> {
    let parser = SexpParser::load(input)?;
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
        let mut out: Box<dyn Write> = Box::new(File::create(filename).unwrap());
        plot(&mut cairo, out, &parser, border, style).unwrap();
    } else {
        let mut out: Box<dyn Write> = Box::new(std::io::stdout());
        plot(&mut cairo, out, &parser, border, style).unwrap();
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
    let mut circuit = Circuit::new(vec!["/home/etienne/elektron/samples/files/spice".to_string()]);
    netlist.dump(&mut circuit)?;
    println!("{}", circuit);
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
    m.add_class::<SearchItem>()?;
    m.add_class::<circuit::Circuit>()?;
    Ok(())
}
