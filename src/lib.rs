use pyo3::prelude::*;

pub extern crate pest;

#[macro_use]
pub extern crate pest_derive;

pub mod reports;
mod draw;
mod error;
//mod notebook;
pub mod plot;
mod python;
pub mod sexp;
mod spice;
mod gerber;

use python::circuit;
use python::model;

use self::python::PyDraw;

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDraw>()?;
    m.add_class::<model::Line>()?;
    m.add_class::<model::Dot>()?;
    m.add_class::<model::Label>()?;
    m.add_class::<model::Nc>()?;
    m.add_class::<model::Element>()?;
    m.add_class::<model::C>()?;
    m.add_class::<model::R>()?;
    m.add_class::<model::Gnd>()?;
    m.add_class::<model::Power>()?;
    m.add_class::<model::Feedback>()?;
    m.add_class::<circuit::Circuit>()?;
    m.add_class::<circuit::Simulation>()?;
    Ok(())
}
