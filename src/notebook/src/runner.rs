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
                        /* Error::Simulation(e) => error(&mut out, "Language is not supported", e.to_string().as_bytes(), &HashMap::new()),
                        Error::Sexp(e) => error(&mut out, "Sexp: ", e.to_string().as_bytes(), &HashMap::new()),
                        Error::Plotter(e) => error(&mut out, "Plotter: ", e.to_string().as_bytes(), &HashMap::new()), */
                        /* Error::Draw(e) => error(
                            &mut out,
                            "Draw: ",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ),
                        Error::PositionNotFound(e) => error(
                            &mut out,
                            "Draw: ",
                            e.to_string().as_bytes(),
                            &HashMap::new(),
                        ), */
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
                        /* Error::FileNotFound(err) => {
                            error(&mut out, err.to_string().as_str(), &[], &HashMap::new())
                        }
                        Error::ParseError(_) => todo!(),
                        Error::LibraryNotFound(_) => todo!(),
                        Error::SymbolNotFound(_) => todo!(),
                        Error::PinNotFound(_, _) => {
                            error(&mut out, err.to_string().as_str(), &[], &HashMap::new())
                        }
                        Error::NoPinsFound(_, _) => todo!(),
                        Error::Name(_) => todo!(),
                        Error::Unknown(_, _) => todo!(),
                        Error::NotFound(_, _) => todo!(),
                        Error::UnknownCircuitElement(_) => todo!(),
                        Error::SpiceModelNotFound(_) => todo!(),
                        Error::ConvertInt { source } => error(
                            &mut out,
                            format!("can not convert int {}", source).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::ConvertFloat { source } => todo!(), */
                        Error::IoError(e) => error(
                            &mut out,
                            format!("can not open file {}", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::NgSpiceError(_) => {
                            todo!()
                        },
                        /* Error::SpiceSimulationError(e) => error(
                            &mut out,
                            format!("Spice simulation returns with an error: {}", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ),
                        Error::PlotterError(e) => error(
                            &mut out,
                            format!("The plotter returns with an error: {}", e).as_str(),
                            &[],
                            &HashMap::new(),
                        ), */
                    }
                }
                /* match cell {
                    parser::Cell::Python(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                    },
                    parser::Cell::Tikz(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                    },
                    parser::Cell::Figure(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                    }
                    parser::Cell::Plot(arguments, _code) => {
                        /* let x: Option<Vec<f64>> = if let Some(ArgType::String(x)) = arguments.get("x") {
                             if x.starts_with("py$") {
                                let key: &str = x.strip_prefix("py$").unwrap();
                                if let Some(item) = locals.get_item(key) {
                                    if let Ok(var) = item.extract() {
                                        Some(var)
                                    } else {
                                        error(&mut out, ERR_CAST, item.get_type().to_string().as_bytes(), arguments);
                                        None
                                    }
                                } else {
                                    error(&mut out, ERR_VAR, key.as_bytes(), arguments);
                                    None
                                }
                            } else {
                                None
                            }
                        } else { None };
                        let y: Option<Vec<Vec<f64>>> = if let Some(ArgType::List(list)) = arguments.get("y") {
                            let mut result: Vec<Vec<f64>> = Vec::new();
                            for x in list {
                                if x.starts_with("py$") {
                                    let key: &str = x.strip_prefix("py$").unwrap();
                                    if let Some(item) = locals.get_item(key) {
                                        if let Ok(var) = item.extract() {
                                            result.push(var)
                                        } else {
                                            error(&mut out, ERR_CAST, item.get_type().to_string().as_bytes(), arguments);
                                        }
                                    } else {
                                        error(&mut out, ERR_VAR, key.as_bytes(), arguments);
                                    }
                                }
                            }
                            Some(result)
                        } else {
                            None
                        };
                        if let (Some(x), Some(y)) = (x, y) {
                            let buffer = plot(x, y);
                            figure(&mut out, buffer.as_bytes(), arguments);
                        } else {
                            //TODO: Handle error
                        } */
                    }
                    parser::Cell::Javascript(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                        /* match parse_variables(&code.join("\n"), &py, globals, locals) {
                            Ok(code) => javascript(&mut out, code.as_bytes(), arguments),
                            Err(err) => ErrWriter::write(&mut out, &err, arguments),
                        } */
                    }
                    parser::Cell::D3(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                    }
                    parser::Cell::Elektron(cell) => {
                        CellWriter::write(&mut out, &py, globals, locals, cell).unwrap();
                        /* if let Some(ArgType::String(command)) = arguments.get("command") {
                            if command == "bom" {
                                bom(&mut out, dir.as_str(), arguments);
                            } else if command == "schema" {
                                schema(&mut out, dir.as_str(), arguments);
                            } else if command == "pcb" {
                                pcb(&mut out, dir.as_str(), arguments);
                            }
                        } */
                    }
                    parser::Cell::Content(line) => {
                        writeln!(out, "{}", line).unwrap();
                    }
                    parser::Cell::Error(err) => {
                        error(&mut out, ERR_PARSE, err.to_string().as_bytes(), &HashMap::new());
                    }
                } */
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
