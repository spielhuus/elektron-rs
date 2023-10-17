use std::collections::HashMap;

pub use tectonic::driver;
pub use tectonic::errors;
pub use tectonic::status;

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::super::runner::LoggingStderr;
use super::super::runner::LoggingStdout;
use super::super::utils::newlines;
use super::write_plot;
use crate::error::Error;

use super::args_to_string;
use super::echo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<PythonCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &PythonCell,
        _: &str,
        dest: &str,
    ) -> Result<(), Error> {
        let code = &cell.1;
        let args = &cell.0;

        //reset the plot buffer
        let el = py.import("elektron").unwrap();
        let plots_fn = el.getattr("reset").unwrap();
        plots_fn.call0().unwrap();

        echo(out, "python", code.join("\n").as_str(), args);
        if let Err(pyerror) = py.run(code.join("\n").as_str(), Some(globals), Some(locals)) {
            Err(Error::Python(pyerror.to_string()))
        } else {
            let sys = py.import("sys").unwrap();
            let resout: LoggingStdout = sys.getattr("stdout").unwrap().extract().unwrap();
            let stdout = newlines(resout.dump());

            let resout: LoggingStderr = sys.getattr("stderr").unwrap().extract().unwrap();
            let errout = newlines(resout.dump());

            if let Some(ArgType::String(result)) = args.get("results") {
                if result != "hide" && !result.is_empty() {
                    writeln!(out, "{{{{< result >}}}}").unwrap();
                    out.write_all(stdout.as_bytes()).unwrap();
                    writeln!(out, "{{{{< /result >}}}}\n").unwrap();
                }
            }

            let plots_fn = el.getattr("plots").unwrap();
            let plots = plots_fn.call0().unwrap();
            if let Ok(plots) = plots.extract::<Vec<Vec<u8>>>() {
                for plot in plots {
                    writeln!(
                        out,
                        "{{{{< figure {}>}}}}",
                        args_to_string(&write_plot(dest, plot, args).unwrap())
                    )
                    .unwrap();
                }
            }

            if let Some(ArgType::String(result)) = args.get("error") {
                if result != "hide" && !errout.is_empty() {
                    writeln!(out, "{{{{< error message=\"Python stderr:\" >}}}}").unwrap();
                    out.write_all(errout.as_bytes()).unwrap();
                    writeln!(out, "{{{{< /error >}}}}\n").unwrap();
                }
            }
            Ok(())
        }
    }
}
