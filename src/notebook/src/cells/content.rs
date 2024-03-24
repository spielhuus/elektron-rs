use std::io::Write;

use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    cells::{CellWrite, CellWriter},
    error::Error,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentCell(pub Vec<String>);
impl CellWrite<ContentCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &ContentCell,
        _: &str,
        _: &str,
    ) -> Result<(), Error> {
        let body = &cell.0;

        match parse_variables(&body.join("\n"), py, globals, locals) {
            Ok(code) => {
                if code.is_empty() {
                    out.write_all("\n".as_bytes())?;
                } else {
                    out.write_all(code.as_bytes())?;
                }
                Ok(())
            }
            Err(err) => Err(Error::VariableNotFound(err.to_string())),
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
    globals: &'a pyo3::types::PyDict,
    locals: &'a pyo3::types::PyDict,
) -> Result<&'a pyo3::PyAny, Error> {
    if key.starts_with("py$") {
        let key: &str = key.strip_prefix("py$").unwrap();
        if let Ok(Some(item)) = locals.get_item(key) {
            Ok(item)
        } else if let Ok(Some(item)) = globals.get_item(key) {
            Ok(item)
        } else {
            Err(Error::Variable(key.to_string()))
        }
    } else if key.starts_with("py@") {
        let key: &str = key.strip_prefix("py@").unwrap();
        let res = py.eval(key, None, None);
        match res {
            Ok(res) => Ok(res),
            Err(err) => Err(Error::Python(err.to_string())),
        }
    } else {
        Err(Error::Variable(key.to_string()))
    }
}

fn parse_variables(
    body: &str,
    py: &pyo3::Python,
    globals: &pyo3::types::PyDict,
    locals: &pyo3::types::PyDict,
) -> Result<String, Error> {
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
