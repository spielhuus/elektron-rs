use std::collections::HashMap;

pub use tectonic::driver;
pub use tectonic::errors;
pub use tectonic::status;

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::super::runner::parse_variables;
use super::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavascriptCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<JavascriptCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &JavascriptCell,
        _: &str,
        _: &str,
    ) -> Result<(), Error> {
        let body = &cell.1;
        // let args = &cell.0;

        match parse_variables(&body.join("\n"), py, globals, locals) {
            Ok(code) => {
                writeln!(out, "{{{{< javascript }}}}").unwrap();
                out.write_all(code.as_bytes()).unwrap();
                writeln!(out, "{{{{< /javascript >}}}}").unwrap();
                Ok(())
            }
            Err(err) => Err(Error::VariableNotFound(err.to_string())),
        }
    }
}
