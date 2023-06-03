use std::collections::HashMap;

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::{Error, write_audio, param};

use super::{args_to_string, get_value, param_or};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<AudioCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &AudioCell,
        _: &str,
        dest: &str,
    ) -> Result<(), Error> {
        let _body = &cell.1;
        let args = &cell.0;

        let ext = param_or!(args, "ext", "wav");
        let data_key = param_or!(args, "data", "py$ret");
        let samplerate = param!(args, "samplerate", Error::PropertyNotFound(String::from("property samperate not found")));

        //get the data from the pyhton context
        let Ok(py_data) = get_value(samplerate, py, globals, locals) else  {
            return Err(Error::VariableNotFound(format!(
                "Variable with name '{}' can not be found.",
                data_key
            )));
        };

        let samplerate = if let Ok(data) = py_data.extract::<u32>() {
            Some(data)
        } else { None }.unwrap();

        let v = get_value(data_key, py, globals, locals)?;
        if let Ok(data) = v.extract::<Vec<f32>>() {
            writeln!(out, "{{{{< audio {}>}}}}", args_to_string(&write_audio(dest, data, ext, samplerate, args)?)).unwrap();
            Ok(())
        } else {
            Err(Error::VariableCastError(String::from(
                "plot value must be of type HashMap<Sting<Vec<f32>>",
            )))
        }
    }
}
