use std::{fmt, io::Write};

use cairo::{Context, Format, ImageSurface};
use ndarray::{arr1, arr2, Array1, Array2, ArrayView};
use pyo3::{Python, py_run, types::PyList};

pub mod cairo_plotter;
pub mod svg;
pub mod themer;
pub mod error;
pub mod gerber;

pub use error::Error;

use simulation::{Netlist, Point};

use sexp::{
    el, utils, PinGraphicalStyle, Sexp, SexpTree, SexpValueQuery, SexpValuesQuery,
    math::{normalize_angle, CalcArc, MathUtils, PinOrientation, Shape, Transform}
};
use self::themer::Themer;

const PIN_NUMER_OFFSET: f64 = 0.6;
const BORDER_RASTER: i32 = 60;

// -----------------------------------------------------------------------------------------------------------
// ---                                             sexp utils                                              ---
// -----------------------------------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Effects {
    pub font_face: String,
    pub font_size: Vec<f64>,
    pub font_thickness: f64,
    pub font_color: Vec<u16>,
    pub bold: bool,
    pub italic: bool,
    pub justify: Vec<String>,
    pub hide: bool,
}
impl Effects {
    pub fn new() -> Self {
        Self {
            font_face: "default".to_string(),
            font_size: Vec::from([1.27, 1.27]),
            font_thickness: 1.27,
            font_color: vec![0, 0, 0, 1],
            bold: false,
            italic: false,
            justify: Vec::new(),
            hide: false,
        }
    }
}

fn effects(element: &Sexp) -> Effects {
    let mut effects = Effects::new();
    if let Some(e) = element.query(el::EFFECTS).next() {
        let font = e.query("font").next();
        if let Some(font) = font {
            if let Some(face) = font.query("face").next() {
                effects.font_face = face.get(0).unwrap();
            }
            if let Some(size) = font.query("size").next() {
                effects.font_size = size.values()
            }
            if let Some(thickness) = font.query("thickness").next() {
                effects.font_thickness = thickness.get(0).unwrap();
            }
            if let Some(color) = font.query("color").next() {
                effects.font_color = color.values()
            }

            let values: Vec<String> = font.values();
            if values.contains(&"bold".to_string()) {
                effects.bold = true;
            }
            if values.contains(&"italic".to_string()) {
                effects.italic = true;
            }
        }

        if let Some(justify) = e.query(el::EFFECTS_JUSTIFY).next() {
            effects.justify = justify.values()
        }

        let values: Vec<String> = e.values();
        if values.contains(&"hide".to_string()) {
            effects.hide = true;
        }
    }
    effects
}

#[derive(Clone)]
pub struct Stroke {
    pub linewidth: f64,
    pub linetype: String,
    pub linecolor: Vec<u16>,
}
impl Stroke {
    pub fn new() -> Self {
        Self {
            linewidth: 0.0,
            linetype: String::from("default"),
            linecolor: vec![0, 0, 0, 1],
        }
    }
}

fn stroke(element: &Sexp) -> Stroke {
    //(stroke (width 2) (type dash_dot_dot) (color 0 255 0 1))
    let mut stroke = Stroke::new();
    let s = element.query(el::STROKE).next().unwrap();

    let linewidth = s.query("width").next();
    if let Some(width) = linewidth {
        stroke.linewidth = width.get(0).unwrap()
    }
    let linetype = s.query("type").next();
    if let Some(linetype) = linetype {
        stroke.linetype = linetype.get(0).unwrap()
    }
    if let Some(color) = s.query("color").next() {
        stroke.linecolor = color.values()
    }

    stroke
}

fn fill_type(element: &Sexp) -> String {
    if let Some(fill) = element.query("fill").next() {
        fill.value("type").unwrap()
    } else {
        String::from("default")
    }
}

// -----------------------------------------------------------------------------------------------------------
// ---                                          The plotter model                                          ---
// -----------------------------------------------------------------------------------------------------------

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

///Draw all PlotItems.
pub trait Draw<T> {
    fn draw(&self, items: &Vec<PlotItem>, document: &mut T);
}

///Draw a PlotItem.
pub trait Drawer<I, T> {
    fn item(&self, item: &I, document: &mut T);
}

