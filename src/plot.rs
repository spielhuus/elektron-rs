use std::io::Write;

use crate::cairo_plotter::{
    Arc, Circle, ImageType, Line, PlotItem, Plotter, Polyline, Rectangle, Text,
};
use crate::sexp::get::{get, Get};
use crate::sexp::parser::SexpParser;
use crate::sexp::test::Test;
use crate::sexp::iterator::libraries;
use crate::sexp::{get_unit, Color, Effects, FillType, LineType, Stroke};
use crate::sexp::{Justify, Sexp};
use crate::shape::{Shape, Transform};
use crate::themes::StyleTypes;
use crate::themes::{Style, StyleContext};
use crate::error::Error;
use ndarray::{arr1, arr2, Array1, Array2};

pub mod paper {
    pub const A4: (f64, f64) = (297.0, 210.0);
}

const BORDER_RASTER: i32 = 60;

macro_rules! text {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr) => {
        PlotItem::TextItem(99, Text::new(
            $pos,
            $angle,
            $content,
            $effects.color.clone(),
            $effects.size,
            $effects.font.as_str(),
            $effects.justify.clone(),
        ))
    };
}

macro_rules! line {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr) => {
    plotter.push(PlotItem::TextItem(99, Text::new(
        pos,
        0.0,
        content,
        effects.color.clone(),
        effects.size,
        effects.font.as_str(),
        effects.justify.clone(),
    )));
    };
}

pub fn text(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let content: String = get!(node, 0).unwrap();
    let effects: Effects = style
        .style(&node, "effects", StyleContext::SchemaProperty)
        .unwrap();
    plotter.push(PlotItem::TextItem(99, Text::new(
        pos,
        0.0,
        content,
        effects.color.clone(),
        effects.size,
        effects.font.as_str(),
        effects.justify.clone(),
    )));
    Ok(())
}

