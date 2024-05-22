use std::{fmt, iter::Iterator};

use ndarray::{arr1, Array1, Array2, ArrayView, Axis};
use itertools::Itertools;
use log::{error, trace};

use crate::{Pts, Sexp, SexpValueQuery, SexpValuesQuery};

//function that splits a spring at the first colon in the name
fn split_name(name: &str) -> (&str, &str) {
    name.split_once(':').unwrap()
}

//TODO think about where to place that
#[cfg(test)]
mod test_utils {
    //test split_name function
    #[test]
    fn split_name_test() {
        assert_eq!(
            ("Connector", "Audiojack_Switch_T"),
            super::split_name("Connector:Audiojack_Switch_T")
        );
    }
}

pub mod el {
    pub const AT: &str = "at";
    pub const EFFECTS: &str = "effects";
    pub const JUNCTION: &str = "junction";
    pub const LIB_ID: &str = "lib_id";
    pub const LIB_SYMBOLS: &str = "lib_symbols";
    pub const MIRROR: &str = "mirror";
    pub const NO_CONNECT: &str = "no_connect";
    pub const PROPERTY: &str = "property";
    pub const PTS: &str = "pts";
    pub const STROKE: &str = "stroke";
    pub const SYMBOL: &str = "symbol";
    pub const WIRE: &str = "wire";
    pub const XY: &str = "xy";
    pub const HIDE: &str = "hide";
}

//-----------------------------------------------------------------------------
// general elements
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Stroke<'a> {
    sexp: &'a Sexp,
}

#[derive(Debug)]
pub struct Effects<'a> {
    sexp: &'a Sexp,
}

//implement display for Effects
impl<'a> fmt::Display for Effects<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.justify();
        write!(f, "Effects(visible({})", self.visible())
        //write!(f, "Justify(visible({})", self.justify().join(' '))
    }
}

pub enum Linetype {
    Dash,
    DashDot,
    DashDotDot,
    Dot,
    Default,
    Solid,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0, 0, 0, 1.0)
    }
}


//  [(justify [left | right] [top | bottom] [mirror])]
pub enum Justify {
    Left,
    Right,
    Top,
    Bottom,
    Mirror,
}

//implemnet from trait for String to Justify
impl From<String> for Justify {
    fn from(s: String) -> Self {
        match s.as_str() {
            "left" => Justify::Left,
            "right" => Justify::Right,
            "top" => Justify::Top,
            "bottom" => Justify::Bottom,
            "mirror" => Justify::Mirror,
            _ => panic!("unknown justify: {}", s),
        }
    }
}

//(stroke
//  (width WIDTH)
//  (type TYPE)
//  (color R G B A)
//)
impl<'a> Stroke<'a> {
    pub fn new(sexp: &'a Sexp) -> Self {
        Self { sexp }
    }
    pub fn width(&self) -> Option<f32> {
        self.sexp.value("width")
    }
    pub fn linetype(&self) -> Option<Linetype> {
        let lt: Option<String> = self.sexp.value("type");
        lt.map(|lt| match lt.as_str() {
            "dash" => Linetype::Dash,
            "dash_dot" => Linetype::DashDot,
            "dash_dot_dot" => Linetype::DashDotDot,
            "dot" => Linetype::Dot,
            "default" => Linetype::Default,
            "solid" => Linetype::Default,
            _ => panic!("unknown linetype: {}", lt),
        })
    }
    pub fn color(&self) -> Option<Color> {
        let mut color = self.sexp.query("color");
        if let Some(color) = color.next() {
            let values: Vec<String> = color.values();
            let (r, g, b, a) = values.iter().collect_tuple().unwrap();
            Some(Color {
                r: r.parse::<u8>().unwrap(),
                g: g.parse::<u8>().unwrap(),
                b: b.parse::<u8>().unwrap(),
                a: a.parse::<f32>().unwrap(),
            })
        } else {
            None
        }
    }
}

