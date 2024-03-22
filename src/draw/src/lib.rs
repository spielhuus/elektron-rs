//! rust backend for the schema drawer
mod error;
mod model;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};

use lazy_static::lazy_static;
use ndarray::{arr1, arr2, Array1};
use regex::Regex;
use uuid::Uuid;

pub use error::Error;
use reports::erc;

use sexp::{
    self, el::{self, EFFECTS}, math::{Bounds, PinOrientation, Shape, Transform, MIRROR}, utils, Builder, Sexp, SexpAtom, SexpParser, SexpProperty, SexpTree, SexpValueQuery, SexpValuesQuery, SexpWriter
};

pub use model::{
    At, Attribute, Direction, Dot, DotPosition, Label, LabelPosition, Nc, Properties, Symbol, To,
};

const LABEL_BORDER: f64 = 2.0;

macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.3}", v).parse::<f64>().unwrap())
    };
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

pub fn from_library(
    lib_id: String,
    library: &Sexp,
    at: Array1<f64>,
    angle: f64,
    unit: usize,
    reference: String,
    value: String,
) -> Result<Sexp, Error> {
    let mut symbol = Sexp::from(el::SYMBOL.to_string());

    //let lib_symbol: String = library.get(0).unwrap();
    symbol.push(SexpAtom::Node(
        sexp::sexp!((lib_id !{lib_id.as_str()}))
            .root()?
            .clone(),
    ))?;
    symbol.push(SexpAtom::Node(sexp::sexp!(
                (at {at[0].to_string().as_str()} 
                    {at[1].to_string().as_str()} 
                    {angle.to_string().as_str()})
                ).root()?.clone()))?;
    symbol.push(SexpAtom::Node(
        sexp::sexp!((unit {unit.to_string().as_str()}))
            .root()?
            .clone(),
    ))?;

    for (count, item) in library.iter().enumerate() {
        match item {
            SexpAtom::Node(n) => {
                if n.name == el::PROPERTY {
                    let mut prop = n.clone();
                    let property_name: String = n.get(0).unwrap();
                    if property_name == el::PROPERTY_REFERENCE {
                        prop.set(1, SexpAtom::Text(reference.to_string()))?;
                    } else if property_name == "Value" {
                        prop.set(1, SexpAtom::Text(value.to_string()))?;
                    }
                    symbol.push(SexpAtom::Node(prop))?;
                } else if n.name != "extends"
                    && n.name != el::SYMBOL 
                    && n.name != "pin_numbers"
                    && n.name != "pin_names"
                    && n.name != "power"
                {
                    symbol.push(SexpAtom::Node(n.clone()))?;
                }
            }
            SexpAtom::Value(v) => {
                symbol.push(SexpAtom::Value(v.clone()))?;
            }
            SexpAtom::Text(t) => {
                if count != 0 {
                    symbol.push(SexpAtom::Text(t.clone()))?;
                }
            }
        };
    }

    Ok(symbol)
}

///get the symbol unit element.
fn subsymbol(library: &Sexp, unit: usize) -> Result<Sexp, Error> {
    for l in library.query(el::SYMBOL) {
        let symbol_name: String = l.get(0).unwrap();
        if utils::unit_number(symbol_name) == unit {
            return Ok(l.clone());
        }
    }
    Err(Error::LibraryNotFound(format!(
        "Symbol unit not found {}",
        unit
    )))
}

///Enum for the set property key
#[derive(Debug, Eq, PartialEq)]
enum PropertyKey {
    Exact(String),
    Range(String, usize, usize),
    From(String, usize),
    To(String, usize),
}

lazy_static! {
    static ref RE_FROM_TO: regex::Regex = Regex::new(r"([A-Z]+)\[([\d]+)\.\.([\d]+)").unwrap();
    static ref RE_FROM: regex::Regex = Regex::new(r"([A-Z]+)\[([\d]+)\.\.\]").unwrap();
    static ref RE_TO: regex::Regex = Regex::new(r"([A-Z]+)\[\.\.([\d]+)\]").unwrap();
    static ref RE_KEY: regex::Regex = Regex::new(r"([A-Z]+)([\d]+)").unwrap();
}

impl PropertyKey {
    pub fn from(key: String) -> Self {
        if let Some(captures) = RE_FROM_TO.captures(&key) {
            Self::Range(
                captures.get(1).unwrap().as_str().to_string(),
                captures.get(2).unwrap().as_str().parse::<usize>().unwrap(),
                captures.get(3).unwrap().as_str().parse::<usize>().unwrap(),
            )
        } else if let Some(captures) = RE_FROM.captures(&key) {
            Self::From(
                captures.get(1).unwrap().as_str().to_string(),
                captures.get(2).unwrap().as_str().parse::<usize>().unwrap(),
            )
        } else if let Some(captures) = RE_TO.captures(&key) {
            Self::To(
                captures.get(1).unwrap().as_str().to_string(),
                captures.get(2).unwrap().as_str().parse::<usize>().unwrap(),
            )
        } else {
            Self::Exact(key)
        }
    }

    pub fn matches(&self, key: String) -> bool {
        let other: (String, usize) = if let Some(captures) = RE_KEY.captures(&key) {
                (captures.get(1).unwrap().as_str().to_string(),
                 captures.get(2).unwrap().as_str().parse::<usize>().unwrap())

        } else {
            return false;
        };

        match self {
            PropertyKey::Exact(k) => key == *k,
            PropertyKey::Range(k, start, end) => {
                other.0 == *k &&
                other.1 >= *start &&
                other.1 <= *end
            }
            PropertyKey::From(k, start) => {
                other.0 == *k &&
                other.1 >= *start
            }
            PropertyKey::To(k, end) => {
                other.0 == *k &&
                other.1 <= *end
            }
        }
    }
}

/// Draw the schematic.
pub struct Draw {
    pos: At,
    pub schema: SexpTree,
    library_path: Vec<String>,
    positions: HashMap<String, Array1<f64>>,
    references: HashMap<String, u32>,
}

