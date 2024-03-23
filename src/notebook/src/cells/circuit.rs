use pyo3::IntoPy;
use std::collections::HashMap;

use crate::draw::{parse, Draw};
use crate::reports::erc::{erc, ErcItem};
use crate::spice;
use elektron_ngspice::{ComplexSlice, NgSpice};
use spice::Cb;

use super::super::cells::{param, params, CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::Error;

use crate::plot::{PlotOptions, PlotSelector, Plotter};

use super::args_to_string;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircuitCell(pub HashMap<String, ArgType>, pub Vec<String>);

impl CellWrite<CircuitCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        _locals: &pyo3::types::PyDict,
        cell: &CircuitCell,
        _path: &str,
    ) -> Result<(), Error> {
        let body = &cell.1;
        let args = &cell.0;

        let key = param!(
            args,
            "id",
            Error::PropertyNotFound(String::from("property 'key' must be set."))
        );
        let spice = if let Ok(spice) = std::env::var("ELEKTRON_SPICE") {
            let res: Vec<String> = spice.split(':').map(|s| s.to_string()).collect();
            res.to_vec()
        } else {
            params!(
                args,
                "spice",
                Error::PropertyNotFound(String::from("property 'spice' must be set."))
            )
            .clone()
        };
        let symbols = if let Ok(symbols) = std::env::var("ELEKTRON_SYMBOLS") {
            let res: Vec<String> = symbols.split(':').map(|s| s.to_string()).collect();
            res.to_vec()
        } else {
            params!(
                args,
                "symbols",
                Error::PropertyNotFound(String::from("property 'symbols' must be set."))
            )
            .clone()
        };
        let command = if let Some(ArgType::String(key)) = args.get("command") {
            vec![key.clone()]
        } else if let Some(ArgType::List(keys)) = args.get("output") {
            keys.to_vec()
        } else {
            Vec::new()
        };

        let result = parse(body.join("\n")).unwrap();
        let mut draw = Draw::new(symbols);
        draw.build(&result).unwrap();

        if command.contains(&String::from("figure")) {
            //TODO: make scale and theme and netlist configurable
            // let svg = plot::SvgPlotter::new(&draw.schema, key, None);
            let mut buffer = Vec::<u8>::new();
            // svg.plot(&mut buffer, false, 5.0, None, true).unwrap();
            Plotter::svg(
                PlotOptions::new(&draw.schema, &mut buffer)
                    .id(key)
                    .border(false)
                    .scale(5.0),
            )?;

            writeln!(out, "{{{{< figure {}>}}}}", args_to_string(args)).unwrap();
            out.write_all(std::str::from_utf8(&buffer).unwrap().as_bytes())
                .unwrap();
            writeln!(out, "{{{{< /figure >}}}}").unwrap();
        }

        let circuit = draw.circuit(spice)?;
        if command.contains(&String::from("netlist")) {
            writeln!(out, "```netlist").unwrap();
            out.write_all(circuit.to_str(true).unwrap().join("\n").as_bytes())
                .unwrap();
            writeln!(out, "\n```").unwrap();
        }

        if command.contains(&String::from("erc")) {
            let erc = erc(&draw.schema).unwrap();
            writeln!(out, "|Reference|Description|Position|").unwrap();
            writeln!(out, "|---------|-----------|--------|").unwrap();
            for item in erc {
                match item {
                    ErcItem::NoReference { reference, at } => {
                        writeln!(out, "|{}|No Reference|{}|", reference, at).unwrap();
                    },
                   ErcItem::ValuesDiffer { reference, at } => {
                        writeln!(out, "|{}|Different values for Symbol unit.|{}|", reference, at).unwrap();
                    },
                    ErcItem::Netlist(err) => {
                        writeln!(out, "| |{}| |", err).unwrap();
                    },
                    ErcItem::NotAllParts { reference, at } => {
                        writeln!(out, "|{}|Not all symbol units on schema.{}| |", reference, at).unwrap();
                    },
                    ErcItem::PinNotConnected { reference, at } => {
                        writeln!(out, "|{}|Pin not connected|{}|", reference, at).unwrap();
                    },
                }

            }
            writeln!(out, "\n").unwrap();
        }

        let mut cb = Cb::new();
        let ng = NgSpice::new(&mut cb)?;
        ng.circuit(circuit.to_str(true).unwrap())?;
        for c in &circuit.controls {
            ng.command(c).unwrap();
        }
        let mut plot_result: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        for plot in ng.all_plots()? {
            let vecs = ng.all_vecs(&plot)?;
            let mut vec_values: HashMap<String, Vec<f64>> = HashMap::new();
            for v in vecs {
                let vals = ng.vector_info(format!("{}.{}", plot, &v).as_str())?;
                let data1 = match vals.data {
                    ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                    ComplexSlice::Complex(list) => list
                        .iter()
                        .map(|f| {
                            if !f.cx_real.is_nan() {
                                f.cx_real
                            } else if !f.cx_imag.is_nan() {
                                f.cx_imag
                            } else {
                                todo!("can not get value from complex: {:?}", f);
                            }
                        })
                        .collect(),
                };
                vec_values.insert(v, data1);
            }
            plot_result.insert(plot, vec_values);
        }

        globals.set_item(key, plot_result.into_py(*py)).unwrap();
        Ok(())
    }
}
