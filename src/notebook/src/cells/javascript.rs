use std::collections::HashMap;

use crate::cells::{CellWrite, CellWriter};
use crate::error::NotebookError;
use crate::notebook::ArgType;

use super::parse_variables;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavascriptCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<JavascriptCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &JavascriptCell,
        input: &str,
        _: &str,
    ) -> Result<(), NotebookError> {
        let body = &cell.2;
        // let args = &cell.0;

        match parse_variables(&body.join("\n"), py, globals, locals) {
            Ok(code) => {
                writeln!(out, "{{{{< javascript }}}}").unwrap();
                out.write_all(code.as_bytes()).unwrap();
                writeln!(out, "{{{{< /javascript >}}}}").unwrap();
                Ok(())
            }
            Err(err) => Err(NotebookError::new(
                input.to_string(),
                String::from("JavascriptCell"),
                String::from("ParseError"),
                format!("Parse error: {}", err.0),
                cell.0,
                cell.0,
                None,
            )),
        }
    }
}
