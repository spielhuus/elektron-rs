mod error;
mod python;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<crate::python::PyDraw>()?;
    m.add_class::<crate::python::model::Line>()?;
    m.add_class::<crate::python::model::Dot>()?;
    m.add_class::<crate::python::model::Label>()?;
    m.add_class::<crate::python::model::Nc>()?;
    m.add_class::<crate::python::model::Element>()?;
    m.add_class::<crate::python::model::C>()?;
    m.add_class::<crate::python::model::R>()?;
    m.add_class::<crate::python::model::Gnd>()?;
    m.add_class::<crate::python::model::Power>()?;
    m.add_class::<crate::python::model::Feedback>()?;
    m.add_class::<crate::python::circuit::Circuit>()?;
    m.add_class::<crate::python::circuit::Simulation>()?;
    Ok(())
}