pub trait GetStroke<'a> {
    fn stroke(&self) -> Option<Stroke<'a>>;
}
macro_rules! impl_stroke {
    ($($t:ty),+ $(,)?) => ($(
        impl<'a> GetStroke<'a> for $t {
            fn stroke(&self) -> Option<Stroke<'a>> {
                let stroke = self.sexp.query(el::STROKE).next();
                stroke.map(|stroke| Stroke { sexp: stroke })
            }
        }
    )+)
}

//(effects
//  (font
//    [(face FACE_NAME)]
//    (size HEIGHT WIDTH)
//    [(thickness THICKNESS)]
//    [bold]
//    [italic]
//    [(line_spacing LINE_SPACING)]
//  )
//  [(justify [left | right] [top | bottom] [mirror])]
//  [hide]
//)
impl<'a> Effects<'a> {
    pub fn new(sexp: &'a Sexp) -> Self {
        Self { sexp }
    }

    pub fn justify(&self) -> Vec<Justify> {
        let justify: Option<String> = self.sexp.value("justify");
        if let Some(justify) = justify {
            //TODO println!("--justify {}", justify);
            Vec::new()
        } else {
            Vec::new()
        }

    }

    pub fn size(&self) -> f32 {
        let size: Option<Array1<f32>> = self.sexp.value("size");
        if let Some(size) = size {
            size[0]
        } else {
            1.0
        }
    }

    pub fn color(&self) -> Option<Color> {
        let mut color = self.sexp.query("color");
        if let Some(color) = color.next() {
            let values: Vec<String> = color.values();
            let (r, g, b, a) = values.iter().collect_tuple().unwrap();
            Some(Color {
                r: r.parse::<u8>().unwrap(),
                g: g.parse::<u8>().unwrap(),
                b: b.parse::<u8>().unwrap(),
                a: a.parse::<f32>().unwrap(),
            })
        } else {
            None
        }
    }

    pub fn visible(&self) -> bool {
        let new_visible: Option<String> = self.sexp.value("hide");
        if let Some(new_visible) = new_visible {
            &new_visible != "yes"
        } else {
            let visible: Vec<String> = self.sexp.values();
            !visible.contains(&el::HIDE.to_string())
        }
    }
}

