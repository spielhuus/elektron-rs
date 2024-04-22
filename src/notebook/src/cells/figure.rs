use std::collections::HashMap;

use crate::error::NotebookError;
use crate::notebook::ArgType;

use super::super::cells::{CellWrite, CellWriter};
use super::super::utils::clean_svg;

use super::{args_to_string, get_value, param_or};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<FigureCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &FigureCell,
        input: &str,
        _: &str,
    ) -> Result<(), NotebookError> {
        let _body = &cell.2;
        let args = &cell.1;

        let data_key = param_or!(args, "data", "py$ret");
        let v = get_value(data_key, py, globals, locals).map_err(|err| {
            NotebookError::new(
                input.to_string(),   
                String::from("FigureCell"),
                String::from("VariableError"),
                err.0,
                cell.0,
                cell.0,
                None,
            )
        })?;
        if let Ok(data) = v.extract::<Vec<u8>>() {
            writeln!(out, "{{{{< figure {}>}}}}", args_to_string(args)).unwrap();
            out.write_all(clean_svg(std::str::from_utf8(&data).unwrap()).as_bytes())
                .unwrap();
            writeln!(out, "{{{{< /figure >}}}}").unwrap();

            Ok(())
        } else {
            Err(NotebookError::new(
                input.to_string(),
                String::from("FigureCell"),
                String::from("ValueError"),
                String::from("plot value must be of type HashMap<String<Vec<f64>>"),
                cell.0,
                cell.0,
                None,
            ))
        }
    }
}
