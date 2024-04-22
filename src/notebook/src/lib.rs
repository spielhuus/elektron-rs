mod cells;
pub mod error;
mod notebook;
mod utils;

use std::collections::HashMap;
use std::io::Write;

use crate::cells::CellDispatch;
use cells::{LoggingStderr, LoggingStdout};
pub use error::NotebookError;
use notebook::Notebook;
use pyo3::prelude::*;
use pyo3::{types::PyDict, Python};

use log::error;

#[derive(Debug, Clone)]
pub struct CodeLine {
    pub line: usize,
    pub code: String,
    pub annotation: Option<String>,
}

impl CodeLine {
    pub fn new(line: usize, code: String, annotation: Option<String>) -> Self {
        Self {
            line,
            code,
            annotation,
        }
    }
}

/// convert a markdown notebook.
///
/// * `input`: input file name.
/// * `out`:  output target.
/// * `source`: the source of the notebook.
/// * `dest`:  the dest of the notebook.
pub fn convert(
    input: &str,
    mut out: Box<dyn Write>,
    source: String,
    dest: String,
) -> Result<(), NotebookError> {
    let nb = Notebook::open(input).map_err(|err| {
        NotebookError::new(
            source,
            "Notebook".to_string(),
            "Can not open notebook".to_string(),
            format!("can not open noteboook at: {:?} ({})", input, err.0),
            0,
            0,
            None,
        )
    })?;
    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        let globals = PyDict::new(py);
        for cell in nb.iter() {
            //capture the python stdout and stderr
            let sys = py.import_bound("sys").unwrap();
            let stdout = LoggingStdout::new();
            let stderr = LoggingStderr::new();
            sys.setattr("stdout", &stdout.into_py(py)).unwrap();
            sys.setattr("stderr", stderr.into_py(py)).unwrap();

            if let Err(err) = cell.write(&mut out, &py, globals, locals, input, &dest) {
                cells::error(&mut out, &err, &HashMap::new());
                error!("{}", err.to_string());
            }
        }
    });

    out.flush().unwrap();
    Ok(())
}