impl Draw {
    /// create a new Draw Object.
    ///
    /// optional Arguments:
    /// paper: Paper size [A4, A3, A2, ...]
    pub fn new(library_path: Vec<String>, kwargs: Option<HashMap<String, String>>) -> Self {
        let paper = if let Some(kwargs) = &kwargs {
            if kwargs.contains_key("paper") {
                kwargs.get("paper").unwrap().clone()
            } else {
                String::from("A4")
            }
        } else { String::from("A4") };
        let mut schema = 
            sexp::sexp!((kicad_sch (version {sexp::KICAD_SCHEMA_VERSION}) (generator !{sexp::KICAD_SCHEMA_GENERATOR})
                (uuid !{sexp::uuid!()})
                (paper !{paper.as_str()})
            ) 
        );
        if let Some(kwargs) = kwargs {
            let mut title_block = Sexp::from(String::from(el::TITLE_BLOCK));
            if let Some(title) = kwargs.get(el::TITLE_BLOCK_TITLE) {
                title_block.push(SexpAtom::Node(sexp::sexp!(
                            (title !{title.as_str()})).root().unwrap().clone())).unwrap();
            }
            if let Some(date) = kwargs.get(el::TITLE_BLOCK_DATE) {
                title_block.push(SexpAtom::Node(sexp::sexp!((date !{date.as_str()})).root().unwrap().clone())).unwrap();
            }
            if let Some(rev) = kwargs.get(el::TITLE_BLOCK_REV) {
                title_block.push(SexpAtom::Node(sexp::sexp!((rev !{rev.as_str()})).root().unwrap().clone())).unwrap();
            }
            if let Some(company) = kwargs.get(el::TITLE_BLOCK_COMPANY) {
                title_block.push(SexpAtom::Node(sexp::sexp!((company !{company.as_str()})).root().unwrap().clone())).unwrap();
            }
            for i in 1..5 {
                if let Some(comment) = kwargs.get(&format!("comment{}", i)) {
                    title_block.push(SexpAtom::Node(sexp::sexp!(
                                (comment {i.to_string().as_str()} 
                                        !{comment.as_str()})
                                ).root().unwrap().clone())).unwrap();
                }
            }
            schema.root_mut().unwrap().push(SexpAtom::Node(title_block)).unwrap();
        }
        schema.root_mut().unwrap().push(SexpAtom::Node(Sexp::from(String::from(el::LIB_SYMBOLS)))).unwrap();
        Self {
            pos: At::Pos((25.4, 25.4)),
            schema,
            library_path,
            positions: HashMap::new(),
            references: HashMap::new(),
        }
    }

    //Build the netlist and get the circuit
    /* pub fn circuit(&self, pathlist: Vec<String>) -> Result<Circuit, Error> {
        let netlist = Netlist::from(&self.schema)?;
        let mut circuit = Circuit::new(String::from("draw circuit"), pathlist);
        netlist.circuit(&mut circuit).unwrap();
        Ok(circuit)
    } */

    ///Set the position.
    pub fn set(&mut self, pos: At) {
        self.pos = pos;
    }

