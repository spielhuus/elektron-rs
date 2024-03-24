/// Parse and access sexp files.
pub mod bom;
pub mod drc;
pub mod erc;
pub mod mouser;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Unable to create netlist file: {0} ({1}).")]
    NetlistFileError(String, String),
    #[error("{0}")]
    SexpError(String),
    #[error("File manipulatuion error {0} ({1}).")]
    IoError(String, String),
    #[error("Cam not parse YAML file: {0} ({1}).")]
    YamlError(String, String),
    #[error("Unable to load partlist: {0} ({1}).")]
    PartlistError(String, String),
}
impl std::convert::From<sexp::Error> for Error {
    fn from(err: sexp::Error) -> Self {
        Error::SexpError(err.to_string())
    }
}
