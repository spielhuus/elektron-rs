use std::collections::HashMap;

pub use tectonic::errors;
pub use tectonic::status;
pub use tectonic::driver;

use crate::Error;
use crate::parser::ArgType;
use crate::runner::get_value;
use crate::utils::{figure, clean_svg};
use crate::writer::{CellWrite, CellWriter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<FigureCell> for CellWriter {
        fn write(
            out: &mut dyn std::io::Write,
            py: &pyo3::Python,
            globals: &pyo3::types::PyDict,
            locals: &pyo3::types::PyDict,
            cell: &FigureCell,
            _: &str,
        ) -> Result<(), Error> {

        let args = &cell.0;

        if let Some(ArgType::String(source)) = args.get("source") {
            match get_value(source, py, globals, locals) {
                Ok(v) => {
                    match &v.extract::<Vec<u8>>() {
                        Ok(value) => {
                            figure(
                                out,
                                clean_svg(std::str::from_utf8(value).unwrap()).as_bytes(),
                                args,
                            );
                            Ok(())
                        },
                        Err(error) => Err(Error::Cast(error.to_string()))
                    }
                },
                Err(error) => Err(Error::VariableNotFound(error.to_string()))
            }
        } else {
            Err(Error::FigureNotSet)
        }
    }
}
