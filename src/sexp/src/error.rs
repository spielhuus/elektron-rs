///Enum of sexp error results.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    ///Can not manipulate file.
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Element Not Found {0}: {1}")]
    NotFound(String, String),
    #[error("Can not parse file: {0}")]
    ParseError(String),
    #[error("Can not laod content: {0}")]
    FileReadError(String),
    #[error("No pins found in {0} for unit {1}")]
    NoPinsFound(String, usize),
    #[error("Can not laod content: {0} ({1})")]
    FileError(String, String),
    #[error("NgSpice Error: \"{0}\"")]
    NgSpiceError(String),
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
