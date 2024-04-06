#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    Plotter(String),
    #[error("{0}")]
    SexpError(String),
    #[error("File not found {0}")]
    FileNotFound(String),
    #[error("File manipulatuion error ({0}).")]
    IoError(String),
    #[error("Unable to load partlist: {0} ({1}).")]
    PartlistError(String, String),
    #[error("Unable to create netlist file: {0} ({1}).")]
    NetlistFileError(String, String),
    #[error("Cam not parse YAML file: {0} ({1}).")]
    YamlError(String, String),
    #[error("NgSpice Error: \"{0}\"")]
    NgSpiceError(String),
}
impl std::convert::From<sexp::Error> for Error {
    fn from(err: sexp::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
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
