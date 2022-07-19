use ndarray::{arr1, Array1, Array2, ArrayView};
//use smallstr::SmallString;
use crate::Error;
use regex::Regex;
use lazy_static::lazy_static;

pub mod elements;
pub mod parser;

lazy_static! {
    pub static ref RE: regex::Regex = Regex::new(r"^.*_(\w*)_(\w*)$").unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineType {
    Dash,
    DashDot,
    DashDotDot,
    Dot,
    Default,
    Solid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Justify {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    Mirror,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Effects {
    pub font: String,
    pub color: Color,
    pub size: f64,
    pub thickness: f64,
    pub bold: bool,
    pub italic: bool,
    pub line_spacing: f64,
    pub justify: Justify,
    pub hide: bool,
}
impl Effects {
    pub fn new(
        font: String,
        color: Color,
        size: f64,
        thickness: f64,
        bold: bool,
        italic: bool,
        line_spacing: f64,
        justify: Justify,
        hide: bool,
    ) -> Effects {
        Effects {
            font,
            color,
            size,
            thickness,
            bold,
            italic,
            line_spacing,
            justify,
            hide,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}
impl From<&Sexp> for Color {
    fn from(node: &Sexp) -> Color {
        Color {
            r: node.get(0).unwrap(),
            g: node.get(0).unwrap(),
            b: node.get(0).unwrap(),
            a: node.get(0).unwrap(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FillType {
    None,
    Outline,
    Background,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub width: f64,
    pub line_type: LineType,
    pub color: Color,
    pub fill: FillType,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub enum Sexp {
    Node(String, Vec<Sexp>),
    Value(String),
    Text(String),
    Empty,
}

macro_rules! get {
    ($node:expr, $key:expr) => {
        $node.get($key)
    };
    ($node:expr, $key:expr, $index:expr) => {
        Get::<_, Vec<&Sexp>>::get($node, $key)
            .unwrap().get(0).unwrap()
            .get($index).unwrap()
    };
}
pub(crate) use get;

/// Access the nodes and values.
pub trait Test<T> {
    fn has(&self, index: T) -> bool;
    fn contains(&self, index: T) -> bool;
}
/// Get the value as String by index.
impl Test<&str> for Sexp {
    fn has(&self, value: &str) -> bool {
        if let Sexp::Node(_, values) = &self {
            for v in values {
                if let Sexp::Value(val) = v {
                    if *val == value {
                        return true;
                    }
                } else if let Sexp::Text(text) = v {
                    if *text == value {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn contains(&self, key: &str) -> bool {
        if let Sexp::Node(_, values) = &self {
            for v in values {
                if let Sexp::Node(name, _) = v {
                    if *name == key {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Access the nodes and values.
pub trait Get<'a, S, T> {
    fn get(&'a self, index: S) -> Result<T, Error>;
}
/// Get the value as String by index.
impl Get<'_, usize, String> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<String, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.to_string())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.to_string())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as float by index.
impl Get<'_, usize, f64> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<f64, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as usize by index.
impl Get<'_, usize, usize> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<usize, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as Array1 by index.
impl Get<'_, &str, Array1<f64>> for Sexp {
    fn get(&self, key: &str) -> Result<Array1<f64>, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        let pos: &Sexp = nodes.get(0).unwrap();
        let x: f64 = pos.get(0).unwrap();
        let y: f64 = pos.get(1).unwrap();
        Ok(arr1(&[x, y]))
    }
}
/// Get the value as Array2 by index.
impl Get<'_, &str, Array2<f64>> for Sexp {
    /// Get the value as String by index.
    fn get(&self, key: &str) -> Result<Array2<f64>, Error> {
        let mut array: Array2<f64> = Array2::zeros((0, 2));
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        let xy: Vec<&Sexp> = nodes.get(0).unwrap().get("xy").unwrap();

        for _xy in xy {
            let x: f64 = _xy.get(0).unwrap();
            let y: f64 = _xy.get(1).unwrap();
            array.push_row(ArrayView::from(&[x, y])).unwrap();
        }
        Ok(array)
    }
}
/// Get the value as String by index.
impl<'a> Get<'a, &str, Vec<&'a Sexp>> for Sexp {
    /// Get the value as String by index.
    fn get(&'a self, key: &str) -> Result<Vec<&'a Sexp>, Error> {
        if let Sexp::Node(_, values) = &self {
            Ok(values.into_iter().filter(|n| {
                if let Sexp::Node(name, _) = n {
                    name == key
                } else { false }
            }).collect())
        } else { Err(Error::ExpectValueNode) }
    }
}

/// Get the value as Effects by index.
impl<'a> Get<'a, &str, Effects> for Sexp {
    fn get(&'a self, key: &str) -> Result<Effects, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        if nodes.len() == 1 {
            let node = nodes.get(0).unwrap();
                let fonts: Vec<&Sexp> = node.get("font").unwrap();
                if fonts.len() == 1 {
                    let font = fonts.get(0).unwrap();
                    // get face 0
                    /* let face_list: Vec<&Sexp> = font.get("face").unwrap();
                    let face_item: &Sexp = face_list.get(0).unwrap();
                    let face: String = face_item.get(0).unwrap(); */

                    let face: String = if font.contains("face") {
                        get!(*font, "face", 0)
                    } else {
                        "default".to_string()
                    };
                    let size: f64 = if font.contains("size") {
                        get!(*font, "size", 0)
                    } else {
                        0.0
                    };
                    let thickness: f64 = if font.contains("thickess") {
                        get!(*font, "thickness", 0)
                    } else {
                        0.0
                    };
                    let line_spacing: f64 = if font.contains("line_spacing") {
                        get!(*font, "line_spacing", 0)
                    } else {
                        0.0
                    };
                    let justify: Justify = if font.contains("justify") {
                        get!(*font, "justify").unwrap()
                    } else {
                        Justify::Center
                    };

                    let effects = Effects::new(
                        face,
                        Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                        size,
                        thickness,
                        font.has("bold"),
                        font.has("italic"),
                        line_spacing,
                        justify,
                        font.has("hide"),
                    );
                    return Ok(effects);
                } else {
                    Err(Error::ParseError)
                }
        } else {
            Err(Error::ParseError)
        }
    }
}

/// Get the value as Stroke by index.
impl<'a> Get<'a, &str, Stroke> for Sexp {
    fn get(&'a self, key: &str) -> Result<Stroke, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        if nodes.len() == 1 {
            let stroke = nodes.get(0).unwrap();

            let width: f64 = if stroke.contains("width") {
                get!(*stroke, "width", 0)
            } else {
                0.0
            };
            let line_type: LineType = if stroke.contains("type") {
                stroke.get("type").unwrap()
            } else {
                LineType::Default
            };
            let color: Color = if stroke.contains("color") {
                let nodes: Vec<&Sexp> = stroke.get("color").unwrap();
                let color: &Sexp = nodes.get(0).unwrap();
                Color::from(color)
            } else {
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }
            };
            let fill: FillType = if self.contains("fill") {
                get!(self, "fill").unwrap()
            } else {
                FillType::None
            };

            Ok(Stroke {
                width,
                line_type,
                color,
                fill,
            })
        } else {
            Err(Error::ParseError)
        }
    }
}

impl<'a> Get<'a, &str, Justify> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<Justify, Error> {
        let mytype: String = get!(self, key, 0);
        if mytype == "right" {
            Ok(Justify::Right)
        } else if mytype == "left" {
            Ok(Justify::Left)
        } else if mytype == "top" {
            Ok(Justify::Top)
        } else if mytype == "bottom" {
            Ok(Justify::Bottom)
        } else if mytype == "mirror" {
            Ok(Justify::Mirror)
        } else {
            Err(Error::JustifyValueError)
        }
    }
}

impl<'a> Get<'a, &str, FillType> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<FillType, Error> {
        let nodes: Vec<&Sexp> = get!(self, key).unwrap();
        let myfill: &Sexp = nodes.get(0).unwrap();
        let mytype: String = get!(myfill, "type", 0);
        if mytype == "none" {
            Ok(FillType::None)
        } else if mytype == "outline" {
            Ok(FillType::Outline)
        } else if mytype == "background" {
            Ok(FillType::Background)
        } else {
            Ok(FillType::None)
        }
    }
}

impl<'a> Get<'a, &str, LineType> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<LineType, Error> {
        let mytype: String = get!(self, key, 0);
        if mytype == "dash" {
            Ok(LineType::Dash)
        } else if mytype == "dash_dot" {
            Ok(LineType::DashDot)
        } else if mytype == "dash_dot_dot" {
            Ok(LineType::DashDotDot)
        } else if mytype == "dot" {
            Ok(LineType::Dot)
        } else if mytype == "default" {
            Ok(LineType::Default)
        } else if mytype == "solid" {
            Ok(LineType::Solid)
        } else {
            Err(Error::LineTypeValueError)
        }
    }
}

pub fn get_unit(node: &Sexp) -> Result<usize, Error> {
    if let Sexp::Node(name, values) = node {
        if name != "symbol" {
            return Err(Error::ExpectSexpNode); //TODO
        }

        if node.contains("unit") {
            let unit: usize = get!(node, "unit", 0);
            return Ok(unit);
        } else {
            let name: String = get!(node, 0).unwrap();
            if let Some(line) = RE.captures_iter(&name).next() {
                return Ok(line[1].parse().unwrap());
            }
        }
    }
    Ok(1)
}

pub fn get_pin<'a>(node: &'a Sexp, index: usize) -> Result<&'a Sexp, Error> {
    let pins = get_pins(node, None)?;
    for p in pins {
        let i: usize = get!(p, "number", 0);
        if index == i {
            return Ok(p);
        }
    }
    Err(Error::PinNotFound(index))
}

/// Get all the pins of a library symbol.
pub fn get_pins<'a>(node: &'a Sexp, number: Option<usize>) -> Result<Vec<&'a Sexp>, Error> {
    let symbols: Vec<&Sexp> = node.get("symbol")?;
    let symbols: Vec<&Sexp> = symbols 
        .iter()
        .filter_map(|symbol| {
            let symbol_unit = get_unit(symbol).unwrap();
            if let Some(number) = number {
                if number == symbol_unit {
                    Option::from(*symbol)
                } else {
                    None
                }
            } else {
                Option::from(*symbol)
            }
        }).collect();

    let mut result: Vec<&Sexp> = Vec::new();
    for symbol in symbols {
        let pins: Vec<&Sexp> = symbol.get("pin")?;
        for pin in pins {
            result.push(pin);
        }
    }
    Ok(result)
}

pub fn get_property(node: &Sexp, key: &str) -> Result<String, Error> {
    let props: Vec<&Sexp> = node.get("property")?;
    let result: Vec<String> = props
        .iter()
        .filter_map(|node| {
            if let Sexp::Node(name, _) = node {
                if name == "property" {
                    let k: String = get!(node, 0).unwrap();
                    if k == key {
                        let res: String = get!(node, 1).unwrap();
                        Option::from(res)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if result.is_empty() {
        Err(Error::PropertyNotFound(key.to_string()))
    } else if result.len() == 1 {
        Ok(result[0].clone())
    } else {
        Err(Error::MoreThenOnPropertyFound(key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
