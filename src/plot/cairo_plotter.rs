use super::{
    plotter::{
        Arc, Circle, Draw, Drawer, ImageType, Line, Outline, PlotItem, PlotterImpl, Polyline,
        Rectangle, Text, Theme,
    },
    theme::Themer,
};
use crate::error::Error;
use crate::sexp::{Pcb, Schema};
use crate::spice::Netlist;
use itertools::Itertools;
use ndarray::{arr2, Array2};
use pangocairo::{create_layout, pango::SCALE, show_layout, update_layout};
use std::io::Write;
extern crate cairo;
use cairo::{Context, Format, ImageSurface, PdfSurface, SvgSurface};

fn rgba_color(color: (f64, f64, f64, f64)) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        (color.0 * 255.0) as u32,
        (color.1 * 255.0) as u32,
        (color.2 * 255.0) as u32,
        (color.3 * 255.0) as u32
    )
}

pub mod paper {
    pub const A4: (f64, f64) = (297.0, 210.0);
}

macro_rules! color {
    ($element:expr, $themer:expr) => {
        if let Some(color) = $element.color {
            if color != (0.0, 0.0, 0.0, 0.0) {
                color
            } else {
                $themer.stroke(&$element.class)
            }
        } else {
            $themer.stroke(&$element.class)
        }
    };
}

macro_rules! stroke {
    ($context:expr, $element:expr, $themer:expr) => {
        let color = color!($element, $themer);
        $context.set_source_rgba(color.0, color.1, color.2, color.3);
        $context.set_line_width($themer.stroke_width($element.width, &$element.class));
    };
}
macro_rules! fill {
    ($context:expr, $element:expr, $themer:expr) => {
        let fill = $themer.fill(&$element.class);
        if let Some(fill) = fill {
            $context.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
            $context.fill().unwrap();
        }
    };
}

/// Plotter implemntation for SVG and PDF file.
pub struct CairoPlotter<'a> {
    context: Context,
    paper_size: (f64, f64),
    image_type: ImageType,
    themer: Themer<'a>,
}

impl<'a> CairoPlotter<'a> {
    pub fn new(image_type: ImageType, theme: Theme) -> CairoPlotter<'a> {
        let surface = ImageSurface::create(
            Format::Rgb24,
            (297.0 * 72.0 / 25.4) as i32,
            (210.0 * 72.0 / 25.4) as i32,
        )
        .unwrap();
        let context = Context::new(&surface).unwrap();
        context.scale(72.0 / 25.4, 72.0 / 25.4);
        CairoPlotter {
            context,
            paper_size: paper::A4,
            image_type,
            themer: Themer::new(theme),
        }
    }
}


