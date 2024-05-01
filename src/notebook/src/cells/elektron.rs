use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::{error::NotebookError, notebook::ArgType, utils::check_directory};

use pyo3::Bound;
use reports::{bom::BomItem, drc, erc};

use plotter::{gerber, pcb::LAYERS, schema::SchemaPlot, svg::SvgPlotter};
use sexp::{SexpParser, SexpTree};

use super::super::cells::{CellWrite, CellWriter};
use super::param_or;

use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElektronCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<ElektronCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        _py: &pyo3::Python,
        _globals: &Bound<pyo3::types::PyDict>,
        _locals: &Bound<pyo3::types::PyDict>,
        cell: &ElektronCell,
        source: &str,
        dest: &str,
    ) -> Result<(), NotebookError> {
        let args = &cell.1;

        let source_path = Path::new(&source)
            .parent()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

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
                    return Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ));
                };
                let mut missing: Vec<BomItem> = Vec::new();
                writeln!(out, "bom:").unwrap();
                for input in input {
                    let input_file = Path::new(&source_path)
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
                            return Err(NotebookError::new(
                                source.to_string(),
                                String::from("ElektronCell"),
                                String::from("IoError"),
                                format!("can not create bom: {:?}", err),
                                cell.0,
                                cell.0,
                                None,
                            ));
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
                        let input_file = Path::new(&source_path)
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
                    Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ))
                }
            } else if command == "erc" {
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "erc:").unwrap();
                    let mut count = 0;
                    for input in input {
                        let input_file = Path::new(&source_path)
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
                    Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ))
                }
            } else if command == "schema" {
                let out_dir = Path::new(dest).join("_files");
                let Some(ArgType::List(input)) = args.get("input") else {
                    return Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ));
                };
                writeln!(out, "schema:").unwrap();
                for input in input {
                    let input_file = Path::new(&source_path)
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

                    check_directory(&output_file).map_err(|err| {
                        NotebookError::new(
                            source.to_string(),
                            String::from("ElektronCell"),
                            String::from("IOError"),
                            err.to_string(),
                            cell.0,
                            cell.0,
                            None,
                        )
                    })?;

                    let mut plotter = SchemaPlot::new()
                        .border(super::flag!(args, "border", false))
                        .theme(param_or!(args, "theme", "").into())
                        .scale(str::parse::<f64>(param_or!(args, "scale", "1.0")).unwrap())
                        .name(input);

                    plotter.open(&input_file).map_err(|err| {
                        NotebookError::new(
                            source.to_string(),
                            String::from("ElektronCell"),
                            String::from("IOError"),
                            err.to_string(),
                            cell.0,
                            cell.0,
                            None,
                        )
                    })?;
                    for page in plotter.iter() {
                        let mut file = if *page.0 == 1 {
                            debug!("write first page to {}", output_file);
                            writeln!(out, "  {}: {}", input, output_file).unwrap();
                            File::create(output_file.clone()).map_err(|err| {
                                NotebookError::new(
                                    source.to_string(),
                                    String::from("ElektronCell"),
                                    String::from("IOError"),
                                    err.to_string(),
                                    cell.0,
                                    cell.0,
                                    None,
                                )
                            })?
                        } else {
                            let output_file = out_dir
                                .join(format!("{}_schema.svg", page.1))
                                .to_str()
                                .unwrap()
                                .to_string();
                            debug!("write page {} to {}", page.1, format!("{}.svg", page.1));
                            writeln!(out, "  {}: {}", page.1, output_file).unwrap();
                            File::create(output_file).map_err(|err| {
                                NotebookError::new(
                                    source.to_string(),
                                    String::from("ElektronCell"),
                                    String::from("IOError"),
                                    err.to_string(),
                                    cell.0,
                                    cell.0,
                                    None,
                                )
                            })?
                        };
                        let mut svg_plotter = SvgPlotter::new(&mut file);
                        plotter.write(page.0, &mut svg_plotter).map_err(|err| {
                            NotebookError::new(
                                source.to_string(),
                                String::from("ElektronCell"),
                                String::from("IOError"),
                                err.to_string(),
                                cell.0,
                                cell.0,
                                None,
                            )
                        })?;
                    }
                }
                Ok(())
            } else if command == "pcb" {
                let out_dir = Path::new(dest).join("_files");
                let layers = if let Some(ArgType::List(layers)) = args.get("layers") {
                    Some(layers.clone())
                } else {
                    None
                };
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "pcb:").unwrap();

                    for input in input {
                        let input_file = Path::new(&source_path)
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

                        plotter::plot(
                            &input_file,
                            &output_file,
                            super::flag!(args, "border", false),
                            param_or!(args, "theme", "").into(),
                            str::parse::<f64>(param_or!(args, "scale", "1.0")).unwrap(),
                            None,
                            layers.clone(),
                        )
                        .map_err(|err| {
                            NotebookError::new(
                                source.to_string(),
                                String::from("ElektronCell"),
                                String::from("PlotPCB"),
                                err.to_string(),
                                cell.0,
                                cell.0,
                                None,
                            )
                        })?;

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
                        //TODO writeln!(out, "    width: {}", size.0).unwrap();
                        //writeln!(out, "    height: {}", size.1).unwrap();
                        writeln!(out, "    layers:").unwrap();
                        if let Some(layers) = &layers {
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
                    Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ))
                }
            } else if command == "gerber" {
                let out_dir = Path::new(dest).join("_files");
                if let Some(ArgType::List(input)) = args.get("input") {
                    writeln!(out, "gerber:").unwrap();
                    for input in input {
                        let input_file = Path::new(&source_path)
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
                            check_directory(&output_file).map_err(|err| {
                                NotebookError::new(
                                    source.to_string(),
                                    String::from("ElektronCell"),
                                    String::from("IOError"),
                                    err.to_string(),
                                    cell.0,
                                    cell.0,
                                    None,
                                )
                            })?;
                            writeln!(out, "  {}: {}", input, output_file).unwrap();
                        }
                    }
                    Ok(())
                } else {
                    Err(NotebookError::new(
                        source.to_string(),
                        String::from("ElektronCell"),
                        String::from("VariableError"),
                        String::from("input not found."),
                        cell.0,
                        cell.0,
                        None,
                    ))
                }
            } else {
                Err(NotebookError::new(
                    source.to_string(),
                    String::from("ElektronCell"),
                    String::from("UnknownCommand"),
                    command.to_string(),
                    cell.0,
                    cell.0,
                    None,
                ))
            }
        } else {
            Err(NotebookError::new(
                source.to_string(),
                String::from("ElektronCell"),
                String::from("NoCommand"),
                String::from("command not found."),
                cell.0,
                cell.0,
                None,
            ))
        }
    }
}
