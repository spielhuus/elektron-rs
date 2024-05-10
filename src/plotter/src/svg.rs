//! Draw the model with svglib
use crate::{
    error::Error, Arc, Circle, Color, Draw, Drawer, Line, PlotItem, PlotterImpl, Polyline,
    Rectangle, Text,
};

use itertools::Itertools;
use ndarray::{Array1, Array2};
use std::io::Write;
use log::trace;

use sexp::math::Round;
use svg::{
    node::element::{self, path::Data, Group, Path},
    Document, Node,
};

mod c {
    pub const START: &str = "start";
    pub const END: &str = "end";
    pub const MIDDLE: &str = "middle";
    pub const LEFT: &str = "left";
    pub const RIGHT: &str = "right";
    pub const CENTER: &str = "center";
    pub const HEIGHT: &str = "height";
    pub const WIDTH: &str = "width";
}

/// Plotter implemntation for SVG files.
pub struct SvgPlotter<'a> {
    out: &'a mut dyn Write,
}

impl<'a> SvgPlotter<'a> {
    pub fn new(out: &'a mut dyn Write) -> Self {
        SvgPlotter { out }
    }

    fn radius(p1: &Array1<f64>, p2: &Array1<f64>) -> f64 {
        ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt()
    }

    // Function to calculate angle between center and point
    fn angle(center: &Array1<f64>, point: &Array1<f64>) -> f64 {
        (point[1] - center[1]).atan2(point[0] - center[0])
    }

    // calculate the svg sweep flac from star, middle and end points.
    pub fn sweep_flag(start: &Array1<f64>, mid: &Array1<f64>, end: &Array1<f64>) -> String {
        if (start[0] - mid[0]) * (end[1] - mid[1])
            > (start[1] - mid[1]) * (end[0] - mid[0]) {
            0
        } else {
            1
        }.to_string()
    }
}

impl<'a> PlotterImpl<'a> for SvgPlotter<'a> {
    fn plot(
        &mut self,
        plot_items: &[PlotItem],
        size: Array2<f64>,
        scale: f64,
        name: String,
    ) -> Result<(), Error> {
        let mut document = Document::new()
            .set(
                "viewBox",
                (size[[0, 0]], size[[0, 1]], size[[1, 0]], size[[1, 1]]),
            )
            .set(c::WIDTH, format!("{}mm", (size[[1, 0]]) * scale))
            .set(c::HEIGHT, format!("{}mm", (size[[1, 1]]) * scale));

        let mut g = Group::new().set("id", name.to_string());

        if scale != 1.0 {
            g = g.set("scale", scale);
        }

        self.draw(plot_items, &mut g);
        document.append(g);
        self.out.write_all(document.to_string().as_bytes())?;
        Ok(())
    }
}

