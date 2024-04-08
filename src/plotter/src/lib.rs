use std::{collections::HashMap, fmt, fs::File, io::Read, sync::Mutex};

use fontdue::{layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle}, Font};

use log::debug; 

use ndarray::{arr1, arr2, Array1, Array2};

//pub mod cairo_plotter;
pub mod error;
pub mod gerber;
pub mod pcb;
pub mod schema;
pub mod svg;
pub mod themer;

pub use error::Error;
use rust_fontconfig::{FcFontCache, FcPattern};

use self::themer::Themer;
use sexp::{el, PaperSize, Sexp, SexpValueQuery, SexpValuesQuery};

use lazy_static::lazy_static;

const BORDER_RASTER: i32 = 60;

// -----------------------------------------------------------------------------------------------------------
// ---                                             sexp utils                                              ---
// -----------------------------------------------------------------------------------------------------------

/// Text Effects
#[derive(Clone, Debug, Default)]
pub struct Effects {
    ///The optional face token indicates the font family. It should be a TrueType font family name
    ///or "KiCad Font" for the KiCad stroke font. (from version 7)
    pub font_face: String,
    ///The size token attributes define the font height and width.
    pub font_size: Vec<f64>,
    ///The thickness token attribute defines the line thickness of the font.
    pub font_thickness: f64,
    ///The color token attribute defines the color of the font.
    pub font_color: Color,
    /// The bold token specifies if the font should be bold.
    pub bold: bool,
    /// The italic token specifies if the font should be italicized.
    pub italic: bool,
    /// The optional justify token attributes define if the text is justified horizontally right or
    /// left and/or vertically top or bottom and/or mirrored. If the justification is not defined,
    /// then the text is center justified both horizontally and vertically and not mirrored.
    pub justify: Vec<String>,
    /// The optional hide token defines if the text is hidden.
    pub hide: bool,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            font_face: "default".to_string(),
            font_size: Vec::from([1.27, 1.27]),
            font_thickness: 1.27,
            font_color: Color::None,
            bold: false,
            italic: false,
            justify: Vec::new(),
            hide: false,
        }
    }
}

impl From<&Sexp> for Effects {
    fn from(element: &Sexp) -> Self {
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
                    let c: Vec<u16> = color.values();
                    effects.font_color = c.into()
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
            effects.hide = if let Some(hide) = e.query("hide").next() {
                let values: Vec<String> = hide.values();

                values.contains(&"yes".to_string())
            } else {
                values.contains(&"hide".to_string())
            };
        }
        effects
    }
}


#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    None,
    Rgb(u16, u16, u16),
    Rgba(u16, u16, u16, u16),
}

impl From<Vec<u16>> for Color {
    fn from(value: Vec<u16>) -> Self {
        if value.len() == 3 {
            Color::Rgb(value[0], value[1], value[2])
        } else if value.len() == 4 {
            Color::Rgba(value[0], value[1], value[2], value[3])
        } else {
            panic!("can not parse color {:?}", value);
        }
    }
}

///implement the rust format trait for Color
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::None => write!(f, ""),
            Color::Rgb(r, g, b) => write!(f, "rgb({},{},{})", r, g, b),
            Color::Rgba(r, g, b, a) => write!(f, "rgba({},{},{},{})", r, g, b, a),
        }
    }
}

/// The stroke token defines how the outlines of graphical objects are drawn.
#[derive(Clone, Default, Debug)]
pub struct Stroke {
    /// The width token attribute defines the line width of the graphic object.
    pub linewidth: f64,
    /// The type token attribute defines the line style of the graphic object. Valid stroke line styles are:
    /// - dash
    /// - dash_dot
    /// - dash_dot_dot (from version 7)
    /// - dot
    /// - default
    /// - solid
    pub linetype: String,
    /// defines the color of the graphic object
    pub linecolor: Color,
    /// defines the fill color of the graphic object, None if not filled.
    pub fillcolor: Color,
}
impl Stroke {
    pub fn new() -> Self {
        Self {
            linewidth: 0.0,
            linetype: String::from("default"),
            linecolor: Color::None,
            fillcolor: Color::None,
        }
    }
}