//-----------------------------------------------------------------------------
// schema elements
// -----------------------------------------------------------------------------
pub enum Element<'a> {
    NoConnect(NoConnect<'a>),
    Symbol(Symbol<'a>),
    Wire(Wire<'a>),
    Junction(Junction<'a>),
}

trait Points {
    fn pts(&self) -> crate::Pts;
}

pub trait At {
    fn at(&self) -> Array1<f32>;
    fn angle(&self) -> f32;
    fn mirror(&self) -> String;
}

macro_rules! impl_at {
    ($($t:ty),+ $(,)?) => ($(
        impl<'a> At for $t {
            fn at(&self) -> Array1<f32> {
                <Sexp as SexpValueQuery<Array1<f32>>>::value(self.sexp, el::AT).expect("expecting x and y are allways set")
            //    .unwrap();
            //Vector2F::new(at[0] as f32, at[1] as f32)
            }
            fn angle(&self) -> f32 {
            let at = <Sexp as SexpValueQuery<Array1<f64>>>::value(self.sexp, el::AT)
                .unwrap();
            at[2] as f32 //TODO
            }
            fn mirror(&self) -> String {
                if let Some(mirror) = <Sexp as SexpValueQuery<String>>::value(self.sexp, el::MIRROR) {
                    mirror
                } else {
                    String::new()
                }
            }
        }
    )+)
}

macro_rules! impl_points {
    ($($t:ty),+ $(,)?) => ($(
        impl<'a> Points for $t {
            fn pts(&self) -> crate::Pts {

                let mut pts: Array2<f32> = Array2::zeros((0, 2));
                for pt in self.sexp.query(el::PTS) {
                    for xy in pt.query(el::XY) {
                        pts.push_row(ArrayView::from(&[
                            xy.get(0).unwrap(),
                            xy.get(1).unwrap(),
                        ]))
                        .unwrap();
                    }
                }
                pts
            }
        }
    )+)
}

impl_at!(NoConnect<'a>);
impl_at!(Junction<'a>);
impl_points!(Wire<'a>);

pub struct Symbol<'a> {
    root: &'a Sexp,
    sexp: &'a Sexp,
}
impl_at!(Symbol<'a>);

impl<'a> Symbol<'a> {
    pub fn lib_id(&self) -> String {
        self.sexp.value(el::LIB_ID).unwrap()
    }
    pub fn unit(&self) -> usize {
        self.sexp.value("unit").unwrap()
    }
    pub fn in_bom(&self) -> bool {
        let in_bom: String = self.sexp.value("in_bom").unwrap();
        in_bom == "yes"
    }
    pub fn on_board(&self) -> bool {
        let in_bom: String = self.sexp.value("on_board").unwrap();
        in_bom == "yes"
    }

    pub fn properties(&self) -> Vec<Properties<'a>> {
        self.sexp
            .query(el::PROPERTY)
            .collect::<Vec<&Sexp>>()
            .iter()
            .map(|x| Properties { sexp: x })
            .collect()
    }

    pub fn property(&self, name: &str) -> Option<String> {
        self.sexp
            .query(el::PROPERTY)
            .collect::<Vec<&Sexp>>()
            .iter()
            .filter_map(|x| {
                if <Sexp as SexpValueQuery::<String>>::get(x, 0).unwrap() == name.to_string() {
                    Some(<Sexp as SexpValueQuery::<String>>::get(x, 1))
                } else {
                    None
                }
            })
            .collect()
    }


    pub fn library(&self) -> LibrarySymbol {
        LibrarySymbol {
            sexp: self
                .root
                .query(el::LIB_SYMBOLS)
                .next()
                .unwrap()
                .query(el::SYMBOL)
                .filter(|x| {
                    let name: String = x.get(0).unwrap();
                    name == self.lib_id()
                })
                .collect::<Vec<&Sexp>>()
                .first()
                .unwrap(),
        }
    }
}

impl<'a> fmt::Display for Symbol<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Symbol(at({}x{}), angle= {}, mirror=({}), ref=\"{}\" value\"{}\" lib_id({}), unit({}))",
            self.at()[0],
            self.at()[1],
            self.angle(),
            self.mirror(),
            self.property("Reference").unwrap_or(String::from("NaN")),
            self.property("Value").unwrap_or(String::from("NaN")),
            self.lib_id(),
            self.unit()
        )
    }
}

pub struct LibrarySymbol<'a> {
    sexp: &'a Sexp,
}

