use pyo3::{exceptions::PyOSError, PyErr};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    ReportsError(String),
    #[error("The ngspice simulation returns with an error: {0}")]
    SpiceSimulationError(String),
    #[error("Can not plot file: {0}")]
    PlotterError(String),

    // sexp errors
    #[error("Can not parse file: {0}")]
    ParseError(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("Symbol not found {0}.")]
    SymbolNotFound(String),
    #[error("Pin {1} not found {0}")]
    PinNotFound(String, String),
    #[error("No pins found in {0} for unit {1}")]
    NoPinsFound(String, usize),

    //draw errors
    #[error("Name not set in {0}")]
    Name(String),
    #[error("Unknown Element in {0}: {1}")]
    Unknown(String, String),
    #[error("Element Not Found {0}: {1}")]
    NotFound(String, String),
    #[error("Position not found {0}")]
    PositionNotFound(String),

    //notebook
    #[error("{0}")]
    GetPythonVariable(String),
    #[error("Cell language is not supported {0}.")]
    LanguageNotSupported(String),
    #[error("{0}")]
    Latex(String),
    #[error("No command set")]
    NoCommand,
    #[error("{0}")]
    Python(String),
    #[error("{0}")]
    UnknownCommand(String),
    #[error("No Input file defined.")]
    NoInputFile(),
    #[error("{0}")]
    PropertyNotFound(String),
    #[error("{0}")]
    Variable(String),
    #[error("{0}")]
    VariableNotFound(String),
    #[error("{0}")]
    VariableCastError(String),
    #[error("`{0}`: {1}")]
    Notebook(String, String),

    // spice
    #[error("Unknown circuit element {0}")]
    UnknownCircuitElement(String),
    #[error("Spice model not found: {0}")]
    SpiceModelNotFound(String),

    //draw
    #[error("Draw Error: \"{0}\"")]
    Draw(String),

    #[error("Cannot convert int.")]
    ConvertInt {
        //TODO:
        #[from]
        source: std::num::ParseIntError,
        //backtrace: Backtrace,
    },
    #[error("Cannot convert float.")]
    ConvertFloat {
        #[from]
        source: std::num::ParseFloatError,
    },
    #[error("File not found {0}.")]
    FileNotFound(String),
    #[error("File manipulatuion error {0}.")]
    IoError(String),
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
impl std::convert::From<std::fmt::Error> for Error {
    fn from(err: std::fmt::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<elektron_ngspice::NgSpiceError> for Error {
    fn from(err: elektron_ngspice::NgSpiceError) -> Self {
        Error::NgSpiceError(err.to_string())
    }
}
impl std::convert::From<sexp::Error> for Error {
    fn from(err: sexp::Error) -> Self {
        Error::IoError(err.to_string())
    }
}
impl std::convert::From<simulation::Error> for Error {
    fn from(err: simulation::Error) -> Self {
        Error::SpiceSimulationError(err.to_string())
    }
}
impl std::convert::From<plotter::Error> for Error {
    fn from(err: plotter::Error) -> Self {
        Error::PlotterError(err.to_string())
    }
}
impl std::convert::From<draw::Error> for Error {
    fn from(err: draw::Error) -> Self {
        Error::PlotterError(err.to_string())
    }
}
impl std::convert::From<reports::Error> for Error {
    fn from(err: reports::Error) -> Self {
        Error::ReportsError(err.to_string())
    }
}