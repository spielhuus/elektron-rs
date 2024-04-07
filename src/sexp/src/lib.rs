//! Library to parse and write the Kicad sexp files.
//!
//! The library provides low level acces to the sexp nodes, no model for the kicad data is provided.
//!
//! # Examples
//!
//! ```
//! // load a Kicad schema and access the root node:
//! use sexp::{SexpParser, SexpTree};
//! let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
//! let tree = SexpTree::from(doc.iter()).unwrap();
//! let root = tree.root().unwrap();
//! assert_eq!("kicad_sch", root.name);
//!
//! // get all symbol elements from the document:
//! use sexp::{el, Sexp};
//! let symbols = root.query(el::SYMBOL).collect::<Vec<&Sexp>>();
//! assert_eq!(151, symbols.len());
//!
//! // find the symbol with the reference "R1":
//! use sexp::SexpProperty;
//! let symbol = tree
//!     .root()
//!     .unwrap()
//!     .query(el::SYMBOL)
//!     .find(|s| {
//!         let name: String = s.property(el::PROPERTY_REFERENCE).unwrap();
//!         name == "R1"
//!     })
//!     .unwrap();
//! assert_eq!(String::from("10"),
//!             <Sexp as SexpProperty<String>>::property(
//!                 symbol,
//!                 el::PROPERTY_VALUE
//!             ).unwrap());
//! ```
//! create a document:
//!
//! ```
//! use sexp::{sexp, Builder};
//! let mut tree = sexp!((kicad_sch (version {sexp::KICAD_SCHEMA_VERSION}) (generator "elektron")
//!     (uuid "e91be4a5-3c12-4daa-bee2-30f8afcd4ab8")
//!     (paper r"A4")
//!     (lib_symbols)
//! ));
//! let root = tree.root().unwrap();
//! assert_eq!("kicad_sch", root.name);
//! ```
//! In the last example a sexp model was created. The macro is straight forward.
//! sexp supports string and quoted string. To define quoted steings directly when you
//! define raw strings. When a quoted string should be created this can either be done
//! with a raw String (`r"some text"`) or when it is created from a variable with a
//! bang (`!{variable}`, `!{func(param)}`).

/// Parse and access sexp files.
use std::{fs, io::Write, str::CharIndices};

use ndarray::{arr1, Array1};

pub mod math;

///Kicad schema file version
pub const KICAD_SCHEMA_VERSION: &str = "20211123";
///Kicad schema generator name.
pub const KICAD_SCHEMA_GENERATOR: &str = "elektron";

///Constants for the element names.
pub mod el {
    pub const AT: &str = "at";
    pub const EFFECTS: &str = "effects";
    pub const EFFECTS_JUSTIFY: &str = "justify";
    pub const GLOBAL_LABEL: &str = "global_label";
    pub const GRAPH_ARC: &str = "arc";
    pub const GRAPH_CIRCLE: &str = "circle";
    pub const GRAPH_POLYLINE: &str = "polyline";
    pub const GRAPH_RECTANGLE: &str = "rectangle";
    pub const GRAPH_START: &str = "start";
    pub const GRAPH_END: &str = "end";
    pub const JUNCTION: &str = "junction";
    pub const JUSTIFY: &str = "justify";
    pub const JUSTIFY_LEFT: &str = "left";
    pub const JUSTIFY_RIGHT: &str = "right";
    pub const LABEL: &str = "label";
    pub const LIB_ID: &str = "lib_id";
    pub const LIB_SYMBOLS: &str = "lib_symbols";
    pub const MIRROR: &str = "mirror";
    pub const NO_CONNECT: &str = "no_connect";
    pub const PROPERTY: &str = "property";
    pub const PROPERTY_REFERENCE: &str = "Reference";
    pub const PROPERTY_VALUE: &str = "Value";
    pub const PIN: &str = "pin";
    pub const PIN_NUMBER: &str = "number";
    pub const PIN_NAMES: &str = "pin_names";
    pub const PIN_NAME: &str = "name";
    pub const PTS: &str = "pts";
    pub const SHEET_INSTANCES: &str = "sheet_instances";
    pub const SYMBOL: &str = "symbol";
    pub const SYMBOL_UNIT: &str = "unit";
    pub const STROKE: &str = "stroke";
    pub const TEXT: &str = "text";
    pub const TITLE_BLOCK: &str = "title_block";
    pub const TITLE_BLOCK_COMMENT: &str = "comment";
    pub const TITLE_BLOCK_COMPANY: &str = "company";
    pub const TITLE_BLOCK_DATE: &str = "date";
    pub const TITLE_BLOCK_REV: &str = "rev";
    pub const TITLE_BLOCK_TITLE: &str = "title";
    pub const WIRE: &str = "wire";
    pub const XY: &str = "xy";
}

///create an UUID.
#[macro_export]
macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string().as_str()
    };
}

///Enum of sexp error results.
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    ///Can not manipulate file.
    #[error("{0}:{1}")]
    SexpError(String, String),
    #[error("Can not laod content: {0} ({1})")]
    IoError(String, String),
    #[error("Library not found {0}.")]
    LibraryNotFound(String),
}
impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(String::from("io::Error"), err.to_string())
    }
}