    //Get the next reference for a letter.
    pub fn next(&mut self, key: String) -> String {
        let entry = self
            .references
            .entry(key.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        format!("{}{}", key, entry)
    }

    //Set the next reference for a letter.
    pub fn counter(&mut self, key: String, count: u32) {
        self.references.insert(key, count - 1);
    }

    //Get the next reference for a letter.
    pub fn last(&mut self, key: String) -> Result<String, Error> {
        if let Some(entry) = self.references.get(&key) {
            Ok(format!("{}{}", &key, entry))
        } else {
            Err(Error::NotFound(String::from("Reference"), key))
        }
    }

    //Set a property value for symbols.
    pub fn property(&mut self, regex: String, key: String, value: String) -> Result<(), Error> {
        let matcher = PropertyKey::from(regex);
        let Ok(root) = self.schema.root_mut() else {
            return Err(Error::SexpError(String::from("root node not found.")));
        };
        for symbol in root.query_mut(el::SYMBOL) {
            let reference = symbol.property(el::PROPERTY_REFERENCE).unwrap();
            if matcher.matches(reference) {
                let mut found = false;
                for prop in symbol.query_mut(el::PROPERTY) {
                    let prop_key: String = prop.get(0).unwrap();
                    if prop_key == key {
                        found = true;
                        prop.set(1, SexpAtom::Value(value.clone()))?;
                        let effects = prop.query_mut(el::EFFECTS).next().unwrap();
                        let values: Vec<String> = effects.values();
                        if !values.contains(&String::from("hide")) {
                            effects.push(SexpAtom::Value(String::from("hide"))).unwrap();
                        }
                        break;
                    }
                }
                if !found {
                    symbol.push(SexpAtom::Node(
                        sexp::sexp!((property !{key.as_str()} !{value.as_str()} (at "0" "0" "0")
                        (effects (font (size "1.27" "1.27")) "hide"))).root().unwrap().clone()))?;
                }
            }
        }
        Ok(())
    }

    ///Get the ERC report as String.
    pub fn erc(&self) -> String {
        let items = erc::erc_from_tree(&self.schema).unwrap();
        let mut res = Vec::<u8>::new();
        for item in items {
            writeln!(res, "{}: {} ({}x{})", item.id, item.reference, item.at[0], item.at[1]).unwrap();
            /* match i {
                erc::ErcItem::NoReference { reference, at } => {
                    writeln!(res, "NoReference: {} ({}x{})", reference, at[0], at[1]).unwrap();
                }
                erc::ErcItem::ValuesDiffer { reference, at } => {
                    writeln!(
                        res,
                        "Different values for: {} ({}x{})",
                        reference, at[0], at[1]
                    )
                    .unwrap();
                }
                erc::ErcItem::Netlist(nl) => {
                    writeln!(res, "Netlist: {}", nl).unwrap();
                }
                erc::ErcItem::NotAllParts { reference, at } => {
                    writeln!(
                        res,
                        "Not all symbol units on schema: {} ({}x{})",
                        reference, at[0], at[1]
                    )
                    .unwrap();
                }
                erc::ErcItem::PinNotConnected { reference, at } => {
                    writeln!(
                        res,
                        "Pin not connected: {} ({}x{})",
                        reference, at[0], at[1]
                    )
                    .unwrap();
                }
            } */
        }
        std::str::from_utf8(&res).unwrap().to_string()
    }

    /// Write the schema to a Writer
    ///
    /// * `file`: the filename
    pub fn write(&self, file: &str) {
        let mut out = File::create(file).unwrap();
        self.schema.root().unwrap().write(&mut out, 0).unwrap();
    }

    fn at(&self, pos: &At) -> Result<Array1<f64>, Error> {
        match pos {
            At::Pos(pos) => Ok(arr1(&[pos.0, pos.1])),
            At::Pin(reference, number) => self.pin_pos(reference.to_string(), number.to_string()),
            At::Dot(name) => self
                .positions
                .get(name)
                .ok_or_else(|| Error::PositionNotFound(name.to_string()))
                .cloned(),
        }
    }

    // return a symbol by reference, there are enventualy multiple symbols with the same ref, the
    // first will be returned.
    fn get_symbol_by_ref(&self, name: &str) -> Result<&Sexp, Error> {
        for sym in self.schema.root().unwrap().query(el::SYMBOL) {
            let prop: Option<String> = sym.property(el::PROPERTY_REFERENCE);
            if let Some(reference) = prop {
                if reference == name {
                    return Ok(sym);
                }
            }
        }
        Err(Error::SymbolNotFound(name.to_string()))
    }

    // return a symbol by reference and unit.
    fn get_symbol_by_unit(&self, name: &str, unit: usize) -> Result<&Sexp, Error> {
        for sym in self.schema.root().unwrap().query(el::SYMBOL) {
            let unit_number: usize = sym.value(el::SYMBOL_UNIT).unwrap();
            let prop: Option<String> = sym.property(el::PROPERTY_REFERENCE);
            if let Some(reference) = prop {
                if reference == name && (unit_number == unit || unit == 0) {
                    return Ok(sym);
                }
            }
        }
        Err(Error::SymbolNotFound(name.to_string()))
    }

    // return a library symbol when it exists or load it from the libraries.
    fn get_library(&self, name: &str) -> Result<Sexp, Error> {

        //get existing library
        for lib in self
            .schema
            .root()
            .unwrap()
            .query(el::LIB_SYMBOLS)
            .next()
            .unwrap()
            .query(el::SYMBOL)
        {
            let lib_name: String = lib.get(0).unwrap();
            if lib_name == name {
                return Ok(lib.clone());
            }
        }

        Err(Error::LibraryNotFound(name.to_string()))
    }

    // return a library symbol when it exists or load it from the libraries.
    fn load_library(&mut self, name: &str) -> Result<Sexp, Error> {
        //get existing library
        for lib in self
            .schema
            .root()
            .unwrap()
            .query(el::LIB_SYMBOLS)
            .next()
            .unwrap()
            .query(el::SYMBOL)
        {
            let lib_name: String = lib.get(0).unwrap();
            if lib_name == name {
                return Ok(lib.clone());
            }
        }

        //or load it
        let lib = library(name, self.library_path.clone())?;
        let extends: Option<String> = lib.value("extends");
        if let Some(extends) = extends {
            let library = &name[0 .. name.find(':').unwrap()];
            let symbol_name = &name[name.find(':').unwrap()+1 .. name.len()];
            let extend_symbol = crate::library(
                format!("{}:{}", library, extends).as_str(),
                self.library_path.clone(),
            )?;

            let mut merged_sym = Sexp::from(el::SYMBOL.to_string());
            merged_sym.push(SexpAtom::Text(name.to_string()))?;
            let mut index = 1;

            for element in extend_symbol.iter() {
                if index > 1 {
                    match element {
                        SexpAtom::Node(element) => {
                            if element.name == el::SYMBOL {
                                let sub_name: String = element.get(0).unwrap();
                                let number = &sub_name
                                    [extends.len() + 1 .. sub_name.len()];
                                let mut subsymbol = element.clone();
                                subsymbol
                                    .set(
                                        0,
                                        SexpAtom::Value(
                                            format!(
                                                "{}_{}",
                                                symbol_name,
                                                number
                                            )
                                            .to_string(),
                                        ),
                                    )
                                    .unwrap();
                                merged_sym.push(SexpAtom::Node(subsymbol.clone())).unwrap();
                            } else if element.name == el::PROPERTY {
                                let prop_name: String = element.get(0).unwrap();
                                let sym_property: Option<Sexp> = lib.property(&prop_name);
                                if let Some(sym_property) = sym_property {
                                    merged_sym
                                        .push(SexpAtom::Node(sym_property.clone()))
                                        .unwrap();
                                } else {
                                    merged_sym.push(SexpAtom::Node(element.clone())).unwrap();
                                }
                            } else {
                                let el = lib.query(&element.name).next();
                                if let Some(el) = el {
                                    merged_sym.push(SexpAtom::Node(el.clone())).unwrap();
                                } else {
                                    merged_sym.push(SexpAtom::Node(element.clone())).unwrap();
                                }
                            }
                        }
                        SexpAtom::Value(element) => todo!("unknown value: {}", element),
                        SexpAtom::Text(_) => todo!(),
                    }
                }

                index += 1;
            }
            self.schema
                .root_mut()
                .unwrap()
                .query_mut(el::LIB_SYMBOLS)
                .next()
                .unwrap()
                .push(SexpAtom::Node(merged_sym.clone()))?;
            return Ok(merged_sym);
        }
        self.schema
            .root_mut()
            .unwrap()
            .query_mut(el::LIB_SYMBOLS)
            .next()
            .unwrap()
            .push(SexpAtom::Node(lib.clone()))?;
        Ok(lib.clone())
    }

    fn align(&self, position: LabelPosition, angle: f64, mirror: String) -> String {
        match position {
            LabelPosition::North => {
                "center"
            },
            LabelPosition::South => {
                "center"
            },
            LabelPosition::West => {
                let orientation = if angle == 180.0 { el::JUSTIFY_LEFT } else { el::JUSTIFY_RIGHT };
                if !mirror.is_empty() && mirror.contains('x') { 
                    //TODO what to do here?
                    // orientation = if orientation == el::JUSTIFY_RIGHT { el::JUSTIFY_LEFT } else { el::JUSTIFY_RIGHT };
                }
                orientation
            },
            LabelPosition::East => {
                let mut orientation = if angle == 180.0 { el::JUSTIFY_RIGHT } else { el::JUSTIFY_LEFT };
                if !mirror.is_empty() { 
                    orientation = if orientation == el::JUSTIFY_RIGHT { el::JUSTIFY_LEFT } else { el::JUSTIFY_RIGHT };
                }
                orientation
            },
            LabelPosition::Offset(_, _) => {
                "center"
            },
        }.to_string()
    }

    fn angle(&self, symbol_angle: f64) -> f64 {
        if symbol_angle == 180.0 {
            0.0
        } else {
            0.0 - symbol_angle
        }
    }

    fn place_property(
        &mut self,
        symbol: &mut Sexp,
        label: Option<LabelPosition>,
    ) -> Result<(), Error> {

        let lib_id: String = symbol.value(el::LIB_ID).unwrap();
        let lib = self.get_library(&lib_id).unwrap();

        //get and sort the shape size
        let _size = Shape::transform(symbol, &symbol.bounds(&lib).unwrap());
        let _size = if _size[[0, 0]] > _size[[1, 0]] {
            arr2(&[
                [_size[[1, 0]], _size[[0, 1]]],
                [_size[[0, 0]], _size[[1, 1]]],
            ])
        } else {
            _size
        };
        let _size = if _size[[0, 1]] > _size[[1, 1]] {
            arr2(&[
                [_size[[0, 0]], _size[[1, 1]]],
                [_size[[1, 0]], _size[[0, 1]]],
            ])
        } else {
            _size
        };

        let symbol_angle: f64 = symbol.query(el::AT).next().unwrap().get(2).unwrap();
        let symbol_mirror: String = if let Some(mirror) = symbol.query(el::MIRROR).next() {
            mirror.get(0).unwrap()
        } else {
            String::new()
        };
        let symbol_position = utils::at(symbol).unwrap();
        let unit: usize = symbol.value(el::SYMBOL_UNIT).unwrap();
        let positions = self.pin_positions(symbol, &lib);
        let mut offset = 0.0;
        let pins = if let Ok(pins) = utils::pins(&lib, unit) {
            pins.len()
        } else { 0 };


        //count the visible fields
        let mut vis_props: Vec<&mut Sexp> = symbol
            .query_mut(el::PROPERTY)
            .filter_map(|node| {
                let value: String = node.get(1).unwrap();
                if value.is_empty() || value == "~" {
                    return None;
                }

                let hide = if let Some(node) = node.query(el::EFFECTS).next() {
                    let values: Vec<String> = node.values();
                    if let Some(hide) = node.query("hide").next() {
                        let values: Vec<String> = hide.values();
                        values.contains(&"yes".to_string())
                    } else {
                        values.contains(&"hide".to_string())
                    }
                } else {
                    true 
                };

                if hide {
                    None 
                } else {
                    Option::from(node)
                }
            }).collect();
        let vis_len = vis_props.len();

        for prop in &mut vis_props {
            let position = if let Some(label) = label.clone() {
                label
            
            } else if pins == 1 {
                if positions.contains(&PinOrientation::Up) {
                    LabelPosition::North
                } else if positions.contains(&PinOrientation::Right) {
                    LabelPosition::East
                } else if positions.contains(&PinOrientation::Left) {
                    LabelPosition::West
                } else if positions.contains(&PinOrientation::Down) {
                    LabelPosition::South
                } else {
                    LabelPosition::North
                    //TODO todo!("unplacable property");
                }

            } else if !positions.contains(&PinOrientation::Up) {
                LabelPosition::North
            } else if !positions.contains(&PinOrientation::Right) {
                LabelPosition::East
            } else if !positions.contains(&PinOrientation::Left) {
                LabelPosition::West
            } else if !positions.contains(&PinOrientation::Down) {
                LabelPosition::South
            } else {
                LabelPosition::North
                //TODO todo!("unplacable property");
            };
                        
            let at = if pins == 1 {
                    match position {
                        LabelPosition::North => {
                            arr1(&[symbol_position[0], _size[[1, 1]] + LABEL_BORDER, 0.0 /*- symbol_angle*/])
                        },
                        LabelPosition::South => {
                            arr1(&[symbol_position[0], _size[[0, 1]] - LABEL_BORDER, 0.0 - symbol_angle])
                        },
                        LabelPosition::West => {
                            arr1(&[_size[[1, 0]] + LABEL_BORDER, symbol_position[1], symbol_angle])
                        }
                        LabelPosition::East => {
                            arr1(&[_size[[0, 0]] - LABEL_BORDER, symbol_position[1], 0.0 - symbol_angle])
                        },
                        LabelPosition::Offset(x, y) => {
                            arr1(&[symbol_position[0] + x, symbol_position[1] - y, 0.0 - symbol_angle])
                        },
                    }
                } else {
                    match position {
                    LabelPosition::North => {
                        let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[0, 1]] - ((vis_len as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        } else {
                            _size[[1, 1]] - ((vis_len as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        };
                        arr1(&[symbol_position[0], top_pos - offset, self.angle(symbol_angle)])
                    },
                    LabelPosition::South => {
                        let bottom_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[1, 1]] + LABEL_BORDER
                        } else {
                            _size[[0, 1]] + LABEL_BORDER
                        };
                        arr1(&[symbol_position[0], bottom_pos - offset, 0.0 - self.angle(symbol_angle)])
                    },
                    LabelPosition::West => {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_len as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        arr1(&[_size[[0, 0]] - LABEL_BORDER / 2.0, top_pos - offset, self.angle(symbol_angle)])
                    },
                    LabelPosition::East => {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_len as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset, self.angle(symbol_angle)])
                    },
                    LabelPosition::Offset(x, y) => {
                        arr1(&[symbol_position[0] + x, symbol_position[1] + y - offset, 0.0 - symbol_angle])
                    },
                }
            };

            let effects = prop.query_mut(el::EFFECTS).next().unwrap();
            let orientation = self.align(position, symbol_angle, symbol_mirror.clone());
            if orientation == "center" {
                if effects.has(el::JUSTIFY) {
                    effects.remove(el::JUSTIFY).unwrap();
                }
            } else if effects.has(el::JUSTIFY) {
                    let justify = effects.query_mut(el::JUSTIFY).next().unwrap();
                    justify.set(0, SexpAtom::Value(orientation.to_string())).unwrap();
            } else {
                effects.insert(1, SexpAtom::Node(
                        sexp::sexp!(
                            (justify {orientation.as_str()}))
                        .root().unwrap().clone())).unwrap();
            }

            prop.set(2, SexpAtom::Node(sexp::sexp!(
                        (at {at[0].to_string().as_str()} 
                            {at[1].to_string().as_str()} 
                            {at[2].to_string().as_str()})
                        ).root().unwrap().clone())).unwrap();
            offset -= LABEL_BORDER;
        }
        Ok(())
    }

    // get the pin position
    // returns an array containing the PinOrientation
    fn pin_positions(&self, symbol: &Sexp, lib: &Sexp) -> HashSet<PinOrientation> {
        let unit: usize = symbol.value(el::SYMBOL_UNIT).unwrap();
        let mut found = HashSet::new();
        let Ok(pins) = utils::pins(lib, unit) else {
            return HashSet::new();
        };
        for pin in pins {
            found.insert(PinOrientation::from(symbol, pin));
        }
        found
    }

    fn pin_pos(&self, reference: String, number: String) -> Result<Array1<f64>, Error> {
        let Ok(symbol) = self.get_symbol_by_ref(reference.as_str()) else {
            return Err(Error::NotFound(reference, number));
        };
        let lib_id: String = symbol.value(el::LIB_ID).unwrap();
        let Ok(library) = self.get_library(lib_id.as_str()) else {
            return Err(Error::LibraryNotFound(lib_id.to_string()));
        };
        for subsymbol in library.query(el::SYMBOL) {
            for pin in subsymbol.query(el::PIN) {
                let pin_number_element = pin.query(el::PIN_NUMBER).next().unwrap();
                let pin_number: String = pin_number_element.get(0).unwrap();
                if pin_number == number {
                    let unit: usize = utils::unit_number(subsymbol.get(0).unwrap());
                    let real_symbol = self.get_symbol_by_unit(reference.as_str(), unit).unwrap();
                    return Ok(Shape::transform(real_symbol, &utils::at(pin).unwrap()));
                }
            }
        }
        Err(Error::PinNotFound(reference, number))
    }
}

