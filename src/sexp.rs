use crate::Error;
use regex::Regex;
use lazy_static::lazy_static;

pub mod elements;
pub mod get;
pub mod parser;
pub mod test;
pub mod iterator;

use crate::sexp::get::{get, Get};
use crate::sexp::test::Test;

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
    pub justify: Vec<Justify>,
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
        justify: Vec<Justify>,
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


pub fn get_unit(node: &Sexp) -> Result<usize, Error> {
    if let Sexp::Node(name, _) = node {
        if name != "symbol" {
            return Err(Error::ExpectSexpNode); //TODO
        }
        if node.contains("unit") {
            let unit: usize = get!(node, "unit", 0);
            return Ok(unit);
        } else {
            if let Sexp::Node(_, values) = node {
                if let Some(value) = values.get(0) {
                    if let Sexp::Text(value) = value {
                        if let Some(line) = RE.captures_iter(&value).next() {
                            return Ok(line[1].parse().unwrap());
                        }
                    } else if let Sexp::Value(value) = value {
                        if let Some(line) = RE.captures_iter(&value).next() {
                            return Ok(line[1].parse().unwrap());
                        }
                    }
                }
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
//    use super::*;
}