impl<'a> LibrarySymbol<'a> {
    //TODO redundnant
    pub fn properties(&self) -> Vec<Properties<'a>> {
        let properties = self.sexp.query(el::PROPERTY).collect::<Vec<&Sexp>>();
        properties.iter().map(|x| Properties { sexp: x }).collect()
    }

    pub fn name(&self) -> String {
        <Sexp as SexpValueQuery<String>>::get(self.sexp, 0).unwrap()
    }

    pub fn pin_names_offset(&self) -> f32 {
        *self
            .sexp
            .query("pin_names")
            .collect::<Vec<&Sexp>>()
            .iter()
            .map(|x| <Sexp as SexpValueQuery<f32>>::value(x, "offset").unwrap())
            .collect::<Vec<f32>>()
            .first()
            .unwrap_or(&1.0)
    }

    pub fn pin_numbers(&self) -> bool {
        let pin_numbers = self.sexp.query("pin_numbers").collect::<Vec<&Sexp>>();
        if pin_numbers.is_empty() {
            true
        } else {
            !<Sexp as SexpValuesQuery<Vec<String>>>::values(
                pin_numbers.first().expect("tested before for is_empty"),
            )
            .contains(&el::HIDE.to_string())
        }
    }
    pub fn pin_names(&self) -> bool {
        let pin_names = self.sexp.query("pin_names").collect::<Vec<&Sexp>>();
        if pin_names.is_empty() {
            true
        } else {
            !<Sexp as SexpValuesQuery<Vec<String>>>::values(
                pin_names.first().expect("tested before for is_empty"),
            )
            .contains(&el::HIDE.to_string())
        }
    }

    pub fn sub_symbols(&'a self) -> Vec<SubSymbol<'a>> {
        self.sexp
            .query(el::SYMBOL)
            .map(|x| SubSymbol {
                sexp: x,
                parent: self,
                lib_id: self.name(),
            })
            .collect()
    }
    pub fn sub_symbols_unit(&'a self, unit: usize) -> Vec<SubSymbol<'a>> {
        self.sexp
            .query(el::SYMBOL)
            .map(|x| SubSymbol {
                sexp: x,
                parent: self,
                lib_id: self.name(),
            })
            .filter(|s| s.unit() == 0 || s.unit() == unit)
            .collect()
    }
}

//-----------------------------------------------------------------------------
// the sub symbol
// -----------------------------------------------------------------------------

pub enum SubSymbolElement<'a> {
    Polyline(Polyline<'a>),
    Pin(Pin<'a>),
}

pub struct Pin<'a> {
    sexp: &'a Sexp,
    parent: &'a SubSymbol<'a>,
}
impl_at!(Pin<'a>);

//(pin
//  PIN_ELECTRICAL_TYPE
//  PIN_GRAPHIC_STYLE
//  POSITION_IDENTIFIER
//  (length LENGTH)
//  (name "NAME" TEXT_EFFECTS)
//  (number "NUMBER" TEXT_EFFECTS)
//)
impl<'a> Pin<'a> {
    pub fn electrical_type(&self) -> String {
        <Sexp as SexpValueQuery<String>>::get(self.sexp, 0).unwrap_or(String::from("default"))
    }
    pub fn graphic_style(&self) -> String {
        <Sexp as SexpValueQuery<String>>::get(self.sexp, 1).unwrap_or(String::from("default"))
    }
    pub fn length(&self) -> f32 {
        <Sexp as SexpValueQuery<f32>>::value(self.sexp, "length").unwrap_or(0.0)
    }
    pub fn name(&self) -> String {
        <Sexp as SexpValueQuery<String>>::get(self.sexp.query("name").next().unwrap(), 0)
            .unwrap_or(String::from("default"))
    }
    pub fn number(&self) -> u32 {
        <Sexp as SexpValueQuery<u32>>::get(self.sexp.query("number").next().unwrap(), 0)
            .unwrap_or(0)
    }
    pub fn parent(&self) -> &SubSymbol {
        self.parent
    }
}

pub struct SubSymbol<'a> {
    sexp: &'a Sexp,
    parent: &'a LibrarySymbol<'a>,
    lib_id: String,
}

impl<'a> SubSymbol<'a> {
    //TODO redundnant
    pub fn properties(&self) -> Vec<Properties<'a>> {
        let properties = self.sexp.query(el::PROPERTY).collect::<Vec<&Sexp>>();
        properties.iter().map(|x| Properties { sexp: x }).collect()
    }

    fn parse_numbers(&self) -> Vec<String> {
        let reference: String = self.sexp.get(0).unwrap(); //TODO could be string
        let (_, lib_name) = split_name(&self.lib_id);
        let numbers = reference.strip_prefix(lib_name).unwrap();
        numbers
            .split('_')
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    }

    pub fn unit(&self) -> usize {
        self.parse_numbers()
            .get(1)
            .unwrap()
            .parse::<usize>()
            .unwrap()
    }

    pub fn number(&self) -> usize {
        self.parse_numbers()
            .get(2)
            .unwrap()
            .parse::<usize>()
            .unwrap()
    }

    pub fn graphic(&'a self) -> Vec<GraphicType<'a>> {
        self.sexp
            .nodes()
            .filter_map(|node| match node.name.as_str() {
                "polyline" => Some(GraphicType::Polyline(Polyline { sexp: node })),
                "rectangle" => Some(GraphicType::Rectangle(Rectangle { sexp: node })),
                "pin" => Some(GraphicType::Pin(Pin {
                    sexp: node,
                    parent: self,
                })),
                _ => {
                    if node.name != "pin" {
                        error!("unknown graphic type: {}", node.name);
                    }
                    None
                }
            })
            .collect()
    }
    pub fn parent(&self) -> &LibrarySymbol {
        self.parent
    }
}

#[derive(Debug)]
pub struct Properties<'a> {
    sexp: &'a Sexp,
}
impl_at!(Properties<'a>);

impl<'a> Properties<'a> {
    pub fn name(&self) -> String {
        self.sexp.get(0).unwrap()
    }
    pub fn value(&self) -> String {
        self.sexp.get(1).unwrap()
    }
    pub fn effects(&self) -> Option<Effects> {
        if let Some(effect) = self.sexp.query(el::EFFECTS).next() {
            Some(Effects { sexp: effect })
        } else {
            None
        }
    }
}

//implement display for properties
impl<'a> fmt::Display for Properties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Property(name: \"{}\", value: \"{}\", at({}x{})), {})",
            self.name(),
            self.value(),
            self.at()[0],
            self.at()[1],
            if let Some(effect) = self.effects() {
                effect.to_string()
            } else {
                String::from("Effects(())")
            }
        )
    }
}

