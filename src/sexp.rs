use ::std::fmt::{Display, Formatter};
use lazy_static::lazy_static;
use ndarray::{arr2, Array2};
use regex::Regex;
use std::collections::HashMap;

pub mod elements;
pub mod get;
pub mod set;
pub mod del;
pub mod parser;
pub mod transform;

use crate::sexp::get::SexpGet;
use crate::sexp::get::get;

lazy_static! {
    pub static ref RE: regex::Regex = Regex::new(r"^.*_(\w*)_(\w*)$").unwrap();
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
        (String::from("xy"), arr2(&[[0., 0.], [0., 0.]]))
    ]);
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
impl From<&SexpNode> for Color {
    fn from(node: &SexpNode) -> Color {
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

#[derive(Debug, Clone)]
pub enum SexpType {
    ChildSexpNode(SexpNode),
    ChildSexpValue(SexpValue),
    ChildSexpText(SexpText),
}
#[derive(Debug, Clone)]
/// A SexpNode item has a name and can contain SexpValues or SexpNodes.
pub struct SexpNode {
    /// the node name
    pub name: String,
    /// the node values.
    pub values: Vec<SexpType>,
}
impl SexpNode {
    pub fn new(name: String) -> SexpNode {
        SexpNode {
            name,
            values: Vec::new(),
        }
    }
    pub fn from(name: String, values: Vec<SexpType>) -> Self {
        SexpNode { name, values }
    }

    /// Test if Node has node by key.
    /// ARGS:
    ///    name: &str the key to search for.
    pub fn contains(&self, name: &str) -> bool {
        self.values.iter().any(|x| match x {
            SexpType::ChildSexpNode(node) => {
                &node.name
            }
            _ => { "" }
        } == name)
    }

    /// Test if Node has a value.
    /// ARGS:
    ///    name: &str the value to search for.
    pub fn has(&self, name: &str) -> bool {
        self.values.iter().any(|x| match x {
            SexpType::ChildSexpValue(value) => {
                &value.value
            }
            _ => { "" }
        } == name)
    }

    pub fn unit(&self) -> Result<usize, Error> {
        if self.name != "symbol" {
            return Err(Error::ExpectSexpNode); //TODO
        }

        if self.contains("unit") {
            let unit: usize = get!(self, "unit", 0);
            return Ok(unit);
        } else {
            let name: String = get!(self, 0);
            if let Some(line) = RE.captures_iter(&name).next() {
                return Ok(line[1].parse().unwrap());
            }
        }
        Ok(1)
    }

    pub fn pin(&self, index: usize) -> Option<&SexpNode> {
        let pins = self.pins(None);
        for pin in pins {
            let i: usize = get!(pin, "number", 0);
            if index == i {
                return Option::from(pin);
            }
        }
        None
    }

    /// Get all the pins of a library symbol.
    pub fn pins(&self, unit: Option<usize>) -> Vec<&SexpNode> {
        let symbols: Vec<&SexpNode> = self
            .values
            .iter()
            .filter_map(|node_type| {
                if let SexpType::ChildSexpNode(node_type) = node_type {
                    Option::from(node_type)
                } else {
                    None
                }
            })
            .filter_map(|symbol| {
                if symbol.name == "symbol" {
                    if let Some(unit) = unit {
                        let symbol_unit = symbol.unit().unwrap();
                        if unit == symbol_unit {
                            Option::from(symbol)
                        } else {
                            None
                        }
                    } else {
                        Option::from(symbol)
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut pins: Vec<&SexpNode> = Vec::new();
        for symbol in &symbols {
            for pin in &symbol.values {
                if let SexpType::ChildSexpNode(p) = pin {
                    if p.name == "pin" {
                        pins.push(p);
                    }
                }
            }
        }
        pins
    }

    pub fn property(&self, key: &str) -> Option<String> {
        let result: Vec<String> = self
            .values
            .iter()
            .filter_map(|node_type| {
                if let SexpType::ChildSexpNode(node_type) = node_type {
                    Option::from(node_type)
                } else {
                    None
                }
            })
            .filter_map(|node| {
                if node.name == "property" {
                    let k: String = get!(node, 0);
                    if k == key {
                        let res: String = get!(node, 1);
                        Option::from(res)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if result.is_empty() {
            None
        } else if result.len() == 1 {
            Option::from(result[0].clone())
        } else {
            panic!("more then one property found for {}", key); //TODO Error instead of panic
        }
    }

    pub fn nodes(&self, key: &str) -> Result<Vec<&SexpNode>, Error> {
        let result: Vec<&SexpNode> = self
            .values
            .iter()
            .filter_map(|node_type| {
                if let SexpType::ChildSexpNode(node_type) = node_type {
                    Option::from(node_type)
                } else {
                    None
                }
            })
            .filter_map(|node| {
                if node.name == key {
                    Option::from(node)
                } else {
                    None
                }
            })
            .collect();

        if result.is_empty() {
            Err(Error::SymbolNotFound(key.to_string()))
        } else {
            Ok(result)
        }
    }
    pub fn nodes_mut(&mut self, key: &str) -> Result<Vec<&mut SexpNode>, Error> {
        let result: Vec<&mut SexpNode> = self
            .values
            .iter_mut()
            .filter_map(|node_type| {
                if let SexpType::ChildSexpNode(node_type) = node_type {
                    Option::from(node_type)
                } else {
                    None
                }
            })
            .filter_map(|node| {
                if node.name == key {
                    Option::from(node)
                } else {
                    None
                }
            })
            .collect();

        if result.is_empty() {
            Err(Error::SymbolNotFound(key.to_string()))
        } else {
            Ok(result)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SexpValue {
    pub value: String,
}
impl SexpValue {
    pub fn new(value: String) -> Self {
        SexpValue { value }
    }
}
#[derive(Debug, Clone)]
pub struct SexpText {
    pub value: String,
}
impl SexpText {
    pub fn new(value: String) -> Self {
        SexpText { value }
    }
}

impl IntoIterator for SexpNode {
    type Item = SexpType;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

pub trait SexpConsumer {
    fn visit(&mut self, node: &SexpNode) -> Result<(), Error>;
    fn start(&mut self, version: &String, name: &String) -> Result<(), Error>;
    fn start_library_symbols(&mut self) -> Result<(), Error>;
    fn end_library_symbols(&mut self) -> Result<(), Error>;
    fn start_sheet_instances(&mut self) -> Result<(), Error>;
    fn end_sheet_instances(&mut self) -> Result<(), Error>;
    fn start_symbol_instances(&mut self) -> Result<(), Error>;
    fn end_symbol_instances(&mut self) -> Result<(), Error>;
    fn end(&mut self) -> Result<(), Error>;
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Can not parse file.")]
    ParseError,
    #[error("can not find symbol {0}.")]
    SymbolNotFound(String),
    #[error("can not find symbol.")]
    ExpectSexpNode,
    #[error("element is not a value node.")]
    ExpectValueNode,
    #[error("Justify value error.")]
    JustifyValueError,
    #[error("LineType value error")]
    LineTypeValueError,
    #[error("File manipulatuion error {0}.")]
    IoError(String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

/* #[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    value: String,
}
impl ParseError {
    pub fn new(msg: &str) -> ParseError {
        ParseError {
            value: msg.to_string(),
        }
    }
} */
/* impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("missing closing quote")
    }
}
impl std::error::Error for ParseError {} */

impl Display for SexpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SexpType::ChildSexpNode(node) => write!(f, "{}, {:?}", node.name, node.values),
            SexpType::ChildSexpValue(value) => write!(f, "{}", value.value),
            SexpType::ChildSexpText(value) => write!(f, "{}", value.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::SexpParser;
    use ndarray::{arr1, arr2, Array1};
    use std::fs::File;
    use std::io::Read;
    use crate::sexp::transform::Bounds;

    struct DebugConsumer {
        version: String,
        generator: String,
        end_called: usize,
    }
    impl SexpConsumer for DebugConsumer {
        fn visit(&mut self, _: &SexpNode) -> Result<(), Error> {
            Ok(())
        }
        fn start(&mut self, version: &String, generator: &String) -> Result<(), Error> {
            self.version = version.to_string();
            self.generator = generator.to_string();
            Ok(())
        }
        fn start_library_symbols(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn end_library_symbols(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn start_sheet_instances(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn end_sheet_instances(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn start_symbol_instances(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn end_symbol_instances(&mut self) -> Result<(), Error> {
            Ok(())
        }
        fn end(&mut self) -> Result<(), Error> {
            self.end_called += 1;
            Ok(())
        }
    }
    impl DebugConsumer {
        fn new() -> DebugConsumer {
            DebugConsumer {
                version: String::new(),
                generator: String::new(),
                end_called: 0,
            }
        }
    }

    #[test]
    fn get_node_name() {
        let content = "(SEXP (NAME VALUE))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();

        let result: Vec<&SexpNode> = sexp
            .values
            .iter()
            .filter_map(|node_type| {
                if let SexpType::ChildSexpNode(node_type) = node_type {
                    Option::from(node_type)
                } else {
                    None
                }
            })
            .filter_map(|node| {
                if node.name == "NAME" {
                    Option::from(node)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(&result[0].name, &String::from("NAME"));
    }
    #[test]
    fn get_node_value() {
        let content = "(SEXP (NAME VALUE))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let name: String = get!(&sexp, "NAME", 0);
        assert_eq!(String::from("VALUE"), name);
    }
    #[test]
    fn get_node_quoted() {
        let content = "(SEXP (NAME \"My VALUE\"))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let name: String = get!(&sexp, "NAME", 0);
        assert_eq!(String::from("My VALUE"), name);
    }
    #[test]
    fn get_node_quoted_backslash() {
        let content = "(SEXP (NAME \"My \\\"upsi\\\" VALUE\"))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let name: String = get!(&sexp, "NAME", 0);
        assert_eq!(String::from("My \\\"upsi\\\" VALUE"), name);
    }
    #[test]
    fn get_schema_header() {
        let content = "(kicad_schema (version 20220101) (generator elektron)
            (symbol \"Device:R\")
        )";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        assert_eq!(sexp.name, String::from("kicad_schema"));
        assert_eq!(consumer.generator, String::from("elektron"));
        assert_eq!(consumer.version, String::from("20220101"));
        //test if end is called
        assert_eq!(consumer.end_called, 1);
    }
    #[test]
    fn get_pos() {
        let content = "(symbol (lib_id \"power:+15V\") (at 127 34.29 0) (unit 1))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let node: Array1<f64> = sexp.get("at").unwrap();
        //assert_eq!(Pos::new(127.0, 34.29), node);
        assert_eq!(arr1(&[127.0, 34.29]), node);
    }

    #[test]
    fn get_nd_pts() {
        let content = "(polyline
          (pts
            (xy -5.08 5.08)
            (xy 5.08 0)
            (xy -5.08 -5.08)
            (xy -5.08 5.08)
          )
          (stroke (width 0.254) (type default) (color 0 0 0 0))
          (fill (type background))
        )";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let node: Array2<f64> = sexp.get("pts").unwrap();
        assert_eq!(
            arr2(&[[-5.08, 5.08], [5.08, 0.0], [-5.08, -5.08], [-5.08, 5.08]]),
            node
        );
    }
    #[test]
    fn get_macro() {
        let content = "(polyline
          (pts
            (xy -5.08 5.08)
            (xy 5.08 0)
            (xy -5.08 -5.08)
            (xy -5.08 5.08)
          )
          (stroke (width 0.254) (type default) (color 0 0 0 0))
          (fill (type background))
        )";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let node: Array2<f64> = get!(sexp, "pts");
        assert_eq!(
            arr2(&[[-5.08, 5.08], [5.08, 0.0], [-5.08, -5.08], [-5.08, 5.08]]),
            node
        );
    }
    #[test]
    fn parse_unit_symbol() {
        let content = "(symbol (lib_id \"power:-15V\") (at 91.44 189.23 180) (unit 4)";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        assert_eq!(4, sexp.unit().unwrap());
    }
    #[test]
    fn parse_unit_subsymbol() {
        let content = "(symbol \"TL072_2_1\")";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        assert_eq!(2, sexp.unit().unwrap());
    }
    #[test]
    fn parse_stroke_type() {
        let content = "(stroke (width 0.254) (type default) (color 0 0 0 0))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let stroke: LineType = get!(sexp, "type");
        assert_eq!(LineType::Default, stroke);
    }
    #[test]
    fn parse_stroke() {
        let content = "(some (stroke (width 0.254) (type default) (color 0 0 0 0)))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let stroke: Stroke = get!(sexp, "stroke");
        assert_eq!(0.254, stroke.width);
        assert_eq!(LineType::Default, stroke.line_type);
        assert_eq!(
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            },
            stroke.color
        );
    }
    #[test]
    fn parse_stroke_fill() {
        let content = "(polyline
          (pts
            (xy -5.08 5.08)
            (xy 5.08 0)
            (xy -5.08 -5.08)
            (xy -5.08 5.08)
          )
          (stroke (width 0.254) (type default) (color 0 0 0 0))
          (fill (type background))
        )";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let stroke: Stroke = get!(sexp, "stroke");
        assert_eq!(0.254, stroke.width);
        assert_eq!(LineType::Default, stroke.line_type);
        assert_eq!(
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            },
            stroke.color
        );
        assert_eq!(FillType::Background, stroke.fill);
    }
    #[test]
    fn parse_effects() {
        let content = "(some (effects (font (size 1.27 1.27)) hide))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let effects: Effects = get!(sexp, "effects");
        assert!(effects.hide);
        assert_eq!(1.27, effects.size);
    }
    #[test]
    fn parse_effects_justify() {
        let content = "(some (effects (font (size 1.27 1.27)) (justify left)))";
        let mut parser = SexpParser::new(content);
        let mut consumer = DebugConsumer::new();
        let sexp = parser.parse(&mut consumer).unwrap();
        let effects: Effects = get!(sexp, "effects");
        assert!(!effects.hide);
        assert_eq!(Justify::Left, effects.justify);
    }
    #[test]
    fn transform_arr1() {
        //TODO
    }
    #[test]
    fn get_pins() {
        match File::open("samples/files/summe/summe.kicad_sch") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                let mut parser = SexpParser::new(&content);
                let mut consumer = DebugConsumer::new();
                let sexp = parser.parse(&mut consumer).unwrap();
                let lib_symbols = sexp.nodes("lib_symbols").unwrap();
                assert_eq!(lib_symbols.len(), 1);
                let symbol = lib_symbols.get(0);
                if let Some(libs) = symbol {
                    let lib_symbols = libs.nodes("symbol").unwrap();
                    assert_eq!(7, lib_symbols.len());
                    let symbol = lib_symbols.get(0);
                    if let Some(s) = symbol {
                        assert_eq!(8, s.pins(None).len());
                    } else {
                        panic!("can not get pins.");
                    }
                } else {
                    panic!("can not get lib symbol.");
                }
            }
            Err(error) => {
                panic!(
                    "Error opening file \"samples/files/summe/summe.kicad_sch\", {}",
                    error
                );
            }
        }
    }
    #[test]
    fn bounds() {
        match File::open("samples/files/summe/summe.kicad_sch") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                let mut parser = SexpParser::new(&content);
                let mut consumer = DebugConsumer::new();
                let sexp = parser.parse(&mut consumer).unwrap();
                let lib_symbols = sexp.nodes("lib_symbols").unwrap();
                assert_eq!(lib_symbols.len(), 1);
                let symbol = lib_symbols.get(0);
                let lib_symbol = if let Some(libs) = symbol {
                    let ls = libs.nodes("symbol").unwrap();
                    //assert_eq!(7, ls.len());
                    let s = ls.get(0).unwrap();
                    let name: String = get!(s, 0);
                    assert_eq!("Amplifier_Operational:TL072", name);
                    symbol
                } else {
                    panic!("no lib symbol loaded!");
                };

                let _symbol = if let Some(libs) = symbol {
                    let lib_symbols = libs.nodes("symbol").unwrap();
                    for s in lib_symbols {
                        let unit: usize = s.unit().unwrap();
                        let reference: String = s.property("Reference").unwrap();
                            println!("Bound: unit:{:?} reference:{:?}", unit, reference);
                        if unit == 1 && reference == "U" {
                            let b = symbol.unwrap().bounds(&lib_symbol.unwrap());
                            println!("{:?}", b);
                        }
                    }
                } else {
                    panic!("symbol not found");
                };
            }
            Err(error) => {
                panic!(
                    "Error opening file \"samples/files/summe/summe.kicad_sch\", {}",
                    error
                );
            }
        }
    }
    /* #[test]
    fn get_pins() {
        match File::open("samples/files/summe/summe.kicad_sch") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                let mut parser = SexpParser::new(&content);
                let mut consumer = DebugConsumer::new();
                let sexp = parser.parse(&mut consumer);
                let symbols = sexp.nodes("symbol").unwrap();
                assert_eq!(symbols.len(), 18);
                let symbol = symbols.get(1);
                if let Some(node) = symbol {
                    if let Some(reference) = node.property("Reference") {
                        assert_eq!("R3", reference);
                        let pins = node.pins();
                        assert_eq!(2, pins.len());

                    } else {
                        panic!("can not get reference");
                    }
                } else {
                }
            },
            Err(error) => {
                panic!("Error opening file \"samples/files/summe/summe.kicad_sch\", {}", error);
            },
        }
    } */
}