pub trait Drawer<T> {
    fn draw(&mut self, item: &T) -> Result<Option<Sexp>, Error>;
}

impl Drawer<Label> for Draw {
    //fn draw(&mut self, item: &Label) -> Result<Option<Sexp>, Error> {
    fn draw(&mut self, item: &Label) -> Result<Option<Sexp>, Error> {
        let angle = item.angle();
        let pos = round!(self.at(&self.pos)?);
        let text = &item
            .get_name()
            .ok_or_else(|| Error::Name(String::from("Label")))?; //TODO silly name

        let align = &["left", "left", "right", "right"];
        let index = (angle / 90.0) as usize;
        let justify = align[index];

        let result = sexp::sexp!(
        (label !{text} (at {pos[0].to_string().as_str()} {pos[1].to_string().as_str()} {angle.to_string().as_str()})
            (effects (font (size "1.27" "1.27")) (justify {justify}))
            (uuid {sexp::uuid!()})
        ));

        //TODO compact
        let l = result.root().unwrap().clone();
        let root = self.schema.root_mut().unwrap();
        root.push(SexpAtom::Node(l))?;
        Ok(Some(result.root().unwrap().clone()))
    }
}

impl Drawer<Nc> for Draw {
    fn draw(&mut self, _: &Nc) -> Result<Option<Sexp>, Error> {
        let pos = round!(self.at(&self.pos)?);

        let result = sexp::sexp!(
            (no_connect (at {pos[0].to_string().as_str()} {pos[1].to_string().as_str()}) (uuid {sexp::uuid!()})
            ));

        self.schema
            .root_mut()
            .unwrap()
            .push(SexpAtom::Node(result.root().unwrap().clone()))?;
        Ok(Some(result.root().unwrap().clone()))
    }
}

