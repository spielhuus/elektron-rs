#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Library not found {1} (2).")]
    DirectoryError(String, String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("{0}")]
    PropertyNotFound(String),
    #[error("Spice model not found: {0}")]
    SpiceModelNotFound(String),
    #[error("Unknown circuit element {0}")]
    UnknownCircuitElement(String),
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("NgSpice Error: \"{0}\"")]
    NgSpiceError(String),
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<ngspice::NgSpiceError> for Error {
    fn from(err: ngspice::NgSpiceError) -> Self {
        Error::IoError(err.to_string())
    }
}
