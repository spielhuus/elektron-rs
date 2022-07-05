use super::cairo_plotter::{Arc, Circle, Line, PlotItem, Plotter, Polyline, Rectangle, Text};
use super::sexp::{
    Color, Effects, Error, FillType, LineType, SexpConsumer, SexpNode, SexpType, Stroke,
};
use crate::sexp::get::{get, SexpGet};
use crate::sexp::transform::Transform;
use crate::themes::{Style, StyleContext};
use ::std::fmt::{Display, Formatter, Result as FmtResult};
use ndarray::{arr2, Array1, Array2};
use std::fs::File;
use std::io::Write;

/// The Plot struct
///
/// Plot a sexp model to a plotter implementation.
///
pub struct Plot {
    plotter: Box<dyn Plotter>,
    index: u8,
    libraries: std::collections::HashMap<String, SexpNode>,
    style: Style,
    file: Option<String>,
    border: bool,
    scale: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleError {
    value: String,
}
impl StyleError {
    pub fn new(msg: &str) -> StyleError {
        StyleError {
            value: msg.to_string(),
        }
    }
}
impl Display for StyleError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("missing closing quote")
    }
}
impl std::error::Error for StyleError {}

/// Access the nodes and values.
pub trait SexpStyle<S, T> {
    fn style(&self, node: &SexpNode, index: S, context: StyleContext) -> Result<T, StyleError>;
}

impl SexpStyle<&str, Effects> for Plot {
    fn style(
        &self,
        node: &SexpNode,
        key: &str,
        context: StyleContext,
    ) -> Result<Effects, StyleError> {
        let effects: Result<Effects, _> = node.get(key);
        let style_effects = self.style.effects(&context);
        match effects {
            Ok(effects) => {
                let font = if effects.font != "" {
                    effects.font.clone()
                } else {
                    style_effects.font.clone()
                };
                let size = if effects.size != 0.0 {
                    effects.size.clone()
                } else {
                    style_effects.size.clone()
                };
                let thickness = if effects.thickness != 0.0 {
                    effects.thickness.clone()
                } else {
                    style_effects.thickness.clone()
                };
                let line_spacing = if effects.line_spacing != 0.0 {
                    effects.line_spacing
                } else {
                    style_effects.line_spacing
                };
                Ok(Effects::new(
                    font,
                    effects.color.clone(),
                    size,
                    thickness,
                    effects.bold,
                    effects.italic,
                    line_spacing,
                    effects.justify.clone(),
                    effects.hide,
                ))
            }
            _ => Ok(style_effects.clone()),
        }
    }
}
impl SexpStyle<&str, Stroke> for Plot {
    fn style(
        &self,
        node: &SexpNode,
        key: &str,
        context: StyleContext,
    ) -> Result<Stroke, StyleError> {
        let stroke: Result<Stroke, _> = node.get(key);
        let style_stroke = self.style.stroke(&context);
        match stroke {
            Ok(stroke) => Ok(Stroke {
                width: if stroke.width != 0.0 {
                    stroke.width
                } else {
                    style_stroke.width
                },
                line_type: stroke.line_type,
                color: if stroke.color
                    != (Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }) {
                    stroke.color.clone()
                } else {
                    style_stroke.color
                },
                fill: stroke.fill,
            }),
            _ => Ok(style_stroke.clone()),
        }
    }
}

