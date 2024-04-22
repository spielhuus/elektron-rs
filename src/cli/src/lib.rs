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

use log::{debug, info, error};

use pyo3::{exceptions::{PyFileNotFoundError, PyIOError}, prelude::*};

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};

use itertools::Itertools;
use rust_fuzzy_search::fuzzy_compare;
use tempfile::NamedTempFile;

use colored::*;

mod error;
mod python;

use crate::error::Error;

use plotter::{
    schema::SchemaPlot, svg::SvgPlotter, Theme
};
use sexp::{el, SexpParser, SexpProperty, SexpTree, SexpValueQuery, State};

use reports::{bom, drc, erc, mouser};
use simulation::{Circuit, Netlist};

mod constant {
    pub const EXT_KICAD_SCH: &str = ".kicad_sch";
    pub const EXT_JSON: &str = ".json";
    pub const EXT_SVG: &str = ".svg";
    pub const EXT_PNG: &str = ".png";
    pub const EXT_PDF: &str = ".pdf";
    pub const EXT_EXCEL: &str = ".xls";
}

///helper function to check if a directory exists and create it if it doesn't.
pub fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
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

/// Create the BOM for a Schema.
///
/// # Arguments
///
/// * `input`    - A Schema filename.
/// * `group`    - group equal items.
/// * `partlist` - A YAML file with the parts description (Optional).
/// * `return`   - Tuple with a `Vec<BomItem>` and the items not found
///                in the partlist, when provided.
#[pyfunction]
pub fn make_bom(
    input: &str,
    group: bool,
    output: Option<String>,
    partlist: Option<String>,
) -> Result<(), Error> {
    env_logger::init();
    info!("Write BOM: input:{}, output:{:?}", input, output);
    let tree = load_sexp(input)?;
    let results = bom::bom(&tree, group, partlist)?;

    if let Some(output) = output {
        if let Some(ext_pos) = output.find('.') {
            let ext = output.split_at(ext_pos).1;
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
                    )));
                };
                let Ok(mut out) = File::create(output.clone()) else {
                    return Err(Error::FileIo(format!(
                        "{} can not create output file: '{}'",
                        "Error:".red(),
                        output.bold()
                    )));
                };
                if let Err(err) = data.write(&mut out) {
                    return Err(Error::FileIo(format!(
                        "{} can not write output file: '{}'",
                        "Error:".red(),
                        err.to_string().bold()
                    )));
                };
                if let Err(err) = out.flush() {
                    return Err(Error::FileIo(format!(
                        "{} can not flush output file: '{}'",
                        "Error:".red(),
                        err.to_string().bold()
                    )));
                }
            } else if ext == constant::EXT_EXCEL {
                if let Err(err) = mouser::mouser(&output, &results.0) {
                    return Err(Error::FileIo(format!(
                        "{} can not create excel file: '{}'",
                        "Error:".red(),
                        err.to_string().bold()
                    )));
                }
            } else {
                return Err(Error::FileIo(format!(
                    "{} Output file type not supported for extension: '{}'",
                    "Error:".red(),
                    ext.bold()
                )));
            }
        } else {
            return Err(Error::FileIo(format!(
                "{} Output file has no extension: '{}'",
                "Error:".red(),
                output.bold()
            )));
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
}

/// plot the document
///
/// The filetype is selected by the output file extension. When no output filename is given the
/// image will be displayed in the console.
///
/// # Arguments
///
/// * `input`    - A Schema filename.
/// * `output`   - The filename of the target image.
#[pyfunction]
pub fn plot(input: &str, output: Option<&str>) -> Result<(), PyErr> {
    env_logger::init();
    if input.ends_with(constant::EXT_KICAD_SCH) {
        info!("Write schema: input:{}, output:{:?}", input, output);
        //load the sexp file.
        if let Some(output) = output {
            if let Some(ext_pos) = output.find('.') {
                let ext = output.split_at(ext_pos).1;
                if ext == constant::EXT_SVG {

                    let mut plotter = SchemaPlot::new()
                        .border(true).theme(Theme::Kicad2020).scale(1.0) //TODO set paramenters
                        .name(input);

                    plotter.open(input)?;
                    for page in plotter.iter() {
                        let mut file = if *page.0 == 1 {
                            debug!("write first page to {}", output);
                            File::create(output)?
                        } else {
                            debug!("write page {} to {}", page.1, format!("{}.svg", page.1));
                            File::create(format!("{}.svg", page.1))?
                        };
                        let mut svg_plotter = SvgPlotter::new(&mut file);
                        plotter.write(page.0, &mut svg_plotter)?;
                    }

                /*TODO  } else if ext == constant::EXT_PNG {
                    let plotter = CairoPlotter::new(
                        input,
                        ImageType::Png,
                        Some(Themer::new(Theme::Kicad2020)), //TODO
                    );
                    let mut buffer = File::create(output).unwrap();
                    plotter
                        .plot(&tree, &mut buffer, true, 1.0, None, false)
                        .unwrap();
                } else if ext == constant::EXT_PDF {
                    let plotter = CairoPlotter::new(
                        input,
                        ImageType::Pdf,
                        Some(Themer::new(Theme::Kicad2020)),
                    );
                    let mut buffer = File::create(output).unwrap();
                    plotter
                        .plot(&tree, &mut buffer, true, 1.0, None, false)
                        .unwrap(); */
                } else {
                    return Err(PyIOError::new_err(format!(
                        "{} Image type not supported for extension: '{}'",
                        "Error:".red(),
                        ext.bold()
                    )));
                }
            } else {
                return Err(PyFileNotFoundError::new_err(format!(
                    "{} Input file does not exist: {}",
                    "Error:".red(),
                    input.bold()
                )));
            }
        } else {
            println!("no output file");
        }
    } else if input.ends_with(".kicad_pcb") {
        info!("Write PCB: input:{}, output:{:?}", input, output);
        if let Some(output) = output {
            plotter::pcb::plot_pcb(
                input.to_string(),
                output.to_string(),
                None, /* TODO */
                None,
            )?; //TODO set layers
        } else {
            println!("no output file");
        }
    } else {
        return Err(PyFileNotFoundError::new_err(format!(
            "{} Input file does not exist: {}",
            "Error:".red(),
            input
        )));
    }
    Ok(())
}

