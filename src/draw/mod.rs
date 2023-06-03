/// Draw a schema from code
mod model;

use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

use itertools::Itertools;
use ndarray::{arr1, arr2, Array1};
use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

use crate::{
    error::Error,
    reports::erc,
    sexp::{self, Bounds, PinOrientation, Shape, Transform, MIRROR, TitleBlock, SchemaElement, Property},
    spice::{Circuit, Netlist},
};

pub use model::{At, Attribute, Direction, Dot, DotPosition, LabelPosition, Label, Nc, Properties, Symbol, To};

const LABEL_BORDER: f64 = 2.0;

macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.3}", v).parse::<f64>().unwrap())
    };
}

fn filter_properties(node: &&mut sexp::Property) -> bool {

    if let Some(effects) = &node.effects {
        !effects.hide
    } else {
        true
    }
}

fn sort_properties(a: &&mut sexp::Property, b: &&mut sexp::Property) -> std::cmp::Ordering {
    a.id.cmp(&b.id)
}

///Enum for the set property key
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
enum PropertyKey {
    Exact(String),
    Range(String, usize, usize),
    From(String, usize),
    To(String, usize),
}

lazy_static! {
    pub static ref RE_FROM_TO: regex::Regex = Regex::new(r"([A-Z]+)\[([\d]+)\.\.([\d]+)").unwrap();
    pub static ref RE_FROM: regex::Regex = Regex::new(r"([A-Z]+)\[([\d]+)\.\.\]").unwrap();
    pub static ref RE_TO: regex::Regex = Regex::new(r"([A-Z]+)\[\.\.([\d]+)\]").unwrap();
    pub static ref RE_KEY: regex::Regex = Regex::new(r"([A-Z]+)([\d]+)").unwrap();
}

#[allow(dead_code)]
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
    pub schema: sexp::Schema,
    libs: sexp::Library,
    positions: HashMap<String, Array1<f64>>,
    references: HashMap<String, u32>,
}

#[allow(dead_code)]
impl Draw {
    /// create a new Draw Object.
    pub fn new(library_path: Vec<String>, kwargs: Option<HashMap<String, String>>) -> Self {
        let mut schema = sexp::Schema::new();
        schema.new_page();
        if let Some(kwargs) = kwargs {
            let mut title_block = TitleBlock::new();
            if let Some(title) = kwargs.get("title") {
                title_block.title = title.to_string();
            }
            if let Some(date) = kwargs.get("date") {
                title_block.date = date.to_string();
            }
            if let Some(rev) = kwargs.get("rev") {
                title_block.rev = rev.to_string();
            }
            if let Some(company) = kwargs.get("company") {
                title_block.company = company.to_string();
            }
            for i in 1..5 {
                if let Some(comment) = kwargs.get(&format!("comment{}", i)) {
                    title_block.comment.push((i, comment.to_string()));
                }
            }
            schema.pages.get_mut(0).unwrap().title_block = Some(title_block);
        }
        Self {
            pos: At::Pos((25.4, 25.4)),
            schema,
            libs: sexp::Library::new(library_path),
            positions: HashMap::new(),
            references: HashMap::new(),
        }
    }

    ///Build the netlist and get the circuit
    pub fn circuit(&self, pathlist: Vec<String>) -> Result<Circuit, Error> {
        let netlist = Netlist::from(&self.schema)?;
        let mut circuit = Circuit::new(String::from("draw circuit"), pathlist);
        netlist.circuit(&mut circuit).unwrap();
        Ok(circuit)
    }

    ///Set the position.
    pub fn set(&mut self, pos: At) {
        self.pos = pos;
    }

