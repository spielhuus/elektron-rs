use super::theme::Themer;

use crate::error::Error;
use cairo::{Context, Format, ImageSurface};
use ndarray::{arr1, arr2, s, Array1, Array2};
use pangocairo::{create_layout, pango::SCALE, update_layout};
use std::{ffi::OsStr, fmt, io::Write, path::Path};

#[derive(Clone)]
///The color theme
pub enum Theme {
    BlackWhite,
    BlueGreenDark,
    BlueTone,
    EagleDark,
    Nord,
    SolarizedDark,
    SolarizedLight,
    WDark,
    WLight,
    ///Kicad alike theme.
    Kicad2020,
    ///Behave Dark Theme
    BehaveDark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Kicad2020
    }
}

impl From<&str> for Theme {
    fn from(theme: &str) -> Self {
        if theme == "BlackWhite" {
            Theme::BlackWhite
        } else if theme == "BlueGreenDark" {
            Theme::BlueGreenDark
        } else if theme == "BlueTone" {
            Theme::BlueTone
        } else if theme == "EagleDark" {
            Theme::EagleDark
        } else if theme == "Nord" {
            Theme::Nord
        } else if theme == "SolarizedDark" {
            Theme::SolarizedDark
        } else if theme == "SolarizedLight" {
            Theme::SolarizedLight
        } else if theme == "WDark" {
            Theme::WDark
        } else if theme == "WLight" {
            Theme::WLight
        } else if theme == "BehaveDark" {
            Theme::BehaveDark
        } else {
            Theme::Kicad2020
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
//The output image type. Availability epends on the plotter.
pub enum FillType {
    NoFill,
    Background,
    Outline,
}

impl FillType {
    pub fn from(name: &str) -> Self {
        if name == "outline" {
            FillType::Outline
        } else if name == "background" {
            FillType::Background
        } else {
            FillType::NoFill
        }
    }
}

impl fmt::Display for FillType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoFill => write!(f, ""),
            Self::Background => write!(f, "fill-background"),
            Self::Outline => write!(f, "fill-outline"),
        }
    }
}

pub fn no_fill(styles: &Vec<Style>) -> bool {
    for style in styles {
        if let Style::Fill(FillType::NoFill) = style {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone)]
//The output image type. Availability epends on the plotter.
pub enum Style {
    Border,
    Wire,
    Bus,
    BusEntry,
    Junction,
    NoConnect,
    NotFound,
    Outline,
    PinDecoration,
    Pin,
    Polyline,
    Text,
    TextSheet,
    TextTitle,
    TextSubtitle,
    TextHeader,
    TextPinName,
    TextPinNumber,
    TextNetname,
    Fill(FillType),
    Layer(String),
    Label,
    Property,
    Segment,
    PadFront,
    PadBack,
    Test,
    NotOnBoard,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Style::Wire => write!(f, "schema-wire"),
            Style::Bus => write!(f, "schema-bus"),
            Style::BusEntry => write!(f, "schema-bus-entry"),
            Style::Junction => write!(f, "schema-junction"),
            Style::NoConnect => write!(f, "no-connect"),
            Style::NotFound => write!(f, "not-found"),
            Style::Outline => write!(f, "schema-outline"),
            Style::Pin => write!(f, "schema-pin"),
            Style::Polyline => write!(f, "schema-polyline"),
            Style::Fill(fill) => write!(f, "{}", fill),
            Style::Layer(layer) => write!(f, "{}", layer),
            Style::TextSheet => write!(f, "text-sheet"),
            Style::TextTitle => write!(f, "text-title"),
            Style::TextSubtitle => write!(f, "text-subtitle"),
            Style::TextHeader => write!(f, "text-header"),
            Style::Label => write!(f, "schema-label"),
            Style::Property => write!(f, "schema-property"),
            Style::TextPinName => write!(f, "schema-pin-name"),
            Style::TextPinNumber => write!(f, "schema-pin-number"),
            Style::TextNetname => write!(f, "schema-netname"),
            Style::Text => write!(f, "schema-text"),
            Style::Border => write!(f, "schema-border"),
            Style::Segment => write!(f, "pcb-segment"),
            Style::PadFront => write!(f, "pad_front"),
            Style::PadBack => write!(f, "pad_back"),
            Style::Test => write!(f, "test"),
            Style::PinDecoration => write!(f, "schema-pin-decoration"),
            Style::NotOnBoard => write!(f, "opaque"),
        }
    }
}

#[derive(Debug)]
//The output image type. Availability epends on the plotter.
pub enum ImageType {
    Svg,
    Png,
    Pdf,
}

impl ImageType {
    //ImageType from String.
    pub fn from(name: &str) -> Option<Self> {
        if name == "pdf" {
            Some(ImageType::Pdf)
        } else if name == "png" {
            Some(ImageType::Png)
        } else if name == "svg" {
            Some(ImageType::Svg)
        } else {
            None
        }
    }
    //ImageType from file extension.
    pub fn from_filename(name: &str) -> Option<Self> {
        let extension = Path::new(name).extension().and_then(OsStr::to_str);
        if let Some(extension) = extension {
            ImageType::from(extension)
        } else {
            None
        }
    }
}