#[derive(Debug, Clone, PartialEq, Eq)]
//The output image type. Availability epends on the plotter. //TODO
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

//TODO this is to much effort
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

pub enum PlotItem {
    Arc(usize, Arc),
    Circle(usize, Circle),
    Line(usize, Line),
    Rectangle(usize, Rectangle),
    Polyline(usize, Polyline),
    Text(usize, Text),
}

//Line element.
pub struct Line {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub linecap: Option<LineCap>,
    pub class: Vec<Style>,
}
impl Line {
    ///Line from absolute points with style.
    pub fn new(
        pts: Array2<f64>,
        stroke: Stroke,
        linecap: Option<LineCap>,
        class: Vec<Style>,
    ) -> Line {
        Line {
            pts,
            stroke,
            linecap,
            class,
        }
    }
}

pub struct Polyline {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub class: Vec<Style>,
}
impl Polyline {
    pub fn new(
        pts: Array2<f64>,
        stroke: Stroke,
        class: Vec<Style>,
    ) -> Polyline {
        Polyline {
            pts,
            stroke,
            class,
        }
    }
}

pub struct Rectangle {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub class: Vec<Style>,
}
impl Rectangle {
    pub fn new(
        pts: Array2<f64>,
        stroke: Stroke,
        class: Vec<Style>,
    ) -> Rectangle {
        Rectangle {
            pts,
            stroke,
            class,
        }
    }
}