#[derive(Debug)]
pub struct Wire<'a> {
    sexp: &'a Sexp,
}
impl_stroke!(Wire<'a>);

#[derive(Debug)]
pub struct Junction<'a> {
    sexp: &'a Sexp,
}

impl<'a> Junction<'a> {
    pub fn diameter(&self) -> f32 {
        let d: f64 = self.sexp.value("diameter").unwrap();
        d as f32 //TODO
    }
}

pub struct NoConnect<'a> {
    sexp: &'a Sexp,
}

impl<'a> Wire<'a> {
    pub fn start(&self) -> Array1<f32> {
        let pts = self.sexp.query(el::PTS).next().unwrap();
        let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
        let xy: Array1<f32> = xy.first().unwrap().values();
        arr1(&[xy[0], xy[1]])
    }
    pub fn end(&self) -> Array1<f32> {
        let pts = self.sexp.query(el::PTS).next().unwrap();
        let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
        let xy: Array1<f32> = xy.get(1).unwrap().values();
        arr1(&[xy[0], xy[1]])
    }
}

pub struct Schema<'a> {
    sexp: &'a Sexp,
    elements: Vec<Element<'a>>,
}

// -----------------------------------------------------------------------------
// graphics
// -----------------------------------------------------------------------------

pub enum GraphicType<'a> {
    Polyline(Polyline<'a>),
    Rectangle(Rectangle<'a>),
    Pin(Pin<'a>),
}

#[derive(Debug)]
pub struct Polyline<'a> {
    sexp: &'a Sexp,
}
impl_stroke!(Polyline<'a>);

#[derive(Debug)]
pub struct Rectangle<'a> {
    sexp: &'a Sexp,
}
impl_stroke!(Rectangle<'a>);

impl<'a> Polyline<'a> {
    pub fn points(&self) -> crate::Pts {

        let mut pts: Array2<f32> = Array2::zeros((0, 2));
        for pt in self.sexp.query(el::PTS) {
            for xy in pt.query(el::XY) {
                pts.push_row(ArrayView::from(&[
                    xy.get(0).unwrap(),
                    xy.get(1).unwrap(),
                ]))
                .unwrap();
            }
        }
        pts
    }
}

impl<'a> fmt::Display for Polyline<'a> {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Polyline(pts: [{:?}])",
            self.points().axis_iter(Axis(1)).map(|pt| format!("[{}x{}]", pt[0], pt[1])).collect::<Vec<String>>().join(", "),
        )
    }
}

impl<'a> Rectangle<'a> {
    pub fn start(&self) -> crate::Pt {
        <Sexp as SexpValueQuery<Array1<f32>>>::value(self.sexp, "start").unwrap()
    }
    pub fn end(&self) -> crate::Pt {
        <Sexp as SexpValueQuery<Array1<f32>>>::value(self.sexp, "end").unwrap()
    }
}

impl<'a> fmt::Display for Rectangle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rectangle(start: {}x{}, end: {}x{})",
            self.start()[0],
            self.start()[1],
            self.end()[0],
            self.end()[1],
        )
    }
}

// -----------------------------------------------------------------------------
// schema
// -----------------------------------------------------------------------------

