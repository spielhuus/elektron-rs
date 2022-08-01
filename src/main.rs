use clap::{Parser, Subcommand};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use viuer::{Config, print_from_file};
use std::env::temp_dir;
use rand::Rng;

pub mod cairo_plotter;
pub mod circuit;
pub mod draw;
pub mod error;
pub mod libraries;
pub mod netlist;
pub mod ngspice;
pub mod plot;
pub mod reports;
pub mod sexp;
pub mod shape;
pub mod themes;

use crate::cairo_plotter::{CairoPlotter, ImageType};
use crate::circuit::{Circuit, Simulation};
use crate::error::Error;
use crate::libraries::Libraries;
use crate::plot::plot;
use crate::reports::bom;
use crate::sexp::parser::SexpParser;
use crate::themes::Style;

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
    },
    Dump {
        #[clap(short, long, value_parser)]
        input: String,
        #[clap(forbid_empty_values = true)]
        /// Name of the package to search
        package_name: String,
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
            let parser = SexpParser::load(input.as_str())?;
            if let Some(filename) = &output {
                let path = std::path::Path::new(&filename);
                let parent = path.parent();
                if let Some(parent) = parent {
                    if parent.to_str().unwrap() != "" && !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
            }
            bom(output, &parser, group).unwrap();
        }
        Command::Plot { input, output, border, theme, scale, } => {
            let scale: f64 = if let Some(scale) = scale { scale } else { 1.0 };
            let image_type = if let Some(output) = &output {
                if output.ends_with(".svg") {
                    ImageType::Svg
                } else if output.ends_with(".png") {
                    ImageType::Png
                } else {
                    ImageType::Pdf
                }
            } else {
                ImageType::Png
            };
            let mut cairo = CairoPlotter::new();
            let style = Style::new();
            let parser = SexpParser::load(input.as_str()).unwrap();

            if let Some(filename) = &output {
                let path = std::path::Path::new(&filename);
                let parent = path.parent();
                if let Some(parent) = parent {
                    if parent.to_str().unwrap() != "" && !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }

                let out: Box<dyn Write> = Box::new(File::create(filename).unwrap());
                plot(&mut cairo, out, &parser, border, scale, style, image_type).unwrap();
            } else {
                let mut rng = rand::thread_rng();
                let num: u32 = rng.gen();
                let filename =
                    String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".svg";
                let out: Box<dyn Write> = Box::new(File::create(&filename).unwrap());
                plot(&mut cairo, out, &parser, border, scale, style, image_type).unwrap();
                print_from_file(&filename, &Config::default()).expect("Image printing failed.");
            };
        }
        Command::Search { term } => {
            let mut libs: Libraries = Libraries::new(vec!["/usr/share/kicad/symbols".to_string()]);
            let res = libs.search(&term)?;
            println!("{:?}", res);
        }
        _ => { /* TODO */ }
    }
    Ok(())
}