impl Drawer<Symbol> for Draw {
    fn draw(&mut self, item: &Symbol) -> Result<Option<Sexp>, Error> {
        let Some(reference) = item.get_reference() else {
            return Err(Error::Name(String::from("Symbol::Reference")));
        };
        let Some(lib_id) = item.get_lib_id() else {
            return Err(Error::Name(String::from("Symbol::LibId")));
        };

        let unit = if let Some(unit) = item.get_property("unit") {
            unit.parse::<usize>().unwrap() //TODO handle error
        } else {
            1
        };

        let value = if let Some(value) = item.get_property("value") {
            value.clone()
        } else {
            String::new()
        };

        let angle: f64 = item.angle();

        let pin = if let Some(anchor) = item.anchor() {
            anchor
        } else {
            String::from("1")
        };

        let end_pin = if pin == "1" {
            String::from("2")
        } else {
            String::from("1")
        };

        //load the library
        let Ok(lib_symbol) = self.load_library(lib_id.as_str()) else {
            return Err(Error::LibraryNotFound(lib_id));
        };

        //calculate the position.
        let pos = self.at(&self.pos)?;

        let mut symbol = if let Some(sym_pin) = utils::pin(&lib_symbol, &pin)
        {
            // transform pin pos
            let theta = -angle.to_radians();
            let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
            let mut verts: Array1<f64> = utils::at(sym_pin).unwrap().dot(&rot);
            verts = if let Some(mirror) = item.mirror() {
                verts.dot(MIRROR.get(&mirror).unwrap())
            } else {
                verts.dot(MIRROR.get(&String::new()).unwrap())
            };
            verts = arr1(&[pos[0], pos[1]]) - &verts;
            verts = round!(verts);

            let mut symbol = from_library(
                lib_id,
                &lib_symbol,
                round!(verts.clone()),
                angle,
                unit,
                reference,
                value,
            )?;

            if item.length().is_some() || item.tox().is_some() || item.toy().is_some() {
                let subsymbol = subsymbol(&lib_symbol, unit).unwrap();
                let pins: Vec<&Sexp> = subsymbol.query(el::PIN).collect();
                if pins.len() == 2 {
                    let (length, rad) = if let Some(length) = item.length() {
                        (length, (item.angle() + 90.0).to_radians())
                    } else if let Some(tox) = item.tox() {
                        let end_pos = self.at(tox)?;
                        (
                            ((end_pos[0] - pos[0]).powf(2.0).sqrt()),
                            if (0.0f64).atan2(end_pos[0] - pos[0]) < 0.0 {
                                (0.0f64).atan2(end_pos[0] - pos[0]) + 369.0f64.to_radians()
                            } else {
                                (0.0f64).atan2(end_pos[0] - pos[0])
                            },
                        )
                    } else if let Some(toy) = item.toy() {
                        let end_pos = self.at(toy)?;
                        (
                            ((end_pos[1] - pos[1]).powf(2.0)).sqrt(),
                            if (end_pos[1] - pos[1]).atan2(0.0) < 0.0 {
                                (end_pos[1] - pos[1]).atan2(0.0) + 360.0f64.to_radians()
                            } else {
                                (end_pos[1] - pos[1]).atan2(0.0)
                            },
                        )
                    } else {
                        panic!("unknown branch in to.");
                    };

                    //get the symbol length
                    let pin0 = utils::pin(&lib_symbol, &pin).unwrap();
                    let pin1 = utils::pin(&lib_symbol, &end_pin).unwrap();
                    let verts0: Array1<f64> = round!(utils::at(pin0).unwrap().dot(&rot));
                    let verts1: Array1<f64> = round!(utils::at(pin1).unwrap().dot(&rot));
                    let sym_len = ((verts1[0] - verts0[0]).powf(2.0)
                        + (verts1[1] - verts0[1]).powf(2.0))
                    .sqrt();

                    if length > sym_len {
                        let wire_len = (length - sym_len) / 2.0;

                        let wire1 = round!(arr2(&[
                            [pos[0], pos[1]],
                            [pos[0] + wire_len * rad.cos(), pos[1] + wire_len * rad.sin()]
                        ]));
                        self.schema.root_mut().unwrap().push(SexpAtom::Node(sexp::sexp!(
                              (wire (pts (xy {wire1[[0, 0]].to_string().as_str()} {wire1[[0, 1]].to_string().as_str()}) 
                                         (xy {wire1[[1, 0]].to_string().as_str()} {wire1[[1, 1]].to_string().as_str()}))
                                (stroke (width "0") (type "default"))
                                (uuid {sexp::uuid!()})
                              )).root().unwrap().clone())).unwrap();

                        let wire2 = round!(arr2(&[
                            [
                                pos[0] + (wire_len + sym_len) * rad.cos(),
                                pos[1] + (wire_len + sym_len) * rad.sin()
                            ],
                            [pos[0] + length * rad.cos(), pos[1] + length * rad.sin()]
                        ]));
                        self.schema.root_mut().unwrap().push(SexpAtom::Node(sexp::sexp!(
                              (wire (pts (xy {wire2[[0, 0]].to_string().as_str()} {wire2[[0, 1]].to_string().as_str()}) 
                                         (xy {wire2[[1, 0]].to_string().as_str()} {wire2[[1, 1]].to_string().as_str()}))
                                (stroke (width "0") (type "default"))
                                (uuid {sexp::uuid!()})
                              )).root().unwrap().clone())).unwrap();

                        // set the symbol position
                        let symbol_at = round!(arr1(&[
                            pos[0] + (wire_len + sym_len / 2.0) * rad.cos(),
                            pos[1] + (wire_len + sym_len / 2.0) * rad.sin()
                        ]));
                        let at = symbol.query_mut(el::AT).next().unwrap();
                        at.set(0, SexpAtom::Value(symbol_at[0].to_string()))
                            .unwrap();
                        at.set(1, SexpAtom::Value(symbol_at[1].to_string()))
                            .unwrap();

                        // set next pos on draw
                        let next_pos = round!(arr1(&[
                            pos[0] + length * rad.cos(),
                            pos[1] + length * rad.sin()
                        ]));
                        self.pos = At::Pos((next_pos[0], next_pos[1]));
                    }
                } else {
                    panic!("only allow with 2 pins");
                }
            } else if let Some(endpin) = utils::pin(&lib_symbol, &end_pin) {
                let pts = Shape::transform(&symbol, &utils::at(endpin).unwrap());
                self.pos = At::Pos((pts[0], pts[1]));
            }
            symbol
        } else {
            from_library(
                lib_id,
                &lib_symbol,
                round!(arr1(&[pos[0], pos[1]])),
                angle,
                unit,
                reference,
                value,
            )?
        };

        //add the mirror element
        if let Some(mirror) = item.mirror() {
            symbol.insert(
                2,
                SexpAtom::Node(sexp::sexp!(("mirror" {&mirror})).root().unwrap().clone()),
            )?;
        }
        if let Some(on_schema) = item.properties.get("on_schema") {
            if on_schema == "false" || on_schema == "no" {
                if symbol.has("on_schema") {
                    let on: &mut Sexp = symbol.query_mut("on_schema").next().unwrap();
                    on.set(0, SexpAtom::Value(on_schema.clone())).unwrap();
                } else {
                    symbol.insert(5, SexpAtom::Node(sexp::sexp!((on_schema {on_schema})).root().unwrap().clone())).unwrap();
                }
            }
        }
        if let Some(on_board) = item.properties.get("on_board") {
            if symbol.has("on_board") {
                let on: &mut Sexp = symbol.query_mut("on_board").next().unwrap();
                on.set(0, SexpAtom::Value(on_board.clone())).unwrap();
            } else {
                symbol.insert(5, SexpAtom::Node(sexp::sexp!((on_board {on_board})).root().unwrap().clone())).unwrap();
            }
        }

        // add the extra properties
        for (k, v) in &item.properties {
            if k != "on_schema" && k != "on_board" && k != "value" && k != "unit" {
                let pos = round!(utils::at(&symbol).unwrap());
                symbol.push(SexpAtom::Node(
                    sexp::sexp!(
                        (property !{k.as_str()} !{v.as_str()} (at {pos[0].to_string().as_str()} {pos[1].to_string().as_str()} "0")
                            (effects (font (size "1.27" "1.27")) "hide")
                        )).root().unwrap().clone()))?;
            }
        }

        //place the properties
        self.place_property(&mut symbol, item.label()).unwrap();

        self.schema
            .root_mut()
            .unwrap()
            .push(SexpAtom::Node(symbol.clone()))?;

        Ok(Some(symbol)) 
    }
}

