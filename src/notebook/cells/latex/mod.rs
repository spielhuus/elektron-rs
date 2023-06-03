use std::collections::HashMap;
use std::fmt::Arguments;

use cairo::{Context, SvgSurface};
use tectonic::config::PersistentConfig;
pub use tectonic::driver;
pub use tectonic::engines::bibtex::BibtexEngine;
pub use tectonic::engines::spx2html::Spx2HtmlEngine;
pub use tectonic::engines::tex::{TexEngine, TexOutcome};
pub use tectonic::engines::xdvipdfmx::XdvipdfmxEngine;
pub use tectonic::errors;
pub use tectonic::status;
use tectonic::status::{MessageKind, StatusBackend};

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::{Error, echo, write_plot};

use super::args_to_string;

struct BufferStatusBackend {
    pub messages: Vec<String>,
}

impl StatusBackend for BufferStatusBackend {
    fn report(&mut self, kind: MessageKind, args: Arguments, err: Option<&anyhow::Error>) {
        let prefix = match kind {
            MessageKind::Note => "note:",
            MessageKind::Warning => "warning:",
            MessageKind::Error => "error:",
        };

        self.messages.push(format!("{}:{}", prefix, args));
        if let Some(e) = err {
            for item in e.chain() {
                self.messages.push(format!("caused by: {}", item));
            }
        }
    }

    fn report_error(&mut self, err: &anyhow::Error) {
        let mut prefix = "error";
        for item in err.chain() {
            self.messages.push(format!("{}: {}", prefix, item));
            prefix = "caused by";
        }
    }

    fn note_highlighted(&mut self, _before: &str, _highlighted: &str, _after: &str) { }

    fn dump_error_logs(&mut self, _: &[u8]) {  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TikzCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<TikzCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        _: &pyo3::Python,
        _: &pyo3::types::PyDict,
        _: &pyo3::types::PyDict,
        cell: &TikzCell,
        _: &str,
        dest: &str,
    ) -> Result<(), Error> {
        let body = &cell.1;
        let args = &cell.0;

        let pdf_data = latex_to_pdf(body.join("\n"));
        echo(out, "latex", body.join("\n").as_str(), args);
        if let Err(pdf_err) = pdf_data {
            return Err(Error::Latex(pdf_err.to_string()));
        } else if let Ok(pdf_data) = pdf_data {
            unsafe {
                let document = poppler::Document::from_data(&pdf_data, None).unwrap();
                let mut buffer: Vec<u8> = Vec::new();
                let page = document.page(0).unwrap();
                let cairo =
                    SvgSurface::for_raw_stream(page.size().0, page.size().1, &mut buffer).unwrap();
                let context = Context::new(&cairo).unwrap();
                document.page(0).unwrap().render(&context);
                cairo.finish_output_stream().unwrap();

                writeln!(out, "{{{{< figure {}>}}}}", args_to_string(&write_plot(dest, buffer, args).unwrap())).unwrap();
            }
        }
        Ok(())
    }
}

pub fn latex_to_pdf<T: AsRef<str>>(latex: T) -> Result<Vec<u8>, Error> {
    let mut status = Box::new(BufferStatusBackend {
        messages: Vec::new(),
    }); // as Box<dyn StatusBackend>;

    let auto_create_config_file = false;
    let Ok(config) = PersistentConfig::open(auto_create_config_file) else {
        return Err(Error::Latex(String::from("failed to open the default configuration file")));
    };

    let only_cached = false;
    let Ok(bundle) = config.default_bundle(only_cached, &mut *status) else {
        return Err(Error::Latex(String::from("failed to load the default resource bundle")));
    };

    let Ok(format_cache_path) = config.format_cache_path() else {
        return Err(Error::Latex(String::from("failed to set up the format cache")));
    };

    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sb = driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer(latex.as_ref().as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(driver::OutputFormat::Pdf)
            .do_not_write_output_files();

        let Ok(mut sess) = sb.create(&mut *status) else {
            return Err(Error::Latex(String::from("failed to initialize the LaTeX processing session")));
        };
        // ctry!(sess.run(&mut *status); "the LaTeX engine failed");
        if sess.run(&mut *status).is_err() {
            return Err(Error::Latex(status.messages.join("\n")));
        }
        sess.into_file_data()
    };

    match files.remove("texput.pdf") {
        Some(file) => Ok(file.data),
        None => Err(Error::Latex(String::from("LaTeX didn't report failure, but no PDF was created (??)"))),
    }
}
