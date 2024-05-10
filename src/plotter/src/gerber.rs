use pyo3::{types::PyDict, Python};
use pyo3::prelude::*;

use crate::Error;

pub fn gerber(input: String, output: String) -> Result<(), Error> {
    Python::with_gil(|py| {
        let list = PyDict::new_bound(py);
        list.set_item("input", input).unwrap();
        list.set_item("output", output.to_string()).unwrap();
        if let Err(err) = py.run_bound(
            r#"from elektron import Pcb
board = Pcb(input)
board.gerber(output)"#,
            Some(&list),
            None,
        ) {
            return Err(Error(format!("python error: {}", err)));
        }
        Ok(())
    })
}
