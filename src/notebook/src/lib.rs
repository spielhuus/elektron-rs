pub mod error;
mod cells;
mod notebook;
mod utils;

use std::collections::HashMap;
use std::io::Write;

use cells::{LoggingStderr, LoggingStdout};
use notebook::Notebook;
use pyo3::prelude::*;
use pyo3::{types::PyDict, Python};
use crate::error::Error;
use crate::cells::CellDispatch;

/// convert a markdown notebook.
///
/// * `input`: input file name.
/// * `out`:  output target.
/// * `source`: the source of the notebook.
/// * `dest`:  the dest of the notebook.
pub fn convert(input: &str, mut out: Box<dyn Write>, source: String, dest: String) -> Result<(), Error> {

    let nb = Notebook::open(input)?;
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            let globals = PyDict::new(py);
            for cell in nb.iter() {
                //capture the python stdout and stderr
                let sys = py.import("sys").unwrap();
                let stdout = LoggingStdout::new();
                let stderr = LoggingStderr::new();
                sys.setattr("stdout", &stdout.into_py(py)).unwrap();
                sys.setattr("stderr", stderr.into_py(py)).unwrap();

                if let Err(err) = cell.write(&mut out, &py, globals, locals, &source, &dest) {
                    match err {
                        Error::Notebook(key, message) => cells::error(
                            &mut out,
                            &key,
                            message.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::Variable(e) => cells::error(
                            &mut out,
                            "Variable not found:",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::Python(e) => cells::error(
                            &mut out,
                            "Python execution error:",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::PropertyNotFound(e) => cells::error(
                            &mut out,
                            "Property not found",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::VariableNotFound(e) => cells::error(
                            &mut out,
                            format!("Variable {} not found", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::VariableCastError(e) => cells::error(
                            &mut out,
                            "Can not cast result",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::Latex(e) => cells::error(
                            &mut out,
                            "Latex execution error",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::NoInputFile() => {
                            cells::error(&mut out, "No input file set", &[], &HashMap::new())
                        }
                        Error::UnknownCommand(e) => cells::error(
                            &mut out,
                            "Command not supported",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::NoCommand => cells::error(&mut out, "No command set", &[], &HashMap::new()),
                        Error::LanguageNotSupported(e) => cells::error(
                            &mut out,
                            "Language is not supported",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::GetPythonVariable(e) => cells::error(
                            &mut out,
                            format!(
                                "can not get variable {} from python context.",
                                e
                            )
                            .as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::IoError(e) => cells::error(
                            &mut out,
                            format!("can not open file {}", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::NgSpiceError(_) => {
                            todo!()
                        },
                    }
                }
            }
        });

        out.flush().unwrap();
    Ok(())
}