/// The paper siues. DIN pagper sizes are used.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaperSize {
    A5,
    A4,
    A3,
    A2,
    A1,
    A0,
}

///Display the paper size.
impl std::fmt::Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

///Parse the paper size from String.
impl std::convert::From<&String> for PaperSize {
    fn from(size: &String) -> Self {
        if size == "A5" {
            Self::A5
        } else if size == "A4" {
            Self::A4
        } else if size == "A3" {
            Self::A3
        } else if size == "A2" {
            Self::A2
        } else if size == "A1" {
            Self::A1
        } else {
            Self::A0
        }
    }
}

///Get the real paper size im mm.
impl std::convert::From<PaperSize> for (f64, f64) {
    fn from(size: PaperSize) -> Self {
        if size == PaperSize::A5 {
            (148.0, 210.0)
        } else if size == PaperSize::A4 {
            (297.0, 210.0)
        } else if size == PaperSize::A3 {
            (420.0, 297.0)
        } else if size == PaperSize::A2 {
            (420.0, 594.0)
        } else if size == PaperSize::A1 {
            (594.0, 841.0)
        } else {
            (841.0, 1189.0)
        }
    }
}

///Graphical styles for a symbol pin.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PinGraphicalStyle {
    Line,
    Inverted,
    Clock,
    InvertedClock,
    InputLow,
    ClockLow,
    OutputLow,
    EdgeClockHigh,
    NonLogic,
}

///Get the pin graphical style from String.
impl std::convert::From<String> for PinGraphicalStyle {
    fn from(pin_type: String) -> Self {
        if pin_type == "line" {
            Self::Line
        } else if pin_type == "inverted" {
            Self::Inverted
        } else if pin_type == "clock" {
            Self::Clock
        } else if pin_type == "inverted_clock" {
            Self::InvertedClock
        } else if pin_type == "input_low" {
            Self::InputLow
        } else if pin_type == "clock_low" {
            Self::ClockLow
        } else if pin_type == "output_low" {
            Self::OutputLow
        } else if pin_type == "edge_clock_high" {
            Self::EdgeClockHigh
        } else if pin_type == "non_logic" {
            Self::NonLogic
        } else {
            println!("unknown pin graphical style {}", pin_type);
            Self::Line
        }
    }
}

///Display pin graphical style.
impl std::fmt::Display for PinGraphicalStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PinGraphicalStyle::Line => write!(f, "line")?,
            PinGraphicalStyle::Inverted => write!(f, "inverted")?,
            PinGraphicalStyle::Clock => write!(f, "clock")?,
            PinGraphicalStyle::InvertedClock => write!(f, "inverted_clock")?,
            PinGraphicalStyle::InputLow => write!(f, "input_low")?,
            PinGraphicalStyle::ClockLow => write!(f, "clock_low")?,
            PinGraphicalStyle::OutputLow => write!(f, "output_low")?,
            PinGraphicalStyle::EdgeClockHigh => write!(f, "edge_clock_high")?,
            PinGraphicalStyle::NonLogic => write!(f, "non_logic")?,
        };
        Ok(())
    }
}

///the types of a sexp atom.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SexpAtom {
    ///Child node.
    Node(Sexp),
    ///Value
    Value(String),
    ///Text surrounded with quotes.
    Text(String),
}

///Sexp Element
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sexp {
    ///name of the node
    pub name: String,
    ///Children of the node.
    nodes: Vec<SexpAtom>,
}

