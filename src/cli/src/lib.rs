//! these methods are exposed over the python APi.
extern crate colored;
extern crate comfy_table;
extern crate draw;
extern crate itertools;
extern crate ndarray;
extern crate ngspice;
extern crate notebook;
extern crate plotter;
extern crate pyo3;
extern crate rand;
extern crate reports;
extern crate rust_fuzzy_search;
extern crate sexp;
extern crate sexp_macro;
extern crate simulation;
extern crate tempfile;
extern crate thiserror;

use log::{info, error};

use pyo3::prelude::*;

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};

use itertools::Itertools;
use rust_fuzzy_search::fuzzy_compare;
use tempfile::NamedTempFile;
use clap::{Parser, Subcommand};

use colored::*;

mod error;
mod python;

use crate::error::Error;

use plotter::Theme;

use sexp::{el, SexpParser, SexpTree, State};

use reports::{bom, drc, erc, mouser};
use simulation::{Circuit, Netlist};


mod constant {
    //pub const EXT_KICAD_SCH: &str = ".kicad_sch";
    pub const EXT_JSON: &str = ".json";
    //pub const EXT_SVG: &str = ".svg";
    //pub const EXT_PNG: &str = ".png";
    //pub const EXT_PDF: &str = ".pdf";
    pub const EXT_EXCEL: &str = ".xls";
}