impl From<&Sexp> for Stroke {
    fn from(element: &Sexp) -> Self {
        //(stroke (width 2) (type dash_dot_dot) (color 0 255 0 1))
        let mut stroke = Stroke::new();
        if let Some(s) = element.query(el::STROKE).next() {

            let linewidth = s.query("width").next();
            if let Some(width) = linewidth {
                stroke.linewidth = width.get(0).unwrap()
            }
            let linetype = s.query("type").next();
            if let Some(linetype) = linetype {
                stroke.linetype = linetype.get(0).unwrap()
            }
            if let Some(color) = s.query("color").next() {
                let colors: Vec<u16> =  color.values();
                stroke.linecolor = colors.into()
            }
        }
        stroke
    }
}

// -----------------------------------------------------------------------------------------------------------
// ---                                          The plotter model                                          ---
// -----------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
///The color theme
pub enum Theme {
    ///Kicad alike theme.
    #[default]
    Kicad2020,
    BlackWhite,
    BlueGreenDark,
    BlueTone,
    EagleDark,
    Nord,
    SolarizedDark,
    SolarizedLight,
    WDark,
    WLight,
    ///Behave Dark Theme
    BehaveDark,
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
pub trait PlotterImpl<'a> {

    ///Plot the data.
    //fn plot<W: Write + 'static>(
    fn plot(
        &mut self,
        plot_items: &[PlotItem],
        size: Array2<f64>,
    ) -> Result<(), Error>;
}

///Draw all PlotItems.
pub trait Draw<T> {
    fn draw(&self, items: &[PlotItem], document: &mut T);
}

