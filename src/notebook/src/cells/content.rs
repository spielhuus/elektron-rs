use std::io::Write;

use lazy_static::lazy_static;
use pyo3::prelude::*;
use regex::Regex;

use crate::{
    cells::{CellWrite, CellWriter},
    error::{NotebookError, ValueError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentCell(pub Vec<String>);
impl CellWrite<ContentCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &Bound<pyo3::types::PyDict>,
        locals: &Bound<pyo3::types::PyDict>,
        cell: &ContentCell,
        input: &str,
        _: &str,
    ) -> Result<(), NotebookError> {
        let body = &cell.0;

        match parse_variables(&body.join("\n"), py, globals, locals) {
            Ok(code) => {
                if code.is_empty() {
                    out.write_all("\n".as_bytes()).map_err(|err| {
                        NotebookError::new(
                            input.to_string(),
                            String::from("ContentCell"),
                            String::from("WriteError"),
                            err.to_string(),
                            0,
                            0,
                            None,
                        )
                    })?;
                } else {
                    out.write_all(code.as_bytes()).map_err(|err| {
                        NotebookError::new(
                            input.to_string(),
                            String::from("ContentCell"),
                            String::from("WriteError"),
                            err.to_string(),
                            0,
                            0,
                            None,
                        )
                    })?;
                }
                Ok(())
            }
            Err(err) => Err(NotebookError::new(
                input.to_string(),
                String::from("ContentCell"),
                String::from("WriteError"),
                err.0,
                0,
                0,
                None,
            )),
        }
    }
}

lazy_static! {
    //TODO pub static ref RE_TOKEN: regex::Regex = Regex::new(r"\{\{(.*)\}\}").unwrap();
    pub static ref RE_TOKEN: regex::Regex = Regex::new(r"\$\{(.*)\}").unwrap();
}

pub fn get_value<'a>(
    key: &str,
    py: &'a pyo3::Python,
    globals: &'a Bound<pyo3::types::PyDict>,
    locals: &'a Bound<pyo3::types::PyDict>,
) -> Result<Bound<'a, pyo3::PyAny>, ValueError> {
    if key.starts_with("py$") {
        let key: &str = key.strip_prefix("py$").unwrap();
        if let Ok(Some(item)) = locals.get_item(key) {
            Ok(item)
        } else if let Ok(Some(item)) = globals.get_item(key) {
            Ok(item)
        } else {
            Err(ValueError(format!("Variable {} not found", key)))
        }
    } else if key.starts_with("py@") {
        let key: &str = key.strip_prefix("py@").unwrap();
        let res = py.eval_bound(key, None, None);
        match res {
            Ok(res) => Ok(res),
            Err(err) => Err(ValueError(format!("Variable {} not found ({})", key, err))),
        }
    } else {
        Err(ValueError(format!("variable must start with 'py$' or 'py@' but is {}", key)))
    }
}

fn parse_variables(
    body: &str,
    py: &pyo3::Python,
    globals: &Bound<pyo3::types::PyDict>,
    locals: &Bound<pyo3::types::PyDict>,
) -> Result<String, ValueError> {
    let mut res: Vec<u8> = Vec::new();
    for line in body.lines() {
        let mut position = 0;
        for cap in RE_TOKEN.captures_iter(line) {
            let token = cap.get(1).map_or("", |m| m.as_str());
            let item = &line[position..cap.get(0).unwrap().start()];
            write!(res, "{}", item).unwrap();

            //search the value
            let val = get_value(token.trim(), py, globals, locals)?;
            write!(res, "{}", val).unwrap();
            position = cap.get(0).unwrap().end();
        }
        let item = &line[position..line.len()];
        writeln!(res, "{}", item).unwrap();
    }
    Ok(std::str::from_utf8(&res).unwrap().to_string())
}
