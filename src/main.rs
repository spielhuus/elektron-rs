use core::slice::Iter;
use memmap2::MmapOptions;
use std::fs::File;
use std::io::Write;
use std::fs;

mod draw;
mod cairo_plotter;
mod sexp;
mod reports;
mod plot;
mod themes;
mod shape;
mod libraries;
mod netlist;

use crate::sexp::SexpParser;
use crate::reports::bom;
use crate::plot::plot;
use crate::cairo_plotter::CairoPlotter;
use crate::themes::Style;
use crate::libraries::Libraries;

use ndarray::{arr1, arr2, s, Array, Array1, Array2};

use self::draw::Draw;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Can not parse file.")]
    ParseError,
    #[error("can not find symbol {0}.")]
    SymbolNotFound(String),
    #[error("can not find symbol.")]
    ExpectSexpNode,
    #[error("element is not a value node.")]
    ExpectValueNode,
    #[error("Justify value error.")]
    JustifyValueError,
    #[error("LineType value error")]
    LineTypeValueError,
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
    #[error("Sexp content not loaded.")]
    NotLoaded,
    #[error("Pin not found for {0}")]
    PinNotFound(usize),
    #[error("Property not found for {0}")]
    PropertyNotFound(String),
    #[error("More then one Property found for {0}")]
    MoreThenOnPropertyFound(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}


fn main() {
//    let pathlist = vec!["/usr/share/kicad/symbols"];
//    for path in &pathlist {
//        for entry in fs::read_dir(path).unwrap() {
//            let dir = entry.unwrap();
//            if dir.path().is_file() {
//                let parser = SexpParser::load(dir.path().to_str().unwrap());
//            }
//        }
//    }

    /* let parser = SexpParser::load("/home/etienne/Documents/elektrophon/content/kraft/kraft/kraft.kicad_sch").unwrap();
    //println!("{:#?}", &parser.nodes);
    parser.save(&mut std::io::stdout()).unwrap(); */

    /* let parser = SexpParser::load("/home/etienne/Documents/elektrophon/content/kraft/kraft/kraft.kicad_sch").unwrap();
    bom(None, &parser, true).unwrap();    */

    /* let parser = SexpParser::load("/home/etienne/Documents/elektrophon/content/kraft/kraft/kraft.kicad_sch").unwrap();
    let mut cairo = CairoPlotter::new();
    let style = Style::new();
    plot(&mut cairo, Option::from("out.svg"), &parser, true, style).unwrap(); */


    let mut libs = Libraries::new(vec![String::from("/usr/share/kicad/symbols/")]);
    libs.search("TL072");


    //draw 
    let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols/")]);
    draw.wire(vec![0.0, 0.0], vec![10.0, 0.0]);
    println!("{:?}", draw.elements);
}