impl Drawer<Dot> for Draw {
    fn draw(&mut self, item: &Dot) -> Result<Option<Sexp>, Error> {
        let pos = self.at(&self.pos)?;

        //save the position if an id is given.
        let id: Option<String> = item.id();
        if let Some(id) = id {
            self.positions.insert(id, pos.clone());
        }

        //create the junction.
        let dot = sexp::sexp!(
          (junction (at {pos[0].to_string().as_str()} {pos[1].to_string().as_str()}) (diameter "0") (color "0" "0" "0" "0")
            (uuid {sexp::uuid!()})
          )
        );

        self.schema
            .root_mut()
            .unwrap()
            .push(SexpAtom::Node(dot.root().unwrap().clone()))?;
        Ok(Some(dot.root().unwrap().clone()))
    }
}

impl Drawer<At> for Draw {
    fn draw(&mut self, item: &At) -> Result<Option<Sexp>, Error> {
        self.pos = item.clone();
        Ok(None)
    }
}

impl Drawer<To> for Draw {
    fn draw(&mut self, item: &To) -> Result<Option<Sexp>, Error> {
        let pos = round!(self.at(&self.pos)?);
        let coord = if let Some(length) = item.length() {
            match item.direction() {
                Direction::Left => {
                    round!(arr2(&[[pos[0], pos[1]], [pos[0] - length, pos[1]]]))
                }
                Direction::Up => {
                    round!(arr2(&[[pos[0], pos[1]], [pos[0], pos[1] - length]]))
                }
                Direction::Down => {
                    round!(arr2(&[[pos[0], pos[1]], [pos[0], pos[1] + length]]))
                }
                _ => {
                    round!(arr2(&[[pos[0], pos[1]], [pos[0] + length, pos[1]]]))
                }
            }
        } else if let Some(tox) = item.tox() {
            let end_pos = self.at(tox)?;
            round!(arr2(&[[pos[0], pos[1]], [end_pos[0], pos[1]]]))
        } else if let Some(toy) = item.toy() {
            let end_pos = self.at(toy)?;
            round!(arr2(&[[pos[0], pos[1]], [pos[0], end_pos[1]]]))
        } else {
            round!(arr2(&[[pos[0], pos[1]], [pos[0] + 2.54, pos[1]]]))
        };

        let wire = sexp::sexp!(
              (wire (pts (xy {coord[[0, 0]].to_string().as_str()} {coord[[0, 1]].to_string().as_str()}) 
                             (xy {coord[[1, 0]].to_string().as_str()} {coord[[1, 1]].to_string().as_str()}))
                (stroke (width "0") (type "default"))
                (uuid {sexp::uuid!()})
            ));

        //set the new start position
        self.pos = At::Pos((coord[[1, 0]], coord[[1, 1]]));
        if let Some(dot) = item.dot() {
            for d in dot {
                match d {
                    DotPosition::Start => {
                        //create the junction.
                        self.schema.root_mut().unwrap().push(SexpAtom::Node(
                            sexp::sexp!(
                              (junction (at {coord[[0, 0]].to_string().as_str()} {coord[[0, 1]].to_string().as_str()}) (diameter "0") (color "0" "0" "0" "0")
                                (uuid {sexp::uuid!()})
                              )
                            ).root().unwrap().clone(),
                        ))?;
                    },
                    DotPosition::End => {
                        //create the junction.
                        self.schema.root_mut().unwrap().push(SexpAtom::Node(
                            sexp::sexp!(
                              (junction (at {coord[[1, 0]].to_string().as_str()} {coord[[1, 1]].to_string().as_str()}) (diameter "0") (color "0" "0" "0" "0")
                                (uuid {sexp::uuid!()})
                              )
                            ).root().unwrap().clone(),
                        ))?;
                    }
                }
            }
        }

        self.schema
            .root_mut()
            .unwrap()
            .push(SexpAtom::Node(wire.root().unwrap().clone()))?;
        Ok(Some(wire.root().unwrap().clone()))
    }
}

