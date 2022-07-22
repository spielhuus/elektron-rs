use std::io::Write;
use ndarray::{arr1, arr2, s, Array1, Array2};
use crate::sexp::{Color, Justify, LineType};
use crate::plot::paper;
extern crate cairo;
use cairo::{Format, Context, SvgSurface, FontSlant, FontWeight, ImageSurface, FontFace};



#[derive(Debug)]
pub struct Line {
    pub pts: Array2<f64>,
    pub linewidth: f64,
    pub linetype: LineType,
    pub color: Color
}
impl Line {
    pub fn new(pts: Array2<f64>, linewidth: f64, linetype: LineType, color: Color) -> Line {
        Line {
            pts,
            linewidth,
            linetype,
            color
        }
    }
}

#[derive(Debug)]
pub struct Rectangle {
    pub pts: Array2<f64>,
    pub color: Color,
    pub linewidth: f64,
    pub linetype: LineType,
    pub fill: Option<Color>,
}
impl Rectangle {
    pub fn new(pts: Array2<f64>, color: Color, linewidth: f64, linetype: LineType, fill: Option<Color>) -> Rectangle {
        Rectangle {
            pts,
            color,
            linewidth,
            linetype,
            fill,
        }
    }
}
#[derive(Debug)]
pub struct Arc {
    pub start: Array1<f64>,
    pub mid: Array1<f64>,
    pub end: Array1<f64>,
    pub linewidth: f64,
    pub linetype: LineType,
    pub color: Color,
    pub fill: Option<Color>
}
impl Arc {
    pub fn new(start: Array1<f64>, mid: Array1<f64>, end: Array1<f64>, linewidth: f64, linetype: LineType, color: Color, fill: Option<Color>) -> Arc {
        Arc {
            start,
            mid,
            end,
            linewidth,
            linetype,
            color,
            fill
        }
    }
}
#[derive(Debug)]
pub struct Circle {
    pub pos: Array1<f64>,
    pub radius: f64,
    pub linewidth: f64,
    pub linetype: LineType,
    pub color: Color,
    pub fill: Option<Color>
}
impl Circle {
    pub fn new(pos: Array1<f64>, radius: f64, linewidth: f64, linetype: LineType, color: Color, fill: Option<Color>) -> Circle {
        Circle {
            pos,
            radius,
            linewidth,
            linetype,
            color,
            fill
        }
    }
}
#[derive(Debug)]
pub struct Polyline {
    pub pts: Array2<f64>,
    pub color: Color,
    pub linewidth: f64,
    pub linetype: LineType,
    pub fill: Option<Color>
}
impl Polyline {
    pub fn new(pts: Array2<f64>, color: Color, linewidth: f64, linetype: LineType, fill: Option<Color>) -> Polyline {
        Polyline {
            pts,
            color,
            linewidth,
            linetype,
            fill
        }
    }
}
#[derive(Debug)]
pub struct Text {
    pub pos: Array1<f64>,
    pub text: String,
    pub color: Color,
    pub fontsize: f64,
    pub font: String,
    pub align: Vec<Justify>,
    pub angle: f64,
}
impl Text {
    pub fn new(pos: Array1<f64>, angle: f64, text: String, color: Color, fontsize: f64, font: &str, align: Vec<Justify>) -> Text {
        Text {
            pos,
            text,
            color,
            fontsize,
            font: font.to_string(),
            align,
            angle,
        }
    }
}

#[derive(Debug)]
pub enum PlotItem {
    ArcItem(Arc),
    CircleItem(Circle),
    LineItem(Line),
    RectangleItem(Rectangle),
    PolylineItem(Polyline),
    TextItem(Text),
}

macro_rules! stroke {
    ($context:expr, $stroke:expr) => {
        $context.set_source_rgba($stroke.color.r, $stroke.color.g, $stroke.color.b, $stroke.color.a);
        $context.set_line_width($stroke.linewidth);
    };
}
macro_rules! fill {
    ($context:expr, $fill:expr) => {
        match &$fill {
            Some(fill) => {
                $context.set_source_rgba(fill.r, fill.g, fill.b, fill.a);
                $context.fill().unwrap();
            }
            _ => {}
        }
    };
}
macro_rules! effects {
    ($context:expr, $effects:expr) => {
        $context.set_font_size($effects.fontsize);
        let face = FontFace::toy_create(
            $effects.font.as_str(),
            FontSlant::Normal,
            FontWeight::Normal).unwrap();
        $context.set_font_face(&face);
        $context.set_source_rgba(0.0, 0.0, 0.0, 1.0); //TODO
    };
}

