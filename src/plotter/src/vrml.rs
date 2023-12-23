use crate::error::Error;
use pyo3::{py_run, types::PyList, Python};

pub fn vrml(input: String, output: String) -> Result<(), Error> {
    Python::with_gil(|py| {
        let list = PyList::new(py, &[input, output.to_string()]);
        py_run!(
            py,
            list,
            r#"
                import pcbnew 
                board = pcbnew.LoadBoard(list[0])    
                print(f"create vrml: {list[0]}, {list[1]}")
                res = pcbnew.ExportVRML(list[0], 0.001, True, True, "components/", 0.0, 0.0)
                print(f"end: {res}")
            "#
        );
    });

    Ok(())
}