pub fn draw_border(
    node: Option<&Sexp>,
    paper_size: (f64, f64),
    plotter: &mut dyn Plotter,
    style: &Style,
) -> Result<(), Error> {
    let stroke: Stroke = style.schema_border();
    let effects: Effects = style.schema_effects();
    //outline
    let pts: Array2<f64> = arr2(&[[5.0, 5.0], [paper_size.0 - 5.0, paper_size.1 - 5.0]]);
    plotter.push(PlotItem::RectangleItem(99, Rectangle::new(
        pts,
        stroke.color.clone(),
        stroke.width,
        stroke.line_type.clone(),
        None,
    )));

    //horizontal raster
    for j in vec![(0.0 as f64, 5.0 as f64), (paper_size.1-5.0, paper_size.1)] {
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.0],
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.1],
            ]);
            plotter.push(PlotItem::RectangleItem(99, Rectangle::new(
                pts,
                stroke.color.clone(),
                0.1,
                stroke.line_type.clone(),
                None,
            )));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(text!(arr1(&[(i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0, j.0 + 2.5]), 0.0, (i + 1).to_string(), effects));
        }
    }

    //vertical raster
    let letters = vec![
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    for j in vec![(0.0 as f64, 5.0 as f64), (paper_size.0 - 5.0, paper_size.0)] {
        for i in 0..(paper_size.1 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [j.0, (i as f64 + 1.0) * BORDER_RASTER as f64],
                [j.1, (i as f64 + 1.0) * BORDER_RASTER as f64],
            ]);
            plotter.push(PlotItem::RectangleItem(99, Rectangle::new(
                pts,
                stroke.color.clone(),
                0.1,
                stroke.line_type.clone(),
                None,
            )));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(text!(arr1(&[j.0 + 2.5, (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0]), 0.0, letters[i as usize].to_string(), effects));
        }
    }

    // the head
    let pts: Array2<f64> = arr2(&[
        [paper_size.0 - 120.0, paper_size.1 - 40.0],
        [paper_size.0 - 5.0, paper_size.1 - 5.0],
    ]);
    plotter.push(PlotItem::RectangleItem(99, Rectangle::new(
        pts,
        stroke.color.clone(),
        stroke.width.clone(),
        stroke.line_type.clone(),
        None,
    )));
    plotter.push(PlotItem::LineItem(99, Line::new(
        arr2(&[
            [paper_size.0 - 120.0, paper_size.1 - 30.0],
            [paper_size.0 - 5.0, paper_size.1 - 30.0],
        ]),
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(99, Line::new(
        arr2(&[
            [paper_size.0 - 120.0, paper_size.1 - 10.0],
            [paper_size.0 - 5.0, paper_size.1 - 10.0],
        ]),
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(99, Line::new(
        arr2(&[
            [paper_size.0 - 120.0, paper_size.1 - 16.0],
            [paper_size.0 - 5.0, paper_size.1 - 16.0],
        ]),
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));

    if let Some(node) = node {
        let left = paper_size.0 - 118.0;
        if node.contains("comment") {
            let comments: Vec<&Sexp> = node.get("comment").unwrap();
            let effects: Effects = style.schema_subtitle_effects();
            for comment in comments {
                if let Sexp::Node(_, values) = comment {
                    let key = if let Sexp::Value(key) = values.get(0).unwrap() {
                        key.clone()
                    } else { String::from("0") };
                    let text = if let Sexp::Text(text) = values.get(1).unwrap() {
                        text.clone()
                    } else { String::from("0") };

                    if key == String::from("1") {
                        plotter.push(text!(arr1(&[left, paper_size.1 - 35.0]), 0.0, text, &effects));
                    } else if key == String::from("2") {
                        plotter.push(text!(arr1(&[left, paper_size.1 - 35.0]), 0.0, text, &effects));
                    } else if key == String::from("3") {
                        plotter.push(text!(arr1(&[left, paper_size.1 - 35.0]), 0.0, text, &effects));
                    } else if key == String::from("4") {
                        plotter.push(text!(arr1(&[left, paper_size.1 - 40.0]), 0.0, text, &effects));
                    }
                }
            }
            /* let effects: Effects = style.schema_title_effects();
            let title: String = get!(node, "title", 0);
            plotter.push(text!(arr1(&[left, paper_size.1 - 15.0]), 0.0, title, effects)); */
        }
        if node.contains("company") {
            let effects: Effects = style.schema_title_effects();
            let title: String = get!(node, "company", 0);
            plotter.push(text!(arr1(&[left, paper_size.1 - 25.0]), 0.0, title, effects));
        }
        if node.contains("title") {
            let effects: Effects = style.schema_title_effects();
            let title: String = get!(node, "title", 0);
            plotter.push(text!(arr1(&[left, paper_size.1 - 13.0]), 0.0, format!("Title: {}", title), effects));
        }
        let effects: Effects = style.schema_effects();
        plotter.push(text!(arr1(&[left, paper_size.1 - 8.0]), 0.0, format!("Paper: {}", String::from("xxx")), &effects));

        if node.contains("date") {
            let effects: Effects = style.schema_effects();
            let title: String = get!(node, "date", 0);
            plotter.push(text!(arr1(&[paper_size.0 - 90.0, paper_size.1 - 8.0]), 0.0, format!("Data: {}", title), &effects));
        }
        if node.contains("rev") {
            let effects: Effects = style.schema_effects();
            let title: String = get!(node, "rev", 0);
            plotter.push(text!(arr1(&[paper_size.0 - 20.0, paper_size.1 - 8.0]), 0.0, format!("Rev: {}", title), &effects));
        }
    }
    Ok(())
}

pub fn no_connect(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let stroke: Stroke = style
        .style(&node, "stroke", StyleContext::SchemaSymbol)
        .unwrap();
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
    let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;

    plotter.push(PlotItem::LineItem(10, Line::new(
        lines1,
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(10, Line::new(
        lines2,
        stroke.width,
        stroke.line_type,
        stroke.color,
    )));

    Ok(())
}

pub fn label(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let effects: Effects = style
        .style(&node, "effects", StyleContext::SchemaProperty)
        .unwrap();
    let _fill_color: Option<Color> = style.color(&FillType::Background); //TODO
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let mut angle: f64 = get!(node, "at", 2);
    if angle >= 180.0 {
        //dont know why this is possible
        angle = angle - 180.0;
    }
    let _text: String = get!(node, 0).unwrap();
    let text = Text::new(
        pos.clone(),
        angle,
        _text,
        effects.color.clone(),
        effects.size,
        effects.font.as_str(),
        effects.justify,
    );
    plotter.push(PlotItem::TextItem(10, text));
    plotter.push(PlotItem::CircleItem(10, Circle::new(
        pos,
        0.5,
        0.3,
        LineType::Default,
        effects.color,
        None,
    )));
    Ok(())
}

pub fn global_label(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let stroke: Stroke = style
        .style(&node, "stroke", StyleContext::SchemaSymbol)
        .unwrap();
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let lines1 = arr2(&[[-1.0, -1.0], [1.0, 1.0]]) + &pos;
    let lines2 = arr2(&[[1.0, 1.0], [-1.0, -1.0]]) + &pos;

    plotter.push(PlotItem::LineItem(10, Line::new(
        lines1,
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(10, Line::new(
        lines2,
        stroke.width,
        stroke.line_type,
        stroke.color,
    )));

    Ok(())
}

pub fn symbol(
    node: &Sexp,
    libs: &std::collections::HashMap<String, &Sexp>,
    plotter: &mut dyn Plotter,
    style: &Style,
) -> Result<(), Error> {
    if node.contains("on_schema") {
        let on_schema: String = get!(node, "on_schema", 0);
        if on_schema == "no" {
            return Ok(());
        }
    }
    let symbol_angle: f64 = get!(node, "at", 2);
    let properties: Vec<&Sexp> = node.get("property").unwrap();
    for property in properties {
        let mut effects: Effects = style
            .style(property, "effects", StyleContext::SchemaProperty)
            .unwrap();
        let value: String = get!(property, 1).unwrap();
        let angle: f64 = get!(property, "at", 2);
        let mut justify: Vec<Justify> = Vec::new();
        for j in effects.justify {
            if angle + symbol_angle >= 180.0 && angle + symbol_angle < 360.0 && j == Justify::Left {
                justify.push(Justify::Right);
            } else if (angle + symbol_angle).abs() >= 180.0
                && angle + symbol_angle < 360.0
                && j == Justify::Right
            {
                justify.push(Justify::Left);
            } else {
                justify.push(j);
            }
        }
        effects.justify = justify;
        let prop_angle = if symbol_angle - angle >= 180.0 {
            symbol_angle - angle - 180.0
        } else {
            symbol_angle - angle
        };
        if !effects.hide {
            plotter.push(text!(get!(property, "at").unwrap(), prop_angle.abs(), value, effects));
        }
    }
    let lib_id: String = get!(node, "lib_id", 0);
    let symbol_unit: usize = get_unit(node).unwrap();
    let lib: Option<&&Sexp> = libs.get(&lib_id);
    if let Some(lib) = lib {
    let units: Vec<&Sexp> = lib.get("symbol").unwrap();
    for _unit in units {
        let unit_number = get_unit(&_unit).unwrap();
        if &unit_number == &0 || &unit_number == &symbol_unit {
            if let Sexp::Node(_, values) = _unit {
                for graph in values {
                    match graph {
                        Sexp::Node(name, _) => {
                            if name == "polyline" {
                                let stroke: Stroke = style
                                    .style(&graph, "stroke", StyleContext::SchemaSymbol)
                                    .unwrap();
                                let fill_color: Option<Color> = style.color(&stroke.fill);
                                let z: usize = if let None = fill_color {
                                    10
                                } else {
                                    1
                                };
                                plotter.push(PlotItem::PolylineItem(z, Polyline::new(
                                    Shape::transform(&node, &get!(graph, "pts").unwrap()),
                                    stroke.color,
                                    stroke.width,
                                    LineType::Default,
                                    fill_color,
                                )));
                            } else if name == "rectangle" {
                                let stroke: Stroke = style
                                    .style(&graph, "stroke", StyleContext::SchemaSymbol)
                                    .unwrap();
                                let fill_color: Option<Color> = style.color(&stroke.fill);
                                let start: Array1<f64> = get!(&graph, "start").unwrap();
                                let end: Array1<f64> = get!(&graph, "end").unwrap();
                                let pts: Array2<f64> =
                                    arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                let z: usize = if let None = fill_color {
                                    10
                                } else {
                                    1
                                };
                                plotter.push(PlotItem::RectangleItem(z, Rectangle::new(
                                    Shape::transform(&node, &pts),
                                    stroke.color,
                                    stroke.width,
                                    stroke.line_type,
                                    fill_color,
                                )));
                            } else if name == "circle" {
                                let stroke: Stroke = style
                                    .style(&graph, "stroke", StyleContext::SchemaSymbol)
                                    .unwrap();
                                let fill_color: Option<Color> = style.color(&stroke.fill);
                                let center: Array1<f64> = get!(&graph, "center").unwrap();
                                let radius: f64 = get!(graph, "radius", 0);
                                let z: usize = if let None = fill_color {
                                    10
                                } else {
                                    1
                                };
                                plotter.push(PlotItem::CircleItem(z, Circle::new(
                                    Shape::transform(&node, &center),
                                    radius,
                                    stroke.width,
                                    stroke.line_type,
                                    stroke.color,
                                    fill_color,
                                )));
                            } else if name == "arc" {
                                let stroke: Stroke = style
                                    .style(&graph, "stroke", StyleContext::SchemaSymbol)
                                    .unwrap();
                                let _fill_color: Option<Color> = style.color(&stroke.fill); //TODO
                                let start: Array1<f64> = get!(&graph, "start").unwrap();
                                let mid: Array1<f64> = get!(&graph, "mid").unwrap();
                                let end: Array1<f64> = get!(&graph, "end").unwrap();

                                let trans = Shape::transform(&node, &start);
                                let arc = Arc::new(
                                    trans,
                                    mid,
                                    end,
                                    stroke.width,
                                    LineType::Default,
                                    stroke.color,
                                    None,
                                );
                                let z: usize = if let None = _fill_color {
                                    10
                                } else {
                                    1
                                };
                                plotter.push(PlotItem::ArcItem(z, arc));
                            } else if name == "pin" {
                                if graph.has("hide") {
                                    break;
                                }
                                let stroke: Stroke = style
                                    .style(&graph, "stroke", StyleContext::SchemaPin)
                                    .unwrap();
                                let pin_pos: Array1<f64> = get!(&graph, "at").unwrap();
                                let length: f64 = get!(graph, "length", 0);
                                let pin_angle: f64 = get!(graph, "at", 2);
                                let pin_line: Array2<f64> = arr2(&[
                                    [pin_pos[0], pin_pos[1]],
                                    [
                                        pin_pos[0] + pin_angle.to_radians().cos() * length,
                                        pin_pos[1] + pin_angle.to_radians().sin() * length,
                                    ],
                                ]);
                                plotter.push(PlotItem::LineItem(10, Line::new(
                                    Shape::transform(&node, &pin_line),
                                    stroke.width,
                                    stroke.line_type,
                                    stroke.color,
                                )));

                                let pin_number: String = get!(graph, "number", 0);
                                let pin_name: String = get!(graph, "name", 0);
                                let show_pin_numbers = if lib.contains("pin_numbers") {
                                    let numbers_hide: String = get!(*lib, "pin_numbers", 0);
                                    numbers_hide != "hide"
                                } else {
                                    true
                                };
                                if show_pin_numbers {
                                    let npos = if pin_angle == 0.0 || pin_angle == 180.0 {
                                        arr1(&[
                                            pin_pos[0]
                                                + pin_angle.to_radians().cos() * length / 2.0,
                                            pin_pos[1] - 1.0,
                                        ])
                                    } else {
                                        arr1(&[
                                            pin_pos[0] - 1.0,
                                            pin_pos[1]
                                                + pin_angle.to_radians().sin() * length / 2.0,
                                        ])
                                    };

                                    let effects = style.effects(&StyleContext::SchemaPinNumber);
                                    plotter.push(text!(Shape::transform(&node, &npos), 0.0, pin_number, effects));
                                }
                                //(pin_names (offset 1.016) hide)
                                let names_offset = if lib.contains("pin_names") {
                                    let pin_names: Vec<&Sexp> = lib.get("pin_names").unwrap();
                                    if pin_names.len() == 1 {
                                        let the_name = pin_names.get(0).unwrap();
                                        let offset: f64 = if the_name.contains("offset") {
                                            get!(*the_name, "offset", 0)
                                        } else {
                                            0.0
                                        };
                                        offset
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                };
                                let names_hide = if lib.contains("pin_names") {
                                    let pin_names: Vec<&Sexp> = lib.get("pin_names").unwrap();
                                    if pin_names.len() == 1 {
                                        let the_name = pin_names.get(0).unwrap();
                                        the_name.has("hide")
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                let name_pos = arr1(&[
                                    pin_pos[0]
                                        + pin_angle.to_radians().cos()
                                            * (length + names_offset * 4.0),
                                    pin_pos[1]
                                        + pin_angle.to_radians().sin()
                                            * (length + names_offset * 4.0),
                                ]);

                                if pin_name != "~" && !names_hide {
                                    plotter.push(PlotItem::TextItem(99, Text::new(
                                        Shape::transform(&node, &name_pos),
                                        0.0,
                                        pin_name,
                                        Color {
                                            r: 0.0,
                                            g: 0.0,
                                            b: 0.0,
                                            a: 1.0,
                                        },
                                        1.25,
                                        "osifont",
                                        vec![Justify::Center],
                                    )));
                                }
                            } else {
                                println!("Unknwon Graph Item: {:?}", graph);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    } else {
        let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]); 
                                plotter.push(PlotItem::RectangleItem(10, Rectangle::new(
                                    Shape::transform(&node, &pts),
                                    Color{r:1.0, g:0.0, b:0.0, a:1.0},
                                    2.0,
                                    LineType::Solid,
                                    None,
                                )));
    }
    Ok(())
}

pub fn plot(
    plotter: &mut dyn Plotter,
    out: Box<dyn Write>,
    sexp_parser: &SexpParser,
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
                    if let Some(v) = v {
                        if let Sexp::Value(size) = v {
                            plotter.paper(size.clone());
                        }
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
            if name == "version"
                || name == "generator"
                || name == "uuid"
                || name == "sheet_instances"
                || name == "symbol_instances"
                || name == "lib_symbols"
                || name == "paper"
                || name == "title_block"
            {
                //just skip those elements
            /* } else if name == "paper" {
                let paper: String = get!(node, 0).unwrap();
                plotter.paper(paper); */
            /* } else if name == "title_block" {
                title_block = Option::from(node.clone()); */
            } else if name == "no_connect" {
                no_connect(node, plotter, &style).unwrap();
            } else if name == "text" {
                text(node, plotter, &style).unwrap();
            } else if name == "wire" {
                let stroke: Stroke = style
                    .style(&node, "stroke", StyleContext::SchemaWire)
                    .unwrap();
                let pts: Array2<f64> = node.get("pts").unwrap();
                plotter.push(PlotItem::LineItem(20, Line::new(
                    pts,
                    stroke.width,
                    stroke.line_type,
                    stroke.color,
                )));
            } else if name == "junction" {
                let stroke: Stroke = style
                    .style(&node, "stroke", StyleContext::SchemaJunction)
                    .unwrap();
                let pos: Array1<f64> = get!(node, "at").unwrap();
                plotter.push(PlotItem::CircleItem(99, Circle::new(
                    pos,
                    0.35,
                    stroke.width,
                    stroke.line_type,
                    stroke.color.clone(),
                    Option::from(stroke.color),
                )));
            } else if name == "label" {
                label(node, plotter, &style).unwrap();
            } else if name == "global_label" {
                global_label(node, plotter, &style).unwrap();
            } else if name == "symbol" {
                symbol(node, &libraries, plotter, &style).unwrap();
            } else {
                println!("node not implemented: {}", name);
            }
        } else {
            panic!("wrong node");
        }
    });

    Ok(plotter.plot(out, border, scale, image_type).unwrap())
}