impl Sexp {
    ///Create a new sexp node with name.
    pub fn from(name: String) -> Self {
        Sexp {
            name,
            nodes: Vec::new(),
        }
    }
    ///Iterator for all elements.
    pub fn iter(&self) -> impl Iterator<Item = &SexpAtom> {
        self.nodes.iter()
    }
    ///get the nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &Sexp> {
        self.nodes.iter().filter_map(|n| {
            if let SexpAtom::Node(node) = n {
                Some(node)
            } else {
                None
            }
        })
    }
    ///Iterator with child nodes.
    pub fn children(&self) -> impl Iterator<Item = &Sexp> {
        self.nodes.iter().filter_map(|n| {
            if let SexpAtom::Node(node) = n {
                Some(node)
            } else {
                None
            }
        })
    }
    ///query child nodes for elements by name.
    pub fn query<'a>(&'a self, q: &'a str) -> impl Iterator<Item = &Sexp> + 'a {
        self.nodes.iter().filter_map(move |n| {
            if let SexpAtom::Node(node) = n {
                if node.name == q {
                    Some(node)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
    ///query child for elements by name and return mutable iterator.
    pub fn query_mut<'a>(&'a mut self, q: &'a str) -> impl Iterator<Item = &mut Sexp> + 'a {
        self.nodes.iter_mut().filter_map(move |n| {
            if let SexpAtom::Node(node) = n {
                if node.name == q {
                    Some(node)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
    pub fn has(&self, q: &str) -> bool {
        self.nodes
            .iter()
            .filter_map(|n| {
                if let SexpAtom::Node(node) = n {
                    if node.name == q {
                        Some(node)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .count()
            > 0
    }

    ///set the child at index, will replace the existing element.
    pub fn set(&mut self, index: usize, element: SexpAtom) -> Result<(), Error> {
        let _res = std::mem::replace(&mut self.nodes[index], element);
        Ok(())
    }
    ///push a node to the end of the child list.
    pub fn push(&mut self, element: SexpAtom) -> Result<(), Error> {
        self.nodes.push(element);
        Ok(())
    }
    ///insert a new node at index, shifts elements right.
    pub fn insert(&mut self, index: usize, element: SexpAtom) -> Result<(), Error> {
        self.nodes.insert(index, element);
        Ok(())
    }
    ///Removes the element by name within the vector, shifting all elements after it to the left.
    pub fn remove(&mut self, name: &str) -> Result<(), Error> {
        for (i, e) in self.nodes.iter().enumerate() {
            if let SexpAtom::Node(node) = e {
                if node.name == name {
                    self.nodes.remove(i);
                    return Ok(());
                }
            }
        }
        Err(Error::SexpError(
            String::from("element not found"),
            name.to_string(),
        ))
    }

    ///Test if node has a property by key.
    pub fn has_property(&self, q: &str) -> bool {
        for p in self.query(el::PROPERTY) {
            let name: String = p.get(0).unwrap();
            if name == q {
                return true;
            }
        }
        false
    }
}

///Sexp document.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SexpTree {
    tree: Sexp,
}

impl<'a> SexpTree {
    ///parse a sexp document for SexpParser Iterator.
    pub fn from<I>(mut iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = State<'a>>,
    {
        let mut stack: Vec<(String, Sexp)> = Vec::new();
        if let Some(State::StartSymbol(name)) = iter.next() {
            stack.push((name.to_string(), Sexp::from(name.to_string())));
        } else {
            return Err(Error::SexpError(
                String::from("Document does not start with a start symbol."),
                String::from("from item"),
            ));
        };
        loop {
            match iter.next() {
                Some(State::Values(value)) => {
                    let len = stack.len();
                    if let Some((_, parent)) = stack.get_mut(len - 1) {
                        parent.nodes.push(SexpAtom::Value(value.to_string()));
                    }
                }
                Some(State::Text(value)) => {
                    let len = stack.len();
                    if let Some((_, parent)) = stack.get_mut(len - 1) {
                        parent.nodes.push(SexpAtom::Text(value.to_string()));
                    }
                }
                Some(State::EndSymbol) => {
                    let len = stack.len();
                    if len > 1 {
                        let (_n, i) = stack.pop().unwrap();
                        if let Some((_, parent)) = stack.get_mut(len - 2) {
                            parent.nodes.push(SexpAtom::Node(i));
                        }
                    }
                }
                Some(State::StartSymbol(name)) => {
                    stack.push((name.to_string(), Sexp::from(name.to_string())));
                }
                None => break,
            }
        }
        let (_n, i) = stack.pop().unwrap();
        Ok(SexpTree { tree: i })
    }
    ///Get the root element.
    pub fn root(&self) -> Result<&Sexp, Error> {
        Ok(&self.tree)
    }
    ///Get mutable root element.
    pub fn root_mut(&mut self) -> Result<&mut Sexp, Error> {
        Ok(&mut self.tree)
    }
}

///Get a sexp property node or value.
///
pub trait SexpProperty<E> {
    fn property(&self, q: &str) -> Option<E>;
}

///Get a sexp property node, return the property value.
impl SexpProperty<String> for Sexp {
    fn property(&self, q: &str) -> Option<String> {
        for p in self.query(el::PROPERTY) {
            let name: String = p.get(0).unwrap();
            if name == q {
                let value: String = p.get(1).unwrap();
                return Some(value);
            }
        }
        None
    }
}

///Get a sexp property node, return the element.
impl SexpProperty<Sexp> for Sexp {
    fn property(&self, q: &str) -> Option<Sexp> {
        for p in self.query(el::PROPERTY) {
            let name: String = p.get(0).unwrap();
            if name == q {
                return Some(p.clone());
            }
        }
        None
    }
}

///get sexp values, will be castet to E.
pub trait SexpValuesQuery<E> {
    ///Return values from a node.
    fn values(&self) -> E;
}

///get sexp values as Strings.
impl SexpValuesQuery<Vec<String>> for Sexp {
    ///Return values from a node.
    fn values(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter_map(|n| {
                if let SexpAtom::Value(value) = n {
                    Some(value.clone())
                } else if let SexpAtom::Text(value) = n {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

///get sexp values as u16.
impl SexpValuesQuery<Vec<u16>> for Sexp {
    ///Return a single value from a node.
    fn values(&self) -> Vec<u16> {
        let vals: Vec<String> = self
            .nodes
            .iter()
            .filter_map(|n| {
                if let SexpAtom::Value(value) = n {
                    Some(value.clone())
                } else if let SexpAtom::Text(value) = n {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();

        vals.iter()
            .map(|v| v.parse::<u16>().unwrap())
            .collect::<Vec<u16>>()
    }
}

///get sexp values as f64.
impl SexpValuesQuery<Vec<f64>> for Sexp {
    ///Return a single value from a node.
    fn values(&self) -> Vec<f64> {
        let vals: Vec<String> = self
            .nodes
            .iter()
            .filter_map(|n| {
                if let SexpAtom::Value(value) = n {
                    Some(value.clone())
                } else if let SexpAtom::Text(value) = n {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();

        vals.iter()
            .map(|v| v.parse::<f64>().unwrap())
            .collect::<Vec<f64>>()
    }
}

///get sexp values as ndarray f64.
impl SexpValuesQuery<Array1<f64>> for Sexp {
    ///Return a single value from a node.
    fn values(&self) -> Array1<f64> {
        let vals: Vec<String> = self
            .nodes
            .iter()
            .filter_map(|n| {
                if let SexpAtom::Value(value) = n {
                    Some(value.clone())
                } else if let SexpAtom::Text(value) = n {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();

        Array1::from(
            vals.iter()
                .map(|v| v.parse::<f64>().unwrap())
                .collect::<Vec<f64>>(),
        )
    }
}

///Get a single sexp value.
///
///Get a sexp value by name or index.
///There could be multiple values, the
///first is returned.
pub trait SexpValueQuery<E> {
    ///Return the first value from a node by name.
    fn value(&self, q: &str) -> Option<E>;
    ///get value at index.
    fn get(&self, index: usize) -> Option<E>;
}

impl SexpValueQuery<String> for Sexp {
    ///Return a single value from a node.
    fn value(&self, q: &str) -> Option<String> {
        if let Some(node) = self.query(q).next() {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(value.to_string());
            }
        }
        None
    }
    ///Return a positional value from the node.
    fn get(&self, index: usize) -> Option<String> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(value.to_string());
        }
        None
    }
}

impl SexpValueQuery<u32> for Sexp {
    fn value(&self, q: &str) -> Option<u32> {
        if let Some(node) = self.query(q).next() {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(value.parse::<u32>().unwrap());
            }
        }
        None
    }
    fn get(&self, index: usize) -> Option<u32> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(value.parse::<u32>().unwrap());
        }
        None
    }
}

impl SexpValueQuery<usize> for Sexp {
    fn value(&self, q: &str) -> Option<usize> {
        if let Some(node) = self.query(q).next() {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(value.parse::<usize>().unwrap());
            }
        }
        None
    }
    fn get(&self, index: usize) -> Option<usize> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(value.parse::<usize>().unwrap());
        }
        None
    }
}

impl SexpValueQuery<bool> for Sexp {
    fn value(&self, q: &str) -> Option<bool> {
        if let Some(node) = self.query(q).next() {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(value == "true" || value == "yes");
            }
        }
        Some(false)
    }
    fn get(&self, index: usize) -> Option<bool> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(value == "true" || value == "yes");
        }
        Some(false)
    }
}

impl SexpValueQuery<f64> for Sexp {
    fn value(&self, q: &str) -> Option<f64> {
        let node = self.query(q).next();
        if let Some(node) = node {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(value.parse::<f64>().unwrap());
            }
        }
        None
    }
    fn get(&self, index: usize) -> Option<f64> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(value.parse::<f64>().unwrap());
        }
        None
    }
}

impl SexpValueQuery<Array1<f64>> for Sexp {
    fn value(&self, q: &str) -> Option<Array1<f64>> {
        if let Some(node) = self.query(q).next() {
            let arr: Array1<f64> = Array1::from(
                <Sexp as SexpValuesQuery<Vec<String>>>::values(node)
                    .iter()
                    .map(|v| v.parse::<f64>().unwrap())
                    .collect::<Vec<f64>>(),
            );
            return Some(arr);
        }
        None
    }
    fn get(&self, index: usize) -> Option<Array1<f64>> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(arr1(&[value.parse::<f64>().unwrap()]));
        }
        None
    }
}

impl SexpValueQuery<PaperSize> for Sexp {
    fn value(&self, q: &str) -> Option<PaperSize> {
        if let Some(node) = self.query(q).next() {
            if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(node).first() {
                return Some(PaperSize::from(value));
            }
        }
        None
    }
    fn get(&self, index: usize) -> Option<PaperSize> {
        if let Some(value) = <Sexp as SexpValuesQuery<Vec<String>>>::values(self).get(index) {
            return Some(PaperSize::from(value));
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum State<'a> {
    StartSymbol(&'a str),
    EndSymbol,
    Values(&'a str),
    Text(&'a str),
}

impl std::convert::From<State<'_>> for u32 {
    fn from(state: State<'_>) -> Self {
        if let State::Values(value) = state {
            return value.parse::<u32>().unwrap();
        }
        if let State::Text(value) = state {
            return value.parse::<u32>().unwrap();
        }
        panic!();
    }
}

impl std::convert::From<State<'_>> for i32 {
    fn from(state: State<'_>) -> Self {
        if let State::Values(value) = state {
            return value.parse::<i32>().unwrap();
        }
        if let State::Text(value) = state {
            return value.parse::<i32>().unwrap();
        }
        panic!();
    }
}
impl std::convert::From<State<'_>> for f64 {
    fn from(state: State<'_>) -> Self {
        if let State::Values(value) = state {
            return value.parse::<f64>().unwrap();
        }
        panic!("Error Parsing to f64: {:?}", state);
    }
}
impl std::convert::From<State<'_>> for String {
    fn from(state: State<'_>) -> Self {
        if let State::Values(value) = state {
            return value.to_string();
        }
        if let State::Text(value) = state {
            return value.to_string();
        }
        panic!("Error Parsing to String: {:?}", state);
    }
}

#[derive(Debug, PartialEq, Clone)]
enum IntState {
    NotStarted,
    Symbol,
    Values,
    BeforeEndSymbol,
}

///Parse sexo document.
pub struct SexpParser {
    content: String,
}

impl SexpParser {
    pub fn from(content: String) -> Self {
        Self { content }
    }
    ///Load the SEXP tree into memory.
    pub fn load(filename: &str) -> Result<Self, Error> {
        match fs::read_to_string(filename) {
            Ok(content) => Ok(Self::from(content)),
            Err(err) => Err(Error::IoError(filename.to_string(), err.to_string())),
        }
    }
    pub fn iter(&self) -> SexpIter<'_> {
        SexpIter::new(&self.content)
    }
}

///Sexp Iterator,
pub struct SexpIter<'a> {
    content: &'a String,
    chars: CharIndices<'a>,
    start_index: usize,
    int_state: IntState,
}

impl<'a> SexpIter<'a> {
    fn new(content: &'a String) -> Self {
        Self {
            content,
            chars: content.char_indices(),
            start_index: 0,
            int_state: IntState::NotStarted,
        }
    }
    ///Seek to the next siebling of the current node.
    pub fn next_siebling(&mut self) -> Option<State<'a>> {
        let mut count: usize = 1;
        loop {
            if let Some(indice) = self.chars.next() {
                match indice.1 {
                    '(' => {
                        count += 1;
                    }
                    ')' => {
                        count -= 1;
                        if count == 0 {
                            self.int_state = IntState::NotStarted;
                            return self.next();
                        }
                    }
                    '\"' => {
                        let mut last_char = '\0';
                        loop {
                            // collect the characters to the next quote
                            if let Some(ch) = self.chars.next() {
                                if ch.1 == '"' && last_char != '\\' {
                                    break;
                                }
                                last_char = ch.1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl<'a> Iterator for SexpIter<'a> {
    type Item = State<'a>;
    ///Get the next node.
    fn next(&mut self) -> Option<Self::Item> {
        if self.int_state == IntState::BeforeEndSymbol {
            self.int_state = IntState::Values;
            return Some(State::EndSymbol);
        }
        while let Some(indice) = self.chars.next() {
            match self.int_state {
                IntState::NotStarted => {
                    if indice.1 == '(' {
                        self.start_index = indice.0 + 1;
                        self.int_state = IntState::Symbol;
                    }
                }
                IntState::Symbol => {
                    if indice.1 == ' ' || indice.1 == '\n' || indice.1 == ')' {
                        let name = &self.content[self.start_index..indice.0];
                        self.start_index = indice.0 + 1;
                        self.int_state = if indice.1 == ')' {
                            IntState::BeforeEndSymbol
                        } else {
                            IntState::Values
                        };
                        return Some(State::StartSymbol(name));
                    }
                }
                IntState::Values => {
                    if indice.1 == ' ' || indice.1 == '\t' || indice.1 == '\n' || indice.1 == ')' {
                        if indice.0 - self.start_index > 0 {
                            let value = &self.content[self.start_index..indice.0];
                            self.start_index = indice.0 + 1;
                            self.int_state = if indice.1 == ')' {
                                IntState::BeforeEndSymbol
                            } else {
                                IntState::Values
                            };
                            return Some(State::Values(value));
                        }
                        self.start_index = indice.0 + 1;
                        if indice.1 == ')' {
                            return Some(State::EndSymbol);
                        }
                    } else if indice.1 == '(' {
                        self.start_index = indice.0 + 1;
                        self.int_state = IntState::Symbol;
                    } else if indice.1 == '"' {
                        let mut last_char = '\0';
                        self.start_index = indice.0 + 1;
                        loop {
                            // collect the characters to the next quote
                            if let Some(ch) = self.chars.next() {
                                if ch.1 == '"' && last_char != '\\' {
                                    let value = &self.content[self.start_index..ch.0];
                                    self.start_index = ch.0 + 1;
                                    self.int_state = if indice.1 == ')' {
                                        IntState::BeforeEndSymbol
                                    } else {
                                        IntState::Values
                                    };
                                    return Some(State::Text(value));
                                }
                                last_char = ch.1;
                            }
                        }
                    }
                }
                IntState::BeforeEndSymbol => {}
            }
        }
        None
    }
}

///Utility methods to access some common nodes.
pub mod utils {
    use super::{el, Sexp, SexpAtom, SexpParser, SexpTree, SexpValueQuery};
    use crate::Error;
    use lazy_static::lazy_static;
    use ndarray::{s, Array1};
    use regex::Regex;

    ///get the position from the at node.
    pub fn at(element: &Sexp) -> Option<Array1<f64>> {
        Some(
            <Sexp as SexpValueQuery<Array1<f64>>>::value(element, el::AT)
                .unwrap()
                .slice_move(s![0..2]),
        )
    }

    ///get the angle from the at node.
    pub fn angle(element: &Sexp) -> Option<f64> {
        element.query(el::AT).next().unwrap().get(2)
    }

    lazy_static! {
        static ref RE: regex::Regex = Regex::new(r"^.*_(\d*)_(\d*)$").unwrap();
    }

    /// extract the unit number from the subsymbol name
    pub fn unit_number(name: String) -> usize {
        if let Some(line) = RE.captures_iter(&name).next() {
            line[1].parse().unwrap()
        } else {
            0
        }
    }

    ///get a pin of a library symbol.
    pub fn pin<'a>(root: &'a Sexp, number: &str) -> Option<&'a Sexp> {
        for _unit in root.query(el::SYMBOL) {
            for pin in _unit.query(el::PIN) {
                let n: String = pin.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                if number == n {
                    return Some(pin);
                }
            }
        }
        None
    }

    ///get all the pins of a library symbol.
    pub fn pins(root: &Sexp, unit: usize) -> Result<Vec<&Sexp>, Error> {
        let mut items: Vec<&Sexp> = Vec::new();
        for _unit in root.query(el::SYMBOL) {
            let number = unit_number(_unit.get(0).unwrap());
            if unit == 0 || number == 0 || number == unit {
                for pin in _unit.query(el::PIN) {
                    items.push(pin);
                }
            }
        }
        if items.is_empty() {
            let name: String = root.get(0).unwrap();
            Err(Error::SexpError(name.clone(), unit.to_string()))
        } else {
            Ok(items)
        }
    }

    ///get the library from the schema document.
    pub fn get_library<'a>(root: &'a Sexp, lib_id: &str) -> Option<&'a Sexp> {
        let libraries: &Sexp = root.query(el::LIB_SYMBOLS).next().unwrap();
        let lib: Vec<&Sexp> = libraries
            .nodes()
            .filter(|l| {
                let identifier: String = l.get(0).unwrap();
                identifier == lib_id
            })
            .collect();
        if lib.len() == 1 {
            Some(lib.first().unwrap())
        } else {
            None
        }
    }

    /// load a library
    ///
    /// # Arguments
    ///
    /// * `name`     - The symbol name.
    /// * `pathlist` - List of library paths.
    /// * `return`   - Library symbol as Sexp struct.
    pub fn library(name: &str, pathlist: Vec<String>) -> Result<Sexp, Error> {
        let t: Vec<&str> = name.split(':').collect();
        for path in &pathlist {
            let filename = &format!("{}/{}.kicad_sym", path, t[0]);
            if let Ok(doc) = SexpParser::load(filename) {
                if let Ok(tree) = SexpTree::from(doc.iter()) {
                    for node in tree.root()?.query(el::SYMBOL) {
                        let sym_name: String = node.get(0).unwrap();
                        if sym_name == t[1] {
                            let mut node = node.clone();
                            node.set(0, SexpAtom::Text(name.to_string()))?;
                            return Ok(node.clone());
                        }
                    }
                }
            }
        }
        Err(Error::LibraryNotFound(name.to_string()))
    }
}

/// internal state of the sexp builder.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderState {
    StartSymbol(String),
    EndSymbol,
    Values(String),
    Text(String),
}

/// utility to build a sexp document.
///
///The sruct us used by the sexp macro.
pub struct Builder {
    pub nodes: Vec<BuilderState>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    pub fn push(&mut self, name: &str) {
        self.nodes.push(BuilderState::StartSymbol(name.to_string()));
    }
    pub fn end(&mut self) {
        self.nodes.push(BuilderState::EndSymbol);
    }
    pub fn value(&mut self, name: &str) {
        self.nodes.push(BuilderState::Values(name.to_string()));
    }
    pub fn text(&mut self, name: &str) {
        self.nodes.push(BuilderState::Text(name.to_string()));
    }
    ///return a SexpTree.
    pub fn sexp(&self) -> Result<SexpTree, Error> {
        let mut iter = self.nodes.iter();
        let mut stack: Vec<(String, Sexp)> = Vec::new();
        if let Some(BuilderState::StartSymbol(name)) = iter.next() {
            stack.push((name.to_string(), Sexp::from(name.to_string())));
        } else {
            return Err(Error::SexpError(
                String::from("Document does not start with a start symbol."),
                String::from("sexp"),
            ));
        };
        loop {
            match iter.next() {
                Some(BuilderState::Values(value)) => {
                    let len = stack.len();
                    if let Some((_, parent)) = stack.get_mut(len - 1) {
                        parent.nodes.push(SexpAtom::Value(value.to_string()));
                    }
                }
                Some(BuilderState::Text(value)) => {
                    let len = stack.len();
                    if let Some((_, parent)) = stack.get_mut(len - 1) {
                        parent.nodes.push(SexpAtom::Text(value.to_string()));
                    }
                }
                Some(BuilderState::EndSymbol) => {
                    let len = stack.len();
                    if len > 1 {
                        let (_n, i) = stack.pop().unwrap();
                        if let Some((_, parent)) = stack.get_mut(len - 2) {
                            parent.nodes.push(SexpAtom::Node(i));
                        }
                    }
                }
                Some(BuilderState::StartSymbol(name)) => {
                    stack.push((name.to_string(), Sexp::from(name.to_string())));
                }
                None => break,
            }
        }
        let (_n, i) = stack.pop().unwrap();
        Ok(SexpTree { tree: i })
    }
}

///call the sexp document builder macro.
#[macro_export]
macro_rules! sexp {
   ($($inner:tt)*) => {
       {
        use sexp_macro::parse_sexp;
        let mut document = Builder::new();
        parse_sexp!(document, $($inner)*);
        document.sexp().unwrap()
       }
    };
}
//pub use sexp;

const NO_NEW_LINE: [&str; 13] = [
    "at",
    "pin_names",
    "offset",
    "in_bom",
    "on_board",
    "font",
    "size",
    "justify",
    "lib_id",
    "effects",
    "width",
    "type",
    "length",
];

///Write the document to a Write trait.
pub trait SexpWriter {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<bool, Error>;
}

impl SexpWriter for Sexp {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<bool, Error> {
        let mut has_new_line = false;
        let mut has_children = false;
        if indent > 0 {
            if NO_NEW_LINE.contains(&self.name.as_str()) {
                out.write_all(b" ")?;
            } else {
                has_new_line = true;
                out.write_all(b"\n")?;
                out.write_all("  ".repeat(indent).as_bytes())?;
            }
        }
        out.write_all(b"(")?;
        out.write_all(self.name.as_bytes())?;
        for node in &self.nodes {
            match node {
                SexpAtom::Node(node) => {
                    has_children |= node.write(out, indent + 1)?;
                }
                SexpAtom::Value(value) => {
                    out.write_all(b" ")?;
                    out.write_all(value.as_bytes())?;
                }
                SexpAtom::Text(value) => {
                    out.write_all(b" \"")?;
                    out.write_all(value.as_bytes())?;
                    out.write_all(b"\"")?;
                }
            }
        }
        if has_children && has_new_line {
            out.write_all(b"\n")?;
            out.write_all("  ".repeat(indent).as_bytes())?;
            out.write_all(b")")?;
        } else if indent == 0 {
            out.write_all(b"\n)\n")?;
        } else {
            out.write_all(b")")?;
        }
        Ok(has_new_line)
    }
}

#[cfg(test)]
mod tests {
    use crate::{sexp, SexpParser, SexpWriter, State};

    use super::Builder;

    #[test]
    fn check_index() {
        let doc = SexpParser::from(String::from(
            r#"(node value1 value2 "value 3" "value 4" "" "value \"four\"" endval)"#,
        ));
        let mut iter = doc.iter();
        let state = iter.next();
        assert_eq!(state, Some(State::StartSymbol("node")));

        let state = iter.next();
        assert_eq!(state, Some(State::Values("value1")));

        let state = iter.next();
        assert_eq!(state, Some(State::Values("value2")));

        let state = iter.next();
        assert_eq!(state, Some(State::Text("value 3")));

        let state = iter.next();
        assert_eq!(state, Some(State::Text("value 4")));

        let state = iter.next();
        assert_eq!(state, Some(State::Text("")));

        let state = iter.next();
        assert_eq!(state, Some(State::Text(r#"value \"four\""#)));

        let state = iter.next();
        assert_eq!(state, Some(State::Values("endval")));

        let state = iter.next();
        assert_eq!(state, Some(State::EndSymbol));
    }

    #[test]
    fn simple_content() {
        let doc = SexpParser::from(String::from(
            r#"(node value1 value2 "value 3" "value 4" "" "value \"four\"" endval)"#,
        ));
        let mut node_name = String::new();
        let mut values = String::new();
        let mut texts = String::new();
        let mut count = 0;
        for state in doc.iter() {
            match state {
                State::StartSymbol(name) => {
                    node_name = name.to_string();
                    count += 1;
                }
                State::EndSymbol => {
                    count -= 1;
                }
                State::Values(value) => {
                    values += value;
                }
                State::Text(value) => {
                    texts += value;
                }
            }
        }
        assert_eq!("node", node_name);
        assert_eq!(values, "value1value2endval");
        assert_eq!(texts, r#"value 3value 4value \"four\""#);
        assert_eq!(count, 0);
    }
    #[test]
    fn next_sub_symbol() {
        let doc = SexpParser::from(String::from("(node value1 (node2))"));
        let mut iter = doc.iter();
        let state = iter.next();
        assert_eq!(state, Some(State::StartSymbol("node")));

        let state = iter.next();
        assert_eq!(state, Some(State::Values("value1")));

        let state = iter.next();
        assert_eq!(state, Some(State::StartSymbol("node2")));
    }

    #[test]
    fn next_sub_symbol_values() {
        let doc = SexpParser::from(String::from("(node value1 (node2 value2))"));
        let mut count = 0;
        let mut ends = 0;
        let mut iter = doc.iter();
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("node", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("value1", *value);
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("node2", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("value2", *value);
        }
        if let Some(State::EndSymbol) = &iter.next() {
            ends -= 1;
        }
        if let Some(State::EndSymbol) = &iter.next() {
            ends -= 1;
        }
        assert_eq!(count, 4);
        assert_eq!(ends, 0);
    }
    #[test]
    fn next_sub_symbol_text() {
        let doc = SexpParser::from(String::from("(node value1 (node2 \"value 2\"))"));
        let mut count = 0;
        let mut ends = 0;
        let mut iter = doc.iter();
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("node", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("value1", *value);
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("node2", *name);
        }
        if let Some(State::Text(value)) = &iter.next() {
            count += 1;
            assert_eq!("value 2", *value);
        }
        if let Some(State::EndSymbol) = &iter.next() {
            ends -= 1;
        }
        if let Some(State::EndSymbol) = &iter.next() {
            ends -= 1;
        }
        assert_eq!(count, 4);
        assert_eq!(ends, 0);
    }
    #[test]
    fn next_sub_symbol_text_escaped() {
        let doc = SexpParser::from(String::from(r#"(node value1 (node2 "value \"2\""))"#));
        let mut count = 0;
        let mut iter = doc.iter();
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            assert_eq!("node", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("value1", *value);
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            assert_eq!("node2", *name);
        }
        if let Some(State::Text(value)) = &iter.next() {
            count += 1;
            assert_eq!(r#"value \"2\""#, *value);
        }
        assert_eq!(count, 4);
    }
    #[test]
    fn next_sub_symbol_line_breaks() {
        let doc = SexpParser::from(String::from("(node value1\n(node2 \"value 2\"\n)\n)"));
        let mut count = 0;
        let mut iter = doc.iter();
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            assert_eq!("node", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("value1", *value);
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            assert_eq!("node2", *name);
        }
        if let Some(State::Text(value)) = &iter.next() {
            count += 1;
            assert_eq!("value 2", *value);
        }
        assert_eq!(count, 4);
    }
    #[test]
    fn parse_stroke() {
        let doc = SexpParser::from(String::from(
            "(stroke (width 0) (type default) (color 0 0 0 0))",
        ));
        let mut count = 0;
        let mut ends = 0;
        let mut iter = doc.iter();
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("stroke", *name);
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("width", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("0", *value);
        }
        if let Some(State::EndSymbol) = &iter.next() {
            count += 1;
            ends -= 1;
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("type", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("default", *value);
        }
        if let Some(State::EndSymbol) = &iter.next() {
            count += 1;
            ends -= 1;
        }
        if let Some(State::StartSymbol(name)) = &iter.next() {
            count += 1;
            ends += 1;
            assert_eq!("color", *name);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("0", *value);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("0", *value);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("0", *value);
        }
        if let Some(State::Values(value)) = &iter.next() {
            count += 1;
            assert_eq!("0", *value);
        }
        if let Some(State::EndSymbol) = &iter.next() {
            count += 1;
            ends -= 1;
        }
        if let Some(State::EndSymbol) = &iter.next() {
            count += 1;
            ends -= 1;
        }
        assert_eq!(iter.next(), None);
        assert_eq!(count, 14);
        assert_eq!(ends, 0);
    }
    #[test]
    fn build_document() {
        /* (kicad_sch (version 20211123) (generator elektron)

            (uuid e91be4a5-3c12-4daa-bee2-30f8afcd4ab8)

            (paper "A4")
            (lib_symbols
            )
        )*/
        let mut document = Builder::new();
        document.push("kicad_sch");
        document.push("version");
        document.value("20211123");
        document.end();
        document.push("generator");
        document.value("elektron");
        document.end();
        document.push("uuid");
        document.value("e91be4a5-3c12-4daa-bee2-30f8afcd4ab8");
        document.end();
        document.push("paper");
        document.text("A4");
        document.end();
        document.push("lib_symbols");
        document.end();

        document.end();
        let tree = document.sexp().unwrap();
        tree.root()
            .unwrap()
            .write(&mut std::io::stdout(), 0)
            .unwrap();
    }
    #[test]
    fn macro_document() {
        let tree = sexp!(("kicad_sch" ("version" {super::KICAD_SCHEMA_VERSION}) ("generator" "elektron")
            ("uuid" "e91be4a5-3c12-4daa-bee2-30f8afcd4ab8")
            ("paper" r"A4")
            ("lib_symbols")
        ));
        let mut writer: Vec<u8> = Vec::new();
        tree.root().unwrap().write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(String::from("(kicad_sch\n  (version 20211123)\n  (generator elektron)\n  (uuid e91be4a5-3c12-4daa-bee2-30f8afcd4ab8)\n  (paper \"A4\")\n  (lib_symbols)\n)\n"), result);
    }
    #[test]
    fn remove_element() {
        let mut tree = sexp!(("kicad_sch" ("version" {super::KICAD_SCHEMA_VERSION}) ("generator" "elektron")
            ("uuid" "e91be4a5-3c12-4daa-bee2-30f8afcd4ab8")
            ("paper" r"A4")
            ("lib_symbols")
        ));
        let mut writer: Vec<u8> = Vec::new();
        tree.root().unwrap().write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(String::from("(kicad_sch\n  (version 20211123)\n  (generator elektron)\n  (uuid e91be4a5-3c12-4daa-bee2-30f8afcd4ab8)\n  (paper \"A4\")\n  (lib_symbols)\n)\n"), result);

        tree.root_mut().unwrap().remove("paper").unwrap();
        let mut writer: Vec<u8> = Vec::new();
        tree.root().unwrap().write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(String::from("(kicad_sch\n  (version 20211123)\n  (generator elektron)\n  (uuid e91be4a5-3c12-4daa-bee2-30f8afcd4ab8)\n  (lib_symbols)\n)\n"), result);
    }
}