pub struct Circle {
    pub pos: Array1<f64>,
    pub radius: f64,
    pub stroke: Stroke,
    pub class: Vec<Style>,
}
impl Circle {
    pub fn new(
        pos: Array1<f64>,
        radius: f64,
        stroke: Stroke,
        class: Vec<Style>,
    ) -> Circle {
        Circle {
            pos,
            radius,
            stroke,
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
    pub stroke: Stroke,
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
        stroke: Stroke,
        class: Vec<Style>,
    ) -> Arc {
        Arc {
            center,
            start,
            end,
            radius,
            start_angle,
            end_angle,
            stroke,
            class,
        }
    }
    /*TODO pub fn from_center(
        center: Array1<f64>,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        stroke: Stroke,
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
            stroke,
            class,
        }
    } */
}

pub struct Text {
    pub pos: Array1<f64>,
    pub angle: f64,
    pub text: String,
    pub effects: Effects,
    pub label: bool,
    pub class: Vec<Style>,
}
impl Text {
    pub fn new(
        pos: Array1<f64>,
        angle: f64,
        text: String,
        effects: Effects,
        label: bool,
        class: Vec<Style>,
    ) -> Text {
        Text {
            pos,
            angle,
            text,
            effects,
            label,
            class,
        }
    }
}

// -----------------------------------------------------------------------------------------------------------
// ---                             collect the plot model from the sexp tree                               ---
// -----------------------------------------------------------------------------------------------------------

/// get the pin position
/// returns an array containing the number of pins:
///   3
/// 0   2
///   1
fn pin_position(symbol: &Sexp, pin: &Sexp) -> Vec<usize> {
    let mut position: Vec<usize> = vec![0; 4];
    let symbol_shift: usize = (utils::angle(symbol).unwrap() / 90.0).round() as usize;

    let lib_pos: usize = (utils::angle(pin).unwrap() / 90.0).round() as usize;
    position[lib_pos] += 1;

    position.rotate_right(symbol_shift);
    if let Some(mirror) = <Sexp as SexpValueQuery<String>>::value(&symbol, el::MIRROR) {
        if mirror == "x" {
            position = vec![position[0], position[3], position[2], position[1]];
        } else if mirror == "y" {
            position = vec![position[2], position[1], position[0], position[3]];
        }
    }
    position
}

pub fn plot(document: &SexpTree, netlist: &Option<Netlist>, paper_size: Option<(f64, f64)>) -> Vec<PlotItem> {
    let mut plot_items = Vec::new();
    for item in document.root().unwrap().nodes() {
        if item.name == el::GLOBAL_LABEL {
            //TODO
        } else if item.name == el::JUNCTION {
            plot_items.push(PlotItem::Circle(
                99,
                Circle::new(
                    utils::at(item).unwrap(),
                    0.4,
                    Stroke::new(),
                    vec![Style::Junction, Style::Fill(FillType::Outline)],
                ),
            ));
        } else if item.name == el::LABEL {
            let angle: f64 = utils::angle(item).unwrap();
            let pos: Array1<f64> = utils::at(item).unwrap();
            let mut angle: f64 = angle;
            let pos: Array1<f64> = if angle < 0.0 {
                arr1(&[pos[0] + 1.0, pos[1]])
            } else if angle < 90.0 {
                arr1(&[pos[0], pos[1] - 1.0])
            } else if angle < 180.0 {
                arr1(&[pos[0] - 1.0, pos[1]])
            } else {
                arr1(&[pos[0], pos[1] + 1.0])
            };
            if angle >= 180.0 {
                angle -= 180.0;
            }
            let effects = effects(item);
            let text: String = item.get(0).unwrap();
            plot_items.push(PlotItem::Text(
                10,
                Text::new(pos, angle, text, effects, false, vec![Style::Label]),
            ));
        } else if item.name == el::NO_CONNECT {
            let pos: Array1<f64> = utils::at(item).unwrap();
            let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
            let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;
            plot_items.push(PlotItem::Line(
                10,
                Line::new(lines1, Stroke::new(), None, vec![Style::NoConnect]),
            ));
            plot_items.push(PlotItem::Line(
                10,
                Line::new(lines2, Stroke::new(), None, vec![Style::NoConnect]),
            ));
        } else if item.name == el::SYMBOL {
            let on_schema: bool = if let Some(on_schema) = item.query("on_schema").next() {
                let v: String = on_schema.get(0).unwrap();
                v == String::from("yes") || v == String::from("true")
            } else {
                true
            };
            if on_schema {
                // let mut items: Vec<PlotItem> = Vec::new();
                for property in item.query(el::PROPERTY) {
                    let mut effects = effects(property);
                    let i_angle = utils::angle(item).unwrap();
                    let p_angle = utils::angle(property).unwrap();
                    let mut justify: Vec<String> = Vec::new();
                    for j in effects.justify {
                        if p_angle + i_angle >= 180.0 && p_angle + i_angle < 360.0 && j == "left" {
                            justify.push(String::from("right"));
                        } else if (p_angle + i_angle).abs() >= 180.0
                            && p_angle + i_angle < 360.0
                            && j == "right"
                        {
                            justify.push(String::from("left"));
                        } else {
                            justify.push(j);
                        }
                    }
                    effects.justify = justify;
                    let prop_angle = if (i_angle - p_angle).abs() >= 360.0 {
                        (i_angle - p_angle).abs() - 360.0
                    } else {
                        (i_angle - p_angle).abs()
                    };
                    if !effects.hide {
                        plot_items.push(PlotItem::Text(
                            10,
                            Text::new(
                                utils::at(property).unwrap(),
                                prop_angle,
                                property.get(1).unwrap(),
                                effects,
                                false,
                                vec![Style::Property],
                            ),
                        ));
                    }
                }
                let lib_id: String = item.value(el::LIB_ID).unwrap();
                let item_unit: usize = item.value(el::SYMBOL_UNIT).unwrap();
                if let Some(lib) = utils::get_library(document.root().unwrap(), &lib_id) {
                    for _unit in lib.query(el::SYMBOL) {
                        //&self.schema.get_library(&item.lib_id).unwrap().symbols {
                        let unit: usize = utils::unit_number(_unit.get(0).unwrap());
                        if unit == 0 || unit == item_unit {
                            for graph in _unit.query(el::GRAPH_POLYLINE) {
                                let mut classes = vec![
                                    Style::Outline,
                                    Style::Fill(FillType::from(&fill_type(graph))),
                                ];
                                let on_board: bool = item.value("on_board").unwrap();
                                if on_board == false {
                                    //Grey out item if it is not on pcb
                                    classes.push(Style::NotOnBoard);
                                }
                                let mut pts: Array2<f64> = Array2::zeros((0, 2));
                                for pt in graph.query("pts") {
                                    for xy in pt.query("xy") {
                                        pts.push_row(ArrayView::from(&[
                                            xy.get(0).unwrap(),
                                            xy.get(1).unwrap(),
                                        ]))
                                        .unwrap();
                                    }
                                }
                                plot_items.push(PlotItem::Polyline(
                                    20,
                                    Polyline::new(
                                        Shape::transform(item, &pts),
                                        Stroke::new(),
                                        classes,
                                    ),
                                ));
                            }
                            for graph in _unit.query(el::GRAPH_RECTANGLE) {
                                let start: Vec<f64> = graph.query("start").next().unwrap().values();
                                let end: Vec<f64> = graph.query("end").next().unwrap().values();
                                let pts: Array2<f64> =
                                    arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                let filltype: String =
                                    graph.query("fill").next().unwrap().value("type").unwrap();
                                let mut classes =
                                    vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                                let on_board: bool = item.value("on_board").unwrap();
                                if on_board == false {
                                    classes.push(Style::NotOnBoard);
                                }
                                let stroke = stroke(graph);
                                plot_items.push(PlotItem::Rectangle(
                                    1,
                                    Rectangle::new(
                                        Shape::transform(item, &pts),
                                        stroke,
                                        classes,
                                    ),
                                ));
                            }
                            for graph in _unit.query(el::GRAPH_CIRCLE) {
                                let filltype: String =
                                    graph.query("fill").next().unwrap().value("type").unwrap();
                                let mut classes =
                                    vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                                let on_board: bool = item.value("on_board").unwrap();
                                if on_board == false {
                                    classes.push(Style::NotOnBoard);
                                }
                                let center: Array1<f64> = graph.value("center").unwrap();
                                let radius: f64 = graph.value("radius").unwrap();
                                let stroke = stroke(graph);
                                plot_items.push(PlotItem::Circle(
                                    1,
                                    Circle::new(
                                        Shape::transform(item, &center),
                                        radius,
                                        stroke,
                                        /* TODO if stroke.linewidth == 0.0 {
                                            None
                                        } else {
                                            Some(stroke.linewidth)
                                        },
                                        None,
                                        None, */
                                        classes,
                                    ),
                                ));
                            }

                            for graph in _unit.query(el::GRAPH_ARC) {
                                let mut arc_start: Array1<f64> = graph.value("start").unwrap();
                                let arc_mid: Array1<f64> = graph.value("mid").unwrap();
                                let mut arc_end: Array1<f64> = graph.value("end").unwrap();
                                let mirror: Option<String> = graph.value(el::MIRROR);
                                let mut start_angle = normalize_angle(
                                    graph.start_angle() + utils::angle(item).unwrap(),
                                );
                                let mut end_angle = normalize_angle(
                                    graph.end_angle() + utils::angle(item).unwrap(),
                                );
                                if let Some(mirror) = mirror {
                                    //TODO: is
                                    //this
                                    //needed?
                                    if mirror == "x" {
                                        start_angle = 180.0 - end_angle;
                                        end_angle = 180.0 - start_angle;
                                    } else {
                                        start_angle = -start_angle;
                                        end_angle = -end_angle;
                                    }
                                    std::mem::swap(&mut arc_start, &mut arc_end);
                                }

                                let classes = vec![
                                    Style::Outline,
                                    Style::Fill(FillType::from(&fill_type(&item))),
                                ];
                                /* TODO if item.on_board == false {
                                    classes.push(Style::NotOnBoard);
                                } */
                                let stroke = stroke(graph);
                                plot_items.push(PlotItem::Arc(
                                    1,
                                    Arc::new(
                                        Shape::transform(item, &graph.center()),
                                        Shape::transform(item, &arc_start),
                                        Shape::transform(item, &arc_end),
                                        graph.radius(),
                                        start_angle,
                                        end_angle,
                                        stroke,
                                        classes,
                                    ),
                                ));
                            }
                            /*        Graph::Text(text) => {
                                        items.push(text!(
                                            Shape::transform(item, &text.at),
                                            text.angle,
                                            text.text.clone(),
                                            text.effects,
                                            vec![Style::Text]
                                        ));
                                    }
                                }
                            } */

                            for pin in _unit.query(el::PIN) {
                                //calculate the pin line
                                //TODO: there are also symbols like inverting and so on (see:
                                //sch_painter.cpp->848)
                                let orientation = PinOrientation::from(item, pin);
                                let pin_length: f64 = pin.value("length").unwrap();
                                let pin_at: Array1<f64> = utils::at(pin).unwrap(); //TODO remove
                                                                                   //all at below
                                let pin_angle: f64 = utils::angle(pin).unwrap();
                                let pin_end = MathUtils::projection(
                                    &pin_at,
                                    utils::angle(pin).unwrap(),
                                    pin_length,
                                );
                                let pin_line: Array2<f64> =
                                    arr2(&[[pin_at[0], pin_at[1]], [pin_end[0], pin_end[1]]]);
                                let pin_graphical_style: String = pin.get(1).unwrap();
                                let pin_graphic_style: PinGraphicalStyle =
                                    PinGraphicalStyle::from(pin_graphical_style);
                                let stroke = Stroke::new(); //TODO stroke(pin);
                                match pin_graphic_style {
                                    PinGraphicalStyle::Line => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke,
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                    }
                                    PinGraphicalStyle::Inverted => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke.clone(),
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                        plot_items.push(PlotItem::Circle(
                                            11,
                                            Circle::new(
                                                Shape::transform(
                                                    item,
                                                    &arr1(&[pin_end[0], pin_end[1]]),
                                                ),
                                                0.5,
                                                stroke,
                                                vec![Style::PinDecoration],
                                            ),
                                        ));
                                    }
                                    PinGraphicalStyle::Clock => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke,
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                        plot_items.push(PlotItem::Polyline(
                                            10,
                                            Polyline::new(
                                                Shape::transform(
                                                    item,
                                                    &arr2(&[
                                                        [pin_end[0], pin_end[1] - 1.0],
                                                        [pin_end[0] + 1.0, pin_end[1]],
                                                        [pin_end[0], pin_end[1] + 1.0],
                                                    ]),
                                                ),
                                                Stroke::new(),
                                                vec![Style::PinDecoration],
                                            ),
                                        ));
                                    }
                                    PinGraphicalStyle::InvertedClock => todo!(),
                                    PinGraphicalStyle::InputLow => todo!(),
                                    PinGraphicalStyle::ClockLow => todo!(),
                                    PinGraphicalStyle::OutputLow => todo!(),
                                    PinGraphicalStyle::EdgeClockHigh => todo!(),
                                    PinGraphicalStyle::NonLogic => todo!(),
                                }

                                let power = <Sexp as SexpValuesQuery<Vec<String>>>::values(lib)
                                    .contains(&String::from("power"));
                                let pin_numbers: Option<String> = lib.value("pin_numbers");
                                let pin_numbers = if let Some(pin_numbers) = pin_numbers {
                                    pin_numbers != "hide"
                                } else {
                                    true
                                };
                                if !power && pin_numbers {
                                    let pos = Shape::transform(item, &utils::at(pin).unwrap())
                                        + match PinOrientation::from(item, pin) {
                                            PinOrientation::Left | PinOrientation::Right => {
                                                arr1(&[
                                                    Shape::pin_angle(item, pin).to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    -PIN_NUMER_OFFSET,
                                                ])
                                            }
                                            PinOrientation::Up | PinOrientation::Down => arr1(&[
                                                PIN_NUMER_OFFSET,
                                                -Shape::pin_angle(item, pin).to_radians().sin()
                                                    * pin_length
                                                    / 2.0,
                                            ]),
                                        };

                                    let pin_number: String =
                                        pin.query("number").next().unwrap().get(0).unwrap();
                                    plot_items.push(PlotItem::Text(
                                        10,
                                        Text::new(
                                            pos,
                                            utils::angle(pin).unwrap(),
                                            pin_number,
                                            Effects::new(),
                                            false,
                                            vec![Style::Label],
                                        ),
                                    ));
                                }

                                let pin_names: Option<String> = lib.value("pin_names");
                                let pin_names = if let Some(pin_names) = pin_names {
                                    pin_names != "hide"
                                } else {
                                    true
                                };
                                let pin_names_offset: f64 =
                                    if let Some(pin_name) = lib.query("pin_names").next() {
                                        if let Some(pin_offset) = pin_name.value("pin_offset") {
                                            pin_offset
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        0.0
                                    };
                                let pin_name: String =
                                    pin.query("name").next().unwrap().get(0).unwrap();
                                if power && pin_name != "~" && pin_names {
                                    if pin_names_offset != 0.0 {
                                        let name_pos = MathUtils::projection(
                                            &utils::at(pin).unwrap(),
                                            utils::angle(pin).unwrap(),
                                            pin_length + pin_names_offset + 0.5,
                                        );
                                        let mut effects = effects(pin);
                                        effects.justify = vec![match orientation {
                                            PinOrientation::Left => String::from("left"),
                                            PinOrientation::Right => String::from("right"),
                                            PinOrientation::Up => String::from("left"),
                                            PinOrientation::Down => String::from("right"),
                                        }];
                                        plot_items.push(PlotItem::Text(
                                            99,
                                            Text::new(
                                                Shape::transform(item, &name_pos),
                                                utils::angle(pin).unwrap(),
                                                pin_name.clone(),
                                                effects,
                                                false,
                                                vec![Style::TextPinName],
                                            ),
                                        ));
                                    } else {
                                        let name_pos = arr1(&[
                                            pin_at[0]
                                                + pin_angle.to_radians().cos()
                                                    * (pin_length/* + lib.pin_names_offset * 8.0 */),
                                            pin_at[1]
                                                + pin_angle.to_radians().sin()
                                                    * (pin_length/* + lib.pin_names_offset * 8.0 */),
                                        ]);
                                        let mut effects = effects(pin);
                                        effects.justify = vec![String::from("center")];
                                        plot_items.push(PlotItem::Text(
                                            99,
                                            Text::new(
                                                Shape::transform(item, &name_pos),
                                                pin_angle,
                                                pin_name.clone(),
                                                effects,
                                                false,
                                                vec![Style::TextPinName],
                                            ),
                                        ));
                                    }
                                }

                                // draw the netlist name
                                let power = lib.query("power").next();
                                if power.is_none() {
                                    if let Some(netlist) = netlist {
                                        let orientation = pin_position(item, pin);
                                        let pin_length: f64 = pin.value("length").unwrap();
                                        let pos = if orientation == vec![1, 0, 0, 0] {
                                            Shape::transform(item, &utils::at(&pin).unwrap())
                                                + arr1(&[
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    1.0,
                                                ])
                                        } else if orientation == vec![0, 1, 0, 0] {
                                            Shape::transform(item, &utils::at(&pin).unwrap())
                                                + arr1(&[
                                                    -1.0,
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                ])
                                        } else if orientation == vec![0, 0, 1, 0] {
                                            Shape::transform(item, &utils::at(&pin).unwrap())
                                                + arr1(&[
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    1.0,
                                                ])
                                        } else if orientation == vec![0, 0, 0, 1] {
                                            Shape::transform(item, &utils::at(&pin).unwrap())
                                                + arr1(&[
                                                    -1.0,
                                                    -utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                ])
                                        } else {
                                            panic!("unknown pin position: {:?}", orientation)
                                            //TODO Error
                                        };

                                        let effects = Effects::new(); //TODO
                                        let pin_pos =
                                            Shape::transform(item, &utils::at(&pin).unwrap());

                                        plot_items.push(PlotItem::Text(
                                            99,
                                            Text::new(
                                                pos,
                                                0.0,
                                                netlist
                                                    .node_name(&Point::new(
                                                        pin_pos[0], pin_pos[1],
                                                    ))
                                                    .unwrap_or_else(|| String::from("NaN")),
                                                effects,
                                                false,
                                                vec![Style::TextNetname],
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]);
                    plot_items.push(PlotItem::Rectangle(
                        10,
                        Rectangle::new(
                            Shape::transform(item, &pts),
                            Stroke::new(),
                            vec![Style::NotFound],
                        ),
                    ));
                }
                // plot_items.push(Some(items));
            }
        } else if item.name == el::WIRE {
            let stroke = stroke(item);
            let pts = item.query(el::PTS).next().unwrap();
            let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
            let xy1: Array1<f64> = xy.get(0).unwrap().values();
            let xy2: Array1<f64> = xy.get(1).unwrap().values();
            plot_items.push(PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[xy1[0], xy1[1]], [xy2[0], xy2[1]]]),
                    stroke,
                    /* stroke,
                    color,
                    linetype, */
                    None,
                    vec![Style::Wire],
                ),
            ));
        } else if item.name == el::TITLE_BLOCK && paper_size.is_some() {
            plot_items.append(&mut border(item, paper_size.unwrap()).unwrap());
        } else {
            println!("Node: {}", item.name);
        }
    }
    plot_items
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

        let layout = pangocairo::create_layout(&context);
        let markup = format!(
            "<span face=\"{}\" size=\"{}\">{}</span>",
            themer.font(Some(item.effects.font_size[0].to_string()), &item.class),
            (themer.font_size(Some(item.effects.font_size[0]), &item.class) * 1024.0) as i32,
            item.text
        );
        layout.set_markup(markup.as_str());
        pangocairo::update_layout(&context, &layout);

        let outline: (i32, i32) = layout.size();
        let outline = (
            outline.0 as f64 / pangocairo::pango::SCALE as f64,
            outline.1 as f64 / pangocairo::pango::SCALE as f64,
        );
        arr1(&[outline.0, outline.1])
    }

