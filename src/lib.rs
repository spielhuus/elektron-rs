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

use crate::sexp::{Sexp, SexpParser};
use crate::reports::bom;
use crate::cairo_plotter::CairoPlotter;
use crate::themes::Style;
use crate::plot::plot;

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
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
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
    output: Option<String>,
    border: bool,
    scale: f64,
) -> Result<(), Error> {
    let mut cairo = CairoPlotter::new();
    let style = Style::new();
    let parser = SexpParser::load(filename).unwrap();
    plot(&mut cairo, Option::from("out.svg"), &parser, true, style).unwrap();
    Ok(())
}

#[pyfunction]
fn search(term: &str, path: Vec<String>) -> String {
    /* let mut libs: Libraries = Libraries::new(path);
    if let Some(res) = libs.search(term) {
        res
    } else {
        String::from("not found!")
    } */
    String::from("not implemented")
}

#[pyfunction]
fn schema_netlist(input: &str, output: Option<String>) -> PyResult<()> {
    /* let out: Box<dyn Write> = if let Some(filename) = output {
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
    netlist.dump()?; */

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
    Ok(())
}
