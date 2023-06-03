use std::io::{self, Write};
use std::path::Path;
use std::{fs::File, io::BufWriter};
use elektron::sexp::Library;
use tempfile::NamedTempFile;

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};

use itertools::Itertools;
use rand::Rng;
use rust_fuzzy_search::fuzzy_compare;
use std::env::temp_dir;
use viuer::{print_from_file, Config};

use clap::{Parser, Subcommand};

pub extern crate pest;

#[macro_use]
pub extern crate pest_derive;

mod draw;
mod error;
mod notebook;
mod plot;
mod reports;
mod sexp;
mod spice;
mod gerber;

use crate::reports::mouser::mouser;
use crate::{
    error::Error,
    notebook::Document,
    plot::{PlotOptions, PlotSelector, Plotter, Theme},
    reports::{
        bom::{bom, BomItem},
        erc::{erc, ErcItem},
        drc::drc,
    },
    sexp::{
        parser::{SexpParser, State},
        Pcb, Schema,
    },
    spice::{Circuit, Netlist},
};

fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn write_bom(results: Vec<BomItem>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
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

    results.iter().for_each(|item| {
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

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Bom {
        #[clap(short, long, value_parser)]
        /// Input filename.
        input: String,
        #[clap(short, long, value_parser)]
        /// Output Filename (defeault: stdout).
        output: Option<String>,
        #[clap(short, long)]
        /// Group the elements.
        group: bool,
        #[clap(short, long, value_parser)]
        /// Partlist
        parts: Option<String>,
    },
    Mouser {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        output: String,
        #[clap(short, long, value_parser)]
        parts: Option<String>,
    },
    Netlist {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        output: Option<String>,
        #[clap(short, long, value_parser)]
        spice: Vec<String>,
    },
    Plot {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        output: Option<String>,
        #[clap(short, long)]
        /// Draw border, otherwise the image will be croped.
        border: bool,
        #[clap(short, long, value_parser)]
        /// Select the color theme.
        theme: Option<String>,
        /// Set the image scale.
        scale: Option<f64>,
    },
    Gerber {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        output: Option<String>,
    },
    Search {
        #[clap(short, long, value_parser)]
        /// Search the symbol libraries for term
        term: String,
        #[clap(short, long, value_parser)]
        path: Vec<String>,
    },
    Symbol {
        #[clap(short, long, value_parser)]
        /// Search the symbol libraries for term
        key: String,
        #[clap(short, long, value_parser)]
        path: Vec<String>,
    },
    Dump {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        /// the output filename, prints to console when not defined.
        output: Option<String>,
    },
    Erc {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        /// the output filename, prints to console when not defined.
        output: Option<String>,
    },
    Drc {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        /// the output filename, prints to console when not defined.
        output: Option<String>,
    },
    Convert {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(short, long, value_parser)]
        output: Option<String>,
    },
    List {
        #[clap(short, long, value_parser)]
        input: String,
    },
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();
    match args.command {
        Command::Convert { input, output } => {

            if let Some(output) = &output {
                check_directory(&output).unwrap();
            }

            //prepare env for notebook output
            std::env::set_var("ELEKTRON_NOTEBOOK", "true");

            //write to a tempf file, otherwise hugo reloads
            let tmpfile = NamedTempFile::new()?;
            let tmppath = tmpfile.into_temp_path();

            let out: Box<dyn Write> = if output.is_some() {
                Box::new(BufWriter::new(File::create(&tmppath).unwrap()))
            } else {
                Box::new(std::io::stdout())
            };
            let mut runner = Document::new();
            runner.parse(&input).unwrap();
            runner
                .run(
                    out,
                    Path::new(&input)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    if let Some(output) = &output {
                        Path::new(&output)
                            .parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    } else {
                        Path::new(&input)
                            .parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    },
                )
                .unwrap();

            if let Some(filename) = output {
                //move the tmpfile to target
                std::fs::copy(tmppath, filename)?;
            }
        }
        Command::Bom {
            input,
            output,
            group,
            parts,
        } => {
            let schema = sexp::Schema::load(input.as_str())?;
            let results = bom(&schema, group, parts).unwrap();

            if let Some(output) = output {
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
                check_directory(&output)?;
                let mut out = File::create(output).unwrap();
                data.write(&mut out)?;
                out.flush()?;
            } else {
                write_bom(results.0);
            }
        }
        Command::Mouser {
            input,
            output,
            parts,
        } => {
            let schema = sexp::Schema::load(input.as_str())?;
            let results = bom(&schema, true, parts).unwrap();
            mouser(&output, &results.0).unwrap();

            if let Some(results) = results.1 {
                write_bom(results);
            }
        }
        Command::Erc { input, output } => {
            let schema = sexp::Schema::load(input.as_str())?;
            let results = erc(&schema).unwrap();

            if let Some(output) = output {
                /* let mut data = json::JsonValue::new_array();
                for item in &results {
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
                elektron::check_directory(&output)?;
                let mut out = File::create(output).unwrap();
                data.write(&mut out)?;
                out.flush()?; */
            } else {
                let mut table = Table::new();
                let mut index: u32 = 1;
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    // .set_width(40)
                    .set_header(vec!["#", "Pos", "Description"]);

                results.iter().for_each(|item| {
                    match item {
                        ErcItem::Netlist(err) => {
                            table.add_row(vec![
                                Cell::new("unable to create netlist"),
                                Cell::new(err.to_string()),
                                Cell::new(""),
                            ]);
                        }
                        ErcItem::NoReference { reference, at } => {
                            table.add_row(vec![
                                Cell::new(reference),
                                Cell::new("No reference for Symbol"),
                                Cell::new(format!("{}:{}", at[0], at[1])),
                            ]);
                        }
                        ErcItem::NotAllParts { reference, at } => {
                            table.add_row(vec![
                                Cell::new(reference),
                                Cell::new("Not all Symbol units on schema"),
                                Cell::new(format!("{}:{}", at[0], at[1])),
                            ]);
                        }
                        ErcItem::ValuesDiffer { reference, at } => {
                            table.add_row(vec![
                                Cell::new(reference),
                                Cell::new("Unit values differ"),
                                Cell::new(format!("{}:{}", at[0], at[1])),
                            ]);
                        }
                        ErcItem::PinNotConnected { reference, at } => {
                            table.add_row(vec![
                                Cell::new(reference),
                                Cell::new("Pin not connected"),
                                Cell::new(format!("{}:{}", at[0], at[1])),
                            ]);
                        }
                    }
                    index += 1;
                });

                println!("{table}");
            }
        }
        Command::Netlist {
            input,
            output,
            spice,
        } => {
            let schema = sexp::Schema::load(input.as_str())?;
            let netlist = Netlist::from(&schema).unwrap();
            let mut circuit = Circuit::new(input, spice);
            netlist.circuit(&mut circuit).unwrap();
            circuit.save(output).unwrap();
        }
        Command::Dump { input, output } => {
            if String::from(&input).ends_with(".kicad_sch") {
                let schema = Schema::load(input.as_str()).unwrap();
                schema.write(output.unwrap().as_str()).unwrap();
            } else {
                let pcb = Pcb::load(input.as_str())?;
                pcb.write(output.unwrap().as_str()).unwrap();
            }
        }
        Command::Plot {
            input,
            output,
            border,
            theme,
            scale,
        } => {
            if input.ends_with(".kicad_sch") {
                let t = if let Some(theme) = theme {
                    theme.as_str().into()
                } else {
                    Theme::Kicad2020
                };
                // let scale = if let Some(scale) = scale { scale } else { 1.0 };
                let schema = Schema::load(input.as_str())?;
                if let Some(filename) = &output {
                    let mut buffer = File::create(filename)?;
                    if filename.ends_with(".png") {
                        Plotter::png(
                            PlotOptions::new(&schema, &mut buffer)
                                .id("TODO")
                                .border(border)
                                .theme(t),
                        )?;
                    } else if filename.ends_with(".pdf") {
                        Plotter::pdf(
                            PlotOptions::new(&schema, &mut buffer)
                                .id("TODO")
                                .border(border)
                                .theme(t),
                        )?;
                    } else {
                        Plotter::svg(
                            PlotOptions::new(&schema, &mut buffer)
                                .id("TODO")
                                .border(border)
                                .theme(t),
                        )?;
                    }
                } else {
                    let mut rng = rand::thread_rng();
                    let num: u32 = rng.gen();
                    let filename = String::new()
                        + temp_dir().to_str().unwrap()
                        + "/"
                        + &num.to_string()
                        + ".png";
                    let mut buffer = File::create(&filename)?;
                    Plotter::png(
                        PlotOptions::new(&schema, &mut buffer)
                            .border(border)
                            .theme(t),
                    )?;
                    print_from_file(&filename, &Config::default()).expect("Image printing failed.");
                };
            } else {
                /* let scale = if let Some(scale) = scale { scale } else { 1.0 };
                let theme = if let Some(theme) = theme {
                    theme
                } else {
                    "kicad_2000".to_string()
                }; */
                let pcb = Pcb::load(input.as_str()).unwrap();
                if let Some(filename) = &output {
                    let mut buffer = File::create(filename)?;
                    Plotter::svg(
                        PlotOptions::new(&pcb, &mut buffer)
                            .id("TODO")
                            .border(border)
                            .theme(Theme::Kicad2020),
                    )?;
                } else {
                    let mut rng = rand::thread_rng();
                    let num: u32 = rng.gen();
                    let filename = String::new()
                        + temp_dir().to_str().unwrap()
                        + "/"
                        + &num.to_string()
                        + ".png";
                    let mut buffer = File::create(&filename)?;
                    Plotter::png(
                        PlotOptions::new(&pcb, &mut buffer)
                            .border(border)
                            .theme(Theme::Kicad2020),
                    )?;
                    print_from_file(&filename, &Config::default()).expect("Image printing failed.");
                };
            }
        }
        Command::Gerber {
            input,
            output,
        } => {
            let pcb = gerber::Pcb::new(input);
            pcb.gerber(output.unwrap());
            /* if let Some(filename) = output {
                let pcb = Pcb::load(input.as_str()).unwrap();
                let mut buffer = File::create(filename)?;
                let svg = GerberPlotter::new();
                svg.plot(
                    &pcb,
                    &mut buffer,
                    false,
                    1.0,
                    None,
                    false,
                );
            } */
        }
        Command::Drc {
            input,
            output,
        } => {
            let results = drc(input, output)?;
                let mut table = Table::new();
                let mut index: u32 = 1;
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    // .set_width(40)
                    .set_header(vec!["#", "title", "description", "severity"]);

                results.iter().for_each(|item| {
                    table.add_row(vec![
                        Cell::new(item.id.to_string()),
                        Cell::new(item.title.to_string()),
                        Cell::new(item.description.to_string()),
                        Cell::new(item.severity.to_string()),
                        // Cell::new(format!("@({}:{}): {}", item.position.0, item.position.1, item.position.2)),
                    ]);
                });

                println!("{table}");
        }
        Command::Symbol { key, path } => {
            let mut library = sexp::Library::new(path);
            let symbol = library.get(key.as_str())?;
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                // .set_width(40)
                .set_header(vec!["Key", "Description"]);

            table.add_row(vec![Cell::new(symbol.lib_id), Cell::new(symbol.power)]);

            println!("{table}");
        }
        Command::Search { term, path } => {
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
                                            if name == "symbol" {
                                                if let Some(State::Text(id)) = iter.next() {
                                                    let score: f32 = fuzzy_compare(
                                                        &id.to_lowercase(),
                                                        &term.to_string().to_lowercase(),
                                                    );
                                                    if score > 0.4 {
                                                        while let Some(node) = iter.next() {
                                                            if let State::StartSymbol(name) = node {
                                                                if name == "property" {
                                                                    if let Some(State::Text(name)) =
                                                                        iter.next()
                                                                    {
                                                                        if name == "ki_description" {
                                                                            if let Some(State::Text(
                                                                                desc,
                                                                            )) = iter.next()
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
                // .set_width(40)
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
        },
        Command::List { input } => {

            let mut data = json::JsonValue::new_array();
            let lib = Library::from(input);
            for symbol in &lib.cache {
                data.push(json::object! {
                    library: symbol.1.lib_id.to_string(),
                    description: symbol.1.get_property("ki_description"),
                }).unwrap();
            }
            io::stdout().write_all(data.to_string().as_bytes()).unwrap();
            io::stdout().write_all("\n".as_bytes()).unwrap();
        }
    }
    Ok(())
}