    ///Bounding box of the Drawing.
    fn arr_outline(&self, boxes: &Array2<f64>) -> Array2<f64> {
        let axis1 = boxes.slice(ndarray::s![.., 0]);
        let axis2 = boxes.slice(ndarray::s![.., 1]);
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
                    if text.effects.justify.contains(&String::from("right")) {
                        x -= outline[0];
                    } else if text.effects.justify.contains(&String::from("top")) {
                        y -= outline[1];
                    } else if !text.effects.justify.contains(&String::from("left"))
                        && !text.effects.justify.contains(&String::from("bottom"))
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

pub fn plot_pcb(input: String, output: String) -> Result<(), Error> {

    Python::with_gil(|py| {
        let list = PyList::new(py, &[input, output.to_string()]);
        py_run!(
            py,
            list,
            r#"
                import pcbnew 
                board = pcbnew.LoadBoard(list[0])    
                plot_controller = pcbnew.PLOT_CONTROLLER(board)
                plot_options = plot_controller.GetPlotOptions()    
                plot_options.SetOutputDirectory(list[1])
                plot_options.SetPlotFrameRef(False)
                #plot_options.SetDrillMarksType(pcbnew.PCB_PLOT_PARAMS.FULL_DRILL_SHAPE)
                plot_options.SetSkipPlotNPTH_Pads(False)
                plot_options.SetMirror(False)
                plot_options.SetFormat(pcbnew.PLOT_FORMAT_SVG)
                plot_options.SetSvgPrecision(4)
                plot_options.SetPlotViaOnMaskLayer(True)    
                plot_controller.OpenPlotfile("mask", pcbnew.PLOT_FORMAT_SVG, "Top mask layer")
                plot_controller.SetColorMode(True)
                plot_controller.SetLayer(pcbnew.F_Mask)
                plot_controller.PlotLayer()
                plot_controller.ClosePlot()"#
        );
    });

    Ok(())
}

fn border(title_block: &Sexp, paper_size: (f64, f64)) -> Option<Vec<PlotItem>> {
    let mut plotter: Vec<PlotItem> = Vec::new();
    //outline
    let pts: Array2<f64> = arr2(&[
        [5.0, 5.0],
        [paper_size.0 - 5.0, paper_size.1 - 5.0],
    ]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(pts, Stroke::new(), vec![Style::Border]),
    ));

    //horizontal raster
    for j in &[
        (0.0_f64, 5.0_f64),
        (paper_size.1 - 5.0, paper_size.1),
    ] {
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.0],
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.1],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, Stroke::new(), vec![Style::Border]),
            ));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0,
                        j.0 + 2.5
                    ]),
                    0.0,
                    (i + 1).to_string(),

                    Effects::new(),
                    false,
                    vec![Style::TextSheet],
                ),
            ));
        }
    }

    //vertical raster
    let letters = vec![
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
        'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    for j in &[
        (0.0_f64, 5.0_f64),
        (paper_size.0 - 5.0, paper_size.0),
    ] {
        for i in 0..(paper_size.1 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [j.0, (i as f64 + 1.0) * BORDER_RASTER as f64],
                [j.1, (i as f64 + 1.0) * BORDER_RASTER as f64],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, Stroke::new(), vec![Style::Border]),
            ));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[
                        j.0 + 2.5,
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0
                    ]),
                    0.0,
                    letters[i as usize].to_string(),
                    Effects::new(),
                    false,
                    vec![Style::TextSheet],
                ),
            ));
        }
    }

    // the head
    let pts: Array2<f64> = arr2(&[
        [paper_size.0 - 120.0, paper_size.1 - 40.0],
        [paper_size.0 - 5.0, paper_size.1 - 5.0],
    ]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(pts, Stroke::new(), vec![Style::Border]),
    ));
    /* plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_size.0 - 120.0, paper_size.1 - 30.0],
                [paper_size.0 - 5.0, paper_size.1 - 30.0],
            ]),
            stroke.width,
            stroke.linetype.clone(),
            LineCap::Butt,
            stroke.color,
        ),
    )); */
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_size.0 - 120.0, paper_size.1 - 10.0],
                [paper_size.0 - 5.0, paper_size.1 - 10.0],
            ]),
            Stroke::new(),
            None,
            vec![Style::Border],
        ),
    ));
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_size.0 - 120.0, paper_size.1 - 16.0],
                [paper_size.0 - 5.0, paper_size.1 - 16.0],
            ]),
            Stroke::new(),
            None,
            vec![Style::Border],
        ),
    ));

    // if let Some(title_block) = item {
    let left = paper_size.0 - 117.0;
    let mut effects: Effects = Effects::new();
    effects.justify.push(String::from("left"));
    for comment in title_block.query(el::TITLE_BLOCK_COMMENT) {
    // for (key, comment) in &title_block.comment {
        let key: usize = comment.get(0).unwrap();
        if key == 1 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_size.1 - 25.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    effects.clone(),
                    false,
                    vec![Style::TextHeader],
                ),
            ));
        } else if key == 2 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_size.1 - 29.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    effects.clone(),
                    false,
                    vec![Style::TextHeader],
                ),
            ));
        } else if key == 3 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_size.1 - 33.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    effects.clone(),
                    false,
                    vec![Style::TextHeader],
                ),
            ));
        } else if key == 4 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_size.1 - 37.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    effects.clone(),
                    false,
                    vec![Style::TextHeader],
                ),
            ));
        }
    }
    if let Some(company) = title_block.query(el::TITLE_BLOCK_COMPANY).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[left, paper_size.1 - 21.0]),
                0.0,
                company.get(0).unwrap(),
                effects.clone(),
                false,
                vec![Style::TextHeader],
            ),
        ));
    }
    if let Some(title) = title_block.query(el::TITLE_BLOCK_TITLE).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[left, paper_size.1 - 13.0]),
                0.0,
                title.get(0).unwrap(),
                effects.clone(),
                false,
                vec![Style::TextHeader],
            ),
        ));
    }

    plotter.push(PlotItem::Text(
        99,
        Text::new(
            arr1(&[left, paper_size.1 - 8.0]),
            0.0,
            String::from("xxx"),
            effects.clone(),
            false,
            vec![Style::TextHeader],
        ),
    ));

    if let Some(date) = title_block.query(el::TITLE_BLOCK_DATE).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[paper_size.0 - 90.0, paper_size.1 - 8.0]),
                0.0,
                date.get(0).unwrap(),
                effects.clone(),
                false,
                vec![Style::TextHeader],
            ),
        ));
    }
    if let Some(rev) = title_block.query(el::TITLE_BLOCK_REV).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[paper_size.0 - 20.0, paper_size.1 - 8.0]),
                0.0,
                format!("Rev: {}", <Sexp as SexpValueQuery::<String>>::get(rev, 0).unwrap()),
                effects.clone(),
                false,
                vec![Style::TextHeader],
            ),
        ));
    }
    Some(plotter)
}