///helper function to check if a directory exists and create it if it doesn't.
pub fn check_directory(path: &Path) -> Result<(), Error> {
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

///load a sexp file and return a SexpTree
fn load_sexp(input: &str) -> Result<SexpTree, Error> {
    let doc = SexpParser::load(input)?;
    let tree = SexpTree::from(doc.iter())?;
    Ok(tree)
}

fn absolute_path(path: &Path) -> String {
    let mut absolute_path = std::env::current_dir().unwrap();
    absolute_path.push(path);
    absolute_path.to_str().unwrap().to_string()
}

/// Search a Kicad symbol.
///
/// # Arguments
///
/// * `term`     - The symbol name.
/// * `path`     - List of library paths.
pub fn search(term: &str, path: Vec<PathBuf>) -> Result<(), Error> {
    let mut results: Vec<(f32, String, String, String)> = Vec::new();
    for p in path {
        for entry in std::fs::read_dir(p).unwrap() {
            let dir = entry.unwrap();
            if dir.path().is_file() {
                let library = dir
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                if dir.path().to_str().unwrap().ends_with(".kicad_sym") {
                    let doc = SexpParser::load(dir.path().to_str().unwrap()).unwrap();
                    let mut iter = doc.iter();

                    if let Some(State::StartSymbol(name)) = &iter.next() {
                        if *name == "kicad_symbol_lib" {
                            iter.next(); //take first symbol
                            while let Some(state) = iter.next_siebling() {
                                if let State::StartSymbol(name) = state {
                                    if name == el::SYMBOL {
                                        if let Some(State::Text(id)) = iter.next() {
                                            let score: f32 = fuzzy_compare(
                                                &id.to_lowercase(),
                                                &term.to_string().to_lowercase(),
                                            );
                                            if score > 0.4 {
                                                while let Some(node) = iter.next() {
                                                    if let State::StartSymbol(name) = node {
                                                        if name == "property" {
                                                            if let Some(State::Text(name)) = iter.next() {
                                                                if name == "Description" {
                                                                    if let Some(State::Text(desc)) =
                                                                        iter.next()
                                                                    {
                                                                        results.push((
                                                                            score,
                                                                            library.to_string(),
                                                                            id.to_string(),
                                                                            desc.to_string(),
                                                                        ));
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            panic!("file is not a symbol library")
                        }
                    }
                }
            }
        }
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Library", "Key", "Description"]);

    results
        .iter()
        .sorted_by(|a, b| b.0.partial_cmp(&a.0).unwrap())
        .for_each(|item| {
            table.add_row(vec![
                Cell::new(item.1.clone()),
                Cell::new(item.2.clone()),
                Cell::new(item.3.clone()),
            ]);
        });

    println!("{table}");
    Ok(())
}

///the elektron command line interface
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// create a BOM from a kicad schematic.
    Bom {
        /// input kicad schema file.
        #[arg(short, long)]
        input: PathBuf,
        /// output file, this can be a json or excel file.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// group the items.
        #[arg(short, long)]
        group: bool,
        /// partlist yaml file, fields of the parts will be added or replaced with the partlist
        /// content.
        #[arg(short, long)]
        partlist: Option<PathBuf>,
    },
    /// convet a notebook
    Convert {
        /// input file
        #[arg(short, long)]
        input: PathBuf,
        /// ouptut file
        #[arg(short, long)]
        output: PathBuf,
    },
    /// run the drc checks on a kicad pcb.
    Drc {
        /// input file
        #[arg(short, long)]
        input: PathBuf,
        /// ouptut file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// run the erc checks on a kicad schematic.
    Erc {
        /// input file
        #[arg(short, long)]
        input: PathBuf,
        /// ouptut file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// plot a schematic or pcb from kicad file.
    Plot {
        /// input file.
        #[arg(short, long)]
        input: std::path::PathBuf,
        /// output file.
        #[arg(short, long)]
        output: std::path::PathBuf,
        /// plot the border
        #[arg(long)]
        border: bool,
        /// color theme
        #[arg(short, long, value_enum, default_value_t=Theme::Kicad2020)]
        theme: Theme,
        /// scale the plot
        #[arg(short, long, default_value_t = 1.0)]
        scale: f64,
        /// select the schema pages to plot
        #[arg(short, long)]
        pages: Option<Vec<usize>>,
        /// select the PCB layers to plot
        #[arg(long)]
        layers: Option<Vec<String>>,
        /// Output the PCB layers to seperate files.
        #[arg(long)]
        split: bool,
    },
    /// search for a symbol in the kicad library.
    Search {
        /// path where the spice models are located.
        #[arg(short, long)]
        path: Vec<PathBuf>,
        /// search term
        term: String,

    },
    /// create a spice netlist from a kicad schema.
    Spice {
        /// input file.
        #[arg(short, long)]
        input: PathBuf,
        /// output file.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// path where the spice models are located.
        #[arg(short, long)]
        path: Vec<PathBuf>,
    },
}

enum FileExtension {
    Schema,
    Pcb,
    Unknown,
}

impl FileExtension {
    fn from(path: &Path) -> Result<Self, Error> {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "kicad_sch" => Ok(FileExtension::Schema),
                "kicad_pcb" => Ok(FileExtension::Pcb),
                _ => Ok(FileExtension::Unknown),
            }
        } else {
            Err(Error::Plotter(format!(
                "File '{}' has no extension",
                path.to_str().unwrap()
            )))
        }
    }
}

/// Search a Kicad symbol.
///
/// # Arguments
///
/// * `term`     - The symbol name.
/// * `path`     - List of library paths.
#[pyfunction]
pub fn main() -> PyResult<()> {
    env_logger::init();
    let mut args: Vec<String> = std::env::args().map(|s| s.to_string()).collect();
    args.remove(0);
    let cli = Cli::parse_from(args);

    if let Err(error) = match cli.command {
        Some(Commands::Bom { input, output, group, partlist }) => {
            let tree = load_sexp(input.to_str().unwrap())?;
            let results = bom::bom(&tree, group, partlist).unwrap();

            if let Some(output) = output {
                let ext = output.extension();
                match ext {
                    None => {
                        return Err(Error::FileIo(format!(
                            "{} Output file has no extension: '{}'",
                            "Error:".red(),
                            output.to_str().unwrap().bold()
                        )).into());
                    },
                    Some(ext) => {
                        let ext = ext.to_str().unwrap();
                        if ext == constant::EXT_JSON {
                            let mut data = json::JsonValue::new_array();
                            for item in &results.0 {
                                data.push(json::object! {
                                    amount: item.amount,
                                    reference: item.references.clone(),
                                    value: item.value.clone(),
                                    footprint: item.footprint.clone(),
                                    datasheet: item.datasheet.clone(),
                                    description: item.description.clone()
                                })
                                .unwrap();
                            }
                            if let Err(err) = check_directory(&output) {
                                return Err(Error::FileIo(format!(
                                    "{} can not create output directory: '{}'",
                                    "Error:".red(),
                                    err.to_string().bold()
                                )).into());
                            };
                            let Ok(mut out) = File::create(output.clone()) else {
                                return Err(Error::FileIo(format!(
                                    "{} can not create output file: '{}'",
                                    "Error:".red(),
                                    output.to_str().unwrap().bold()
                                )).into());
                            };
                            if let Err(err) = data.write(&mut out) {
                                return Err(Error::FileIo(format!(
                                    "{} can not write output file: '{}'",
                                    "Error:".red(),
                                    err.to_string().bold()
                                )).into());
                            };
                            if let Err(err) = out.flush() {
                                return Err(Error::FileIo(format!(
                                    "{} can not flush output file: '{}'",
                                    "Error:".red(),
                                    err.to_string().bold()
                                )).into());
                            }
                        } else if ext == constant::EXT_EXCEL {
                            if let Err(err) = mouser::mouser(&output, &results.0) {
                                return Err(Error::FileIo(format!(
                                    "{} can not create excel file: '{}'",
                                    "Error:".red(),
                                    err.to_string().bold()
                                )).into());
                            }
                        } else {
                            return Err(Error::FileIo(format!(
                                "{} Output file type not supported for extension: '{}'",
                                "Error:".red(),
                                ext.bold()
                            )).into());
                        }
                    }
                }
            } else {
                let mut table = Table::new();
                table
                    .load_preset(comfy_table::presets::UTF8_FULL)
                    .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    // .set_width(40)
                    .set_header(vec![
                        "#",
                        "Ref",
                        "Value",
                        "Footprint",
                        "Datasheet",
                        "Description",
                    ]);

                results.0.iter().for_each(|item| {
                    table.add_row(vec![
                        Cell::new(item.amount),
                        Cell::new(item.references.join(" ")),
                        Cell::new(item.value.clone()),
                        Cell::new(item.footprint.clone()),
                        Cell::new(item.datasheet.clone()),
                        Cell::new(item.description.clone()),
                    ]);
                });

                println!("{table}");
            }
            Ok(())
        },
        Some(Commands::Drc { input, output }) => {
            info!("Write DRC: input:{}, output:{:?}", input.to_str().unwrap(), output);
            let results = match drc::drc(input.to_str().unwrap()) {
                Ok(result) => result,
                Err(error) => {
                    return Err(Error::FileIo(format!(
                        "{} can not load drc information from pcb: {} ({}))",
                        "Error".red(),
                        input.to_str().unwrap(),
                        error
                    )).into());
                }
            };
            if let Some(output) = output {
                let mut data = json::JsonValue::new_array();
                for item in &results.errors {
                    let mut pos = json::array![];
                    for item in &item.items {
                        pos.push(json::object! {
                            description: item.description.clone(),
                            pos: format!("{}x{}", item.pos.0, item.pos.1),
                            uuid: item.uuid.clone(),
                        }).unwrap();
                    }
                    data.push(json::object! {
                        type: item.drc_type.clone(),
                        severity: item.severity.clone(),
                        description: item.description.clone(),
                        items: pos,
                    })
                    .unwrap();
                }
                check_directory(&output)?;
                let mut out = File::create(output)?;
                data.write(&mut out)?;
                out.flush()?;

            } else {

                println!("DRC: {:?}", results);
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["Type", "Severity", "Description", "Position"]);

                results.errors.iter().for_each(|item| {
                    table.add_row(vec![
                        Cell::new(item.error_type.to_string()),
                        Cell::new(item.severity.clone()),
                        Cell::new(item.description.to_string()),
                        Cell::new(item.items.iter().map(|item| item.to_string()).collect::<Vec<String>>().join("\n")),
                    ]);
                });

                println!("{table}");
            }
            Ok(())
        },
        Some(Commands::Erc { input, output }) => {
            info!("Write ERC: input:{}, output:{:?}", input.to_str().unwrap(), output);
            let Ok(results) = erc::erc(&input) else {
                return Err(Error::FileIo(format!(
                    "{} can not load drc information from eschema: {})",
                    "Error".red(),
                    input.to_str().unwrap(),
                )).into());
            };
            if let Some(output) = output {
                let mut data = json::JsonValue::new_array();
                for item in &results {
                    data.push(json::object! {
                        id: String::from("NoReference"),
                        reference: item.reference.to_string(),
                        position: item.at.to_string(),
                    })
                    .unwrap();
                }
                check_directory(&output)?;
                let mut out = File::create(output)?;
                data.write(&mut out)?;
                out.flush()?;
            } else {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec!["ID", "Reference", "Position"]);

                results.iter().for_each(|item| {
                    table.add_row(vec![
                        Cell::new(item.id.clone()),
                        Cell::new(item.reference.clone()),
                        Cell::new(item.at.to_string()),
                    ]);
                });

                println!("{table}");
            }
            Ok(())
        },
        Some(Commands::Plot { input, output, border, theme, scale, pages, layers, split}) => {
            match FileExtension::from(&input) {
                Ok(FileExtension::Schema) => {
                    Ok(plotter::Schema::new(&input)
                        .border(border)
                        .theme(theme)
                        .scale(scale)
                        .pages(pages)
                        .split(split)
                        .plot(&output)?)
                },
                Ok(FileExtension::Pcb) => {
                    let _ = plotter::Pcb::new(&input)
                        .border(border)
                        .theme(theme)
                        .scale(scale)
                        .layers(layers)
                        .split(split)
                        .plot(&output)?;
                    Ok(())
                },
                _ => {
                    Err(Error::Plotter(format!("Input file '{}' has unknown type.", input.to_str().unwrap())))
                }
            }
        },
        Some(Commands::Convert { input, output }) => {
            info!("Write notebook: input:{}, output:{:?}", input.to_str().unwrap(), output);

            check_directory(&output).unwrap();

            //prepare env for notebook output
            std::env::set_var("ELEKTRON_NOTEBOOK", "true");

            //write to a tempf file, otherwise hugo reloads
            let tmpfile = NamedTempFile::new()?;
            let tmppath = tmpfile.into_temp_path();

            let out: Box<dyn Write> = Box::new(BufWriter::new(File::create(&tmppath).unwrap()));

            if let Err(err) = notebook::convert(
                &absolute_path(&input),
                out,
                Path::new(&input)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                Path::new(&output)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            ) {
                error!("{}", err.to_string());
            }

            if let Err(err) = std::fs::copy(tmppath, &output) {
                Err(Error::FileIo(format!(
                    "Can not write destination markdown file {} ({})",
                    output.to_str().unwrap(), err
                )))
            } else {
                Ok(())
            }
        },
        Some(Commands::Search { path, term }) => {
            search(&term, path)
        },
        Some(Commands::Spice { input, output, path }) => {
            let tree = load_sexp(input.to_str().unwrap())?;
            let netlist = Netlist::from(&tree);
            if let Ok(netlist) = netlist {
                let path =
                    path.iter().map(|p| p.to_str().unwrap().to_string()).collect::<Vec<String>>();

                let mut circuit = Circuit::new(input.to_str().unwrap().to_string(), path);
                netlist.circuit(&mut circuit).unwrap();
                let output = output.map(|o|
                    o.to_str().unwrap().to_string());
                circuit.save(output).unwrap();
            } else {
                println!(
                    "{} Can not create spice netlist from schema: {}",
                    "Error:".red(),
                    input.to_str().unwrap()
                );
            }
            Ok(())
        },
        None => { Err(Error::NoCommand) },
    } {
        error!("{}", error);
    }

    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(main))?;
    m.add_class::<crate::python::PyDraw>()?;
    m.add_class::<crate::python::model::Line>()?;
    m.add_class::<crate::python::model::Dot>()?;
    m.add_class::<crate::python::model::Label>()?;
    m.add_class::<crate::python::model::Nc>()?;
    m.add_class::<crate::python::model::Element>()?;
    m.add_class::<crate::python::model::C>()?;
    m.add_class::<crate::python::model::R>()?;
    m.add_class::<crate::python::model::Gnd>()?;
    m.add_class::<crate::python::model::Power>()?;
    m.add_class::<crate::python::model::Feedback>()?;
    m.add_class::<crate::python::circuit::Circuit>()?;
    m.add_class::<crate::python::circuit::Simulation>()?;
    Ok(())
}