/// output the spice netlist.
///
/// # Arguments
///
/// * `input`    - A Schema filename.
/// * `path`     - Path to the spice library
/// * `output`   - output file name, when no filename is given the result will be printed to the console.
#[pyfunction]
pub fn make_spice(input: &str, path: Vec<String>, output: Option<String>) -> Result<(), Error> {
    let tree = load_sexp(input)?;
    let netlist = Netlist::from(&tree);
    if let Ok(netlist) = netlist {
        let mut circuit = Circuit::new(input.to_string(), path);
        netlist.circuit(&mut circuit).unwrap();
        circuit.save(output).unwrap();
    } else {
        println!(
            "{} Can not create spice netlist from schema: {}",
            "Error:".red(),
            input
        );
    }
    Ok(())
}
fn absolute_path(path: &str) -> String {
    let mut absolute_path = std::env::current_dir().unwrap();
    absolute_path.push(path);
    absolute_path.to_str().unwrap().to_string()
}

/// Convert a notebook page
///
/// # Arguments
///
/// * `input`    - notebook markdown file.
/// * `output`   - destination markdown file.
/// * `return`   - possible error.
#[pyfunction]
pub fn convert(input: &str, output: &str) -> Result<(), Error> {
    env_logger::init();
    info!("Write notebook: input:{}, output:{:?}", input, output);

    check_directory(output).unwrap();

    //prepare env for notebook output
    std::env::set_var("ELEKTRON_NOTEBOOK", "true");

    //write to a tempf file, otherwise hugo reloads
    let tmpfile = NamedTempFile::new()?;
    let tmppath = tmpfile.into_temp_path();

    let out: Box<dyn Write> = Box::new(BufWriter::new(File::create(&tmppath).unwrap()));

    if let Err(err) = notebook::convert(
        &absolute_path(input),
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

    if let Err(err) = std::fs::copy(tmppath, output) {
        Err(Error::FileIo(format!(
            "Can not write destination markdown file {} ({})",
            output, err
        )))
    } else {
        Ok(())
    }
}

/// Run the ERC checks for a Schema.
///
/// # Arguments
///
/// * `input` - A Schema filename.
/// * `group`    - group equal items.
/// * `partlist` - A YAML file with the parts description.
/// * `return`   - Tuple with a 'Vec<BomItem>' and the items not found
///                in the partlist, when provided.
#[pyfunction]
pub fn make_erc(input: &str, output: Option<String>) -> Result<(), Error> {
    let Ok(results) = erc::erc(input) else {
        return Err(Error::FileIo(format!(
            "{} can not load drc information from eschema: {})",
            "Error".red(),
            input,
        )));
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
}

/// Run the DRC checks for a Schema.
///
/// # Arguments
///
/// * `input` - A Schema filename.
/// * `partlist` - A YAML file with the parts description.
/// * `return`   - Tuple with a `Vec<BomItem>` and the items not found
///                in the partlist, when provided.
#[pyfunction]
pub fn make_drc(input: &str, output: Option<String>) -> Result<(), Error> {
    env_logger::init();
    info!("Write DRC: input:{}, output:{:?}", input, output);
    let results = match drc::drc(input.to_string()) {
        Ok(result) => result,
        Err(error) => {
            return Err(Error::FileIo(format!(
                "{} can not load drc information from pcb: {} ({}))",
                "Error".red(),
                input,
                error
            )));
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
}

/// Search a Kicad symbol.
///
/// # Arguments
///
/// * `term`     - The symbol name.
/// * `path`     - List of library paths.
#[pyfunction]
pub fn search(term: &str, path: Vec<String>) -> Result<(), Error> {
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
                                                            if let Some(State::Text(name)) =
                                                                iter.next()
                                                            {
                                                                if name == "ki_description" {
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

/// Search a Kicad symbol.
///
/// # Arguments
///
/// * `term`     - The symbol name.
/// * `path`     - List of library paths.
#[pyfunction]
pub fn list(input: &str) -> Result<(), Error> {
    let mut data = json::JsonValue::new_array();

    if let Ok(doc) = SexpParser::load(input) {
        if let Ok(tree) = SexpTree::from(doc.iter()) {
            for node in tree.root()?.query(el::SYMBOL) {
                let sym_name: String = node.get(0).unwrap();
                let sym_desc: String = node.property("ki_description").unwrap();
                data.push(json::object! {
                    library: sym_name,
                    description: sym_desc,
                })
                .unwrap();
            }
        }
    }

    std::io::stdout()
        .write_all(data.to_string().as_bytes())
        .unwrap();
    std::io::stdout().write_all("\n".as_bytes()).unwrap();
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn elektron(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(make_bom))?;
    m.add_wrapped(wrap_pyfunction!(plot))?;
    m.add_wrapped(wrap_pyfunction!(make_spice))?;
    m.add_wrapped(wrap_pyfunction!(convert))?;
    m.add_wrapped(wrap_pyfunction!(make_erc))?;
    m.add_wrapped(wrap_pyfunction!(make_drc))?;
    m.add_wrapped(wrap_pyfunction!(search))?;
    m.add_wrapped(wrap_pyfunction!(list))?;
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
