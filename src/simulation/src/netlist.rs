use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

use indexmap::IndexMap;
use ndarray::{arr1, s, Array1};

use crate::{
    circuit::Circuit,
    error::Error,
};
use sexp::{
    el, utils, Sexp, SexpProperty, SexpTree, SexpValueQuery, SexpValuesQuery,
    Shape, Transform,
};

///return the pin name, pin and unit number from a libary symbol.
fn pin_names(symbol: &Sexp) -> Result<HashMap<String, (Sexp, usize)>, Error> {
    let mut pins = HashMap::new();
    for symbol in symbol.query(el::SYMBOL) {
        let unit = utils::unit_number(symbol.get(0).unwrap());
        //search the pins
        for pin in symbol.query(el::PIN) {
            let number = pin.query(el::PIN_NUMBER).next().unwrap();
            pins.insert(number.get(0).unwrap(), (pin.clone(), unit));
        }
    }
    Ok(pins)
}

///A point in the schema.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
}
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        (self.x == other.x) && (self.y == other.y)
    }
}
impl Eq for Point {}
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}
impl std::convert::From<Array1<f64>> for Point {
    fn from(pts: Array1<f64>) -> Self {
        Point::new(pts[0], pts[1])
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.x, self.y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodePositions<'a> {
    Pin(Point, &'a Sexp, &'a Sexp),
    Wire(Point, Point),
    Label(Point, &'a Sexp),
    GlobalLabel(Point, &'a Sexp),
    NoConnect(Point),
    Junction(Point),
}

#[derive(Clone, Debug)]
pub struct Node {
    identifier: Option<String>,
    points: Vec<Point>,
    // pins: Vec<Pin>,
}

/// create a new node with values.
impl Node {
    pub fn from(identifier: Option<String>, points: Vec<Point>) -> Self {
        Self { identifier, points }
    }
}

/// The Netlist struct
///
/// Create a netlist as a graph.
///
pub struct Netlist<'a> {
    schema: &'a SexpTree,
    // elements: Vec<&'a SchemaElement>,
    // used_elements: Vec<&'a Wire>,
    symbols: IndexMap<String, Vec<&'a Sexp>>,
    pub nodes: Vec<Node>, //TODO only public for tests
    node_positions: Vec<(Point, NodePositions<'a>)>,
}

impl<'a> Netlist<'a> {
    pub fn from(schema: &'a SexpTree) -> Result<Self, Error> {
        let symbols = Self::get_symbols(schema.root().unwrap())?;
        let node_positions = Netlist::positions(schema.root().unwrap())?;
        let mut netlist = Self {
            schema,
            symbols,
            nodes: Vec::new(),
            node_positions,
        };

        let used_vec = &mut Vec::new();
        let used = &Rc::new(RefCell::new(used_vec));
        let mut used_pins: Vec<&NodePositions> = Vec::new();
        for pos in &netlist.node_positions {
            if let NodePositions::Pin(point, p, s) = &pos.1 {
                if !used_pins.contains(&&pos.1) {
                    used_pins.push(&pos.1);
                    used.borrow_mut().clear();
                    used.borrow_mut().push(&pos.1);
                    if let Some(nodes) = Netlist::next_node(&pos.0, &netlist.node_positions, used) {
                        let mut identifier: Option<String> = None;
                        let mut points: Vec<Point> = vec![point.to_owned()];
                        let mut pins: Vec<Sexp> = vec![p.to_owned().clone()];
                        let lib_id: String = s.value(el::LIB_ID).unwrap();
                        if lib_id.starts_with("power:") {
                            identifier = s.property(el::PROPERTY_VALUE);
                        }
                        for node in &nodes {
                            match node {
                                NodePositions::Pin(point, p, s) => {
                                    let lib_id: String = s.value(el::LIB_ID).unwrap();
                                    if lib_id.starts_with("power:") {
                                        identifier = s.property(el::PROPERTY_VALUE);
                                    }
                                    pins.push(p.to_owned().clone());
                                    points.push(*point);
                                    used_pins.push(node);
                                }
                                NodePositions::Junction(point) => {
                                    points.push(*point);
                                    used_pins.push(&pos.1);
                                }
                                NodePositions::Wire(_, p2) => {
                                    points.push(*point);
                                    points.push(*p2);
                                    used_pins.push(node);
                                }
                                NodePositions::NoConnect(point) => {
                                    points.push(*point);
                                    used_pins.push(node);
                                    identifier = Some(String::from("NC"));
                                }
                                NodePositions::Label(point, l) => {
                                    identifier = Some(l.get(0).unwrap());
                                    points.push(*point);
                                    used_pins.push(node);
                                }
                                NodePositions::GlobalLabel(point, l) => {
                                    identifier = Some(l.get(0).unwrap());
                                    points.push(*point);
                                    used_pins.push(node);
                                }
                            }
                        }
                        netlist.nodes.push(Node::from(identifier, points));
                    }
                }
            }
        }

        let mut name = 1;
        for n in &mut netlist.nodes {
            if n.identifier.is_none() {
                n.identifier = Some(name.to_string());
                name += 1;
            }
        }
        Ok(netlist)
    }

    //Get the symbols, with units collected.
    pub fn get_symbols(schema: &'a Sexp) -> Result<IndexMap<String, Vec<&Sexp>>, Error> {
        let mut symbols: IndexMap<String, Vec<&Sexp>> = IndexMap::new();
        for symbol in schema.query(el::SYMBOL) {
            if let Some(reference) = symbol.property(el::PROPERTY_REFERENCE) {
                symbols.entry(reference).or_default().push(symbol);
            } else {
                let text: String = symbol.value(el::LIB_ID).unwrap();
                return Err(Error::PropertyNotFound(text));
            }
        }
        Ok(symbols)
    }

    ///get all the positions of the elements.
    pub fn positions(schema: &'a Sexp) -> Result<Vec<(Point, NodePositions)>, Error> {
        let mut positions: Vec<(Point, NodePositions)> = Vec::new();
        //for node in schema.nodes {
        for symbol in schema.children() {
            if &symbol.name == el::SYMBOL {
                let lib_id: String = symbol.value(el::LIB_ID).unwrap();
                if lib_id.starts_with("Mechanical:") {
                    continue;
                }
                let Some(lib_symbol) = utils::get_library(schema, &lib_id) else {
                    return Err(Error::LibraryNotFound(lib_id));
                };
                for pin in utils::pins(lib_symbol, symbol.value(el::SYMBOL_UNIT).unwrap())
                    .unwrap()
                    .iter()
                {
                    let pin_pos = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin, el::AT)
                        .unwrap()
                        .slice_move(s![0..2]);
                    let point: Point = Shape::transform(symbol, &pin_pos).into();
                    positions.push((point, NodePositions::Pin(point, pin, symbol)));
                }
            } else if &symbol.name == el::NO_CONNECT {
                let at: Array1<f64> = symbol.value(el::AT).unwrap();
                positions.push((
                    arr1(&[at[0], at[1]]).into(),
                    NodePositions::NoConnect(Point::new(at[0], at[1])),
                ));
            } else if &symbol.name == el::JUNCTION {
                let at: Array1<f64> = symbol.value(el::AT).unwrap();
                positions.push((
                    arr1(&[at[0], at[1]]).into(),
                    NodePositions::Junction(Point::new(at[0], at[1])),
                ));
            } else if &symbol.name == el::WIRE {
                let pts = symbol.query(el::PTS).next().unwrap();
                let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
                let xy1: Array1<f64> = xy.get(0).unwrap().values();
                let xy2: Array1<f64> = xy.get(1).unwrap().values();
                positions.push((
                    Point::new(xy1[0], xy1[1]),
                    NodePositions::Wire(Point::new(xy1[0], xy1[1]), Point::new(xy2[0], xy2[1])),
                ));
            } else if &symbol.name == el::LABEL {
                let at: Array1<f64> = symbol.value(el::AT).unwrap();
                positions.push((
                    at.clone().into(),
                    NodePositions::Label(Point::new(at[0], at[1]), symbol),
                ));
            } else if &symbol.name == el::GLOBAL_LABEL {
                let at: Array1<f64> = symbol.value(el::AT).unwrap();
                positions.push((
                    at.clone().into(),
                    NodePositions::GlobalLabel(Point::new(at[0], at[1]), symbol),
                ));
            }
        }
        Ok(positions)
    }

    ///Get the node name for the Point.
    pub fn node_name(&self, point: &Point) -> Option<String> {
        for n in &self.nodes {
            if n.points.contains(point) {
                return n.identifier.clone();
            }
        }
        None
    }

    ///Get the connected endpoints to this elements.
    pub fn next_node(
        pos: &'a Point,
        elements: &'a Vec<(Point, NodePositions)>,
        used: &Rc<RefCell<&'a mut Vec<&'a NodePositions<'a>>>>,
    ) -> Option<Vec<&'a NodePositions<'a>>> {
        for (p, e) in elements {
            if !used.borrow().contains(&e) {
                match e {
                    NodePositions::Label(_, _) => {
                        if p == pos {
                            used.borrow_mut().push(e);
                            let mut found_nodes: Vec<&'a NodePositions> = vec![e];
                            loop {
                                if let Some(nodes) = &Self::next_node(p, elements, used) {
                                    found_nodes.extend(nodes);
                                    used.borrow_mut().extend(nodes);
                                } else {
                                    return Some(found_nodes);
                                }
                            }
                        }
                    }
                    NodePositions::GlobalLabel(..) => {
                        if p == pos {
                            return Some(vec![e]);
                        }
                    }
                    NodePositions::Junction(..) => {
                        if p == pos {
                            used.borrow_mut().push(e);
                            let mut found_nodes: Vec<&'a NodePositions> = Vec::new();
                            loop {
                                if let Some(nodes) = &Self::next_node(p, elements, used) {
                                    found_nodes.extend(nodes);
                                    used.borrow_mut().extend(nodes);
                                } else {
                                    return Some(found_nodes);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        for (p, e) in elements {
            if !used.borrow().contains(&e) {
                match e {
                    NodePositions::Pin(_point, _pin, _symbol) => {
                        if p == pos {
                            return Some(vec![e]);
                        }
                    }
                    NodePositions::Wire(_, wire) => {
                        let next = if p == pos {
                            used.borrow_mut().push(e);
                            Self::next_node(wire, elements, used)
                        } else if wire == pos {
                            used.borrow_mut().push(e);
                            Self::next_node(p, elements, used)
                        } else {
                            None
                        };
                        if next.is_some() {
                            return next;
                        }
                    }
                    NodePositions::NoConnect(..) => {
                        if p == pos {
                            return Some(vec![e]);
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn circuit(&self, circuit: &mut Circuit) -> Result<(), Error> {
        //Create a spice entry for each referenca
        for (reference, symbols) in &self.symbols {
            let lib_id: String = symbols.first().unwrap().value(el::LIB_ID).unwrap();
            //but not for the power symbols
            if lib_id.starts_with("power:") {
                continue;
            }

            let first_symbol = &symbols.first().unwrap();

            //skip symbol when Netlist_Enabled is 'N'
            let netlist_enabled: Option<String> = first_symbol.property("Spice_Netlist_Enabled"); //TODO differenet
                                                                                                  //name in new
                                                                                                  //KiCAD verison
            if let Some(enabled) = netlist_enabled {
                if enabled == "N" {
                    continue;
                }
            }

            //create the pin order
            let lib_symbols = self
                .schema
                .root()
                .unwrap()
                .query(el::LIB_SYMBOLS)
                .next()
                .unwrap();
            let lib = lib_symbols
                .query(el::SYMBOL)
                .filter(|s| {
                    let name: String = s.get(0).unwrap();
                    name == lib_id
                })
                .next()
                .unwrap();
            let my_pins = pin_names(lib).unwrap();
            let mut pin_sequence: Vec<String> = my_pins.keys().map(|s| s.to_string()).collect();
            pin_sequence.sort_by_key(|x| x.parse::<i32>().unwrap()); //TODO could be string

            //when Node_Sequence is defined, use it
            let netlist_sequence: Option<String> = first_symbol.property("Spice_Node_Sequence"); //TODO
            if let Some(sequence) = netlist_sequence {
                pin_sequence.clear();
                let splits: Vec<&str> = sequence.split(' ').collect();
                for s in splits {
                    pin_sequence.push(s.to_string());
                }
            }

            let mut nodes = Vec::new();
            for n in pin_sequence {
                let pin = my_pins.get(&n).unwrap();
                for symbol in symbols {
                    let unit: usize = symbol.value(el::SYMBOL_UNIT).unwrap();
                    if unit == pin.1 {
                        let at = pin.0.query(el::AT).next().unwrap(); //TODO better at
                        let x: f64 = at.get(0).unwrap();
                        let y: f64 = at.get(1).unwrap();
                        let pts = Shape::transform(*symbol, &arr1(&[x, y]));
                        let p0 = Point::new(pts[0], pts[1]);
                        if let Some(nn) = self.node_name(&p0) {
                            nodes.push(nn);
                        } else {
                            nodes.push(String::from("NF"));
                        }
                    }
                }
            }

            //write the spice netlist item
            let spice_primitive: Option<String> = first_symbol.property("Spice_Primitive"); //TODO
            let spice_model = first_symbol.property("Spice_Model");
            let spice_value = first_symbol.property("Value");
            if let Some(primitive) = spice_primitive {
                if primitive == "X" {
                    circuit.circuit(reference.to_string(), nodes, spice_model.unwrap())?;
                } else if primitive == "Q" {
                    circuit.bjt(
                        reference.to_string(),
                        nodes[0].clone(),
                        nodes[1].clone(),
                        nodes[2].clone(),
                        spice_model.unwrap(),
                    );
                } else if primitive == "J" {
                    circuit.jfet(
                        reference.to_string(),
                        nodes[0].clone(),
                        nodes[1].clone(),
                        nodes[2].clone(),
                        spice_model.unwrap(),
                    );
                } else if primitive == "D" {
                    circuit.diode(
                        reference.to_string(),
                        nodes[0].clone(),
                        nodes[1].clone(),
                        spice_model.unwrap(),
                    );
                } else {
                    println!(
                        "Other node with 'X' -> {}{} - - {}",
                        primitive,
                        reference,
                        spice_value.unwrap()
                    );
                }
            } else if reference.starts_with('R') {
                circuit.resistor(
                    reference.clone(),
                    nodes[0].clone(),
                    nodes[1].clone(),
                    spice_value.unwrap(),
                );
            } else if reference.starts_with('C') {
                circuit.capacitor(
                    reference.clone(),
                    nodes[0].clone(),
                    nodes[1].clone(),
                    spice_value.unwrap(),
                );
            // } else if std::env::var("ELEKTRON_DEBUG").is_ok() {
            } else {
                println!(
                    "Unkknwon Reference: {} ({:?}) {}",
                    reference,
                    nodes,
                    spice_value.unwrap()
                );
            }
        }

        Ok(())
    }
}
