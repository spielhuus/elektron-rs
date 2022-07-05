use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::fs::File;
use std::io::{Read, Write};

use crate::sexp::parser::SexpParser;
use crate::sexp::Error;

use self::libraries::Libraries;

pub mod cairo_plotter;
pub mod libraries;
pub mod plot;
pub mod reports;
pub mod schema;
pub mod sexp;
pub mod sexp_write;
pub mod themes;
pub mod netlist;

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

#[pyfunction]
fn bom(input: &str, output: Option<String>, group: bool) -> PyResult<()> {
    let out: Box<dyn Write> = if let Some(filename) = output {
        Box::new(File::create(filename).unwrap())
    } else {
        Box::new(std::io::stdout())
    };
    let mut file = File::open(input)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let mut parser = SexpParser::new(&content);
    let mut bom = reports::Bom::new(out, group);
    parser.parse(&mut bom)?;
    Ok(())
}

#[pyfunction]
fn schema_plot(
    filename: &str,
    output: Option<String>,
    border: bool,
    scale: f64,
) -> Result<(), Error> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let mut parser = SexpParser::new(&content);
    let cairo_plotter = cairo_plotter::CairoPlotter::new();
    let mut plot = plot::Plot::new(Box::new(cairo_plotter), output, border, scale);
    parser.parse(&mut plot).unwrap();
    Ok(())
}

#[pyfunction]
fn search(term: &str, path: Vec<String>) -> String {
    let mut libs: Libraries = Libraries::new(path);
    if let Some(res) = libs.search(term) {
        res
    } else {
        String::from("not found!")
    }
}

#[pyfunction]
fn schema_netlist(input: &str, output: Option<String>) -> PyResult<()> {
    let out: Box<dyn Write> = if let Some(filename) = output {
        Box::new(File::create(filename).unwrap())
    } else {
        Box::new(std::io::stdout())
    };
    let mut file = File::open(input)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let mut parser = SexpParser::new(&content);
    let mut netlist = netlist::Netlist::new();
    parser.parse(&mut netlist)?;
    netlist.dump()?;
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(bom, m)?)?;
    m.add_function(wrap_pyfunction!(schema_plot, m)?)?;
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(schema_netlist, m)?)?;
    m.add_class::<schema::Schema>()?;
    Ok(())
}
