use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::reports::bom::BomItem;
use crate::reports::drc::drc;
use crate::reports::erc::{ErcItem, erc};
use crate::sexp::{Pcb, Schema};

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::param_or;
use crate::error::Error;

use crate::reports;
use crate::plot::{PlotOptions, PlotSelector, Plotter};

fn check_directory(filename: &str) {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
}

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
                if let Some(ArgType::List(input)) = args.get("input") {
                    let mut missing: Vec<BomItem> = Vec::new();
                    writeln!(out, "bom:").unwrap();
                    for input in input {
                        let input_file = Path::new(&source)
                            .join(input)
                            .join(format!("{}.kicad_sch", input))
                            .to_str()
                            .unwrap()
                            .to_string();
                        let schema = Schema::load(input_file.as_str())?;
                        let res = reports::bom::bom(&schema, group, partlist.clone());

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
                            },
                            Err(err) => {
                                panic!("can not create bom: {:?}", err);
                            }
                        }
                    }
                    writeln!(out, "bom_missing:").unwrap();
                    writeln!(out, "  items:").unwrap();
                    let mut count = 0;
                    for item in missing {
                        writeln!(out, "    -").unwrap();
                        writeln!(out, "       amount: {}", item.amount).unwrap();
                        writeln!(out, "       value: {}", item.value).unwrap();
                        writeln!(out, "       references: {}", item.references.join(" "))
                            .unwrap();
                        writeln!(out, "       description: {}", item.description).unwrap();
                        writeln!(out, "       footprint: {}", item.footprint).unwrap();
                        count += 1;
                    }
                    writeln!(out, "  count: {}", count).unwrap();
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
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
                        let res = drc(input_file, None).unwrap();
                        if !res.is_empty() {
                            //output the erc as frontmatter
                            writeln!(out, "  {}:", input).unwrap();
                            for item in res {
                                count += 1;
                                writeln!(out, "    -").unwrap();
                                writeln!(out, "       id: {}", item.id.to_string()).unwrap();
                                writeln!(out, "       severity: {}", item.severity.to_string()).unwrap();
                                writeln!(out, "       title: {}", item.title.to_string()).unwrap();
                                writeln!(out, "       description: {}", item.description.to_string()).unwrap();
                                writeln!(out, "       pos:").unwrap();
                                writeln!(out, "       -").unwrap();
                                for pos in item.position {
                                    writeln!(out, "         x: {}", pos.0).unwrap();
                                    writeln!(out, "         y: {}", pos.1).unwrap();
                                    writeln!(out, "         reference: {}", pos.2).unwrap();
                                }
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
                        let schema = Schema::load(input_file.as_str())?;
                        let res = erc(&schema).unwrap();
                        count += res.len();
                        if !res.is_empty() {
                            //output the erc as frontmatter
                            writeln!(out, "  {}:", input).unwrap();
                            for item in res {
                                match item {
                                    ErcItem::NoReference { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(out, "       description: No Reference for symbol.").unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    },
                                    ErcItem::ValuesDiffer { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(out, "       description: Values for Symbol units differ.").unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    },
                                    ErcItem::Netlist(err) => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       description: Unable to build netlist: {}", err).unwrap();
                                    },
                                    ErcItem::NotAllParts { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(out, "       description: No all Symbol units on schema.").unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    },
                                    ErcItem::PinNotConnected { reference, at } => {
                                        writeln!(out, "    -").unwrap();
                                        writeln!(out, "       reference: {}", reference).unwrap();
                                        writeln!(out, "       description: Pin not connected.").unwrap();
                                        writeln!(out, "       at: {}:{}", at[0], at[1]).unwrap();
                                    },
                                }
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
                let border = if let Some(ArgType::String(group)) = args.get("border") {
                    group == "TRUE" || group == "true"
                } else {
                    false
                };
                if let Some(ArgType::List(input)) = args.get("input") {
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

                        let schema = Schema::load(input_file.as_str())?;
                        check_directory(&output_file);
                        let mut buffer = File::create(&output_file)?;
                        Plotter::svg(
                            PlotOptions::new(&schema, &mut buffer)
                                .id(input)
                                .border(border)
                                .theme(param_or!(args, "theme", "").into()),
                        )?;
                        writeln!(out, "  {}: {}", input, output_file).unwrap();
                    }
                    Ok(())
                } else {
                    Err(Error::NoInputFile())
                }
            } else if command == "pcb" {
                let out_dir = Path::new(dest).join("_files");
                let border = if let Some(ArgType::String(group)) = args.get("border") {
                    group == "TRUE" || group == "true"
                } else {
                    false
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

                        let schema = Pcb::load(input_file.as_str())?;
                        /* TODO: plot::plot_pcb(
                            &schema,
                            output_file.as_str(),
                            1.0,
                            border,
                            "kicad_2000",
                        )
                        .unwrap(); */
                        check_directory(&output_file);
                        let mut buffer = File::create(&output_file)?;
                        Plotter::svg(
                            PlotOptions::new(&schema, &mut buffer)
                                .id(input)
                                .border(border)
                                .theme(param_or!(args, "theme", "").into()),
                        )?;
                        writeln!(out, "  {}: {}", input, output_file).unwrap();
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

                        let pcb = crate::gerber::Pcb::new(input_file.to_string());
                        pcb.gerber(output_file.to_string());
                        //check_directory(&output_file);
                        writeln!(out, "  {}: {}", input, output_file).unwrap();
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