impl<'a> Schema<'a> {
    pub fn new(sexp: &'a Sexp) -> Self {
        let mut elements = Vec::new();
        for element in &sexp.nodes {
            match element {
                crate::SexpAtom::Node(ref node) => match node.name.as_str() {
                    el::SYMBOL => elements.push(Element::Symbol(Symbol {
                        root: sexp,
                        sexp: node,
                    })),
                    el::WIRE => elements.push(Element::Wire(Wire { sexp: node })),
                    el::NO_CONNECT => elements.push(Element::NoConnect(NoConnect { sexp: node })),
                    el::JUNCTION => elements.push(Element::Junction(Junction { sexp: node })),
                    _ => {
                        error!("unknown element: {}", node.name);
                    }
                },
                crate::SexpAtom::Value(_) => {}
                crate::SexpAtom::Text(_) => {}
            }
        }
        Schema { sexp, elements }
    }
    pub fn paper(&self) -> String {
        self.sexp.value("paper").unwrap()
    }
    pub fn paper_size(&self) -> Array1<f32> {
        let size: String = self.sexp.value("paper").unwrap();

        if size == "A5" {
            arr1(&[148.0, 210.0])
        } else if size == "A4" {
            arr1(&[297.0, 210.0])
        } else if size == "A3" {
            arr1(&[420.0, 297.0])
        } else if size == "A2" {
            arr1(&[420.0, 594.0])
        } else if size == "A1" {
            arr1(&[594.0, 841.0])
        } else {
            arr1(&[841.0, 1189.0])
        }
    }
}

pub struct SchemaIter<'a> {
    items: std::slice::Iter<'a, Element<'a>>,
}