#[derive(Debug)]
//Line CAP, endings.
pub enum LineCap {
    Butt,
    Round,
    Square,
}

impl fmt::Display for LineCap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LineCap::Butt => write!(f, "butt"),
            LineCap::Round => write!(f, "round"),
            LineCap::Square => write!(f, "square"),
        }
    }
}

macro_rules! text {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr, $class:expr) => {
        PlotItem::Text(
            99,
            Text::new(
                $pos,
                $angle,
                $content,
                $effects.color.clone(),
                $effects.font_size.0,
                $effects.font.as_str(),
                $effects.justify.clone(),
                false,
                $class,
            ),
        )
    };
}
pub(crate) use text;

macro_rules! effects {
    ($context:expr, $effects:expr) => {
        $context.set_font_size($effects.fontsize);
        let face = FontFace::toy_create(
            $effects.font.as_str(),
            FontSlant::Normal,
            FontWeight::Normal,
        )
        .unwrap();
        $context.set_font_face(&face);
        $context.set_source_rgba(0.0, 0.0, 0.0, 1.0); //TODO
    };
}
pub(crate) use effects;

//Line element.
pub struct Line {
    pub pts: Array2<f64>,
    pub width: Option<f64>,
    pub color: Option<(f64, f64, f64, f64)>,
    pub style: Option<String>,
    pub linecap: Option<LineCap>,
    pub class: Vec<Style>,
}
impl Line {
    ///Line from absolute points with style.
    pub fn new(
        pts: Array2<f64>,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        linecap: Option<LineCap>,
        class: Vec<Style>,
    ) -> Line {
        Line {
            pts,
            width,
            color,
            style,
            linecap,
            class,
        }
    }
}

pub struct Rectangle {
    pub pts: Array2<f64>,
    pub width: Option<f64>,
    pub color: Option<(f64, f64, f64, f64)>,
    pub style: Option<String>,
    pub class: Vec<Style>,
}
impl Rectangle {
    pub fn new(
        pts: Array2<f64>,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        class: Vec<Style>,
    ) -> Rectangle {
        Rectangle {
            pts,
            width,
            color,
            style,
            class,
        }
    }
}

pub struct Arc {
    pub center: Array1<f64>,
    pub start: Array1<f64>,
    pub end: Array1<f64>,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
    pub width: Option<f64>,
    pub color: Option<(f64, f64, f64, f64)>,
    pub style: Option<String>,
    pub class: Vec<Style>,
}
impl Arc {
    pub fn new(
        center: Array1<f64>,
        start: Array1<f64>,
        end: Array1<f64>,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        class: Vec<Style>,
    ) -> Arc {
        Arc {
            center,
            start,
            end,
            radius,
            start_angle,
            end_angle,
            width,
            color,
            style,
            class,
        }
    }
    pub fn from_center(
        center: Array1<f64>,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        class: Vec<Style>,
    ) -> Self {
        let start = arr1(&[
            center[0] + radius * start_angle.to_radians().cos(),
            center[1] + radius * start_angle.to_radians().sin(),
        ]);
        let end = arr1(&[
            center[0] + radius * end_angle.to_radians().cos(),
            center[1] + radius * end_angle.to_radians().sin(),
        ]);
        Arc {
            center,
            start,
            end,
            radius,
            start_angle,
            end_angle,
            width,
            color,
            style,
            class,
        }
    }
}

pub struct Circle {
    pub pos: Array1<f64>,
    pub radius: f64,
    pub width: Option<f64>,
    pub color: Option<(f64, f64, f64, f64)>,
    pub style: Option<String>,
    pub class: Vec<Style>,
}
impl Circle {
    pub fn new(
        pos: Array1<f64>,
        radius: f64,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        class: Vec<Style>,
    ) -> Circle {
        Circle {
            pos,
            radius,
            width,
            color,
            style,
            class,
        }
    }
}

pub struct Polyline {
    pub pts: Array2<f64>,
    pub width: Option<f64>,
    pub color: Option<(f64, f64, f64, f64)>,
    pub style: Option<String>,
    pub class: Vec<Style>,
}
impl Polyline {
    pub fn new(
        pts: Array2<f64>,
        width: Option<f64>,
        color: Option<(f64, f64, f64, f64)>,
        style: Option<String>,
        class: Vec<Style>,
    ) -> Polyline {
        Polyline {
            pts,
            width,
            color,
            style,
            class,
        }
    }
}

pub struct Text {
    pub pos: Array1<f64>,
    pub text: String,
    pub color: (f64, f64, f64, f64),
    pub fontsize: f64,
    pub font: String,
    pub align: Vec<String>,
    pub angle: f64,
    pub label: bool,
    pub class: Vec<Style>,
}
impl Text {
    pub fn new(
        pos: Array1<f64>,
        angle: f64,
        text: String,
        color: (f64, f64, f64, f64),
        fontsize: f64,
        font: &str,
        align: Vec<String>,
        label: bool,
        class: Vec<Style>,
    ) -> Text {
        Text {
            pos,
            text,
            color,
            fontsize,
            font: font.to_string(),
            align,
            angle,
            label,
            class,
        }
    }
}

