use pyo3::{Python, py_run, types::PyList};

pub fn gerber(input: String, output: String) {
    Python::with_gil(|py| {
        let list = PyList::new(py, &[input, output.to_string()]);
        py_run!(
            py,
            list,
            r#"from elektron import Pcb
            board = Pcb(list[0])
            board.gerber(list[1])"#
        );
    });
}
