use std::io::Write;

use super::border::draw_border;
use super::cairo_plotter::{
    Arc, Circle, ImageType, Line, PlotItem, Plotter, Polyline, Rectangle, Text,
};
use crate::error::Error;
use crate::sexp::document::Document;
use crate::sexp::get::{get, Get};
use crate::sexp::iterator::libraries;
use crate::sexp::test::Test;
use crate::sexp::{get_unit, Color, Effects, FillType, LineType, Stroke};
use crate::sexp::{Justify, Sexp};
use crate::shape::{Shape, Transform};
use crate::themes::StyleTypes;
use crate::themes::{Style, StyleContext};
use ndarray::{arr1, arr2, Array1, Array2};

pub fn pcb(
    plotter: &mut dyn Plotter,
    out: Box<dyn Write>,
    sexp_parser: &Document,
    border: bool,
    scale: f64,
    style: Style,
    image_type: ImageType,
) -> Result<(), Error> {
    let libraries = libraries(sexp_parser).unwrap();

    if border {
        sexp_parser.iter().for_each(|node| {
            if let Sexp::Node(name, values) = node {
                if name == "paper" {
                    let v = values.get(0);
                    if let Some(Sexp::Value(size)) = v {
                        plotter.paper(size.clone());
                    }
                }
            }
        });
        sexp_parser.iter().for_each(|node| {
            if let Sexp::Node(name, _) = node {
                if name == "title_block" {
                    draw_border(Option::from(node), plotter.get_paper(), plotter, &style).unwrap();
                }
            }
        });
    }

    sexp_parser.iter().for_each(|node| {
        if let Sexp::Node(name, _) = node {
            if name == "gr_line" {

            }
            println!("name {}", name);
        } else {
            panic!("wrong node");
        }
    });

    plotter.plot(out, border, scale, image_type).unwrap();
    Ok(())
}