impl<'a> PlotterImpl<'a, Schema> for CairoPlotter<'a> {
    fn plot<W: Write + 'static>(
        &self,
        schema: &Schema,
        out: &mut W,
        border: bool,
        scale: f64,
        pages: Option<Vec<usize>>,
        netlist: bool,
    ) -> Result<(), Error> {
        use super::schema::PlotIterator;
        let netlist = if netlist {
            Some(Netlist::from(schema).unwrap())
        } else {
            None
        };
        for page in 0..schema.pages() {
            if pages.as_ref().is_none() || pages.as_ref().unwrap().contains(&page) {
                if border {
                    let paper_size: (f64, f64) =
                        schema.page(page).unwrap().paper_size.clone().into();
                    let title_block = &schema.page(page).unwrap().title_block;
                    let iter = schema
                        .iter(page)?
                        .plot(schema, title_block, paper_size, border, &netlist)
                        .flatten()
                        .collect();
                    match self.image_type {
                        ImageType::Svg => unsafe {
                            let surface = SvgSurface::for_raw_stream(
                                self.paper_size.0 * 96.0 / 25.4,
                                self.paper_size.1 * 96.0 / 25.4,
                                out,
                            )
                            .unwrap();
                            let mut context = Context::new(&surface).unwrap();
                            context.scale(96.0 / 25.4, 96.0 / 25.4);
                            self.draw(&iter, &mut context);
                            surface.finish_output_stream().unwrap();
                        },
                        ImageType::Png => {
                            let surface = ImageSurface::create(
                                Format::Rgb24,
                                (self.paper_size.0 * 96.0 / 25.4) as i32,
                                (self.paper_size.1 * 96.0 / 25.4) as i32,
                            )
                            .unwrap();
                            let mut context = Context::new(&surface).unwrap();
                            context.scale(96.0 / 25.4, 96.0 / 25.4);
                            context.save()?;
                            context.set_source_rgb(0.0, 0.0, 0.0); //TODO: get from css
                            context.paint()?;
                            context.restore()?;
                            self.draw(&iter, &mut context);
                            surface.write_to_png(out)?;
                        }
                        ImageType::Pdf => unsafe {
                            let surface = PdfSurface::for_raw_stream(
                                self.paper_size.0 * 96.0 / 25.4,
                                self.paper_size.1 * 96.0 / 25.4,
                                out,
                            )
                            .unwrap();
                            let mut context = Context::new(&surface).unwrap();
                            context.scale(96.0 / 25.4, 96.0 / 25.4);
                            self.draw(&iter, &mut context);
                            surface.finish_output_stream().unwrap();
                        },
                    }
                } else {
                    let iter = schema
                        .iter(page)?
                        .plot(schema, &None, (0.0, 0.0), border, &netlist)
                        .flatten()
                        .collect();
                    let size =
                        self.bounds(&iter, &self.themer) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    match self.image_type {
                        ImageType::Svg => {
                            let size = self.bounds(&iter, &self.themer)
                                + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                            unsafe {
                                let surface = SvgSurface::for_raw_stream(
                                    (size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale,
                                    (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale,
                                    out,
                                )?;
                                let mut context = Context::new(&surface)?;
                                context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                                context.translate(-size[[0, 0]], -size[[0, 1]]);
                                self.draw(&iter, &mut context);
                                surface.finish_output_stream().unwrap();
                            }
                        }
                        ImageType::Png => {
                            let size = self.bounds(&iter, &self.themer)
                                + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                            let surface = ImageSurface::create(
                                Format::Rgb24,
                                ((size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale) as i32,
                                ((size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale) as i32,
                            )?;
                            let mut context = Context::new(&surface)?;
                            context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                            context.translate(-size[[0, 0]], -size[[0, 1]]);
                            context.save()?;
                            context.set_source_rgb(1.0, 1.0, 1.0); //TODO: get from css
                            context.paint()?;
                            context.restore()?;
                            self.draw(&iter, &mut context);
                            surface.write_to_png(out)?;
                        }
                        ImageType::Pdf => unsafe {
                            let size = self.bounds(&iter, &self.themer)
                                + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                            let surface = PdfSurface::for_raw_stream(
                                (size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale,
                                (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale,
                                out,
                            )?;
                            let mut context = Context::new(&surface)?;
                            context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                            context.translate(-size[[0, 0]], -size[[0, 1]]);
                            self.draw(&iter, &mut context);
                            surface.finish_output_stream().unwrap();
                        },
                    }
                };
            }
        }
        Ok(())
    }
}

impl<'a> PlotterImpl<'a, Pcb> for CairoPlotter<'a> {
    fn plot<W: Write + 'static>(
        &self,
        schema: &Pcb,
        out: &mut W,
        border: bool,
        scale: f64,
        _pages: Option<Vec<usize>>,
        _netlist: bool,
    ) -> Result<(), Error> {
        use super::pcb::PcbPlotIterator;
        /* for page in 0..schema.pages() {
        if pages.as_ref().is_none() || pages.as_ref().unwrap().contains(&page) { */

        if border {
            let paper_size: (f64, f64) = schema.paper_size.clone().into();
            let title_block = &schema.title_block;
            let iter = schema
                .iter()?
                .plot(schema, &Some(title_block.clone()), paper_size, border)
                .flatten()
                .collect();
            match self.image_type {
                ImageType::Svg => unsafe {
                    let surface = SvgSurface::for_raw_stream(
                        self.paper_size.0 * 96.0 / 25.4,
                        self.paper_size.1 * 96.0 / 25.4,
                        out,
                    )
                    .unwrap();
                    let mut context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    self.draw(&iter, &mut context);
                    surface.finish_output_stream().unwrap();
                },
                ImageType::Png => {
                    let surface = ImageSurface::create(
                        Format::Rgb24,
                        (self.paper_size.0 * 96.0 / 25.4) as i32,
                        (self.paper_size.1 * 96.0 / 25.4) as i32,
                    )
                    .unwrap();
                    let mut context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    self.draw(&iter, &mut context);
                    surface.write_to_png(out)?;
                }
                ImageType::Pdf => unsafe {
                    let surface = PdfSurface::for_raw_stream(
                        self.paper_size.0 * 96.0 / 25.4,
                        self.paper_size.1 * 96.0 / 25.4,
                        out,
                    )
                    .unwrap();
                    let mut context = Context::new(&surface).unwrap();
                    context.scale(96.0 / 25.4, 96.0 / 25.4);
                    self.draw(&iter, &mut context);
                    surface.finish_output_stream().unwrap();
                },
            }
        } else {
            let iter = schema
                .iter()?
                .plot(schema, &None, (0.0, 0.0), border)
                .flatten()
                .collect();
            //TODO: let size = self.bounds(&iter, &self.themer) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
            match self.image_type {
                ImageType::Svg => {
                    let size =
                        self.bounds(&iter, &self.themer) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    unsafe {
                        let surface = SvgSurface::for_raw_stream(
                            (size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale,
                            (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale,
                            out,
                        )?;
                        let mut context = Context::new(&surface)?;
                        context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                        context.translate(-size[[0, 0]], -size[[0, 1]]);
                        self.draw(&iter, &mut context);
                        surface.finish_output_stream().unwrap();
                    }
                }
                ImageType::Png => {
                    let size =
                        self.bounds(&iter, &self.themer) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    let surface = ImageSurface::create(
                        Format::Rgb24,
                        ((size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale) as i32,
                        ((size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale) as i32,
                    )?;
                    let mut context = Context::new(&surface)?;
                    context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                    context.translate(-size[[0, 0]], -size[[0, 1]]);
                    self.draw(&iter, &mut context);
                    surface.write_to_png(out)?;
                }
                ImageType::Pdf => unsafe {
                    let size =
                        self.bounds(&iter, &self.themer) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
                    let surface = PdfSurface::for_raw_stream(
                        (size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale,
                        (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale,
                        out,
                    )?;
                    let mut context = Context::new(&surface)?;
                    context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
                    context.translate(-size[[0, 0]], -size[[0, 1]]);
                    self.draw(&iter, &mut context);
                    surface.finish_output_stream().unwrap();
                },
            }
        };
        /* }
        } */
        Ok(())
    }
}
impl Outline for CairoPlotter<'_> {}

impl<'a> Draw<Context> for CairoPlotter<'a> {
    fn draw(&self, items: &Vec<PlotItem>, document: &mut Context) {
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

impl<'a> Drawer<Text, Context> for CairoPlotter<'a> {
    fn item(&self, text: &Text, context: &mut Context) {
        context.save().unwrap();
        let layout = create_layout(&self.context);
        let markup = format!(
            "<span face=\"{}\" foreground=\"{}\" size=\"{}\">{}</span>",
            self.themer.font(Some(text.font.to_string()), &text.class),
            rgba_color(text.color),
            (self.themer.font_size(Some(text.fontsize), &text.class) * 1024.0) as i32,
            text.text
        );
        layout.set_markup(markup.as_str());
        update_layout(context, &layout);

        let outline: (i32, i32) = layout.size();
        let outline = (
            outline.0 as f64 / SCALE as f64,
            outline.1 as f64 / SCALE as f64,
        );
        let mut x = text.pos[0];
        let mut y = text.pos[1];

        if !text.label {
            if text.angle == 0.0 || text.angle == 180.0 {
                if text.align.contains(&String::from("right")) {
                    x -= outline.0 as f64;
                } else if !text.align.contains(&String::from("left")) {
                    x -= outline.0 as f64 / 2.0;
                }
                if text.align.contains(&String::from("bottom")) {
                    y -= outline.1 as f64;
                } else if !text.align.contains(&String::from("top")) {
                    y -= outline.1 as f64 / 2.0;
                }
            } else if text.angle == 90.0 || text.angle == 270.0 {
                if text.align.contains(&String::from("right")) {
                    y += outline.0 as f64;
                } else if !text.align.contains(&String::from("left")) {
                    y += outline.0 as f64 / 2.0;
                }
                if text.align.contains(&String::from("bottom")) {
                    x -= outline.1 as f64;
                } else if !text.align.contains(&String::from("top")) {
                    x -= outline.1 as f64 / 2.0;
                }
            } else {
                println!("text angle is: {} ({})", text.angle, text.text);
            }
            context.move_to(x, y);
            let angle = if text.angle >= 180.0 {
                text.angle - 180.0
            } else {
                text.angle
            };
            context.rotate(-angle * std::f64::consts::PI / 180.0);
            show_layout(context, &layout);
            context.stroke().unwrap();
        } else {
            let label_left = 0.4;
            let label_up = 0.1;
            let contur = arr2(&[
                [0.0, 0.],
                [2.0 * label_left, -outline.1 / 2.0 - label_up],
                [3.0 * label_left + outline.0, -outline.1 / 2.0 - label_up],
                [3.0 * label_left + outline.0, outline.1 / 2.0 + label_up],
                [2.0 * label_left, outline.1 / 2.0 + label_up],
                [0.0, 0.0],
            ]);
            let theta = -text.angle.to_radians();
            let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
            let verts: Array2<f64> = contur.dot(&rot);
            let verts = &text.pos + verts;
            context.move_to(text.pos[0], text.pos[1]);
            for row in verts.rows() {
                context.line_to(row[0], row[1]);
            }
            context.stroke().unwrap();

            //adjust the text
            if text.angle == 0.0 {
                x += 2.0 * label_left;
                y -= outline.1 / 2.0;
            } else if text.angle == 180.0 {
                x -= 2.0 * label_left + outline.0;
                y -= outline.1 / 2.0;
            } //TODO 90, 270
            context.move_to(x, y);
            let angle = if text.angle >= 180.0 {
                text.angle - 180.0
            } else {
                text.angle
            };
            context.rotate(-angle * std::f64::consts::PI / 180.0);
            show_layout(context, &layout);
            context.stroke().unwrap();
        }
        context.restore().unwrap();
    }
}

impl<'a> Drawer<Line, Context> for CairoPlotter<'a> {
    fn item(&self, line: &Line, context: &mut Context) {
        stroke!(context, line, self.themer);
        /*TODO: match line.linecap {
            LineCap::Butt => context.set_line_cap(cairo::LineCap::Butt),
            LineCap::Round => context.set_line_cap(cairo::LineCap::Round),
            LineCap::Square => context.set_line_cap(cairo::LineCap::Square),
        } */
        context.move_to(line.pts[[0, 0]], line.pts[[0, 1]]);
        context.line_to(line.pts[[1, 0]], line.pts[[1, 1]]);
        context.stroke().unwrap();
    }
}

impl<'a> Drawer<Polyline, Context> for CairoPlotter<'a> {
    fn item(&self, line: &Polyline, context: &mut Context) {
        stroke!(context, line, self.themer);
        let mut first: bool = true;
        for pos in line.pts.rows() {
            if first {
                context.move_to(pos[0], pos[1]);
                first = false;
            } else {
                context.line_to(pos[0], pos[1]);
                context.stroke_preserve().unwrap();
            }
        }
        fill!(context, line, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Rectangle, Context> for CairoPlotter<'a> {
    fn item(&self, rectangle: &Rectangle, context: &mut Context) {
        stroke!(context, rectangle, self.themer);
        context.rectangle(
            rectangle.pts[[0, 0]],
            rectangle.pts[[0, 1]],
            rectangle.pts[[1, 0]] - rectangle.pts[[0, 0]],
            rectangle.pts[[1, 1]] - rectangle.pts[[0, 1]],
        );
        context.stroke_preserve().unwrap();
        fill!(context, rectangle, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Circle, Context> for CairoPlotter<'a> {
    fn item(&self, circle: &Circle, context: &mut Context) {
        stroke!(context, circle, self.themer);
        context.arc(circle.pos[0], circle.pos[1], circle.radius, 0., 10.);
        context.stroke_preserve().unwrap();
        fill!(context, circle, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Arc, Context> for CairoPlotter<'a> {
    fn item(&self, arc: &Arc, context: &mut Context) {
        /* TODO: stroke!(context, arc, self.themer);
        context.arc(arc.start[0], arc.start[1], arc.mid[1], 0., 10.);
        context.stroke_preserve().unwrap();
        fill!(context, arc, self.themer);
        context.stroke().unwrap() */
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::sexp::Schema;

    use super::super::plotter::{PlotterImpl, Theme};
    use super::{rgba_color, CairoPlotter};

    #[test]
    fn convert_color() {
        assert_eq!("#000000FF", rgba_color((0.0, 0.0, 0.0, 1.0)));
        assert_eq!("#FFFFFFFF", rgba_color((1.0, 1.0, 1.0, 1.0)));
    }
    #[test]
    fn plt_jfet() {
        let doc = Schema::load("files/jfet.kicad_sch").unwrap();
        let png = CairoPlotter::new(crate::plot::plotter::ImageType::Png, Theme::Kicad2020);

        let mut buffer = Vec::<u8>::new();
        let mut buffer = File::create("target/jfet.png").unwrap();
        png.plot(&doc, &mut buffer, true, 1.0, None, false).unwrap();

        // assert!(!buffer.is_empty());
    }
}
