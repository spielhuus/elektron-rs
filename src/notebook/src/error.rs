use std::fmt;
use colored::Colorize;

use crate::CodeLine;

#[derive(Debug, Clone)]
pub struct ValueError(pub String);

#[derive(Debug, Clone)]
pub struct NotebookError {
    pub filename: String,
    pub cell: String,
    pub title: String,
    pub message: String,
    pub line: usize,
    pub start_line: usize,
    pub code: Option<Box<Vec<CodeLine>>>,
}

impl NotebookError {
    pub fn new(
        filename: String,
        cell: String,
        title: String,
        message: String,
        line: usize,
        start_line: usize,
        code: Option<Box<Vec<CodeLine>>>,
    ) -> Self {
        Self {
            filename,
            cell,
            title,
            message,
            line,
            start_line,
            code,
        }
    }
}

impl std::error::Error for NotebookError {}

impl fmt::Display for NotebookError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lines = self.code.as_ref().map(|code| {
            code.iter()
                .map(|line| {
                    if let Some(annotation) = &line.annotation {
                        format!("{}\t = {}\t{}", line.line, line.code, annotation.red()).bold().to_string()
                    } else {
                        format!("{}\t | {}", line.line, line.code)
                    }
                })
                .collect::<Vec<String>>()
        });

        if let Some(lines) = lines {
            write!(
                f,
                "Python: {}:{}\n{}",
                self.filename,
                self.line,
                lines.join("\n")
            )
        } else {
            write!(
                f,
                "{}::{}::{} in {}::{}",
                self.cell, self.title, self.message, self.filename, self.line
            )
        }
    }
}
