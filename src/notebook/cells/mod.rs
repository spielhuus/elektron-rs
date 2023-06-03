use std::fs::File;
use std::path::Path;
use std::{collections::HashMap, io::Write};

use lazy_static::lazy_static;
use pyo3::PyAny;
use pyo3::{types::PyDict, Python};
use rand::{thread_rng, Rng};
use regex::Regex;

use super::parser::ArgType;
use super::utils::Symbols;
use crate::error::Error;

//TODO: mod circuit;
mod audio;
mod d3;
mod elektron;
mod figure;
mod javascript;
mod latex;
mod plot;
mod python;

fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub use self::{
    d3::D3Cell, elektron::ElektronCell, figure::FigureCell, audio::AudioCell,
    javascript::JavascriptCell, latex::TikzCell, plot::PlotCell, python::PythonCell,
};

lazy_static! {
    pub static ref RE_TOKEN: regex::Regex = Regex::new(r"\$\{(.*)\}").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cell {
    Audio(AudioCell),
    Python(PythonCell),
    Tikz(TikzCell),
    Figure(FigureCell),
    Plot(PlotCell),
    Javascript(JavascriptCell),
    Elektron(ElektronCell),
    D3(D3Cell),
    Content(ContentCell),
    //TODO: Circuit(CircuitCell), */
    Error(String),
}

pub trait CellDispatch {
    fn write(
        &self,
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        source: &str,
        dest: &str,
    ) -> Result<(), Error>;
}

impl CellDispatch for Cell {
    fn write(
        &self,
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        source: &str,
        dest: &str,
    ) -> Result<(), Error> {
        match self {
            Cell::Audio(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Python(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Tikz(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Javascript(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Elektron(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::D3(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Figure(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Plot(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Content(cell) => CellWriter::write(out, py, globals, locals, cell, source, dest),
            Cell::Error(cell) => {
                todo!("Cell Error: {:?}", cell);
              }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentCell(pub HashMap<String, ArgType>, pub Vec<String>);
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
        let body = &cell.1;
        let _args = &cell.0;

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

pub struct CellWriter;
pub trait CellWrite<T> {
    fn write(
        out: &mut dyn Write,
        py: &Python,
        globals: &PyDict,
        locals: &PyDict,
        cell: &T,
        source: &str,
        dest: &str,
    ) -> Result<(), Error>;
}

pub fn error(out: &mut dyn Write, errtype: &str, content: &[u8], args: &HashMap<String, ArgType>) {
    if let Some(ArgType::String(result)) = args.get("error") {
        if result == "hide" {
            return;
        }
    }
    writeln!(out, "{{{{< error message=\"{}\" >}}}}", errtype).unwrap();
    out.write_all(content).unwrap();
    writeln!(out, "{{{{< /error >}}}}\n").unwrap();
}

fn echo(out: &mut dyn Write, lang: &str, code: &str, args: &HashMap<String, ArgType>) {
    if let Some(ArgType::String(echo)) = args.get("echo") {
        if echo == "FALSE" {
            return;
        }
    }
    writeln!(out, "```{}", lang).unwrap();
    out.write_all(code.as_bytes()).unwrap();
    writeln!(out, "\n```").unwrap();
}

macro_rules! param {
    ($args:expr, $key:expr, $err:expr) => {
        if let Some(ArgType::String(key)) = $args.get($key) {
            Ok(key)
        } else {
            Err($err)
        }?
    };
}
pub(crate) use param;

/* macro_rules! params {
    ($args:expr, $key:expr, $err:expr) => {
        if let Some(ArgType::List(key)) = $args.get($key) {
            Ok(key)
        } else {
            Err($err)
        }?
    };
}
pub(crate) use params; */

macro_rules! param_or {
    ($args:expr, $key:expr, $or:expr) => {
        if let Some(ArgType::String(key)) = $args.get($key) {
            key
        } else {
            $or
        }
    };
}
pub(crate) use param_or;

pub fn write_plot(path: &str, plot: Vec<u8>, args: &HashMap<String, ArgType>) -> Result<HashMap<String, ArgType>, Error> {

    let out_dir = Path::new(path).join("_files");
    let rand_string: String = thread_rng()
        .sample_iter(&Symbols)
        .take(30)
        .map(char::from)
        .collect();
    let output_file = out_dir
        .join(format!("{}.svg", rand_string))
        .to_str()
        .unwrap()
        .to_string();
    check_directory(&output_file)?;

    let mut outfile = File::create(&output_file)?;
    outfile.write_all(&plot)?;
    let mut myargs = args.clone();
    if let Some(ArgType::Options(opts)) = myargs.get_mut("options") {
        opts.insert(String::from("path"), ArgType::String(format!("_files/{}.svg", rand_string)));
    } else {
        let mut map = HashMap::new();
        map.insert(String::from("path"), ArgType::String(format!("_files/{}.svg", rand_string)));
        myargs.insert(String::from("options"), ArgType::Options(map));
    }
    Ok(myargs)
}

pub fn write_audio(path: &str, audio: Vec<f32>, ext: &str, fs: u32, args: &HashMap<String, ArgType>) -> Result<HashMap<String, ArgType>, Error> {

    let out_dir = Path::new(path).join("_files");
    let rand_string: String = thread_rng()
        .sample_iter(&Symbols)
        .take(30)
        .map(char::from)
        .collect();
    let output_file = out_dir
        .join(format!("{}.{}", rand_string, ext))
        .to_str()
        .unwrap()
        .to_string();
    check_directory(&output_file)?;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: fs,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(output_file, spec).unwrap();
    for float in audio {
        writer.write_sample(float).unwrap();
    }
    let mut myargs = args.clone();
    if let Some(ArgType::Options(opts)) = myargs.get_mut("options") {
        opts.insert(String::from("path"), ArgType::String(format!("_files/{}.{}", rand_string, ext)));
    } else {
        let mut map = HashMap::new();
        map.insert(String::from("path"), ArgType::String(format!("_files/{}.{}", rand_string, ext)));
        myargs.insert(String::from("options"), ArgType::Options(map));
    }
    Ok(myargs)
}

pub fn args_to_string(args: &HashMap<String, ArgType>) -> String {
    let mut result = String::new();
    let mut first = true;
    for (key, value) in args {
        if key.starts_with("fig.") {
            if !first {
                result += " ";
            } else {
                first = false;
            }
            result += format!("{}=\"{}\"", key.strip_prefix("fig.").unwrap(), value).as_str();
        } else if key == "options" {
            if let ArgType::Options(opts) = value {
                for (key, value) in opts {
                    if !first {
                        result += " ";
                    } else {
                        first = false;
                    }
                    result += format!("{}=\"{}\"", key, value).as_str();
                }
            }
        }
    }
    result
}

pub fn get_value<'a>(
    key: &str,
    py: &'a Python,
    globals: &'a PyDict,
    locals: &'a PyDict,
) -> Result<&'a PyAny, Error> {
    if key.starts_with("py$") {
        let key: &str = key.strip_prefix("py$").unwrap();
        if let Some(item) = locals.get_item(key) {
            Ok(item)
        } else if let Some(item) = globals.get_item(key) {
            Ok(item)
        } else {
            Err(Error::GetPythonVariable(format!(
                "Figure: Variable with name '{}' can not be found.",
                key
            )))
        }
    } else if key.starts_with("py@") {
        let key: &str = key.strip_prefix("py@").unwrap();
        let res = py.eval(key, None, None);
        match res {
            Ok(res) => Ok(res),
            Err(err) => Err(Error::Python(err.to_string())),
        }
    } else if key.contains('.') {
        let k = key.split('.').next().unwrap();
        if let Some(item) = locals.get_item(k) {
            Ok(item)
        } else if let Some(item) = globals.get_item(k) {
            Ok(item)
        } else {
            Err(Error::GetPythonVariable(format!(
                "Variable with name '{}' can not be found.",
                k
            )))
        }
    } else if let Some(item) = locals.get_item(key) {
            Ok(item)
        } else if let Some(item) = globals.get_item(key) {
            Ok(item)
        } else {
            Err(Error::GetPythonVariable(format!(
                "Figure: Variable with name '{}' can not be found.",
                key
            )))
    }
}

fn parse_variables(
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