pub enum PlotItem {
    Arc(usize, Arc),
    Circle(usize, Circle),
    Line(usize, Line),
    Rectangle(usize, Rectangle),
    Polyline(usize, Polyline),
    Text(usize, Text),
}

///Plotter trait
pub trait PlotterImpl<'a, T> {
    ///Plot the data.
    fn plot<W: Write + 'static>(
        &self,
        doc: &T,
        out: &mut W,
        border: bool,
        scale: f64,
        pages: Option<Vec<usize>>,
        netlist: bool,
    ) -> Result<(), Error>;
}

/// Plot an item of the document.
pub trait ItemPlot<T> {
    fn item(&self, item: &T) -> Option<Vec<PlotItem>>;
}

///Draw all PlotItems.
pub trait Draw<T> {
    fn draw(&self, items: &Vec<PlotItem>, document: &mut T);
}

///Draw a PlotItem.
pub trait Drawer<I, T> {
    fn item(&self, item: &I, document: &mut T);
}

pub trait Outline {
    //Get the text size.
    fn text_size(&self, item: &Text, themer: &Themer) -> Array1<f64> {
        let surface = ImageSurface::create(
            Format::Rgb24,
            (297.0 * 72.0 / 25.4) as i32,
            (210.0 * 72.0 / 25.4) as i32,
        )
        .unwrap();
        let context = Context::new(&surface).unwrap();
        context.scale(72.0 / 25.4, 72.0 / 25.4);

        let layout = create_layout(&context);
        let markup = format!(
            "<span face=\"{}\" size=\"{}\">{}</span>",
            themer.font(Some(item.font.to_string()), &item.class),
            (themer.font_size(Some(item.fontsize), &item.class) * 1024.0) as i32,
            item.text
        );
        layout.set_markup(markup.as_str());
        update_layout(&context, &layout);

        let outline: (i32, i32) = layout.size();
        let outline = (
            outline.0 as f64 / SCALE as f64,
            outline.1 as f64 / SCALE as f64,
        );
        arr1(&[outline.0, outline.1])
    }

    ///Bounding box of the Drawing.
    fn arr_outline(&self, boxes: &Array2<f64>) -> Array2<f64> {
        let axis1 = boxes.slice(s![.., 0]);
        let axis2 = boxes.slice(s![.., 1]);
        arr2(&[
            [
                *axis1
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *axis2
                    .iter()
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ],
            [
                *axis1
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
                *axis2
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap(),
            ],
        ])
    }

    /// Calculate the drawing area.
    fn bounds(&self, items: &Vec<PlotItem>, themer: &Themer) -> Array2<f64> {
        let mut __bounds: Array2<f64> = Array2::default((0, 2));
        items.iter().for_each(|item| {
            let arr: Option<Array2<f64>> = match item {
                PlotItem::Arc(_, arc) => Option::from(arr2(&[
                    [arc.start[0], arc.start[1]],
                    [arc.end[0], arc.end[1]],
                    [arc.center[0], arc.center[1]],
                ])),
                PlotItem::Line(_, line) => Option::from(arr2(&[
                    [line.pts[[0, 0]], line.pts[[0, 1]]],
                    [line.pts[[1, 0]], line.pts[[1, 1]]],
                ])),
                PlotItem::Text(_, text) => {
                    let outline = self.text_size(text, themer);
                    let mut x = text.pos[0];
                    let mut y = text.pos[1];
                    if text.align.contains(&String::from("right")) {
                        x -= outline[0];
                    } else if text.align.contains(&String::from("top")) {
                        y -= outline[1];
                    } else if !text.align.contains(&String::from("left"))
                        && !text.align.contains(&String::from("bottom"))
                    {
                        x -= outline[0] / 2.0;
                        y -= outline[1] / 2.0;
                    }
                    Option::from(arr2(&[[x, y], [x + outline[0], y + outline[1]]]))
                }
                PlotItem::Circle(_, circle) => Option::from(arr2(&[
                    [circle.pos[0] - circle.radius, circle.pos[1] - circle.radius],
                    [circle.pos[0] + circle.radius, circle.pos[1] + circle.radius],
                ])),
                PlotItem::Polyline(_, polyline) => Option::from(self.arr_outline(&polyline.pts)),
                PlotItem::Rectangle(_, rect) => Option::from(arr2(&[
                    [rect.pts[[0, 0]], rect.pts[[0, 1]]],
                    [rect.pts[[1, 0]], rect.pts[[1, 1]]],
                ])),
            };
            if let Some(array) = arr {
                for row in array.rows() {
                    __bounds.push_row(row).unwrap();
                }
            }
        });
        self.arr_outline(&__bounds)
    }
}
