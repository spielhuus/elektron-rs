use std::fs::File;

use crate::shape::{Shape, Transform};
use crate::sexp::Sexp;
use crate::sexp::SexpParser;
use crate::sexp::Get;
use crate::sexp::{Effects, LineType, FillType, Stroke, get, Color, get_unit};
use crate::themes::StyleTypes;
use crate::themes::{Style, StyleContext};
use crate::Error;
use crate::cairo_plotter::{Arc, Circle, Line, PlotItem, Plotter, Polyline, Rectangle, Text};
use ndarray::{arr2, Array1, Array2};

pub fn text(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {

    Ok(())
}

pub fn no_connect(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let stroke: Stroke = style
        .style(&node, "stroke", StyleContext::SchemaSymbol)
        .unwrap();
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let lines1 = arr2(&[[-1.0, -1.0], [1.0, 1.0]]) + &pos;
    let lines2 = arr2(&[[1.0, 1.0], [-1.0, -1.0]]) + &pos;

    plotter.push(PlotItem::LineItem(Line::new(
        lines1,
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(Line::new(
        lines2,
        stroke.width,
        stroke.line_type,
        stroke.color,
    )));

    Ok(())
}

pub fn label(node: &Sexp, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {
    let effects: Effects =
        style.style(&node, "effects", StyleContext::SchemaProperty)
        .unwrap();
    let _fill_color: Option<Color> = style.color(&FillType::Background); //TODO
    let pos: Array1<f64> = get!(node, "at").unwrap();
    let _text: String = get!(node, 0).unwrap();
    let text = Text::new(
        pos.clone(),
        _text,
        effects.color.clone(),
        effects.size,
        effects.font.as_str(),
        effects.justify,
    );
    plotter.push(PlotItem::TextItem(text));
    plotter.push(PlotItem::CircleItem(Circle::new(
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

    plotter.push(PlotItem::LineItem(Line::new(
        lines1,
        stroke.width.clone(),
        stroke.line_type.clone(),
        stroke.color.clone(),
    )));
    plotter.push(PlotItem::LineItem(Line::new(
        lines2,
        stroke.width,
        stroke.line_type,
        stroke.color,
    )));

    Ok(())
}

pub fn symbol(node: &Sexp, libs: &std::collections::HashMap<String, &Sexp>, plotter: &mut dyn Plotter, style: &Style) -> Result<(), Error> {

    let properties: Vec<&Sexp> = node.get("property").unwrap();
    for property in properties {
        let effects: Effects = 
            style.style(property, "effects", StyleContext::SchemaProperty)
            .unwrap();
        let value: String = get!(property, 1).unwrap();
        if !effects.hide {
            plotter.push(PlotItem::TextItem(Text::new(
                get!(property, "at").unwrap(), //.get(0).unwrap(),
                value,
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                effects.size,
                effects.font.as_str(),
                effects.justify,
            )));
        }
    }
    let lib_id: String = get!(node, "lib_id", 0);
    let symbol_unit: usize = get_unit(node).unwrap();
    let lib: &Sexp = &libs.get(&lib_id).unwrap();
    let units: Vec<&Sexp> = lib.get("symbol").unwrap();
    for _unit in units {
        let unit_number = get_unit(&_unit).unwrap();
        if &unit_number == &0 || &unit_number == &symbol_unit {
            if let Sexp::Node(_, values) = _unit {
            for graph in values {
                match graph {
                    Sexp::Node(name, values) => {
                        if name == "polyline" {
                            let stroke: Stroke = style
                                .style(&graph, "stroke", StyleContext::SchemaSymbol)
                                .unwrap();
                            let fill_color: Option<Color> = style.color(&stroke.fill);
                            plotter.push(PlotItem::PolylineItem(Polyline::new(
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
                            plotter.push(PlotItem::RectangleItem(Rectangle::new(
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
                            plotter.push(PlotItem::CircleItem(Circle::new(
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
                            plotter.push(PlotItem::ArcItem(arc));
                        } else if name == "pin" {
                            let stroke: Stroke = style
                                .style(&graph, "stroke", StyleContext::SchemaSymbol)
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
                            plotter.push(PlotItem::LineItem(Line::new(
                                Shape::transform(&node, &pin_line),
                                stroke.width,
                                stroke.line_type,
                                stroke.color,
                            )));

                            // drae the pin number
                            //                                    let offset = arr1(&[0.0, -0.5]);
                            //                                    let offset = node.transform(&arr1(&[0.0, 0.0]), &offset, &pin_angle, &mirror);
                            //                                    let pin_number: Array1<f64> = arr1(&[
                            //                                        pin_pos[0] + pin_angle.to_radians().cos() * length / 2.,
                            //                                        pin_pos[1] + pin_angle.to_radians().sin() * length / 2.
                            //                                    ]) + offset;
                            //                                    let trans = node.transform(&pos, &pin_number, &angle, &mirror);
                            //                                    let circle = Circle::new(trans, 0.3, 0.2, LineType::Default, Color{r: 1.0, g: 0.0, b: 0.0, a: 0.0}, None);
                            //                                    self.plotter.push(PlotItem::CircleItem(circle));

                            //                                    number_pos = transform(symbol,
                            //                                                           (pin_pos[0] + (np.cos(math.radians(pin.angle)) * pin.length / 2),
                            //                                                            pin_pos[1] + (np.sin(math.radians(pin.angle)) * pin.length / 2)))
                            //
                            //                                    name_pos = transform(symbol,
                            //                                                         (pin_pos[0] + (np.cos(math.radians(pin.angle)) * (pin.length + 1)),
                            //                                                          pin_pos[1] + (np.sin(math.radians(pin.angle)) * (pin.length + 1))))
                            //
                            //                                    stroke = _merge_stroke(None, cast(StrokeDefinition, self.theme['pin']))
                            //                                    plotter.line(SchemaPlot._pos(pin_line, self.offset, self.scale),
                            //                                                 stroke['width']*self.scale, stroke['color'])
                            //
                            //                                    number_effects = _merge_text_effects(
                            //                                            pin.number[1], cast(TextEffects, self.theme['pin_number']))
                            //                                    name_effects = _merge_text_effects(
                            //                                            pin.name[1], cast(TextEffects, self.theme['pin_name']))
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
    Ok(())
}

fn libraries(sexp_parser: &SexpParser) -> Result<std::collections::HashMap<String, &Sexp>, Error> {
   let mut libraries: std::collections::HashMap<String, &Sexp> = std::collections::HashMap::new();
   for element in sexp_parser.values() {
       if let Sexp::Node(name, values) = element {
           if name == "lib_symbols" {
               for value in values {
                   let name: String = value.get(0).unwrap();
                   libraries.insert(String::from(name), value);
               }
           }
       } //TODO create error
   }
   Ok(libraries)
}

pub fn plot(plotter: &mut dyn Plotter, filename: Option<&str>, sexp_parser: &SexpParser, border: bool, style: Style) -> Result<(), Error> {

    let libraries = libraries(sexp_parser).unwrap();

    sexp_parser.values()
        .for_each(|node| {
            if let Sexp::Node(name, _) = node {
                if name == "paper" || name == "version" || name == "generator" || 
                   name == "uuid" || name == "sheet_instances" || 
                   name == "symbol_instances" || name == "lib_symbols" {
                    //just skip those elements
                } else if name == "title_block" {
                    //TODO
                } else if name == "no_connect" {
                    no_connect(node, plotter, &style).unwrap();
                } else if name == "text" {
                    //TODO
                } else if name == "wire" {
                    let stroke: Stroke = 
                        style.style(&node, "stroke", StyleContext::SchemaSymbol)
                        .unwrap();
                    let pts: Array2<f64> = node.get("pts").unwrap();
                    plotter.push(
                        PlotItem::LineItem(Line::new(pts, stroke.width, stroke.line_type, stroke.color)));
                } else if name == "junction" {
                    let stroke: Stroke = style
                        .style(&node, "stroke", StyleContext::SchemaSymbol)
                        .unwrap();
                    let fill_color: Option<Color> = style.color(&stroke.fill);
                    let pos: Array1<f64> = get!(node, "at").unwrap();
                    plotter.push(PlotItem::CircleItem(Circle::new(
                        pos,
                        0.3,
                        0.1,
                        stroke.line_type,
                        stroke.color,
                        fill_color,
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
    
    let mut file = Box::from(File::create(filename.unwrap()).unwrap());
    plotter.plot(file, border, 1.0);
    Ok(())
}
