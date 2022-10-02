//use pyo3::exceptions::PyOSError;
//use pyo3::prelude::*;

use pyo3::{exceptions::PyOSError, PyErr};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Can not parse file.")]
    ParseError,
    /* #[error("can not find symbol.")]
    ExpectSexpNode,
    #[error("element is not a value node.")]
    ExpectValueNode,
    #[error("Justify value error.")]
    JustifyValueError,
    #[error("LineType value error")]
    LineTypeValueError,
    #[error("Sexp content not loaded.")]
    NotLoaded,
    #[error("More then one Property found for {0}")]
    MoreThenOnPropertyFound(String), */
    #[error("Pin not found for {0}")]
    PinNotFound(u32),
    #[error("can not find symbol {0}.")]
    SymbolNotFound(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Can not find Theme item: {0}{1}")]
    Theme(String, String),
    #[error("Spice model not found: {0}")]
    SpiceModelNotFound(String),
    #[error("Unknown circuit element {0}")]
    UnknownCircuitElement(String),
    #[error("No pins found in {0} for unit {1}")]
    NoPinsFound(String, u32),
    #[error("Property \"{0}\" not found in \"{1}\"")]
    PropertyNotFound(String, String),
    #[error("Library \"{0}\" not found in schema")]
    LinraryNotFound(String),
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
impl std::convert::From<cairo::Error> for Error {
    fn from(err: cairo::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<cairo::IoError> for Error {
    fn from(err: cairo::IoError) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}
