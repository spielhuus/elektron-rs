use pyo3::{types::PyDict, Python};

use crate::Error;

pub fn gerber(input: String, output: String) -> Result<(), Error> {
    Python::with_gil(|py| {
        let list = PyDict::new(py);
        list.set_item("input", input).unwrap();
        list.set_item("output", output.to_string()).unwrap();
        if let Err(err) = py.run(
            r#"from elektron import Pcb
board = Pcb(input)
board.gerber(output)"#,
            Some(list),
            None,
        ) {
            return Err(Error::IoError(format!("python error: {}", err)));
        }
        Ok(())
    })
}
