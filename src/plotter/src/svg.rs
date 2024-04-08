//! Draw the model with svglib
use crate::Color;
use crate::{
    error::Error, Arc, Circle, Draw, Drawer, Line, PlotItem, PlotterImpl, Polyline, Rectangle, Text,
};

use itertools::Itertools;
use ndarray::Array2;
use std::io::Write;

use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::Node;
use svg::{node, node::element, Document};

/// Plotter implemntation for SVG files.
pub struct SvgPlotter<'a> {
    name: &'a str,
    out: &'a mut dyn Write,
    scale: f64,
}

impl<'a> SvgPlotter<'a> {
    pub fn new(out: &'a mut dyn Write) -> Self {
        SvgPlotter {
            name: "",
            out,
            scale: 1.0,
        }
    }
}

impl<'a> PlotterImpl<'a> for SvgPlotter<'a> {
    fn plot(&mut self, plot_items: &[PlotItem], size: Array2<f64>) -> Result<(), Error> {
        let mut document = Document::new()
            .set(
                "viewBox",
                (size[[0, 0]], size[[0, 1]], size[[1, 0]], size[[1, 1]]),
            )
            .set("width", format!("{}mm", (size[[1, 0]]) * self.scale))
            .set("height", format!("{}mm", (size[[1, 1]]) * self.scale));

        let mut g = element::Group::new().set("id", self.name);
        self.draw(plot_items, &mut g);
        document.append(g);
        self.out.write_all(document.to_string().as_bytes())?;
        Ok(())
    }
}

impl<'a> Draw<element::Group> for SvgPlotter<'a> {
    fn draw(&self, items: &[PlotItem], document: &mut element::Group) {
        items
            .iter()
            .sorted_by(|a, b| {
                let za = match a {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };
                let zb = match b {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };

                Ord::cmp(&za, &zb)
            })
            .for_each(|item| match item {
                PlotItem::Arc(_, arc) => self.item(arc, document),
                PlotItem::Circle(_, circle) => self.item(circle, document),
                PlotItem::Line(_, line) => self.item(line, document),
                PlotItem::Rectangle(_, rectangle) => self.item(rectangle, document),
                PlotItem::Polyline(_, line) => self.item(line, document),
                PlotItem::Text(_, text) => self.item(text, document),
            });
    }
}

impl<'a> Drawer<Text, element::Group> for SvgPlotter<'a> {
    fn item(&self, text: &Text, document: &mut element::Group) {
        let align = if text.effects.justify.contains(&String::from("left")) {
            "start"
        } else if text.effects.justify.contains(&String::from("right")) {
            "end"
        } else if text.effects.justify.contains(&String::from("center")) {
            "middle"
        } else if !text.effects.justify.is_empty() {
            "start"
        } else {
            "middle"
        };
        let angle = if text.angle >= 180.0 {
            text.angle - 180.0
        } else {
            text.angle
        };
        //svg rotates are reversed
        let angle = if angle == 90.0 {
            -90.0
        } else if angle == 270.0 {
            90.0
        } else {
            angle
        };
        let mut t = element::Text::new()
            .set(
                "transform",
                format!(
                    "translate({},{}) rotate({})",
                    text.pos[0], text.pos[1], angle
                ),
            )
            .set("text-anchor", align)
            .set("font-family", text.effects.font_face.to_string())
            .set(
                "font-size",
                format!("{}pt", text.effects.font_size.first().unwrap()),
            )
            .set("fill-color", text.effects.font_color.to_string())
            .add(node::Text::new(text.text.clone()));

        if text.effects.justify.contains(&"top".to_string()) {
            t = t.set("dominant-baseline", "hanging");
        } else if !text.effects.justify.contains(&"bottom".to_string()) {
            t = t.set("dominant-baseline", "middle");
        }
        document.append(t);
    }
}

impl<'a> Drawer<Line, element::Group> for SvgPlotter<'a> {
    fn item(&self, line: &Line, document: &mut element::Group) {
        let data = Data::new()
            .move_to((line.pts[[0, 0]], line.pts[[0, 1]]))
            .line_to((line.pts[[1, 0]], line.pts[[1, 1]]));
        let path = Path::new()
            .set("stroke", line.stroke.linecolor.to_string())
            .set("stroke-width", line.stroke.linewidth)
            .set("d", data);

        /*TODO if let Some(linecap) = &line.linecap {
            style.push(format!("stroke-linecap:{};", linecap));
        } */
        document.append(path);
    }
}