    ///Get the next reference for a letter.
    pub fn next(&mut self, key: String) -> String {
        let entry = self
            .references
            .entry(key.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        format!("{}{}", key, entry)
    }

    ///Set the next reference for a letter.
    pub fn counter(&mut self, key: String, count: u32) {
        self.references.insert(key, count-1);
    }

    ///Get the next reference for a letter.
    pub fn last(&mut self, key: String) -> Result<String, Error> {
        if let Some(entry) = self.references.get(&key) {
            Ok(format!("{}{}", &key, entry))
        } else {
            Err(Error::NotFound(String::from("Reference"), key))
        }
    }

    ///Set a property value for symbols.
    pub fn property(&mut self, regex: String, key: String, value: String, id: u32) -> Result<(), Error> {
        let matcher = PropertyKey::from(regex);
        for s in self.schema.iter_all_mut() {
            if let SchemaElement::Symbol(symbol) = s {
                let reference = symbol.get_property("Reference").unwrap();
                if matcher.matches(reference) {
                    if symbol.has_property(&key) {
                        symbol.set_property(&key, &value);
                    } else {
                        symbol.property.push(Property::new(key.clone(), value.clone(), id, arr1(&[0.0, 0.0]), 0.0, None));
                    }
                }
            }
        }
        Ok(())
    }

    ///(Get the ERC report as String.
    pub fn erc(&self) -> String {
        let items = erc::erc(&self.schema).unwrap();
        let mut res = Vec::<u8>::new();
        for i in items {
            match i {
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
            }
        }
        std::str::from_utf8(&res).unwrap().to_string()
    }

    //write the schema to a Writer
    pub fn write(&self, file: &str) {
        self.schema.write(file).unwrap();
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

    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<sexp::LibrarySymbol, Error> {
        if let Some(lib) = self.schema.get_library(name) {
            Ok(lib.clone())
        } else {
            let mut lib = self.libs.get(name)?;
            if !lib.extends.is_empty() {
                let library = &name[0..name.find(':').unwrap()];
                let mut extend_symbol = self
                    .libs
                    .get(format!("{}:{}", library, lib.extends.as_str()).as_str())?;

                //copy properties
                let mut properties = Vec::new();
                for prop in &extend_symbol.property {
                    let mut found = false;
                    for lib_property in &lib.property {
                        if lib_property.key == prop.key {
                            found = true;
                            properties.push(lib_property.clone());
                            break;
                        }
                    }
                    if !found {
                        properties.push(prop.clone());
                    }
                }
                extend_symbol.property = properties;
                
                for subsymbol in &mut extend_symbol.symbols {
                    let number = &subsymbol.lib_id
                        [subsymbol.lib_id.find('_').unwrap()..subsymbol.lib_id.len()];
                    subsymbol.lib_id = format!("{}{}", lib.lib_id, number);
                }
                extend_symbol.lib_id = name.to_string();
                lib = extend_symbol;
            }
            lib.lib_id = name.to_string();
            self.schema.page_mut(0).unwrap().libraries.push(lib.clone());
            Ok(lib)
        }
    }

    fn place_property(&mut self, symbol: &mut sexp::Symbol, label: Option<LabelPosition>) -> Result<(), Error> {
        let vis_field = symbol
            .property
            .iter()
            .filter_map(|node| {
                if let Some(effects) = &node.effects {
                    if !effects.hide {
                        Option::from(node)
                    } else {
                        None
                    }
                } else {
                    Option::from(node)
                }
            })
            .count();

        let lib = self.get_library(&symbol.lib_id).unwrap();

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
        let positions = self.pin_positions(symbol, &lib);
        let mut offset = 0.0;
        let pins = lib.pins(symbol.unit)?.len();
        if let Some(position) = label {
            symbol
                .property
                .iter_mut()
                .filter(filter_properties)
                .sorted_by(sort_properties)
                .for_each(|p| {
                    if position == LabelPosition::North {
                        let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[0, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        } else {
                            _size[[1, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        };
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], top_pos - offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if position == LabelPosition::East {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("left"));
                        } else {
                            let mut effects = sexp::Effects::new();
                            effects.justify.push(String::from("left"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if position == LabelPosition::West {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("right"));
                        } else {
                            let mut effects = sexp::Effects::new();
                            effects.justify.push(String::from("right"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[0, 0]] - LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if position == LabelPosition::South {
                        let bottom_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[1, 1]] + LABEL_BORDER
                        } else {
                            _size[[0, 1]] + LABEL_BORDER
                        };

                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], bottom_pos + offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset += LABEL_BORDER;
                    }
                });
            Ok(())
            
        } else if pins == 1 {
            symbol
                .property
                .iter_mut()
                .filter(filter_properties)
                .sorted_by(sort_properties)
                .for_each(|p| {
                    if positions.contains(&PinOrientation::Left) {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push("left".to_string());
                        }
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER, symbol.at[1]]);
                        p.angle = 0.0 - symbol.angle;
                    } else if positions.contains(&PinOrientation::Up) {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], _size[[1, 1]] + LABEL_BORDER]);
                        p.angle = 0.0 - symbol.angle;
                    } else if positions.contains(&PinOrientation::Right) {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push("right".to_string());
                        }
                        p.at = arr1(&[_size[[0, 0]] - LABEL_BORDER, symbol.at[1]]);
                        p.angle = 0.0 - symbol.angle;
                    } else if positions.contains(&PinOrientation::Down) {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], _size[[0, 1]] - LABEL_BORDER]);
                        p.angle = 0.0 - symbol.angle;
                    }
                });
            Ok(())
        } else {
            symbol
                .property
                .iter_mut()
                .filter(filter_properties)
                .sorted_by(sort_properties)
                .for_each(|p| {
                    if !positions.contains(&PinOrientation::Up) {
                        let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[0, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        } else {
                            _size[[1, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
                        };
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], top_pos - offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if !positions.contains(&PinOrientation::Right) {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("left"));
                        } else {
                            let mut effects = sexp::Effects::new();
                            effects.justify.push(String::from("left"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if !positions.contains(&PinOrientation::Left) {
                        let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                            - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("right"));
                        } else {
                            let mut effects = sexp::Effects::new();
                            effects.justify.push(String::from("right"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[0, 0]] - LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    } else if !positions.contains(&PinOrientation::Down) {
                        let bottom_pos = if _size[[0, 1]] < _size[[1, 1]] {
                            _size[[1, 1]] + LABEL_BORDER
                        } else {
                            _size[[0, 1]] + LABEL_BORDER
                        };

                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], bottom_pos + offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset += LABEL_BORDER;
                    }
                });
            Ok(())
        }
    }

    /// get the pin position
    /// returns an array containing the PinOrientation
    fn pin_positions(
        &self,
        symbol: &sexp::Symbol,
        lib: &sexp::LibrarySymbol,
    ) -> HashSet<PinOrientation> {
        let mut found = HashSet::new();
        for pin in lib.pins(symbol.unit).unwrap() {
            found.insert(PinOrientation::from(symbol, pin));
        }
        found
    }

    fn pin_pos(&self, reference: String, number: String) -> Result<Array1<f64>, Error> {
        let Some(symbol) = self.schema.get_symbol(reference.as_str(), 1) else {
            return Err(Error::NotFound(reference, number));
        };
        let Some(library) = self.schema.get_library(symbol.lib_id.as_str()) else {
            return Err(Error::LibraryNotFound(symbol.lib_id.to_string()));
        };
        for subsymbol in &library.symbols {
            for pin in &subsymbol.pin {
                if pin.number.0 == number {
                    let real_symbol = self
                        .schema
                        .get_symbol(reference.as_str(), subsymbol.unit)
                        .unwrap();
                    return Ok(Shape::transform(real_symbol, &pin.at));
                }
            }
        }
        Err(Error::PinNotFound(reference, number))
    }
}

pub trait Drawer<T> {
    fn draw(&mut self, item: &T) -> Result<Option<SchemaElement>, Error>;
}

impl Drawer<Label> for Draw {
    fn draw(&mut self, item: &Label) -> Result<Option<SchemaElement>, Error> {
        let angle: f64 = item.angle();
        let pos = self.at(&self.pos)?;
        let mut label = sexp::Label::new(
            round!(pos),
            angle,
            &item
                .get_name()
                .ok_or_else(|| Error::Name(String::from("Label")))?,
            sexp::uuid!(),
        );
        let mut effects = sexp::Effects::new();
        let align = &["left", "left", "right", "right"];
        let pos = (angle / 90.0) as usize;
        effects.justify = vec![align[pos].to_string()];
        label.effects = effects;
        self.schema.push(0, sexp::SchemaElement::Label(label.clone()))?;
        Ok(Some(SchemaElement::Label(label)))
    }
}

impl Drawer<Nc> for Draw {
    fn draw(&mut self, _: &Nc) -> Result<Option<SchemaElement>, Error> {
        let pos = self.at(&self.pos)?;
        let label = sexp::NoConnect::new(
            round!(pos),
            sexp::uuid!(),
        );
        self.schema.push(0, sexp::SchemaElement::NoConnect(label.clone()))?;
        Ok(Some(SchemaElement::NoConnect(label)))
    }
}

impl Drawer<Symbol> for Draw {
    fn draw(&mut self, item: &Symbol) -> Result<Option<SchemaElement>, Error> {
        let Some(reference) = item.get_reference() else {
            return Err(Error::Name(String::from("Symbol::Reference")));
        };
        let Some(lib_id) = item.get_lib_id() else {
            return Err(Error::Name(String::from("Symbol::LibId")));
        };

        let unit = if let Some(unit) = item.get_property("unit") {
            unit.parse::<u32>()?
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
        let Ok(lib_symbol) = self.get_library(lib_id.as_str()) else {
            return Err(Error::LibraryNotFound(lib_id));
        };

        //calculate the position.
        let pos = self.at(&self.pos)?;

        let mut symbol = if let Ok(sym_pin) = lib_symbol.get_pin(pin.to_string()) {
            // transform pin pos
            let theta = -angle.to_radians();
            let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
            let mut verts: Array1<f64> = sym_pin.at.dot(&rot);
            verts = if let Some(mirror) = item.mirror() {
                verts.dot(MIRROR.get(&mirror).unwrap())
            } else {
                verts.dot(MIRROR.get(&String::new()).unwrap())
            };
            verts = arr1(&[pos[0], pos[1]]) - &verts;
            verts = round!(verts);

            let mut symbol = sexp::Symbol::from_library(
                &lib_symbol,
                round!(verts.clone()),
                angle,
                unit,
                reference,
                value,
            );

            if item.length().is_some() || item.tox().is_some() || item.toy().is_some() {
                let pins = lib_symbol.pins(unit)?;
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
                    let verts0: Array1<f64> = round!(lib_symbol.get_pin(pin.to_string()).unwrap().at.dot(&rot));
                    let verts1: Array1<f64> = round!(lib_symbol.get_pin(end_pin).unwrap().at.dot(&rot));
                    let sym_len =
                        ((verts1[0] - verts0[0]).powf(2.0) + (verts1[1] - verts0[1]).powf(2.0)).sqrt();

                    if length > sym_len {
                        let wire_len = (length - sym_len) / 2.0;
                        self.schema.push(
                            0,
                            sexp::SchemaElement::Wire(sexp::Wire::new(
                                round!(arr2(&[
                                    [pos[0], pos[1]],
                                    [pos[0] + wire_len * rad.cos(), pos[1] + wire_len * rad.sin()]
                                ])),
                                sexp::Stroke::new(),
                                sexp::uuid!(),
                            )),
                        )?;
                        self.schema.push(
                            0,
                            sexp::SchemaElement::Wire(sexp::Wire::new(
                                round!(arr2(&[
                                    [
                                        pos[0] + (wire_len + sym_len) * rad.cos(),
                                        pos[1] + (wire_len + sym_len) * rad.sin()
                                    ],
                                    [pos[0] + length * rad.cos(), pos[1] + length * rad.sin()]
                                ])),
                                sexp::Stroke::new(),
                                sexp::uuid!(),
                            )),
                        )?;
                        symbol.at = round!(arr1(&[
                            pos[0] + (wire_len + sym_len / 2.0) * rad.cos(),
                            pos[1] + (wire_len + sym_len / 2.0) * rad.sin()
                        ]));
                        let next_pos = round!(arr1(&[
                            pos[0] + length * rad.cos(),
                            pos[1] + length * rad.sin()
                        ]));
                        self.pos = At::Pos((next_pos[0], next_pos[1]));
                    }
                } else {
                    panic!("only allow with 2 pins");
                }
            } else if let Ok(endpin) = lib_symbol.get_pin(end_pin) {
                let pts = Shape::transform(&symbol, &endpin.at);
                self.pos = At::Pos((pts[0], pts[1]));
            }
            symbol
        } else {
            sexp::Symbol::from_library(
                &lib_symbol,
                round!(arr1(&[pos[0], pos[1]])),
                angle,
                unit,
                reference,
                value,
            )
        };

        //create the properties
        if let Some(mirror) = item.mirror() {
            symbol.mirror = Some(mirror);
        }
        symbol.on_schema = if let Some(on_schema) = item.properties.get("on_schema") {
            on_schema == "yes"
        } else {
            true
        };
        symbol.on_board = if let Some(on_board) = item.properties.get("on_board") {
            on_board == "yes"
        } else {
            true
        };

        // add the extra properties
        for (k, v) in &item.properties {
            if k != "on_schema" && k != "on_board" && k != "value" && k != "unit" {
                symbol.property.push(sexp::Property::new(
                    k.to_string(),
                    v.to_string(),
                    0,
                    round!(symbol.at.clone()),
                    0.0,
                    Some(sexp::Effects::hidden()),
                ));
            }
        }

        if let Ok(_) = lib_symbol.get_pin(pin.to_string()) {
            self.place_property(&mut symbol, item.label()).unwrap();
        }
        self.schema.push(0, sexp::SchemaElement::Symbol(symbol.clone()))?;

        Ok(Some(SchemaElement::Symbol(symbol)))
    }
}

impl Drawer<Dot> for Draw {
    fn draw(&mut self, item: &Dot) -> Result<Option<SchemaElement>, Error> {
        let pos = self.at(&self.pos)?;

        //save the position if an id is given.
        let id: Option<String> = item.id();
        if let Some(id) = id {
            self.positions.insert(id, pos.clone());
        }
        //create the junction.
        let junction = sexp::Junction::new(pos, sexp::uuid!());
        self.schema.push(
            0,
            SchemaElement::Junction(junction.clone())
        )?;
        Ok(Some(SchemaElement::Junction(junction)))
    }
}

impl Drawer<At> for Draw {
    fn draw(&mut self, item: &At) -> Result<Option<SchemaElement>, Error> {
        self.pos = item.clone();
        Ok(None)
    }
}

impl Drawer<To> for Draw {
    fn draw(&mut self, item: &To) -> Result<Option<SchemaElement>, Error> {
        let pos = self.at(&self.pos)?;
        let wire = if let Some(length) = item.length() {
            match item.direction() {
                Direction::Left => sexp::Wire::new(
                    round!(arr2(&[[pos[0], pos[1]], [pos[0] - length, pos[1]]])),
                    sexp::Stroke::new(),
                    sexp::uuid!(),
                ),
                Direction::Up => sexp::Wire::new(
                    round!(arr2(&[[pos[0], pos[1]], [pos[0], pos[1] - length]])),
                    sexp::Stroke::new(),
                    sexp::uuid!(),
                ),
                Direction::Down => sexp::Wire::new(
                    round!(arr2(&[[pos[0], pos[1]], [pos[0], pos[1] + length]])),
                    sexp::Stroke::new(),
                    sexp::uuid!(),
                ),
                _ => sexp::Wire::new(
                    round!(arr2(&[[pos[0], pos[1]], [pos[0] + length, pos[1]]])),
                    sexp::Stroke::new(),
                    sexp::uuid!(),
                ),
            }
        } else if let Some(tox) = item.tox() {
            let end_pos = self.at(tox)?;
            sexp::Wire::new(
                arr2(&[[pos[0], pos[1]], [end_pos[0], pos[1]]]),
                sexp::Stroke::new(),
                sexp::uuid!(),
            )
        } else if let Some(toy) = item.toy() {
            let end_pos = self.at(toy)?;
            sexp::Wire::new(
                arr2(&[[pos[0], pos[1]], [pos[0], end_pos[1]]]),
                sexp::Stroke::new(),
                sexp::uuid!(),
            )
        } else {
            sexp::Wire::new(
                round!(arr2(&[[pos[0], pos[1]], [pos[0] + 2.54, pos[1]]])),
                sexp::Stroke::new(),
                sexp::uuid!(),
            )
        };

        self.pos = At::Pos((wire.pts[[1, 0]], wire.pts[[1, 1]]));
        if let Some(dot) = item.dot() {
            for d in dot {
                match d {
                    DotPosition::Start => {
                        //create the junction.
                        self.schema.push(
                            0,
                            sexp::SchemaElement::Junction(sexp::Junction::new(arr1(&[wire.pts[[0, 0]], wire.pts[[0, 1]]]), sexp::uuid!())),
                        )?;
                    },
                    DotPosition::End => {
                        //create the junction.
                        self.schema.push(
                            0,
                            sexp::SchemaElement::Junction(sexp::Junction::new(arr1(&[wire.pts[[1, 0]], wire.pts[[1, 1]]]), sexp::uuid!())),
                        )?;
                    }
                } 
            }
        }
        self.schema.push(0, sexp::SchemaElement::Wire(wire.clone()))?;
        Ok(Some(SchemaElement::Wire(wire)))
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{arr1, arr2};

    use crate::{
        draw::{At, Direction, Dot, Draw, Drawer, Symbol, To, PropertyKey},
        sexp::SchemaElement,
    };

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

    #[test]
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
    }

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
