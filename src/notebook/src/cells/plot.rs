use std::collections::HashMap;

use plotters::prelude::*;
use pyo3::prelude::*;

use crate::cells::{CellWrite, CellWriter};
use crate::error::NotebookError;
use crate::notebook::ArgType;

use super::args_to_string;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<PlotCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        _py: &pyo3::Python,
        _globals: &Bound<pyo3::types::PyDict>,
        locals: &Bound<pyo3::types::PyDict>,
        cell: &PlotCell,
        input: &str,
        _: &str,
    ) -> Result<(), NotebookError> {
        let _body = &cell.2;
        let args = &cell.1;

        let x: Option<Vec<f64>> = if let Some(ArgType::String(x)) = args.get("x") {
            if x.starts_with("py$") {
                let key: &str = x.strip_prefix("py$").unwrap();
                if let Ok(Some(item)) = locals.get_item(key) {
                    if let Ok(var) = item.extract() {
                        Some(var)
                    } else {
                        return Err(NotebookError::new(
                            input.to_string(),
                            String::from("PlotCell"),
                            String::from("VariableError"),
                            format!("Variable with name '{}' can not be found.", key),
                            cell.0,
                            cell.0,
                            None,
                        ));
                    }
                } else {
                    return Err(NotebookError::new(
                        input.to_string(),
                        String::from("PlotCell"),
                        String::from("VariableError"),
                        format!("Variable with name '{}' can not be found.", key),
                        cell.0,
                        cell.0,
                        None,
                    ));
                }
            } else {
                None
            }
        } else {
            None
        };
        let y: Option<Vec<Vec<f64>>> = if let Some(ArgType::List(list)) = args.get("y") {
            let mut result: Vec<Vec<f64>> = Vec::new();
            for x in list {
                if x.starts_with("py$") {
                    let key: &str = x.strip_prefix("py$").unwrap();
                    if let Ok(Some(item)) = locals.get_item(key) {
                        if let Ok(var) = item.extract() {
                            result.push(var)
                        } else {
                            return Err(NotebookError::new(
                                input.to_string(),
                                String::from("PlotCell"),
                                String::from("VariableError"),
                                format!("Variable cast error '{}'.", key),
                                cell.0,
                                cell.0,
                                None,
                            ));
                        }
                    } else {
                        return Err(NotebookError::new(
                            input.to_string(),
                            String::from("PlotCell"),
                            String::from("VariableError"),
                            format!("Variable with name '{}' can not be found.", key),
                            cell.0,
                            cell.0,
                            None,
                        ));
                    }
                }
            }
            Some(result)
        } else {
            None
        };
        if let (Some(x), Some(y)) = (x, y) {
            let buffer = plot(x, y);
            writeln!(out, "{{{{< figure {}>}}}}", args_to_string(args)).unwrap();
            out.write_all(buffer.as_bytes()).unwrap();
            writeln!(out, "{{{{< /figure >}}}}").unwrap();
            Ok(())
        } else {
            Err(NotebookError::new(
                input.to_string(),
                String::from("PlotCell"),
                String::from("ValueError"),
                String::from("x and y property not set."),
                cell.0,
                cell.0,
                None,
            ))
        }

        /* match parse_variables(&body.join("\n"), &py, globals, locals) {
            Ok(code) => {
                writeln!(out, "{{{{< javascript }}}}").unwrap();
                out.write_all(code.as_bytes()).unwrap();
                writeln!(out, "{{{{< /javascript >}}}}").unwrap();
                Ok(())
            }
            Err(err) => Err(Error::VariableNotFound(err.to_string()))
        } */
    }
}

fn plot(x: Vec<f64>, y: Vec<Vec<f64>>) -> String {
    let mut buffer = String::new();
    {
        // get the min and max values of the dataset.
        let max_x = x.iter().copied().fold(f64::NAN, f64::max);
        let min_x = x.iter().copied().fold(f64::NAN, f64::min);
        let mut max_y = 0.0;
        let mut min_y = 0.0;
        for inner in &y {
            let inner_max_y = inner.iter().copied().fold(f64::NAN, f64::max);
            if max_y < inner_max_y {
                max_y = inner_max_y;
            }
            let inner_min_y = inner.iter().copied().fold(f64::NAN, f64::min);
            if min_y > inner_min_y {
                min_y = inner_min_y;
            }
        }

        let root = SVGBackend::with_string(&mut buffer, (640, 480)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(min_x..max_x, min_y..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        for y in y {
            chart
                .draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    RED,
                ))
                .unwrap()
                .label("y = x^2");
        }

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        root.present().unwrap();
    }
    buffer
}