pub trait Plotter {
    fn push(&mut self, item: PlotItem);
    fn text_size(&self, item: &Text) -> Array1<f64>;
    fn bounds(&self) -> Array2<f64>;
    fn plot(&mut self, file: Box<dyn Write>, border: bool, scale: f64);
    fn paper(&mut self, paper: String);
}

/// Plotter implemntation for SVG and PDF file.
pub struct CairoPlotter {
    items: Vec<PlotItem>,
    context: Context,
    paper_size: (f64, f64),
}
impl CairoPlotter {
    pub fn new() -> CairoPlotter {
        let surface = ImageSurface::create(Format::Rgb24, (297.0 * 72.0 / 25.4) as i32, (210.0 * 72.0 / 25.4) as i32).unwrap();
        let context = Context::new(&surface).unwrap();
        context.scale(72.0 / 25.4, 72.0 / 25.4);
        CairoPlotter {
            items: Vec::new(),
            context,
            paper_size: paper::A4,
        }
    }
    fn arr_outline(&self, boxes: &Array2<f64>) -> Array2<f64>{
        let axis1 = boxes.slice(s![.., 0]);
        let axis2 = boxes.slice(s![.., 1]);
        arr2(&[
            [
                axis1
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap().clone(),
                axis2
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap().clone(),
            ],
            [
                axis1
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap().clone(),
                axis2
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap().clone(),
            ],
        ])
    }
}

impl Plotter for CairoPlotter {
    /// Push a plot item to the plotter.
    fn push(&mut self, item: PlotItem) {
        self.items.push(item);
    }
    /// get the text size in pixels.
    fn text_size(&self, item: &Text) -> Array1<f64>{
        effects!(self.context, item);
        let extends = self.context.text_extents(item.text.as_str()).unwrap();
        arr1(&[extends.width, extends.height])
    }
    /// Calculate the drawing area.
    fn bounds(&self) -> Array2<f64> {
        let mut __bounds: Array2<f64> = Array2::default((0, 2));
        for item in self.items.iter() {
            let arr: Option<Array2<f64>>;
            match item {
                PlotItem::ArcItem(arc) => {
                    arr = Option::from(arr2(&[[arc.start[0], arc.start[1]], [arc.end[0], arc.end[1]]]));
                },
                PlotItem::LineItem(line) => {
                    arr = Option::from(arr2(&[[line.pts[[0, 0]], line.pts[[0, 1]]], 
                         [line.pts[[1, 0]], line.pts[[1, 1]]]]));
                },
                PlotItem::TextItem(text) => {
                    let outline = self.text_size(&text);
                    let mut x = text.pos[0];
                    let mut y = text.pos[1];
                    if text.align.contains(&Justify::Right) {
                        x = x - outline[0];
                    } else if text.align.contains(&Justify::Top) {
                        y = y - outline[1];
                    } else if !text.align.contains(&Justify::Left) &&
                              !text.align.contains(&Justify::Bottom) {
                        x = x - outline[0] / 2.0;
                        y = y - outline[1] / 2.0;
                    }
                    arr = Option::from(arr2(&[[x, y], 
                           [x + outline[0], y + outline[1]]]));
                },
                PlotItem::CircleItem(circle) => {
                    arr = Option::from(arr2(&[[circle.pos[0] - circle.radius, circle.pos[1] - circle.radius], 
                           [circle.pos[0] + circle.radius, circle.pos[1] + circle.radius]]));
                },
                PlotItem::PolylineItem(polyline) => {
                    arr = Option::from( self.arr_outline(&polyline.pts));
                },
                PlotItem::RectangleItem(rect) => {
                    arr = Option::from(arr2(&[[rect.pts[[0, 0]], rect.pts[[0, 1]]], 
                         [rect.pts[[1, 0]], rect.pts[[1, 1]]]]));
                },
            }
            if let Some(array) = arr {
                for row in array.rows() {
                    __bounds.push_row(row).unwrap();
                }
            }
        }
        self.arr_outline(&__bounds)
    }