///Draw a PlotItem.
pub trait Drawer<I, T> {
    fn item(&self, item: &I, document: &mut T);
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

impl From<&Sexp> for FillType {
    fn from(element: &Sexp) -> Self {
        if let Some(fill) = element.query("fill").next() {
            let t: String = fill.value("type").unwrap();
            FillType::from(&t)
        } else {
            FillType::from(&String::from("default"))
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

#[derive(Debug, Clone, Eq, PartialEq)]
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
    GlobalLabel,
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
    FCu,
    In1Cu,
    In2Cu,
    In3Cu,
    In4Cu,
    In5Cu,
    In6Cu,
    In7Cu,
    In8Cu,
    In9Cu,
    In10Cu,
    In11Cu,
    In12Cu,
    In13Cu,
    In14Cu,
    In15Cu,
    In16Cu,
    In17Cu,
    In18Cu,
    In19Cu,
    In20Cu,
    In21Cu,
    In22Cu,
    In23Cu,
    In24Cu,
    In25Cu,
    In26Cu,
    In27Cu,
    In28Cu,
    In29Cu,
    In30Cu,
    BCu,
    BAdhes,
    FAdhes,
    BPaste,
    FPaste,
    BSilkS,
    FSilkS,
    BMask,
    FMask,
    DwgsUser,
    CmtsUser,
    Eco1User,
    Eco2User,
    EdgeCuts,
    Margin,
    BCrtYd,
    FCrtYd,
    BFab,
    FFab,
    User1,
    User2,
    User3,
    User4,
    User5,
    User6,
    User7,
    User8,
    User9,
    None,
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
            Style::GlobalLabel => write!(f, "schema-global-label"),
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

            Style::BCu => write!(f, "B_Cu"),
            Style::FCu => write!(f, "F_Cu"),
            Style::In1Cu => write!(f, "In1_Cu"),
            Style::In2Cu => write!(f, "In2_Cu"),
            Style::In3Cu => write!(f, "In3_Cu"),
            Style::In4Cu => write!(f, "In4_Cu"),
            Style::In5Cu => write!(f, "In5_Cu"),
            Style::In6Cu => write!(f, "In6_Cu"),
            Style::In7Cu => write!(f, "In7_Cu"),
            Style::In8Cu => write!(f, "In8_Cu"),
            Style::In9Cu => write!(f, "In9_Cu"),
            Style::In10Cu => write!(f, "In10_Cu"),
            Style::In11Cu => write!(f, "In11_Cu"),
            Style::In12Cu => write!(f, "In12_Cu"),
            Style::In13Cu => write!(f, "In13_Cu"),
            Style::In14Cu => write!(f, "In14_Cu"),
            Style::In15Cu => write!(f, "In15_Cu"),
            Style::In16Cu => write!(f, "In16_Cu"),
            Style::In17Cu => write!(f, "In17_Cu"),
            Style::In18Cu => write!(f, "In18_Cu"),
            Style::In19Cu => write!(f, "In19_Cu"),
            Style::In20Cu => write!(f, "In20_Cu"),
            Style::In21Cu => write!(f, "In21_Cu"),
            Style::In22Cu => write!(f, "In22_Cu"),
            Style::In23Cu => write!(f, "In23_Cu"),
            Style::In24Cu => write!(f, "In24_Cu"),
            Style::In25Cu => write!(f, "In25_Cu"),
            Style::In26Cu => write!(f, "In26_Cu"),
            Style::In27Cu => write!(f, "In27_Cu"),
            Style::In28Cu => write!(f, "In28_Cu"),
            Style::In29Cu => write!(f, "In29_Cu"),
            Style::In30Cu => write!(f, "In30_Cu"),
            Style::BAdhes => write!(f, "B_Adhes"),
            Style::FAdhes => write!(f, "F_Adhes"),
            Style::BPaste => write!(f, "B_Paste"),
            Style::FPaste => write!(f, "F_Paste"),
            Style::BSilkS => write!(f, "B_SilkS"),
            Style::FSilkS => write!(f, "F_SilkS"),
            Style::BMask => write!(f, "B_Mask"),
            Style::FMask => write!(f, "F_Mask"),
            Style::DwgsUser => write!(f, "Dwgs_User"),
            Style::CmtsUser => write!(f, "Cmts_User"),
            Style::Eco1User => write!(f, "Eco1_User"),
            Style::Eco2User => write!(f, "Eco2_User"),
            Style::EdgeCuts => write!(f, "Edge_Cuts"),
            Style::Margin => write!(f, "Margin"),
            Style::BCrtYd => write!(f, "B_Crtyd"),
            Style::FCrtYd => write!(f, "F_Crtyd"),
            Style::BFab => write!(f, "B_Fab"),
            Style::FFab => write!(f, "F_Fab"),
            Style::User1 => write!(f, "User_1"),
            Style::User2 => write!(f, "User_2"),
            Style::User3 => write!(f, "User_3"),
            Style::User4 => write!(f, "User_4"),
            Style::User5 => write!(f, "User_5"),
            Style::User6 => write!(f, "User_6"),
            Style::User7 => write!(f, "User_7"),
            Style::User8 => write!(f, "User_8"),
            Style::User9 => write!(f, "User_9"),

            Style::None => write!(f, "none"),
        }
    }
}

impl From<String> for Style {
    fn from(style: String) -> Self {
        if style.to_lowercase() == "b_cu" {
            Style::BCu
        } else if style.to_lowercase() == "f_cu" {
            Style::FCu
        } else if style.to_lowercase() == "in1_cu" {
            Style::In1Cu
        } else if style.to_lowercase() == "in2_cu" {
            Style::In2Cu
        } else if style.to_lowercase() == "in3_cu" {
            Style::In3Cu
        } else if style.to_lowercase() == "in4_cu" {
            Style::In4Cu
        } else if style.to_lowercase() == "in5_cu" {
            Style::In5Cu
        } else if style.to_lowercase() == "in6_cu" {
            Style::In6Cu
        } else if style.to_lowercase() == "in7_cu" {
            Style::In7Cu
        } else if style.to_lowercase() == "in8_cu" {
            Style::In8Cu
        } else if style.to_lowercase() == "in9_cu" {
            Style::In9Cu
        } else if style.to_lowercase() == "in10_cu" {
            Style::In10Cu
        } else if style.to_lowercase() == "in11_cu" {
            Style::In11Cu
        } else if style.to_lowercase() == "in12_cu" {
            Style::In12Cu
        } else if style.to_lowercase() == "in13_cu" {
            Style::In13Cu
        } else if style.to_lowercase() == "in14_cu" {
            Style::In14Cu
        } else if style.to_lowercase() == "in15_cu" {
            Style::In15Cu
        } else if style.to_lowercase() == "in16_cu" {
            Style::In16Cu
        } else if style.to_lowercase() == "in17_cu" {
            Style::In17Cu
        } else if style.to_lowercase() == "in18_cu" {
            Style::In18Cu
        } else if style.to_lowercase() == "in19_cu" {
            Style::In19Cu
        } else if style.to_lowercase() == "in20_cu" {
            Style::In20Cu
        } else if style.to_lowercase() == "in21_cu" {
            Style::In21Cu
        } else if style.to_lowercase() == "in22_cu" {
            Style::In22Cu
        } else if style.to_lowercase() == "in23_cu" {
            Style::In23Cu
        } else if style.to_lowercase() == "in24_cu" {
            Style::In24Cu
        } else if style.to_lowercase() == "in25_cu" {
            Style::In25Cu
        } else if style.to_lowercase() == "in26_cu" {
            Style::In26Cu
        } else if style.to_lowercase() == "in27_cu" {
            Style::In27Cu
        } else if style.to_lowercase() == "in28_cu" {
            Style::In28Cu
        } else if style.to_lowercase() == "in29_cu" {
            Style::In29Cu
        } else if style.to_lowercase() == "in30_cu" {
            Style::In30Cu
        } else if style.to_lowercase() == "b_cu" {
            Style::BCu
        } else if style.to_lowercase() == "b_adhes" {
            Style::BAdhes
        } else if style.to_lowercase() == "f_adhes" {
            Style::FAdhes
        } else if style.to_lowercase() == "b_paste" {
            Style::BPaste
        } else if style.to_lowercase() == "f_paste" {
            Style::FPaste
        } else if style.to_lowercase() == "b_silks" {
            Style::BSilkS
        } else if style.to_lowercase() == "f_silks" {
            Style::FSilkS
        } else if style.to_lowercase() == "b_mask" {
            Style::BMask
        } else if style.to_lowercase() == "f_mask" {
            Style::FMask
        } else if style.to_lowercase() == "dwgs_user" {
            Style::DwgsUser
        } else if style.to_lowercase() == "cmts_user" {
            Style::CmtsUser
        } else if style.to_lowercase() == "eco1_user" {
            Style::Eco1User
        } else if style.to_lowercase() == "eco2_user" {
            Style::Eco2User
        } else if style.to_lowercase() == "edge_cuts" {
            Style::EdgeCuts
        } else if style.to_lowercase() == "margin" {
            Style::Margin
        } else if style.to_lowercase() == "b_crtyd" {
            Style::BCrtYd
        } else if style.to_lowercase() == "f_crtyd" {
            Style::FCrtYd
        } else if style.to_lowercase() == "b_fab" {
            Style::BFab
        } else if style.to_lowercase() == "f_fab" {
            Style::FFab
        } else if style.to_lowercase() == "user_1" {
            Style::User1
        } else if style.to_lowercase() == "user_2" {
            Style::User2
        } else if style.to_lowercase() == "user_3" {
            Style::User3
        } else if style.to_lowercase() == "user_4" {
            Style::User4
        } else if style.to_lowercase() == "user_5" {
            Style::User5
        } else if style.to_lowercase() == "user_6" {
            Style::User6
        } else if style.to_lowercase() == "user_7" {
            Style::User7
        } else if style.to_lowercase() == "user_8" {
            Style::User8
        } else if style.to_lowercase() == "user_9" {
            Style::User9
        } else {
            Style::None
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
}
impl Line {
    ///Line from absolute points with style.
    pub fn new(
        pts: Array2<f64>,
        stroke: Stroke,
        linecap: Option<LineCap>,
    ) -> Line {
        Line {
            pts,
            stroke,
            linecap,
        }
    }
}

pub struct Polyline {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
}

impl Polyline {
    pub fn new(pts: Array2<f64>, stroke: Stroke) -> Polyline {
        Polyline { pts, stroke }
    }
}

pub struct Rectangle {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
}

impl Rectangle {
    pub fn new(pts: Array2<f64>, stroke: Stroke) -> Rectangle {
        Rectangle { pts, stroke }
    }
}

#[derive(Debug)]
pub struct Circle {
    pub pos: Array1<f64>,
    pub radius: f64,
    pub stroke: Stroke,
}

impl Circle {
    pub fn new(pos: Array1<f64>, radius: f64, stroke: Stroke) -> Circle {
        Circle {
            pos,
            radius,
            stroke,
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
}
impl Arc {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Array1<f64>,
        start: Array1<f64>,
        end: Array1<f64>,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        stroke: Stroke,
    ) -> Arc {
        Arc {
            center,
            start,
            end,
            radius,
            start_angle,
            end_angle,
            stroke,
        }
    }
}

pub struct Text {
    pub pos: Array1<f64>,
    pub angle: f64,
    pub text: String,
    pub effects: Effects,
    pub label: bool,
}

impl Text {
    pub fn new(
        pos: Array1<f64>,
        angle: f64,
        text: String,
        effects: Effects,
        label: bool,
    ) -> Text {
        Text {
            pos,
            angle,
            text,
            effects,
            label,
        }
    }
}

// -----------------------------------------------------------------------------------------------------------
// ---                             collect the plot model from the sexp tree                               ---
// -----------------------------------------------------------------------------------------------------------

fn cached_font(text: &str, effects: &Effects) -> Array1<f64> {

    lazy_static! {
        static ref FONT_CACHE: FcFontCache = FcFontCache::build();
        static ref FONTS: Mutex<HashMap<String, Font>> = Mutex::new(HashMap::new());
    }

    let mut last = FONTS.lock().unwrap();
    if !last.contains_key(&effects.font_face) {
        let result = FONT_CACHE.query(&FcPattern {
          name: Some(String::from(&effects.font_face)),
          .. Default::default()
        });

        let result = result.unwrap().path.to_string();
        let mut f = File::open(result).unwrap();
        let mut font = Vec::new();
        f.read_to_end(&mut font).unwrap();

        last.insert(effects.font_face.to_string(), Font::from_bytes(font, fontdue::FontSettings::default()).unwrap());
    }
    
    let fonts = &[last.get(&effects.font_face).unwrap()];
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    layout.reset(&LayoutSettings {
        ..LayoutSettings::default()
    });
    layout.append(fonts, &TextStyle::new(text, (*effects.font_size.first().unwrap() * 1.33333333) as f32, 0));
    let width: usize = layout.glyphs().iter().map(|g| g.width).sum();

    arr1(&[width as f64, *effects.font_size.first().unwrap() * 1.33333333])
}

pub trait Outline {
    fn text_pos(&self, at: Array1<f64>, text: String, angle: f64, effects: Effects) -> Array2<f64> {
        let themer = Themer::new(Theme::default()); //TODO should we pass a theme here?
        let size = self.text_size(
            &Text::new(
                arr1(&[0.0, 0.0]),
                angle,
                text,
                themer.get_effects(effects.clone(), &[Style::Property]),
                false,
            ),
        );

        let mut x = at[0];
        let mut y = at[1];
        if effects.justify.contains(&String::from("right")) {
            x -= size[0];
        } else if !effects.justify.contains(&String::from("left")) {
            x -= size[0] / 2.0;
        }
        if effects.justify.contains(&String::from("top")) {
            y -= size[1];
        } else if !effects.justify.contains(&String::from("bottom")) {
            y -= size[1] / 2.0;
        }
        arr2(&[[x, y], [x + size[0], y + size[1]]])
    }

    /// Get the text boundery box.
    ///
    /// * `item`: The text item.
    /// * `themer`: The themer selection.
    fn text_size(&self, item: &Text) -> Array1<f64> {
        cached_font(&item.text, &item.effects)
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
    fn bounds(&self, items: &[PlotItem]) -> Array2<f64> {
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
                    let outline = self.text_size(text);
                    let x = text.pos[0];
                    let y = text.pos[1];
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

pub fn border(title_block: &Sexp, paper_size: PaperSize, themer: &Themer) -> Option<Vec<PlotItem>> {
    let mut plotter: Vec<PlotItem> = Vec::new();
    let paper_dimension: (f64, f64) = paper_size.clone().into();
 
    //outline
    let pts: Array2<f64> = arr2(&[[5.0, 5.0], [paper_dimension.0 - 5.0, paper_dimension.1 - 5.0]]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(pts, themer.get_stroke(Stroke::new(), &[Style::Border])),
    ));

    //horizontal raster
    for j in &[(0.0_f64, 5.0_f64), (paper_dimension.1 - 5.0, paper_dimension.1)] {
        for i in 0..(paper_dimension.0 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.0],
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.1],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, themer.get_stroke(Stroke::new(), &[Style::Border])),
            ));
        }
        for i in 0..(paper_dimension.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0,
                        j.0 + 2.5,
                    ]),
                    0.0,
                    (i + 1).to_string(),
                    themer.get_effects(Effects::new(), &[Style::TextSheet]),
                    false,
                ),
            ));
        }
    }

    //vertical raster
    let letters = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    for j in &[(0.0_f64, 5.0_f64), (paper_dimension.0 - 5.0, paper_dimension.0)] {
        for i in 0..(paper_dimension.1 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [j.0, (i as f64 + 1.0) * BORDER_RASTER as f64],
                [j.1, (i as f64 + 1.0) * BORDER_RASTER as f64],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, themer.get_stroke(Stroke::new(), &[Style::Border])),
            ));
        }
        for i in 0..(paper_dimension.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[
                        j.0 + 2.5,
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0,
                    ]),
                    0.0,
                    letters[i as usize].to_string(),
                    themer.get_effects(Effects::new(),&[Style::TextSheet]),
                    false,
                ),
            ));
        }
    }

    // the head
    let pts: Array2<f64> = arr2(&[
        [paper_dimension.0 - 120.0, paper_dimension.1 - 40.0],
        [paper_dimension.0 - 5.0, paper_dimension.1 - 5.0],
    ]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(pts, themer.get_stroke(Stroke::new(), &[Style::Border])),
    ));
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_dimension.0 - 120.0, paper_dimension.1 - 10.0],
                [paper_dimension.0 - 5.0, paper_dimension.1 - 10.0],
            ]),
            themer.get_stroke(Stroke::new(), &[Style::Border]),
            None,
        ),
    ));
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_dimension.0 - 120.0, paper_dimension.1 - 16.0],
                [paper_dimension.0 - 5.0, paper_dimension.1 - 16.0],
            ]),
            themer.get_stroke(Stroke::new(), &[Style::Border]),
            None,
        ),
    ));

    // if let Some(title_block) = item {
    let left = paper_dimension.0 - 117.0;
    let mut effects: Effects = Effects::new();
    effects.justify.push(String::from("left"));
    for comment in title_block.query(el::TITLE_BLOCK_COMMENT) {
        // for (key, comment) in &title_block.comment {
        let key: usize = comment.get(0).unwrap();
        if key == 1 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_dimension.1 - 25.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    themer.get_effects(effects.clone(), &[Style::TextHeader]),
                    false,
                ),
            ));
        } else if key == 2 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_dimension.1 - 29.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    themer.get_effects(effects.clone(), &[Style::TextHeader]),
                    false,
                ),
            ));
        } else if key == 3 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_dimension.1 - 33.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    themer.get_effects(effects.clone(), &[Style::TextHeader]),
                    false,
                ),
            ));
        } else if key == 4 {
            plotter.push(PlotItem::Text(
                99,
                Text::new(
                    arr1(&[left, paper_dimension.1 - 37.0]),
                    0.0,
                    comment.get(1).unwrap(),
                    themer.get_effects(effects.clone(), &[Style::TextHeader]),
                    false,
                ),
            ));
        }
    }
    if let Some(company) = title_block.query(el::TITLE_BLOCK_COMPANY).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[left, paper_dimension.1 - 21.0]),
                0.0,
                company.get(0).unwrap(),
                themer.get_effects(effects.clone(), &[Style::TextHeader]),
                false,
            ),
        ));
    }
    if let Some(title) = title_block.query(el::TITLE_BLOCK_TITLE).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[left, paper_dimension.1 - 13.0]),
                0.0,
                title.get(0).unwrap(),
                themer.get_effects(effects.clone(), &[Style::TextHeader]),
                false,
            ),
        ));
    }

    plotter.push(PlotItem::Text(
        99,
        Text::new(
            arr1(&[left, paper_dimension.1 - 8.0]),
            0.0,
            paper_size.to_string(),
            themer.get_effects(effects.clone(), &[Style::TextHeader]),
            false,
        ),
    ));

    if let Some(date) = title_block.query(el::TITLE_BLOCK_DATE).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[paper_dimension.0 - 90.0, paper_dimension.1 - 8.0]),
                0.0,
                date.get(0).unwrap(),
                themer.get_effects(effects.clone(), &[Style::TextHeader]),
                false,
            ),
        ));
    }
    if let Some(rev) = title_block.query(el::TITLE_BLOCK_REV).next() {
        plotter.push(PlotItem::Text(
            99,
            Text::new(
                arr1(&[paper_dimension.0 - 20.0, paper_dimension.1 - 8.0]),
                0.0,
                format!(
                    "Rev: {}",
                    <Sexp as SexpValueQuery::<String>>::get(rev, 0).unwrap()
                ),
                themer.get_effects(effects.clone(), &[Style::TextHeader]),
                false,
            ),
        ));
    }
    Some(plotter)
}

