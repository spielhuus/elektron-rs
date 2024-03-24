//! Draw the model with svglib
use crate::{
    error::Error, no_fill, themer::Themer, Arc, Circle, Draw, Drawer, FillType, Line, Outline,
    PlotItem, PlotterImpl, Polyline, Rectangle, Style, Text, Theme,
};

use sexp::{el, PaperSize, Sexp, SexpProperty, SexpTree, SexpValueQuery};
use simulation::Netlist;

use itertools::Itertools;
use ndarray::arr2;
use std::collections::HashMap;
use std::io::Write;

use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::Node;
use svg::{node, node::element, Document};

use log::{debug, log_enabled, Level};

/// Plotter implemntation for SVG files.
pub struct SvgPlotter<'a> {
    name: &'a str,
    themer: Option<Themer<'a>>,
}

impl<'a> SvgPlotter<'a> {
    pub fn new(name: &'a str, theme: Option<Themer<'a>>) -> Self {
        SvgPlotter {
            name,
            themer: theme,
        }
    }
}

impl Outline for SvgPlotter<'_> {}

impl<'a> PlotterImpl<'a, SexpTree> for SvgPlotter<'a> {
    fn plot<W: Write>(
        &self,
        schema: &SexpTree,
        out: &mut W,
        border: bool,
        scale: f64,
        pages: Option<Vec<usize>>,
        netlist: bool,
    ) -> Result<(), Error> {
        //load the netlist
        let netlist = if netlist {
            Some(Netlist::from(schema).unwrap())
        } else {
            None
        };

        //collect all the sheets
        let mut schema_pages: HashMap<usize, String> = HashMap::new();
        let sheet_instance = schema.root().unwrap().query(el::SHEET_INSTANCES).next();
        if let Some(sheet_instance) = sheet_instance {
            for page in sheet_instance.query("path") {
                let path: String = page.get(0).unwrap();
                let number: usize = page.value("page").unwrap();
                schema_pages.insert(number, path);
            }
        } else {
            schema_pages.insert(1, String::from("/"));
        }
        for page in schema.root().unwrap().query("sheet") {
            let sheetfile: Sexp = page.property("Sheetfile").unwrap();
            let path: String = sheetfile.get(1).unwrap();
            let instances = page.query("instances").next().unwrap();
            let project = instances.query("project").next().unwrap();
            let sheetpath = project.query("path").next().unwrap();
            let number: usize = sheetpath.value("page").unwrap();
            schema_pages.insert(number, path);
        }

        //and finally plot the pages.
        for page in schema_pages.iter().sorted() {
            if log_enabled!(Level::Info) {
                debug!("plot page {} '{}'", page.0, page.1);
            }
            if pages.as_ref().is_none() || pages.as_ref().unwrap().contains(page.0) {
                let document = if border {
                    let paper_size: (f64, f64) =
                        <Sexp as SexpValueQuery<PaperSize>>::value(schema.root().unwrap(), "paper")
                            .unwrap()
                            .into();

                    let plot_items = crate::schema::plot(schema, &netlist, Some(paper_size));

                    let mut document = Document::new()
                        .set("viewBox", (0, 0, paper_size.0, paper_size.1))
                        .set("width", format!("{}mm", paper_size.0))
                        .set("height", format!("{}mm", paper_size.1));
                    let mut g = element::Group::new();
                    g = g.set("id", self.name);
                    if scale != 1.0 {
                        g = g.set("scale", scale);
                    }
                    self.draw(&plot_items, &mut g);
                    document.append(g);
                    if let Some(themer) = &self.themer {
                        document.append(element::Style::new(
                            themer.css(), /* TODO             format!(
                                          "<![CDATA[\n{}\n]]>",
                                          themer.css() */
                        ));
                    }
                    document
                } else {
                    let plot_items = crate::schema::plot(schema, &netlist, None);

                    let size = self.bounds(
                        &plot_items,
                        self.themer
                            .as_ref()
                            .unwrap_or(&Themer::new(Theme::Kicad2020)),
                    ) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    let mut document = Document::new()
                        .set(
                            "viewBox",
                            (
                                size[[0, 0]],
                                size[[0, 1]],
                                size[[1, 0]] - size[[0, 0]],
                                size[[1, 1]] - size[[0, 1]],
                            ),
                        )
                        .set("width", (size[[1, 0]] - size[[0, 0]]) * scale)
                        .set("height", (size[[1, 1]] - size[[0, 1]]) * scale);
                    let mut g = element::Group::new().set("id", self.name);
                    self.draw(&plot_items, &mut g);
                    if let Some(themer) = &self.themer {
                        document.append(element::Style::new(themer.css()));
                        /*  TODO  "<![CDATA[\n{}\n]]>",
                            themer.css()
                        ))); */
                    }
                    document.append(g);
                    document
                };
                out.write_all(document.to_string().as_bytes())?;
            }
        }
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
            .set("class", text.class.iter().map(|i| i.to_string()).join(" "))
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
        // .close();
        let mut path = Path::new()
            //.set("fill", "none")
            .set(
                "class",
                line.class
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            )
            .set("d", data);
        let mut style = Vec::new();
        if line.stroke.linewidth != 0.0 {
            style.push(format!("stroke-width:{};", line.stroke.linewidth));
        }
        if let Some(linecap) = &line.linecap {
            style.push(format!("stroke-linecap:{};", linecap));
        }
        if !style.is_empty() {
            path = path.set("style", style.join(" "));
        }
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
            .set("class", line.class.iter().map(|i| i.to_string()).join(" "))
            .set("d", data);

        if no_fill(&line.class) {
            path = path.set("fill", "none");
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
        let path = Path::new()
            .set("fill", "none")
            .set(
                "class",
                rectangle.class.iter().map(|i| i.to_string()).join(" "),
            )
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
            .set(
                "class",
                circle.class.iter().map(|i| i.to_string()).join(" "),
            );
        if circle.stroke.linewidth != 0.0 {
            c = c.set(
                "style",
                format!("stroke-width: {};", circle.stroke.linewidth),
            );
        }
        if no_fill(&circle.class) {
            c = c.set("fill", "none");
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

        let mut fill = None;
        for cls in &arc.class {
            if let Style::Fill(f) = cls {
                if *f != FillType::NoFill {
                    fill = Some(cls);
                }
            }
        }
        if let Some(fill) = fill {
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
                .set("class", fill.to_string());
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
            .set("class", arc.class.iter().map(|i| i.to_string()).join(" "));

        if arc.stroke.linewidth != 0.0 {
            c = c.set("stroke-width", arc.stroke.linewidth);
        }
        document.append(c);
    }
}

#[cfg(test)]
mod tests {}