    fn plot(&mut self, file: Box<dyn Write>, border: bool, scale: f64) {
        let (context, surface) = if border {
            let surface = SvgSurface::for_stream(self.paper_size.0 * 96.0 / 25.4, self.paper_size.1 * 96.0 / 25.4, file).unwrap(); //TODO paper size
            let context = Context::new(&surface).unwrap();
            context.scale(96.0 / 25.4, 96.0 / 25.4);
            (context, surface)
        } else {
            println!("without border");
            let size = self.bounds() + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
            let surface = SvgSurface::for_stream((size[[1, 0]] - size[[0, 0]]) * 72.0 / 25.4 * scale, 
                                       (size[[1, 1]] - size[[0, 1]]) * 72.0 / 25.4 * scale, file).unwrap();
            let context = Context::new(&surface).unwrap();
            context.scale(72.0 / 25.4 * scale, 72.0 / 25.4 * scale);
            context.translate(-size[[0, 0]], -size[[0, 1]]);
            (context, surface)
        };
        
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().unwrap();

        for item in &self.items {
            match item {
                PlotItem::LineItem(line) => {
                    stroke!(context, line);
                    context.move_to(line.pts[[0, 0]], line.pts[[0, 1]]);
                    context.line_to(line.pts[[1, 0]], line.pts[[1, 1]]);
                    context.stroke().unwrap();
                }
                PlotItem::PolylineItem(line) => {
                    stroke!(context, line);
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
                    fill!(context, line.fill);
                    context.stroke().unwrap()
                }
                PlotItem::RectangleItem(rectangle) => {
                    stroke!(context, rectangle);
                    context.rectangle(rectangle.pts[[0, 0]], rectangle.pts[[0, 1]],
                                      rectangle.pts[[1, 0]] - rectangle.pts[[0, 0]],
                                      rectangle.pts[[1, 1]] - rectangle.pts[[0, 1]]);
                    context.stroke_preserve().unwrap();
                    fill!(context, rectangle.fill);
                    context.stroke().unwrap()
                }
                PlotItem::CircleItem(circle) => {
                    stroke!(context, circle);
                    context.arc(circle.pos[0], circle.pos[1], circle.radius, 0., 10.);
                    context.stroke_preserve().unwrap();
                    fill!(context, circle.fill);
                    context.stroke().unwrap()
                }
                PlotItem::ArcItem(circle) => {
                    stroke!(context, circle);
                    context.arc(circle.start[0], circle.start[1], circle.mid[1], 0., 10.);
                    context.stroke_preserve().unwrap();
                    fill!(context, circle.fill);
                    context.stroke().unwrap()
                }
                PlotItem::TextItem(text) => {
                    context.save().unwrap();
                    effects!(context, text);
                    let mut x = text.pos[0];
                    let mut y = text.pos[1];
                    let outline = self.text_size(&text);
                    // context.arc(x, y, 0.2, 0., 10.);
                    if text.align.contains(&Justify::Right) {
                        x = x - outline[0];
                    } else if text.align.contains(&Justify::Left) {
                        x = x; //  outline[0];
                    } else {
                        x = x - outline[0] / 2.0;
                    }
                    if text.align.contains(&Justify::Top) {
                        y = y - outline[1];
                    } else if text.align.contains(&Justify::Bottom) {
                        y = y;
                    } else {
                        y = y + outline[1] / 2.0;
                    }
                    context.move_to(x, y);
                    context.rotate(text.angle * 3.14 / 180.0);
                    //context.show_text(format!("{:?}, {:?}", text.text.as_str(), text.align).as_str()).unwrap();
                    context.show_text(text.text.as_str()).unwrap();
                    context.stroke().unwrap();
                    context.restore().unwrap();
                }
            }
        }
        surface.finish_output_stream().unwrap();
    }
    fn paper(&mut self, paper_size: String) {
        if paper_size == String::from("A4") {
            self.paper_size = paper::A4;
        } // TODO other paper sizes
    }

}
