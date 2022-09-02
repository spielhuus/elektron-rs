use std::io::Write;
use std::{env::temp_dir, fs::File};

use clap::{Parser, Subcommand};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};
use elektron::Error;

use elektron::sexp::{SexpParser, State};
use itertools::Itertools;
use rand::Rng;
use rust_fuzzy_search::fuzzy_compare;
use viuer::{print_from_file, Config};

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
        input: String,
        #[clap(short, long, value_parser)]
        output: Option<String>,
        #[clap(short, long)]
        /// Group the elements.
        group: bool,
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
    Search {
        #[clap(forbid_empty_values = true)]
        /// Search the symbol libraries for term
        term: String,
        #[clap(short, long, value_parser)]
        path: Vec<String>,
    },
    Symbol {
        #[clap(forbid_empty_values = true)]
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
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match args.command {
        Command::Bom {
            input,
            output,
            group,
        } => {
            let results = elektron::bom(input.as_str(), group)?;

            if let Some(output) = output {
                let mut data = json::JsonValue::new_array();
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
                out.flush()?;
            } else {
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
        }
        Command::Netlist {
            input,
            output,
            spice,
        } => {
            elektron::netlist(input.as_str(), output, spice)?;
        }
        Command::Dump { input, output } => {
            elektron::dump(input.as_str(), output)?;
        }
        Command::Plot {
            input,
            output,
            border,
            theme,
            scale,
        } => {
            if let Some(filename) = &output {
                elektron::plot(input.as_str(), filename.as_str(), scale, border, theme).unwrap();
            } else {
                let mut rng = rand::thread_rng();
                let num: u32 = rng.gen();
                let filename =
                    String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".png";
                elektron::plot(input.as_str(), filename.as_str(), scale, border, theme).unwrap();
                print_from_file(&filename, &Config::default()).expect("Image printing failed.");
            };
        }
        Command::Symbol { key, path } => {
            let symbol = elektron::get_library(&key, path)?;
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
            let mut results: Vec<(f32, String, String)> = Vec::new();
            for p in path {
                for entry in std::fs::read_dir(p).unwrap() {
                    let dir = entry.unwrap();
                    if dir.path().is_file() {
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

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                // .set_width(40)
                .set_header(vec!["Key", "Description"]);

            results
                .iter()
                .sorted_by(|a, b| b.0.partial_cmp(&a.0).unwrap())
                .for_each(|item| {
                    table.add_row(vec![Cell::new(item.1.clone()), Cell::new(item.2.clone())]);
                });

            println!("{table}");
        }
    }
    Ok(())
}
