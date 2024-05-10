use pyo3::{exceptions::PyIOError, PyErr};

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::error::Error for Error {}

///implement display for Error
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> Self {
        PyIOError::new_err(err.to_string())
    }
}

impl std::convert::From<sexp::Error> for Error {
    fn from(err: sexp::Error) -> Self {
        Error(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(err.to_string())
    }
}

impl std::convert::From<cairo::Error> for Error {
    fn from(err: cairo::Error) -> Self {
        Error(err.to_string())
    }
}

impl std::convert::From<cairo::IoError> for Error {
    fn from(err: cairo::IoError) -> Self {
        Error(err.to_string())
    }
}