impl SexpConsumer for Plot {
    fn visit(&mut self, node: &SexpNode) -> Result<(), Error> {
        if self.index == 1 && node.name == "symbol" {
            self.libraries.insert(get!(node, 0), node.clone());
        } else if self.index == 0 && node.name == "symbol" {
            for property in &node.nodes("property").unwrap() {
                let effects: Effects = self
                    .style(property, "effects", StyleContext::SchemaProperty)
                    .unwrap();
                let value: String = get!(property, 1);
                if !effects.hide {
                    self.plotter.push(PlotItem::TextItem(Text::new(
                        get!(property, "at"),
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
            let unit: usize = node.unit().unwrap();
            for _unit in &self
                .libraries
                .get(&lib_id)
                .unwrap()
                .nodes("symbol")
                .unwrap()
            {
                let unit_number = _unit.unit().unwrap();
                if &unit_number == &0 || &unit_number == &unit {
                    for graph in &_unit.values {
                        match graph {
                            SexpType::ChildSexpNode(child) => {
                                if &child.name == "polyline" {
                                    let stroke: Stroke = self
                                        .style(child, "stroke", StyleContext::SchemaSymbol)
                                        .unwrap();
                                    let fill_color: Option<Color> = self.style.color(&stroke.fill);
                                    self.plotter.push(PlotItem::PolylineItem(Polyline::new(
                                        node.transform(&get!(child, "pts")),
                                        stroke.color,
                                        stroke.width,
                                        LineType::Default,
                                        fill_color,
                                    )));
                                } else if &child.name == "rectangle" {
                                    let stroke: Stroke = self
                                        .style(child, "stroke", StyleContext::SchemaSymbol)
                                        .unwrap();
                                    let fill_color: Option<Color> = self.style.color(&stroke.fill);
                                    let start: Array1<f64> = get!(child, "start");
                                    let end: Array1<f64> = get!(child, "end");
                                    let pts: Array2<f64> =
                                        arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                    self.plotter.push(PlotItem::RectangleItem(Rectangle::new(
                                        node.transform(&pts),
                                        stroke.color,
                                        stroke.width,
                                        stroke.line_type,
                                        fill_color,
                                    )));
                                } else if &child.name == "circle" {
                                    let stroke: Stroke = self
                                        .style(child, "stroke", StyleContext::SchemaSymbol)
                                        .unwrap();
                                    let fill_color: Option<Color> = self.style.color(&stroke.fill);
                                    let center: Array1<f64> = get!(child, "center");
                                    let radius: f64 = get!(child, "radius", 0);
                                    self.plotter.push(PlotItem::CircleItem(Circle::new(
                                        node.transform(&center),
                                        radius,
                                        stroke.width,
                                        stroke.line_type,
                                        stroke.color,
                                        fill_color,
                                    )));
                                } else if &child.name == "arc" {
                                    let stroke: Stroke = self
                                        .style(child, "stroke", StyleContext::SchemaSymbol)
                                        .unwrap();
                                    let _fill_color: Option<Color> = self.style.color(&stroke.fill); //TODO
                                    let start: Array1<f64> = get!(child, "start");
                                    let mid: Array1<f64> = get!(child, "mid");
                                    let end: Array1<f64> = get!(child, "end");

                                    let trans = node.transform(&start);
                                    let arc = Arc::new(
                                        trans,
                                        mid,
                                        end,
                                        stroke.width,
                                        LineType::Default,
                                        stroke.color,
                                        None,
                                    );
                                    self.plotter.push(PlotItem::ArcItem(arc));
                                } else if &child.name == "pin" {
                                    let stroke: Stroke = self
                                        .style(child, "stroke", StyleContext::SchemaSymbol)
                                        .unwrap();
                                    let pin_pos: Array1<f64> = get!(child, "at");
                                    let length: f64 = get!(child, "length", 0);
                                    let pin_angle: f64 = get!(child, "at", 2);
                                    let pin_line: Array2<f64> = arr2(&[
                                        [pin_pos[0], pin_pos[1]],
                                        [
                                            pin_pos[0] + pin_angle.to_radians().cos() * length,
                                            pin_pos[1] + pin_angle.to_radians().sin() * length,
                                        ],
                                    ]);
                                    self.plotter.push(PlotItem::LineItem(Line::new(
                                        node.transform(&pin_line),
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
        } else if self.index == 0 && node.name == "wire" {
            let stroke: Stroke = self
                .style(node, "stroke", StyleContext::SchemaSymbol)
                .unwrap();
            self.plotter.push(PlotItem::LineItem(Line::new(
                get!(node, "pts"),
                stroke.width,
                stroke.line_type,
                stroke.color,
            )));
        } else if self.index == 0 && node.name == "junction" {
            let stroke: Stroke = self
                .style(node, "stroke", StyleContext::SchemaSymbol)
                .unwrap();
            let fill_color: Option<Color> = self.style.color(&stroke.fill);
            self.plotter.push(PlotItem::CircleItem(Circle::new(
                get!(node, "at"),
                0.3,
                0.1,
                stroke.line_type,
                stroke.color,
                fill_color,
            )));
        } else if self.index == 0 && node.name == "label" {
            let effects: Effects = self
                .style(node, "effects", StyleContext::SchemaProperty)
                .unwrap();
            let _fill_color: Option<Color> = self.style.color(&FillType::Background); //TODO
            let pos: Array1<f64> = get!(&node, "at");
            let _text: String = get!(&node, 0);
            let text = Text::new(
                pos.clone(),
                _text,
                effects.color.clone(),
                effects.size,
                effects.font.as_str(),
                effects.justify,
            );
            self.plotter.push(PlotItem::TextItem(text));
            self.plotter.push(PlotItem::CircleItem(Circle::new(
                pos,
                0.5,
                0.3,
                LineType::Default,
                effects.color,
                None,
            )));
        } else if self.index == 0 && node.name == "global_label" {
            let effects: Effects = self
                .style(node, "effects", StyleContext::SchemaProperty)
                .unwrap();
            let _fill_color: Option<Color> = self.style.color(&FillType::Background); //TODO
            let pos: Array1<f64> = get!(&node, "at");
            let _text: String = get!(&node, 0);
            let text = Text::new(
                pos.clone(),
                _text,
                effects.color.clone(),
                effects.size,
                effects.font.as_str(),
                effects.justify,
            );
            self.plotter.push(PlotItem::TextItem(text));
            self.plotter.push(PlotItem::CircleItem(Circle::new(
                pos,
                0.5,
                0.3,
                LineType::Default,
                effects.color,
                None,
            )));
        } else if self.index == 0 && node.name == "no_connect" {
            let stroke: Stroke = self
                .style(node, "stroke", StyleContext::SchemaSymbol)
                .unwrap();
            let pos: Array1<f64> = get!(&node, "at");
            let lines1 = arr2(&[[-1.0, -1.0], [1.0, 1.0]]) + &pos;
            let lines2 = arr2(&[[1.0, 1.0], [-1.0, -1.0]]) + &pos;

            self.plotter.push(PlotItem::LineItem(Line::new(
                lines1,
                stroke.width.clone(),
                stroke.line_type.clone(),
                stroke.color.clone(),
            )));
            self.plotter.push(PlotItem::LineItem(Line::new(
                lines2,
                stroke.width,
                stroke.line_type,
                stroke.color,
            )));

        //        } else if self.index == 0 && node.name == "text" {
        //            self.text(get!(&node, "at"), get!(&node, 0), self.theme.fill_outline(), self.theme.label_effects());
        } else if self.index == 0 && node.name != "path" {
            println!("uknown element {:?}", &node);
        }
        Ok(())
    }
    fn start_library_symbols(&mut self) -> Result<(), Error> {
        self.index += 1;
        Ok(())
    }
    fn end_library_symbols(&mut self) -> Result<(), Error> {
        self.index -= 1;
        Ok(())
    }
    fn start(&mut self, _: &String, _: &String) -> Result<(), Error> {
        Ok(())
    }
    fn start_sheet_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_sheet_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn start_symbol_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_symbol_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end(&mut self) -> Result<(), Error> {
        let out: Box<dyn Write> = if let Some(filename) = &self.file {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        self.plotter.plot(out, self.border, self.scale);
        Ok(())
    }
}

impl Plot {
    pub fn new(plotter: Box<dyn Plotter>, file: Option<String>, border: bool, scale: f64) -> Self {
        Plot {
            plotter,
            index: 0,
            libraries: std::collections::HashMap::new(),
            style: Style::new(),
            file,
            border,
            scale,
        }
    }
}
