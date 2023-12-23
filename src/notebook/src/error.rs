#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    PropertyNotFound(String),
    #[error("Cell language is not supported {0}.")]
    LanguageNotSupported(String),
    #[error("`{0}`: {1}")]
    Notebook(String, String),
    #[error("{0}")]
    VariableNotFound(String),
    #[error("{0}")]
    VariableCastError(String),
    #[error("{0}")]
    Variable(String),
    #[error("{0}")]
    Python(String),
    #[error("{0}")]
    GetPythonVariable(String),
    #[error("{0}")]
    Latex(String),
    #[error("No command set")]
    NoCommand,
    #[error("{0}")]
    UnknownCommand(String),
    #[error("No Input file defined.")]
    NoInputFile(),
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
impl std::convert::From<plotter::Error> for Error {
    fn from(err: plotter::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