#[cfg(test)]
mod tests {
    use ndarray::arr1;
    use crate::{
        At, Attribute, Direction, Draw, Drawer, Label, Nc, To, PropertyKey,
    };
    use sexp::{SexpValueQuery, SexpWriter, el, SexpProperty};

    #[test]
    fn drawer_label() {
        let mut draw = Draw::new(vec![], None);
        let mut label = Label::new();
        label.add_name(String::from("MyLabel"));
        draw.draw(&At::Pos((10.0, 10.0))).unwrap();
        let sexp = draw.draw(&label).unwrap().unwrap();

        let uuid: String = sexp.value("uuid").unwrap();

        let mut writer: Vec<u8> = Vec::new();
        sexp.write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(format!("(label \"MyLabel\" (at 10 10 0) (effects (font (size 1.27 1.27)) (justify left))\n  (uuid {})\n)\n", uuid), result);
    }
    #[test]
    fn drawer_no_connect() {
        let mut draw = Draw::new(vec![], None);
        let nc = Nc::new();
        draw.draw(&At::Pos((10.0, 10.0))).unwrap();
        let sexp = draw.draw(&nc).unwrap().unwrap();

        let uuid: String = sexp.value("uuid").unwrap();

        let mut writer: Vec<u8> = Vec::new();
        sexp.write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(
            format!("(no_connect (at 10 10)\n  (uuid {})\n)\n", uuid),
            result
        );
    }
    #[test]
    fn wire_to() {
        let mut draw = Draw::new(vec![], None);
        let mut to = To::new();

        draw.draw(&At::Pos((10.0, 10.0))).unwrap();
        to.attributes.push(Attribute::Length(25.4));
        to.attributes.push(Attribute::Direction(Direction::Right));

        let sexp = draw.draw(&to).unwrap().unwrap();

        let uuid: String = sexp.value("uuid").unwrap();

        let mut writer: Vec<u8> = Vec::new();
        sexp.write(&mut writer, 0).unwrap();
        let result = std::str::from_utf8(&writer).unwrap();
        assert_eq!(format!("(wire\n  (pts\n    (xy 10 10)\n    (xy 35.4 10)\n  )\n  (stroke (width 0) (type default))\n  (uuid {})\n)\n", uuid), result);
    }
    #[test]
    fn get_library_symbol() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);
        let lib = draw.load_library("Amplifier_Operational:AD8015").unwrap();

        let result: String = lib.get(0).unwrap();
        assert_eq!("Amplifier_Operational:AD8015", result);
        /* let mut writer: Vec<u8> = Vec::new();
        lib.write(&mut std::io::stdout() /* &mut writer */, 0)
            .unwrap();
        let result = std::str::from_utf8(&writer).unwrap(); */
    }
    #[test]
    fn get_library_symbol_extends() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);
        let lib = draw.load_library("Amplifier_Operational:LM324").unwrap();

        let result: String = lib.get(0).unwrap();
        assert_eq!("Amplifier_Operational:LM324", result);
    }
    #[test]
    fn from_library_symbol() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);
        let lib = draw.load_library("Amplifier_Operational:LM324").unwrap();
        let symbol = super::from_library(
            String::from("Amplifier_Operational:LM324"),
            &lib,
            arr1(&[10.0, 10.0]),
            90.0,
            1,
            String::from("U1"),
            String::from("OPAMP_VALUE"),
        )
        .unwrap();

        let result: String = symbol.value(el::LIB_ID).unwrap();
        assert_eq!("Amplifier_Operational:LM324", result);
        let result: String = symbol.property(el::PROPERTY_REFERENCE).unwrap();
        assert_eq!("U1", result);
        let result: String = symbol.property(el::PROPERTY_VALUE).unwrap();
        assert_eq!("OPAMP_VALUE", result);
    }

    #[test]
    fn next_reference() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);
        assert_eq!("R1", draw.next(String::from("R")));
        assert_eq!("R2", draw.next(String::from("R")));
        assert_eq!("R3", draw.next(String::from("R")));
        assert_eq!("Q1", draw.next(String::from("Q")));
    }

    #[test]
    fn last_reference() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);
        assert_eq!("R1", draw.next(String::from("R")));
        assert_eq!("R2", draw.next(String::from("R")));
        assert_eq!("R2", draw.last(String::from("R")).unwrap());
        assert_eq!("R3", draw.next(String::from("R")));
        assert_eq!("Q1", draw.next(String::from("Q")));
        assert_eq!("R3", draw.last(String::from("R")).unwrap());
        assert_eq!("Q1", draw.last(String::from("Q")).unwrap());
    }

    /* #[test]
    fn element_length() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R0".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        draw.draw(&element).unwrap();

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R90".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(90.0));
        draw.draw(&element).unwrap();

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R180".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(180.0));
        draw.draw(&element).unwrap();

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R270".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(270.0));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [25.4, 34.29]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 41.91], [25.4, 50.8]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[25.4, 38.1]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((50.8, 25.4)));
    } */

    /*
    #[test]
    fn element_length_0() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [25.4, 34.29]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 41.91], [25.4, 50.8]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[25.4, 38.1]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((25.4, 50.8)));
    }
    #[test]
    fn element_length_90() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(90.0));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [16.51, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[8.89, 25.4], [0.0, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[12.7, 25.4]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((0.0, 25.4)));
    }
    #[test]
    fn element_length_180() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(180.0));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [25.4, 16.51]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 8.89], [25.4, 0.0]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[25.4, 12.7]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((25.4, 0.0)));
    }
    #[test]
    fn element_length_270() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Length(25.4));
        element.attributes.push(super::Attribute::Rotate(270.0));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [34.29, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[41.91, 25.4], [50.8, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[38.1, 25.4]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((50.8, 25.4)));
    }
    #[test]
    fn element_tox() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(90.0));
        element
            .attributes
            .push(super::Attribute::Tox(At::Pos((50.8, 25.4))));
        draw.draw(&element).unwrap();

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R2".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(90.0));
        element
            .attributes
            .push(super::Attribute::Tox(At::Pos((0.0, 25.4))));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [34.29, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[41.91, 25.4], [50.8, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[38.1, 25.4]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((0.0, 25.4)));
    }
    #[test]
    fn element_toy() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        // element.attributes.push(super::Attribute::Rotate(90.0));
        element
            .attributes
            .push(super::Attribute::Toy(At::Pos((25.4, 50.8))));
        draw.draw(&element).unwrap();

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();
        let mut element = Symbol::new();
        element.set_reference("R2".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(180.0));
        element
            .attributes
            .push(super::Attribute::Toy(At::Pos((25.4, 0.0))));
        draw.draw(&element).unwrap();

        let mut iter = draw.schema.iter_all();
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [25.4, 34.29]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 41.91], [25.4, 50.8]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[25.4, 38.1]), symbol.at);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((25.4, 0.0)));
    }

    #[test]
    fn elements() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        draw.draw(&At::Pos((25.4, 25.4))).unwrap();

        let mut d1 = Dot::new();
        d1.attributes.push(super::Attribute::Id("d1".to_string()));
        draw.draw(&d1).unwrap();

        let mut w1 = To::new();
        w1.attributes
            .push(super::Attribute::Direction(Direction::Right));
        w1.attributes.push(super::Attribute::Length(5.08));
        draw.draw(&w1).unwrap();

        let mut element = Symbol::new();
        element.set_reference("R1".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(90.0));
        draw.draw(&element).unwrap();

        let mut w1 = To::new();
        w1.attributes
            .push(super::Attribute::Direction(Direction::Right));
        w1.attributes.push(super::Attribute::Length(5.08));
        draw.draw(&w1).unwrap();

        let mut w2 = To::new();
        w2.attributes
            .push(super::Attribute::Direction(Direction::Up));
        w2.attributes.push(super::Attribute::Length(2.0 * 5.08));
        draw.draw(&w2).unwrap();

        let mut element = Symbol::new();
        element.set_reference("R2".to_string());
        element.set_lib_id("Device:R".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(270.0));
        element
            .attributes
            .push(super::Attribute::Tox(At::Dot("d1".to_string())));
        draw.draw(&element).unwrap();

        let mut w3 = To::new();
        w3.attributes
            .push(super::Attribute::Toy(At::Dot("d1".to_string())));
        draw.draw(&w3).unwrap();

        let mut iter = draw.schema.iter_all();
        assert!(iter.next().is_some());
        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 25.4], [30.48, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[34.29, 25.4]), symbol.at);
        } else {
            panic!("not a symbol");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[38.1, 25.4], [43.18, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[43.18, 25.4], [43.18, 15.24]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[43.18, 15.24], [38.1, 15.24]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[30.48, 15.24], [25.4, 15.24]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        if let SchemaElement::Symbol(symbol) = iter.next().unwrap() {
            assert_eq!(arr1(&[34.29, 15.24]), symbol.at);
        } else {
            panic!("not a symbol");
        }

        if let SchemaElement::Wire(wire) = iter.next().unwrap() {
            assert_eq!(arr2(&[[25.4, 15.24], [25.4, 25.4]]), wire.pts);
        } else {
            panic!("not a wire ");
        }

        assert_eq!(draw.pos, At::Pos((25.4, 25.4)));
    }
    #[test]
    fn mounting_hole() {
        let mut draw = Draw::new(vec![String::from("/usr/share/kicad/symbols")], None);

        let mut element = Symbol::new();
        element.set_reference("H1".to_string());
        element.set_lib_id("Mechanical:MountingHole".to_string());
        element
            .properties
            .insert("value".to_string(), "100k".to_string());
        element.attributes.push(super::Attribute::Rotate(270.0));
        element
            .attributes
            .push(super::Attribute::Tox(At::Dot("d1".to_string())));
        draw.draw(&element).unwrap();
    }
    */
    #[test]
    fn property_match() {
        assert_eq!(PropertyKey::Range(String::from("R"), 1, 10), PropertyKey::from(String::from("R[1..10]")));
        assert_eq!(PropertyKey::From(String::from("R"), 3), PropertyKey::from(String::from("R[3..]")));
        assert_eq!(PropertyKey::To(String::from("R"), 4), PropertyKey::from(String::from("R[..4]")));
        assert_eq!(PropertyKey::Exact(String::from("R1")), PropertyKey::from(String::from("R1")));

        assert!(PropertyKey::from(String::from("R[1..10]")).matches(String::from("R5")));
        assert!(PropertyKey::from(String::from("R[..10]")).matches(String::from("R5")));
        assert!(PropertyKey::from(String::from("R[4..]")).matches(String::from("R5")));
        assert!(PropertyKey::from(String::from("R5")).matches(String::from("R5")));

        assert!(!PropertyKey::from(String::from("R[1..10]")).matches(String::from("R11")));
        assert!(!PropertyKey::from(String::from("R[..10]")).matches(String::from("R11")));
        assert!(!PropertyKey::from(String::from("R[4..]")).matches(String::from("R3")));
        assert!(!PropertyKey::from(String::from("R5")).matches(String::from("R2")));
    }
}
