use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::check_directory;

use reports::{ 
    drc, erc, bom::BomItem,
};

use sexp::{SexpParser, SexpTree};
use plotter::{
    svg::SvgPlotter,
    themer::Themer,
    PlotterImpl, plot_pcb, gerber,
};

use super::super::cells::{CellWrite, CellWriter};
use super::super::parser::ArgType;
use super::param_or;
use crate::error::Error;

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
                                    writeln!(
                                        out,
                                        "       references: {}",
                                        item.references.join(" ")
                                    )
                                    .unwrap();
                                    writeln!(out, "       description: {}", item.description)
                                        .unwrap();
                                    writeln!(out, "       footprint: {}", item.footprint).unwrap();
                                }
                            }
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
                        writeln!(out, "       references: {}", item.references.join(" ")).unwrap();
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


                        let res = match drc::drc(input_file) {
                            Ok(res) => res,
                            Err(message) => todo!("{}", message),
                        };

                        if !res.is_empty() {
                            //output the erc as frontmatter
                            writeln!(out, "  {}:", input).unwrap();
                            for item in res {
                                count += 1;
                                writeln!(out, "    -").unwrap();
                                writeln!(out, "       id: {}", item.id).unwrap();
                                writeln!(out, "       severity: {}", item.severity)
                                    .unwrap();
                                writeln!(out, "       title: {}", item.title).unwrap();
                                writeln!(
                                    out,
                                    "       description: {}",
                                    item.description
                                )
                                .unwrap();
                                writeln!(out, "       pos:").unwrap();
                                writeln!(out, "       -").unwrap();
                                writeln!(out, "         x: {}", item.position.0).unwrap();
                                writeln!(out, "         y: {}", item.position.1).unwrap();
                                writeln!(out, "         reference: {}", item.position.2).unwrap();
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
                                writeln!(out, "       reference: {}", item.reference).unwrap();
                                writeln!(
                                    out,
                                    "       description: No Reference for symbol."
                                )
                                .unwrap();
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

                        let doc = SexpParser::load(input_file.as_str()).unwrap();
                        let tree = SexpTree::from(doc.iter()).unwrap();

                        let svg_plotter = SvgPlotter::new(
                            input_file.as_str(),
                            Some(Themer::new(param_or!(args, "theme", "").into())),
                        );

                        check_directory(&output_file)?;
                        let mut buffer = File::create(&output_file)?;
                        svg_plotter
                            .plot(&tree, &mut buffer, border, 1.0, None, false) //TODO select pages
                            .unwrap();

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

                        plot_pcb(input_file.to_string(), output_file.to_string())?;
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

                        gerber::gerber(input_file.to_string(), output_file.to_string());
                        check_directory(&output_file)?;
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
