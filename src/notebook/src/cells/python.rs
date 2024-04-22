use std::collections::HashMap;

use pyo3::prelude::*;

use log::trace;

use crate::{error::NotebookError, notebook::ArgType, CodeLine};

use super::{
    args_to_string, echo, newlines, write_plot, CellWrite, CellWriter, LoggingStderr, LoggingStdout,
};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    pub static ref RE_LINE_NUMBER: regex::Regex = Regex::new(r"line ([0-9]*)").unwrap();
}

fn get_linenumber(err_text: &str) -> Option<usize> {
    let cap = RE_LINE_NUMBER.captures_iter(err_text).next();
    if let Some(cap) = cap {
        let text1 = cap.get(1).map_or("", |m| m.as_str());
        Some(text1.parse::<usize>().unwrap())
    } else {
        None
    }
}

pub const TRACEBACK_LINES: usize = 3; // LINES

fn pyerr(
    py: &pyo3::Python,
    file: &str,
    code: &[String],
    line_num: usize,
    err: PyErr,
) -> NotebookError {
    let line = if let Some(traceback) = err.traceback_bound(*py) {
        if let Ok(tb) = traceback.format() {
            get_linenumber(&tb)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(line) = line {
        let start = if line > TRACEBACK_LINES { line - 3 } else { 0 };
        let end = if line + TRACEBACK_LINES < code.len() {
            line + TRACEBACK_LINES
        } else {
            code.len()
        };
        let lines = code[start..end]
            .iter()
            .enumerate()
            .map(|(i, s)| {
                if i + start + 1 == line {
                    CodeLine::new(
                        start + i + line_num + 1,
                        s.to_string(),
                        Some(err.to_string()),
                    )
                } else {
                    CodeLine::new(start + i + line_num + 1, s.to_string(), None)
                }
            })
            .collect::<Vec<CodeLine>>();

        NotebookError::new(
            file.to_string(),
            String::from("Python"),
            String::from("ExecuteError"),
            err.to_string(),
            line + line_num,
            start + line_num,
            Some(Box::new(lines)),
        )
    } else {
        NotebookError::new(
            file.to_string(),
            String::from("Python"),
            String::from("ExecuteError"),
            err.to_string(),
            line_num,
            line_num,
            None,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);

impl CellWrite<PythonCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &Bound<pyo3::types::PyDict>,
        locals: &Bound<pyo3::types::PyDict>,
        cell: &PythonCell,
        input: &str,
        dest: &str,
    ) -> Result<(), NotebookError> {
        let code = &cell.2;
        let args = &cell.1;

        //reset the plot buffer
        let el = py.import_bound("elektron").map_err(|_| {
            NotebookError::new(
                input.to_string(),
                String::from("PythonCell"),
                String::from("ImportError"),
                String::from("can not get elektron module"),
                cell.0,
                cell.0,
                None,
            )
        })?;
        let plots_fn = el.getattr("reset").map_err(|_| {
            NotebookError::new(
                input.to_string(),
                String::from("PythonCell"),
                String::from("AttributeError"),
                String::from("can not get elektron reset function"),
                cell.0,
                cell.0,
                None,
            )
        })?;
        plots_fn.call0().map_err(|_| {
            NotebookError::new(
                input.to_string(),
                String::from("PythonCell"),
                String::from("AttributeError"),
                String::from("can not call elektron reset function"),
                cell.0,
                cell.0,
                None,
            )
        })?;

        trace!("run: {}, {:?}", code.join("\n").as_str(), args);
        echo(out, "python", code.join("\n").as_str(), cell.0, args);

        py.run_bound(code.join("\n").as_str(), Some(globals), Some(locals))
            .map_err(|err| pyerr(py, input, code, cell.0, err))?;

        let sys = py.import_bound("sys").unwrap();
        let resout: LoggingStdout = sys.getattr("stdout").unwrap().extract().unwrap();
        let stdout = newlines(resout.dump());

        let resout: LoggingStderr = sys.getattr("stderr").unwrap().extract().unwrap();
        let errout = newlines(resout.dump());

        if let Some(ArgType::String(result)) = args.get("results") {
            if result != "hide" && !result.is_empty() {
                writeln!(out, "{{{{< result >}}}}").map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
                out.write_all(stdout.as_bytes()).map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
                writeln!(out, "{{{{< /result >}}}}\n").map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
            }
        }

        let plots_fn = el.getattr("plots").unwrap();
        let plots = plots_fn.call0().unwrap();
        if let Ok(plots) = plots.extract::<Vec<Vec<u8>>>() {
            for plot in plots {
                writeln!(
                    out,
                    "{{{{< figure {}>}}}}",
                    args_to_string(&write_plot(dest, plot, args).map_err(|err| {
                        NotebookError::new(
                            input.to_string(),
                            String::from("PythonCell"),
                            String::from("IoError"),
                            err.to_string(),
                            cell.0,
                            cell.0,
                            None,
                        )
                    })?)
                )
                .map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?
            }
        }

        if let Some(ArgType::String(result)) = args.get("error") {
            if result != "hide" && !errout.is_empty() {
                writeln!(out, "{{{{< error message=\"Python stderr:\" >}}}}").map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
                out.write_all(errout.as_bytes()).map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
                writeln!(out, "{{{{< /error >}}}}\n").map_err(|err| {
                    NotebookError::new(
                        input.to_string(),
                        String::from("PythonCell"),
                        String::from("IoError"),
                        err.to_string(),
                        cell.0,
                        cell.0,
                        None,
                    )
                })?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_lienmumber() {
        assert_eq!(
            4,
            super::get_linenumber("SyntaxError: invalid syntax (<string>, line 4)").unwrap()
        );
        assert_eq!(
            231,
            super::get_linenumber(
                r#"Traceback (most recent call last):
  File "<string>", line 231, in <module>"#
            )
            .unwrap()
        );
    }
}