#[cfg(test)]
mod tests {
    use ndarray::{arr1, arr2};
    use crate::{
        schema::Themer, Arc, Circle, Effects, Line, Outline, PlotItem, Polyline, Rectangle, Stroke, Style, Text, Theme
    };

    #[test]
    fn from_style() {
        assert_eq!(Style::BCu, Style::from(String::from("B_Cu")));
    }

    #[test]
    fn test_text_size() {
        let themer = Themer::new(Theme::Kicad2020);
        let mut effects = Effects::new();
        effects.font_face = String::from("osifont");
        effects.font_size = vec![10.0, 10.0];
        let text = Text::new(
            arr1(&[100.0, 100.0]),
            0.0,
            String::from("teststring"),
            themer.get_effects(Effects::new(), &[Style::TextPinName]),
            false,
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.text_size(&text);
        assert_eq!(arr1(&[10.0, 1.0666666640000002]), bounds);
    }
    #[test]
    fn test_bounds_circle() {
        let circle = Circle::new(arr1(&[100.0, 100.0]), 0.45, Stroke::new());
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(
            &[PlotItem::Circle(0, circle)],
        );

        assert_eq!(
            arr2(&[[100.0 - 0.45, 100.0 - 0.45], [100.0 + 0.45, 100.0 + 0.45]]),
            bounds
        );
    }
    #[test]
    fn test_bounds_rect() {
        let rect = Rectangle::new(
            arr2(&[[100.0, 100.0], [150.0, 150.0]]),
            Stroke::new(),
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(
            &[PlotItem::Rectangle(0, rect)],
        );

        assert_eq!(arr2(&[[100.0, 100.0], [150.0, 150.0]]), bounds);
    }
    #[test]
    fn test_bounds_line() {
        let line = Line::new(
            arr2(&[[100.0, 100.0], [150.0, 150.0]]),
            Stroke::new(),
            None,
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(&[PlotItem::Line(0, line)]);

        assert_eq!(arr2(&[[100.0, 100.0], [150.0, 150.0]]), bounds);
    }
    #[test]
    fn test_bounds_polyline() {
        let line = Polyline::new(
            arr2(&[[100.0, 100.0], [150.0, 150.0], [75.0, 75.0]]),
            Stroke::new(),
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(
            &[PlotItem::Polyline(0, line)],
        );
        assert_eq!(arr2(&[[75.0, 75.0], [150.0, 150.0]]), bounds);
    }
    #[test]
    fn test_bounds_arc() {
        let arc = Arc::new(
            arr1(&[100.0, 100.0]),
            arr1(&[99.0, 99.0]),
            arr1(&[101.0, 101.0]),
            0.25,
            0.0,
            360.0,
            Stroke::new(),
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(&[PlotItem::Arc(0, arc)]);

        assert_eq!(arr2(&[[99.0, 99.0], [101.0, 101.0]]), bounds);
    }
    #[test]
    fn test_bounds_text() {
        let themer = Themer::new(Theme::Kicad2020);
        let mut effects = Effects::new();
        effects.font_face = String::from("osifont");
        effects.font_size = vec![1.25, 1.2];
        let text = Text::new(
            arr1(&[100.0, 100.0]),
            0.0,
            String::from("teststring"),
            themer.get_effects(effects, &[Style::TextPinName]),
            false,
        );
        struct TestOutline;
        impl Outline for TestOutline {}

        let outline = TestOutline;
        let bounds = outline.bounds(&[PlotItem::Text(0, text)]);
        assert_eq!(arr2(&[[100.0, 100.0], [110.0, 101.066666664]]), bounds);
    }
    /* #[test]
    fn plt_schema() {
        /* let doc = SexpParser::load("tests/dco.kicad_sch").unwrap();
        let tree = sexp::SexpTree::from(doc.iter()).unwrap(); */

        let mut plotter = SchemaPlot::new()
            .border(false).theme(Theme::Kicad2020).scale(2.0);

        //plotter.open("tests/dco.kicad_sch");
        plotter.open("/home/etienne/github/elektrophon/src/hall/main/main.kicad_sch");
        //plotter.open("/home/etienne/github/elektrophon/src/resonanz/main/main.kicad_sch");
        for page in plotter.iter() {
            let mut buffer = Vec::<u8>::new();
            let mut file = File::create("out.svg").unwrap();
            let mut svg_plotter = SvgPlotter::new(&mut file);
            plotter.write(page.0, &mut svg_plotter).unwrap();
        }
        
        // plotter.plot(&mut svg_plotter);

        /* svg_plotter
            .plot(&tree, &mut buffer, true, 1.0, None, false)
            .unwrap(); */

        // assert!(!buffer.is_empty());
        assert!(true);
    } */
}
