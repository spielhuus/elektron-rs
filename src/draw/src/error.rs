use pyo3::{exceptions::PyOSError, PyErr};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("sexp parse error {0}.")]
    SexpError(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("Element Not Found {0}: {1}")]
    NotFound(String, String),
    #[error("Position not found {0}")]
    PositionNotFound(String),
    #[error("Symbol not found {0}.")]
    SymbolNotFound(String),
    #[error("Name not set in {0}")]
    Name(String),
    #[error("Pin {1} not found {0}")]
    PinNotFound(String, String),
    #[error("NgSpice Error: \"{0}\"")]
    NgSpiceError(String),
}
impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<sexp::Error> for Error {
    fn from(err: sexp::Error) -> Self {
        Error::SexpError(err.to_string())
    }
}