impl<'a> Draw<Group> for SvgPlotter<'a> {
    fn draw(&self, items: &[PlotItem], document: &mut Group) {
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

impl<'a> Drawer<Text, Group> for SvgPlotter<'a> {
    fn item(&self, text: &Text, document: &mut Group) {
        let align = if text.effects.justify.contains(&String::from(c::LEFT)) {
            c::START
        } else if text.effects.justify.contains(&String::from(c::RIGHT)) {
            c::END
        } else if text.effects.justify.contains(&String::from(c::CENTER)) {
            c::MIDDLE
        } else if !text.effects.justify.is_empty() {
            c::START
        } else {
            c::MIDDLE
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

        let mut offset = 0.0;
        for line in text.text.split("\\n") {
            let mut t = element::Text::new(line)
                .set(
                    "transform",
                    format!(
                        "translate({},{}) rotate({})",
                        text.pos[0],
                        text.pos[1] + offset,
                        angle
                    ),
                )
                .set("text-anchor", align)
                .set("font-family", text.effects.font_face.to_string())
                .set(
                    "font-size",
                    format!("{}pt", text.effects.font_size.first().unwrap()),
                )
                .set("fill", text.effects.font_color.to_string());
            //.add(node::Text::new(line));

            if text.effects.justify.contains(&"top".to_string()) {
                t = t.set("dominant-baseline", "hanging");
            } else if !text.effects.justify.contains(&"bottom".to_string()) {
                t = t.set("dominant-baseline", "middle");
            }

            if let Some(cls) = &text.class {
                t = t.set("class", cls.as_str());
            }
            document.append(t);
            offset += text.effects.font_size.first().unwrap() + 0.3;
        }
    }
}

impl<'a> Drawer<Line, Group> for SvgPlotter<'a> {
    fn item(&self, line: &Line, document: &mut Group) {
        let data = Data::new()
            .move_to((line.pts[[0, 0]], line.pts[[0, 1]]))
            .line_to((line.pts[[1, 0]], line.pts[[1, 1]]));

        let mut path = Path::new()
            .set("stroke", line.stroke.linecolor.to_string())
            .set("stroke-width", line.stroke.linewidth)
            .set("d", data);

        if let Some(cls) = &line.class {
            path = path.set("class", cls.as_str());
        }
        if let Some(linecap) = &line.linecap {
            trace!("set linecap: {}", linecap);
            path = path.set("stroke-linecap", linecap.to_string());
        }
        document.append(path);
    }
}

impl<'a> Drawer<Polyline, Group> for SvgPlotter<'a> {
    fn item(&self, line: &Polyline, document: &mut Group) {
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

        if let Some(cls) = &line.class {
            path = path.set("class", cls.as_str());
        }
        document.append(path);
    }
}

impl<'a> Drawer<Rectangle, Group> for SvgPlotter<'a> {
    fn item(&self, rectangle: &Rectangle, document: &mut Group) {
        let mut x0 = rectangle.pts[[0, 0]];
        let mut y0 = rectangle.pts[[0, 1]];
        let mut x1 = rectangle.pts[[1, 0]];
        let mut y1 = rectangle.pts[[1, 1]];

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if y0 > y1 {
            std::mem::swap(&mut y0, &mut y1);
        }

        let fill = if matches!(rectangle.stroke.fillcolor, Color::None) {
            "none".to_string()
        } else {
            rectangle.stroke.fillcolor.to_string()
        };

        let mut rect = element::Rectangle::new()
            .set("x", x0)
            .set("y", y0)
            .set("width", x1 - x0)
            .set("height", y1 - y0)
            .set("fill", fill)
            .set("stroke", rectangle.stroke.linecolor.to_string())
            .set("stroke-width", rectangle.stroke.linewidth);

        if let Some(rx) = &rectangle.rx {
            rect = rect.set("rx", rx.to_string());
        }

        if let Some(cls) = &rectangle.class {
            rect = rect.set("class", cls.as_str());
        }

        document.append(rect);
    }
}

impl<'a> Drawer<Circle, Group> for SvgPlotter<'a> {
    fn item(&self, circle: &Circle, document: &mut Group) {
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
        if let Some(cls) = &circle.class {
            c = c.set("class", cls.as_str());
        }
        document.append(c);
    }
}

impl<'a> Drawer<Arc, Group> for SvgPlotter<'a> {
    fn item(&self, arc: &Arc, document: &mut Group) {

        let radius = Self::radius(&arc.center, &arc.start);
        let mut start_angle = Self::angle(&arc.center, &arc.start);
        let mut end_angle = Self::angle(&arc.center, &arc.end);
        let sweep_flag = Self::sweep_flag(&arc.start, &arc.mid, &arc.end);

        if arc.angle != 0.0 {
            start_angle += arc.angle;
            end_angle += arc.angle;
        }

        let large_arc_flag = if end_angle - start_angle > 180.0 {
            "1"
        } else {
            "0"
        };

        if !matches!(arc.stroke.fillcolor, Color::None) {
            let mut c = Path::new()
                .set(
                    "d",
                    format!(
                        "M{:.2} {:.2} A{:.2} {:.2} 0.0 {} {} {:.2} {:.2}",
                        arc.start[0],
                        arc.start[1],
                        radius,
                        radius,
                        large_arc_flag,
                        sweep_flag,
                        arc.end[0],
                        arc.end[1],
                    ),
                )
                .set("fill", arc.stroke.fillcolor.to_string());

            if let Some(cls) = &arc.class {
                c = c.set("class", cls.as_str());
            }
            document.append(c);
        }

        let mut c = Path::new()
            .set(
                "d",
                format!(
                    "M{:.2} {:.2} A{:.2} {:.2} 0.0 {} {} {:.2} {:.2}",
                    arc.start[0],
                    arc.start[1],
                    radius,
                    radius,
                    large_arc_flag,
                    sweep_flag,
                    arc.end[0],
                    arc.end[1]
                ),
            )
            .set("fill", "none")
            .set("stroke", arc.stroke.linecolor.to_string())
            .set("stroke-width", arc.stroke.linewidth.rnd());

        if let Some(cls) = &arc.class {
            c = c.set("class", cls.as_str());
        }
        document.append(c);
    }
}

#[cfg(test)]
mod test {
    use ndarray::arr1;
    use::sexp::math::Round;

    #[test]
    fn radius() {
        assert_eq!(
            1.36,
            super::SvgPlotter::radius(&arr1(&[84.76, 126.74]), &arr1(&[86.12, 126.74])).rnd()
        );
        assert_eq!(
            1.36,
            super::SvgPlotter::radius(&arr1(&[86.12, 126.74]), &arr1(&[87.48, 126.74])).rnd()
        );
    }

    #[test]
    fn angle() {
        assert_eq!(
            std::f64::consts::PI.rnd(),
            super::SvgPlotter::angle(&arr1(&[86.12, 126.74]), &arr1(&[84.76, 126.74])).rnd()
        );
        assert_eq!(
            0.0,
            super::SvgPlotter::angle(&arr1(&[86.12, 126.74]), &arr1(&[87.48, 126.74])).rnd()
        );
    }
}