impl<'a> Drawer<Polyline, element::Group> for SvgPlotter<'a> {
    fn item(&self, line: &Polyline, document: &mut element::Group) {
        let mut data = Data::new();
        let mut first: bool = true;
        for pos in line.pts.rows() {
            if first {
                data = data.move_to((pos[0], pos[1]));
                first = false;
            } else {
                data = data.line_to((pos[0], pos[1]));
            }
        }
        // data = data.close();
        let mut path = Path::new()
            .set("stroke", line.stroke.linecolor.to_string())
            .set("stroke-width", line.stroke.linewidth)
            .set("d", data);

        if matches!(line.stroke.fillcolor, Color::None) {
            path = path.set("fill", "none");
        } else {
            path = path.set("fill", line.stroke.fillcolor.to_string());
        }
        document.append(path);
    }
}

impl<'a> Drawer<Rectangle, element::Group> for SvgPlotter<'a> {
    fn item(&self, rectangle: &Rectangle, document: &mut element::Group) {
        let data = Data::new()
            .move_to((rectangle.pts[[0, 0]], rectangle.pts[[0, 1]]))
            .line_to((rectangle.pts[[1, 0]], rectangle.pts[[0, 1]]))
            .line_to((rectangle.pts[[1, 0]], rectangle.pts[[1, 1]]))
            .line_to((rectangle.pts[[0, 0]], rectangle.pts[[1, 1]]))
            .line_to((rectangle.pts[[0, 0]], rectangle.pts[[0, 1]]))
            .close();

        let fill = if matches!(rectangle.stroke.fillcolor, Color::None) {
            "none".to_string()
        } else {
            rectangle.stroke.fillcolor.to_string()
        };

        let path = Path::new()

            .set("fill", fill)
            .set("stroke", rectangle.stroke.linecolor.to_string())
            .set("stroke-width", rectangle.stroke.linewidth)
            .set("d", data);
        document.append(path);
    }
}

impl<'a> Drawer<Circle, element::Group> for SvgPlotter<'a> {
    fn item(&self, circle: &Circle, document: &mut element::Group) {
        let mut c = element::Circle::new()
            .set("cx", circle.pos[0])
            .set("cy", circle.pos[1])
            .set("r", circle.radius)
            .set("stroke", circle.stroke.linecolor.to_string())
            .set("stroke-width", circle.stroke.linewidth);

        if matches!(circle.stroke.fillcolor, Color::None) {
            c = c.set("fill", "none");
        } else {
            c = c.set("fill", circle.stroke.fillcolor.to_string());
        }
        document.append(c);
    }
}

impl<'a> Drawer<Arc, element::Group> for SvgPlotter<'a> {
    fn item(&self, arc: &Arc, document: &mut element::Group) {
        let mut theta1 = arc.start_angle.to_radians();

        if theta1 < 0.0 {
            theta1 += std::f64::consts::PI * 2.0;
        }

        let mut theta2 = arc.end_angle.to_radians();

        if theta2 < 0.0 {
            theta2 += std::f64::consts::PI * 2.0;
        }

        if theta2 < theta1 {
            theta2 = std::f64::consts::PI * 2.0;
        }

        // flag for large or small arc. 0 means less than 180 degrees
        let flg_arc = if (theta2 - theta1).abs() > std::f64::consts::PI {
            1.0
        } else {
            0.0
        };

        if matches!(arc.stroke.fillcolor, Color::None) {
            let c = element::Path::new()
                .set(
                    "d",
                    format!(
                        "M{} {} A{} {} 0.0 {} {} {} {} L {} {} Z",
                        arc.start[0],
                        arc.start[1],
                        arc.radius,
                        arc.radius,
                        flg_arc,
                        0,
                        arc.end[0],
                        arc.end[1],
                        arc.center[0],
                        arc.center[1]
                    ),
                )
                .set("fill", arc.stroke.fillcolor.to_string());
            document.append(c);
        }
        let mut c = element::Path::new()
            .set(
                "d",
                format!(
                    "M{} {} A{} {} 0.0 {} {} {} {}",
                    arc.start[0],
                    arc.start[1],
                    arc.radius,
                    arc.radius,
                    flg_arc,
                    0,
                    arc.end[0],
                    arc.end[1]
                ),
            )
            .set("fill", "none")
            .set("stroke", arc.stroke.linecolor.to_string())
            .set("stroke-width", arc.stroke.linewidth);

        if arc.stroke.linewidth != 0.0 {
            c = c.set("stroke-width", arc.stroke.linewidth);
        }
        document.append(c);
    }
}
