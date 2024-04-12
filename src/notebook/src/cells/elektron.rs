use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::{notebook::ArgType, utils::check_directory};

use reports::{bom::BomItem, drc, erc};

use plotter::{
    gerber, pcb::{plot_pcb, LAYERS}, schema::SchemaPlot, svg::SvgPlotter
};
use sexp::{SexpParser, SexpTree};

use super::super::cells::{CellWrite, CellWriter};
use super::param_or;
use crate::error::Error;

use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElektronCell(pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<ElektronCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        _py: &pyo3::Python,
        _globals: &pyo3::types::PyDict,
        _locals: &pyo3::types::PyDict,
        cell: &ElektronCell,
        source: &str,
        dest: &str,
    ) -> Result<(), Error> {
        let args = &cell.0;

        if let Some(ArgType::String(command)) = args.get("command") {
            if command == "bom" {
                let group = if let Some(ArgType::String(group)) = args.get("group") {
                    group == "TRUE" || group == "true"
                } else {
                    false
                };
                let partlist = if let Some(ArgType::String(group)) = args.get("partlist") {
                    Some(group.to_string())
                } else {
                    None
                };
                let Some(ArgType::List(input)) = args.get("input") else {
                    return Err(Error::NoInputFile());
                };
                let mut missing: Vec<BomItem> = Vec::new();
                writeln!(out, "bom:").unwrap();
                for input in input {
                    let input_file = Path::new(&source)
                        .join(input)
                        .join(format!("{}.kicad_sch", input))
                        .to_str()
                        .unwrap()
                        .to_string();

                    let doc = SexpParser::load(input_file.as_str()).unwrap();
                    let tree = SexpTree::from(doc.iter()).unwrap();
                    let res = reports::bom::bom(&tree, group, partlist.clone());

                    match res {
                        Ok(res) => {
                            if let Some(mut m) = res.1 {
                                missing.append(&mut m);
                            }
                            //output the bom as frontmatter
                            writeln!(out, "  {}:", input).unwrap();
                            for item in res.0 {
                                writeln!(out, "    -").unwrap();
                                writeln!(out, "       amount: {}", item.amount).unwrap();
                                writeln!(out, "       value: {}", item.value).unwrap();
                                writeln!(out, "       references: {}", item.references.join(" "))
                                    .unwrap();
                                writeln!(out, "       description: {}", item.description).unwrap();
                                writeln!(out, "       footprint: {}", item.footprint).unwrap();
                            }
                        }
                        Err(err) => {
                            return Err(Error::IoError(format!("can not create bom: {:?}", err)));
                        }
                    }
                }
                writeln!(out, "bom_missing:").unwrap();
                writeln!(out, "  items:").unwrap();
                for item in &missing {
                    writeln!(out, "    -").unwrap();
                    writeln!(out, "       amount: {}", item.amount).unwrap();
                    writeln!(out, "       value: {}", item.value).unwrap();
                    writeln!(out, "       references: {}", item.references.join(" ")).unwrap();
                    writeln!(out, "       description: {}", item.description).unwrap();
                    writeln!(out, "       footprint: {}", item.footprint).unwrap();
                }
                writeln!(out, "  count: {}", missing.len()).unwrap();
                Ok(())
            } else if command == "drc" {
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "drc:").unwrap();
                    let mut count = 0;
                    for input in input {
                        let input_file = Path::new(&source)
                            .join(input)
                            .join(format!("{}.kicad_pcb", input))
                            .to_str()
                            .unwrap()
                            .to_string();

                        let res = match drc::drc(input_file) {
                            Ok(res) => res,
                            Err(message) => todo!("{}", message),
                        };
                        //output the erc as frontmatter
                        writeln!(out, "  {}:", input).unwrap();
                        for item in res.errors {
                            count += 1;
                            writeln!(out, "    -").unwrap();
                            writeln!(out, "       type: {}", item.error_type).unwrap();
                            writeln!(out, "       severity: {}", item.severity).unwrap();
                            writeln!(out, "       description: {}", item.description).unwrap();
                            writeln!(out, "       items:").unwrap();
                            for i in item.items {
                                writeln!(out, "       -").unwrap();
                                writeln!(out, "         description: {}", i.description).unwrap();
                                writeln!(out, "         pos_x: {}", i.pos.0).unwrap();
                                writeln!(out, "         pos_y: {}", i.pos.1).unwrap();
                            }
                        }
                    }
                    writeln!(out, "  count: {}", count).unwrap();
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
            } else if command == "erc" {
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "erc:").unwrap();
                    let mut count = 0;
                    for input in input {
                        let input_file = Path::new(&source)
                            .join(input)
                            .join(format!("{}.kicad_sch", input))
                            .to_str()
                            .unwrap()
                            .to_string();

                        let res = erc::erc(input_file.as_str()).unwrap();
                        count += res.len();
                        if !res.is_empty() {
                            //output the erc as frontmatter
                            writeln!(out, "  {}:", input).unwrap();
                            for item in res {
                                writeln!(out, "    -").unwrap();
                                writeln!(out, "       id: {}", item.id).unwrap();
                                writeln!(out, "       reference: {}", item.reference).unwrap();
                                writeln!(out, "       description: {}.", item.description).unwrap();
                                writeln!(out, "       at: {}:{}", item.at[0], item.at[1]).unwrap();

                                /* match item {
                                    ErcItem::NoReference { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(
                                            out,
                                            "       description: No Reference for symbol."
                                        )
                                        .unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    }
                                    ErcItem::ValuesDiffer { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(
                                            out,
                                            "       description: Values for Symbol units differ."
                                        )
                                        .unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    }
                                    ErcItem::Netlist(err) => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(
                                            out,
                                            "       description: Unable to build netlist: {}",
                                            err
                                        )
                                        .unwrap();
                                    }
                                    ErcItem::NotAllParts { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(
                                            out,
                                            "       description: No all Symbol units on schema."
                                        )
                                        .unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    }
                                    ErcItem::PinNotConnected { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(out, "       description: Pin not connected.")
                                            .unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    }
                                } */
                            }
                        }
                    }
                    writeln!(out, "  count: {}", count).unwrap();
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
            } else if command == "schema" {
                let out_dir = Path::new(dest).join("_files");
                let Some(ArgType::List(input)) = args.get("input") else {
                    return Err(Error::NoInputFile());
                };
                writeln!(out, "schema:").unwrap();
                for input in input {
                    let input_file = Path::new(&source)
                        .join(input)
                        .join(format!("{}.kicad_sch", input))
                        .to_str()
                        .unwrap()
                        .to_string();
                    let output_file = out_dir
                        .join(format!("{}_schema.svg", input))
                        .to_str()
                        .unwrap()
                        .to_string();

                    debug!(
                        "write schema '{}' to '{}'",
                        input,
                        out_dir
                            .join(format!("{}_schema.svg", input))
                            .to_str()
                            .unwrap()
                            .to_string()
                    );

                    check_directory(&output_file)?;

                    let mut plotter = SchemaPlot::new()
                        .border(super::flag!(args, "border", false))
                        .theme(param_or!(args, "theme", "").into())
                        .scale(str::parse::<f64>(param_or!(args, "scale", "1.0")).unwrap())
                        .name(input);

                    plotter.open(&input_file)?;
                    for page in plotter.iter() {
                        let mut file = BufWriter::new(File::create(output_file.clone())?);
                        let mut svg_plotter = SvgPlotter::new(&mut file);
                        plotter.write(page.0, &mut svg_plotter).unwrap();
                    }

                    writeln!(out, "  {}: {}", input, output_file).unwrap();
                }
                Ok(())
            } else if command == "pcb" {
                let out_dir = Path::new(dest).join("_files");
                let layers = if let Some(ArgType::List(layers)) = args.get("layers") {
                    Some(layers)
                } else {
                    None
                };
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "pcb:").unwrap();

                    for input in input {
                        let input_file = Path::new(&source)
                            .join(input)
                            .join(format!("{}.kicad_pcb", input))
                            .to_str()
                            .unwrap()
                            .to_string();
                        let output_file = out_dir
                            .join(format!("{}_pcb.svg", input))
                            .to_str()
                            .unwrap()
                            .to_string();

                        let size = plot_pcb(
                            input_file.to_string(),
                            output_file.to_string(),
                            layers,
                            None,
                        )?;

                        debug!(
                            "write pcb '{}' to '{}'",
                            input,
                            out_dir
                                .join(format!("{}_pcb.svg", input))
                                .to_str()
                                .unwrap()
                                .to_string()
                        );

                        writeln!(out, "  -").unwrap();
                        writeln!(out, "    name: {}", input).unwrap();
                        writeln!(out, "    file: {}", output_file).unwrap();
                        writeln!(out, "    width: {}", size.0).unwrap();
                        writeln!(out, "    height: {}", size.1).unwrap();
                        writeln!(out, "    layers:").unwrap();
                        if let Some(layers) = layers {
                            for layer in layers {
                                writeln!(out, "    - {}", layer).unwrap();
                            }
                        } else {
                            for layer in LAYERS {
                                writeln!(out, "    - {}", layer).unwrap();
                            }
                        }
                    }
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
            } else if command == "gerber" {
                let out_dir = Path::new(dest).join("_files");
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "gerber:").unwrap();
                    for input in input {
                        let input_file = Path::new(&source)
                            .join(input)
                            .join(format!("{}.kicad_pcb", input))
                            .to_str()
                            .unwrap()
                            .to_string();
                        let output_file = out_dir
                            .join(format!("{}_gerber.zip", input))
                            .to_str()
                            .unwrap()
                            .to_string();

                        debug!("write gerber '{}' to '{}'", input_file, output_file);
                        if let Err(err) =
                            gerber::gerber(input_file.to_string(), output_file.to_string())
                        {
                            println!("gerber error {}", err); //TODO add to notebook
                        } else {
                            check_directory(&output_file)?;
                            writeln!(out, "  {}: {}", input, output_file).unwrap();
                        }
                    }
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
            } else {
                Err(Error::UnknownCommand(command.to_string()))
            }
        } else {
            Err(Error::NoCommand)
        }
    }
}
