use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, Write},
    path::Path,
};

use lazy_static::lazy_static;
use pyo3::prelude::*;
use pyo3::{types::PyDict, Python};
use regex::Regex;

use super::cells::CellDispatch;
use super::{cells::error, parser};
use crate::error::Error;

lazy_static! {
    pub static ref RE_TOKEN: regex::Regex = Regex::new(r"\{\{(.*)\}\}").unwrap();
}

#[pyclass]
#[derive(Clone)]
pub struct LoggingStdout {
    content: Vec<u8>,
}

#[pymethods]
impl LoggingStdout {
    #[new]
    fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }
    fn write(&mut self, data: &str) {
        self.content.write_all(data.as_bytes()).unwrap();
    }
    pub fn dump(&self) -> String {
        String::from_utf8(self.content.clone()).unwrap()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LoggingStderr {
    content: Vec<u8>,
}
#[pymethods]
impl LoggingStderr {
    #[new]
    fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }
    fn write(&mut self, data: &str) {
        self.content.write_all(data.as_bytes()).unwrap();
    }
    pub fn dump(&self) -> String {
        String::from_utf8(self.content.clone()).unwrap()
    }
}

///open a file and return a line iterator.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}



pub struct Document {
    parser: parser::CellParser,
}

impl Default for Document {
    fn default() -> Self {
        Self { parser: parser::CellParser::new() }
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            parser: parser::CellParser::new(),
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<(), Error> {
        let lines = read_lines(input)?;
        for line in lines.flatten() {
            self.parser.push(&line)?;
        }
        Ok(())
    }

    pub fn run(&self, mut out: Box<dyn Write>, source: String, dest: String) -> Result<(), Error> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            let globals = PyDict::new(py);
            for cell in self.parser.iter() {
                //capture the python stdout and stderr
                let sys = py.import("sys").unwrap();
                let stdout = LoggingStdout::new(); //into_py(py);
                let stderr = LoggingStderr::new(); //into_py(py);
                sys.setattr("stdout", &stdout.into_py(py)).unwrap();
                sys.setattr("stderr", stderr.into_py(py)).unwrap();

                if let Err(err) = cell.write(&mut out, &py, globals, locals, &source, &dest) {
                    match err {
                        Error::Notebook(key, message) => error(
                            &mut out,
                            &key,
                            message.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        // Error::Cast(e) => error(&mut out, "Can not cast result:", e.to_string().as_bytes(), &HashMap::new()),
                        Error::Variable(e) => error(
                            &mut out,
                            "Variable not found:",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::Python(e) => error(
                            &mut out,
                            "Python execution error:",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        // Error::FileNotFound(e) => error(&mut out, "File not found", e.to_string().as_bytes(), &HashMap::new()),
                        Error::PropertyNotFound(e) => error(
                            &mut out,
                            "Property not found",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::VariableNotFound(e) => error(
                            &mut out,
                            format!("Variable {} not found", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::VariableCastError(e) => error(
                            &mut out,
                            "Can not cast result",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::Latex(e) => error(
                            &mut out,
                            "Latex execution error",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        // Error::FigureNotSet => error(&mut out, "No variable for the figure set", &[], &HashMap::new()),
                        Error::NoInputFile() => {
                            error(&mut out, "No input file set", &[], &HashMap::new())
                        }
                        Error::UnknownCommand(e) => error(
                            &mut out,
                            "Command not supported",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::NoCommand => error(&mut out, "No command set", &[], &HashMap::new()),
                        Error::LanguageNotSupported(e) => error(
                            &mut out,
                            "Language is not supported",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::GetPythonVariable(e) => error(
                            &mut out,
                            format!(
                                "can not get variable {} from python context.",
                                e
                            )
                            .as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::IoError(e) => error(
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
}

pub fn get_value<'a>(
    key: &str,
    py: &'a Python,
    globals: &'a PyDict,
    locals: &'a PyDict,
) -> Result<&'a PyAny, Error> {
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

pub fn parse_variables(
    body: &str,
    py: &Python,
    globals: &PyDict,
    locals: &PyDict,
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