impl<'a> Iterator for SchemaIter<'a> {
    type Item = &'a Element<'a>;
    ///Get the next node.
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

impl<'a> IntoIterator for &'a Schema<'a> {
    type Item = &'a Element<'a>;
    type IntoIter = SchemaIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        SchemaIter {
            items: self.elements.iter(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{SexpParser, SexpTree};

    #[test]
    fn paper() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        assert_eq!(String::from("A4"), schema.paper());
    }
    #[test]
    fn paper_size() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        assert_eq!(arr1(&[297.0, 210.0]), schema.paper_size());
    }
    //TODO ubtil i implement the elements this test will change all the time
    //#[test]
    //fn iterate_elements() {
    //    let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
    //    let sexp = SexpTree::from(doc.iter()).unwrap();
    //    let schema = Schema::new(sexp.root().unwrap());
    //    assert_eq!(403, schema.into_iter().count());
    //    assert_eq!(403, schema.into_iter().count());
    //}
    #[test]
    fn iterate_wires() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let wires = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Wire(w) => Some(w),
                _ => None,
            })
            .collect::<Vec<&Wire<'_>>>();
        assert_eq!(159, wires.len());

        //test pts
        let pts = wires.first().unwrap().pts();
        assert_eq!(2, pts.len());
        assert_eq!(arr1(&[179.07, 34.29]), pts.row(0));
        assert_eq!(arr1(&[179.07, 49.53]), pts.row(1));
    }
    #[test]
    fn iterate_no_connect() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let no_connect = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::NoConnect(nc) => Some(nc),
                _ => None,
            })
            .collect::<Vec<&NoConnect<'_>>>();
        assert_eq!(4, no_connect.len());

        //test pts
        let at = no_connect.first().unwrap().at();
        assert_eq!(arr1(&[53.34, 73.66]), at);
    }
    #[test]
    fn iterate_junction() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let no_connect = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::NoConnect(nc) => Some(nc),
                _ => None,
            })
            .collect::<Vec<&NoConnect<'_>>>();
        assert_eq!(4, no_connect.len());

        //test pts
        let at = no_connect.first().unwrap().at();
        assert_eq!(arr1(&[53.34, 73.66]), at);
    }
    #[test]
    fn iterate_symbol() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();
        assert_eq!(151, symbol.len());

        //(symbol (lib_id "Connector:AudioJack2_SwitchT") (at 48.26 43.18 0) (unit 1)
        //  (in_bom yes) (on_board yes) (dnp no)
        //  (uuid 00000000-0000-0000-0000-00005d64a5b4)
        assert_eq!(
            String::from("Connector:AudioJack2_SwitchT"),
            symbol.first().unwrap().lib_id()
        );
        assert_eq!(1, symbol.first().unwrap().unit());
        assert!(symbol.first().unwrap().in_bom());
        assert!(symbol.first().unwrap().on_board());
        //TODO assert!(symbol.first().unwrap().pin_names());
    }
    #[test]
    fn test_symbol_unit() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();
        assert_eq!(151, symbol.len());

        let unit = symbol.get(118).unwrap();
        assert_eq!(3, unit.unit());
        assert_eq!(String::from("Amplifier_Operational:TL072"), unit.lib_id());
    }

    #[test]
    fn iterate_library() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();
        assert_eq!(151, symbol.len());

        assert_eq!(
            String::from("Connector:AudioJack2_SwitchT"),
            symbol.first().unwrap().library().name()
        );
    }

    #[test]
    fn iterate_subsymbol() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();
        assert_eq!(151, symbol.len());
        let library = symbol.first().unwrap().library();
        let subsymbols = library.sub_symbols();
        let ss = subsymbols.first().unwrap();
        assert_eq!(0, ss.unit());
        assert_eq!(1, ss.number());
    }

    #[test]
    fn iterate_subsymbol_multi() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();

        assert_eq!(151, symbol.len());

        let symbol = symbol.get(118).unwrap();
        assert_eq!(String::from("Amplifier_Operational:TL072"), symbol.lib_id());
        let library = symbol.library();
        let subsymbols = library.sub_symbols();
        assert_eq!(3, subsymbols.len());

        assert_eq!(1, subsymbols.first().unwrap().unit());
        assert_eq!(1, subsymbols.first().unwrap().number());
        assert_eq!(2, subsymbols.get(1).unwrap().unit());
        assert_eq!(1, subsymbols.get(1).unwrap().number());
        assert_eq!(3, subsymbols.get(2).unwrap().unit());
        assert_eq!(1, subsymbols.get(2).unwrap().number());
    }

    #[test]
    fn iterate_properties() {
        let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let symbol = schema
            .into_iter()
            .filter_map(|e| match e {
                Element::Symbol(s) => Some(s),
                _ => None,
            })
            .collect::<Vec<&Symbol<'_>>>();
        assert_eq!(151, symbol.len());

        assert_eq!(
            String::from("Connector:AudioJack2_SwitchT"),
            symbol.first().unwrap().lib_id()
        );

        let properties = symbol.first().unwrap().properties();
        assert_eq!(8, properties.len());
        assert_eq!(
            String::from("Reference"),
            properties.first().unwrap().name()
        );
        assert_eq!(String::from("J2"), properties.first().unwrap().value());
        assert!(properties.first().unwrap().effects().unwrap().visible());
        assert_eq!(String::from("IN"), properties.get(1).unwrap().value());
        assert!(properties.get(1).unwrap().effects().unwrap().visible());
        assert_eq!(
            String::from("elektrophon:Jack_3.5mm_WQP-PJ398SM_Vertical"),
            properties.get(2).unwrap().value()
        );
        assert!(!properties.get(2).unwrap().effects().unwrap().visible());
        assert_eq!(String::from("~"), properties.get(3).unwrap().value());
        assert!(!properties.get(3).unwrap().effects().unwrap().visible());
        assert_eq!(
            String::from("3.5mm Eurorack Jacks"),
            properties.get(4).unwrap().value()
        );
        assert!(!properties.get(4).unwrap().effects().unwrap().visible());
        assert_eq!(String::from("SPICE"), properties.get(5).unwrap().value());
        assert!(!properties.get(5).unwrap().effects().unwrap().visible());
        assert!(!properties.get(6).unwrap().effects().unwrap().visible());
        assert_eq!(
            String::from("S=1 T=2 TN=3"),
            properties.get(7).unwrap().value()
        );
        assert!(!properties.get(7).unwrap().effects().unwrap().visible());

        //(effects (font (size 1.27 1.27)) (justify right) hide)
        let effects = properties.first().unwrap().effects();
        //assert_eq!(String::from("right"), effects.justify());
        //assert_eq!(String::from(el::HIDE), effects.show());
    }
}
