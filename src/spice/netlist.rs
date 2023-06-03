use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

use indexmap::IndexMap;
use ndarray::Array1;

use crate::error::Error;
use crate::sexp::{GlobalLabel, Label, Pin, Schema, SchemaElement, Shape, Symbol, Transform};

use super::Circuit;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: f64,
    y: f64,
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
    Pin(Point, &'a Pin, &'a Symbol),
    Wire(Point, Point),
    Label(Point, &'a Label),
    GlobalLabel(Point, &'a GlobalLabel),
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
    schema: &'a Schema,
    // elements: Vec<&'a SchemaElement>,
    // used_elements: Vec<&'a Wire>,
    symbols: IndexMap<String, Vec<&'a Symbol>>,
    nodes: Vec<Node>,
    node_positions: Vec<(Point, NodePositions<'a>)>,
}

impl<'a> Netlist<'a> {
    pub fn from(schema: &'a Schema) -> Result<Self, Error> {
        let symbols = Netlist::get_symbols(schema)?;
        let node_positions = Netlist::positions(schema)?;
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
                        let mut pins: Vec<Pin> = vec![p.to_owned().clone()];
                        if s.lib_id.starts_with("power:") {
                            identifier = s.get_property("Value");
                        }
                        for node in &nodes {
                            match node {
                                NodePositions::Pin(point, p, s) => {
                                    if s.lib_id.starts_with("power:") {
                                        identifier = s.get_property("Value");
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
                                    identifier = Some(l.text.clone());
                                    points.push(*point);
                                    used_pins.push(node);
                                }
                                NodePositions::GlobalLabel(point, l) => {
                                    identifier = Some(l.text.clone());
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
    fn get_symbols(schema: &'a Schema) -> Result<IndexMap<String, Vec<&Symbol>>, Error> {
        let mut symbols: IndexMap<String, Vec<&Symbol>> = IndexMap::new();
        for node in schema.iter_all() {
            if let SchemaElement::Symbol(symbol) = node {
                if let Some(reference) = symbol.get_property("Reference") {
                    symbols.entry(reference).or_default().push(symbol);
                } else {
                    return Err(Error::PropertyNotFound(
                        symbol.lib_id.to_string(),
                        //TODO String::from("Reference"),
                    ));
                }
            }
        }
        Ok(symbols)
    }

    ///get all the positions of the elements.
    fn positions(schema: &'a Schema) -> Result<Vec<(Point, NodePositions)>, Error> {
        let mut positions: Vec<(Point, NodePositions)> = Vec::new();
        for node in schema.iter_all() {
            match node {
                SchemaElement::Symbol(symbol) => {
                    if symbol.lib_id.starts_with("Mechanical:") {
                        continue;
                    }
                    let Some(lib_symbol) = schema.get_library(symbol.lib_id.as_str()) else {
                        return Err(Error::LibraryNotFound(symbol.lib_id.clone()));
                    };
                    for pin in lib_symbol.pins(symbol.unit).unwrap() {
                        let point: Point = Shape::transform(symbol, &pin.at).into();
                        positions.push((point, NodePositions::Pin(point, pin, symbol)));
                    }
                }
                SchemaElement::NoConnect(nc) => {
                    positions.push((
                        nc.at.clone().into(),
                        NodePositions::NoConnect(Point::new(nc.at[0], nc.at[1])),
                    ));
                }
                SchemaElement::Junction(junction) => {
                    positions.push((
                        junction.at.clone().into(),
                        NodePositions::Junction(Point::new(junction.at[0], junction.at[1])),
                    ));
                }
                SchemaElement::Wire(wire) => {
                    positions.push((
                        Point::new(wire.pts.row(0)[0], wire.pts.row(0)[1]),
                        NodePositions::Wire(
                            Point::new(wire.pts.row(0)[0], wire.pts.row(0)[1]),
                            Point::new(wire.pts.row(1)[0], wire.pts.row(1)[1]),
                        ),
                    ));
                }
                SchemaElement::Label(label) => {
                    positions.push((
                        label.at.clone().into(),
                        NodePositions::Label(Point::new(label.at[0], label.at[1]), label),
                    ));
                }
                SchemaElement::GlobalLabel(label) => {
                    positions.push((
                        label.at.clone().into(),
                        NodePositions::GlobalLabel(Point::new(label.at[0], label.at[1]), label),
                    ));
                }
                SchemaElement::Bus(_) => todo!(),
                SchemaElement::BusEntry(_) => todo!(),
                SchemaElement::HierarchicalLabel(_) => todo!(),
                SchemaElement::Text(_) => {}
                SchemaElement::Polyline(_) => {}
                SchemaElement::Sheet(_) => {}
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
    fn next_node(
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
            //but not for the power symbols
            if symbols.first().unwrap().lib_id.starts_with("power:") {
                continue;
            }

            let first_symbol = &symbols.first().unwrap();

            //skip symbol when Netlist_Enabled is 'N'
            let netlist_enabled = first_symbol.get_property("Spice_Netlist_Enabled");
            if let Some(enabled) = netlist_enabled {
                if enabled == "N" {
                    continue;
                }
            }

            //create the pin order
            let my_pins = self
                .schema
                .get_library(&first_symbol.lib_id)
                .unwrap()
                .pin_names()
                .unwrap();
            let mut pin_sequence: Vec<String> = my_pins.keys().map(|s| s.to_string()).collect();
            pin_sequence.sort_by_key(|x| x.parse::<i32>().unwrap());

            //when Node_Sequence is defined, use it
            let netlist_sequence = first_symbol.get_property("Spice_Node_Sequence");
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
                    if symbol.unit == pin.1 {
                        let pts = Shape::transform(*symbol, &pin.0.at);
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
            let spice_primitive = first_symbol.get_property("Spice_Primitive");
            let spice_model = first_symbol.get_property("Spice_Model");
            let spice_value = first_symbol.get_property("Value");
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{Circuit, Netlist, NodePositions, Point};
    use crate::sexp::Schema;

    #[test]
    fn test_positions() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();
        assert_eq!(11, positions.len());
        let mut iter = positions.iter();
        assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 41.91, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 52.07, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 41.91, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 55.88, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 44.45, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 52.07, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 54.61, y: 91.44 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
        assert_eq!(Point { x: 54.61, y: 91.44 }, iter.next().unwrap().0);
        assert!(iter.next().is_none());
    }
    #[test]
    fn test_next_node() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();

        let mut found = false;
        for pos in &positions {
            if let NodePositions::Pin(_, p, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "R1" && p.number.0 == "1" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(1, node.len());
                    if let NodePositions::Label(_, label) = node[0] {
                        assert_eq!("IN", label.text);
                        found = true;
                    } else {
                        panic!("found node is not a label");
                    }
                }
            }
        }
        assert!(found);
    }
    #[test]
    fn test_next_node_with_junction() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();

        let mut found = 0;
        for pos in &positions {
            if let NodePositions::Pin(_, p, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "R1" && p.number.0 == "2" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(2, node.len());
                    if let NodePositions::Label(_, label) = node[0] {
                        assert_eq!("OUT", label.text);
                        found += 1;
                    } else {
                        panic!("found node is not a label");
                    }
                    if let NodePositions::Pin(_, p, s) = node[1] {
                        assert_eq!("C1", s.get_property("Reference").unwrap());
                        assert_eq!("1", p.number.0);
                        found += 1;
                    } else {
                        panic!("found node is not a label");
                    }
                }
            }
        }
        assert_eq!(2, found);
    }
    #[test]
    fn test_next_node_gnd() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();

        let mut found = 0;
        for pos in &positions {
            if let NodePositions::Pin(_, p, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "#PWR01" && p.number.0 == "1" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(1, node.len());
                    if let NodePositions::Pin(_, _, _) = node[0] {
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                }
            }
        }
        assert_eq!(1, found);
    }
    #[test]
    fn test_next_node_summe() {
        let schema = Schema::load("files/summe.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();

        let mut found = 0;
        for pos in &positions {
            if let NodePositions::Pin(_np, p, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "R3" && p.number.0 == "1" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(2, node.len());
                    if let NodePositions::Pin(pos, _, _) = node[0] {
                        assert_eq!(Point::new(87.63, 33.02), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Pin(pos, _, _) = node[1] {
                        assert_eq!(Point::new(82.55, 43.18), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                }
            }
        }
        assert_eq!(2, found);
    }
    #[test]
    fn test_next_node_svf() {
        let schema = Schema::load("files/svf.kicad_sch").unwrap();
        let positions = Netlist::positions(&schema).unwrap();

        let mut found = 0;
        for pos in &positions {
            if let NodePositions::Pin(_np, p, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "C1" && p.number.0 == "2" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(3, node.len());
                    if let NodePositions::Pin(pos, _, _) = node[0] {
                        assert_eq!(Point::new(40.64, 25.4), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Pin(pos, _, _) = node[1] {
                        assert_eq!(Point::new(44.45, 12.7), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Pin(pos, _, _) = node[2] {
                        assert_eq!(Point::new(71.12, -7.62), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                } else if s.get_property("Reference").unwrap() == "U3" && p.number.0 == "2" {
                    let used = &mut vec![&pos.1];
                    let node = Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                        .unwrap();
                    assert_eq!(4, node.len());
                    if let NodePositions::Pin(pos, _, _) = node[0] {
                        assert_eq!(Point::new(109.22, 25.4), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Label(pos, _) = node[1] {
                        assert_eq!(Point::new(109.22, 12.7), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Pin(pos, _, _) = node[2] {
                        assert_eq!(Point::new(105.41, 12.7), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                    if let NodePositions::Pin(pos, _, _) = node[3] {
                        assert_eq!(Point::new(78.74, -7.62), *pos);
                        found += 1;
                    } else {
                        panic!("found node is not a pin");
                    }
                }
            }
        }
        assert_eq!(7, found);
    }
    #[test]
    fn test_get_symbols() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let symbols = Netlist::get_symbols(&schema).unwrap();
        let mut keys = symbols.keys();
        assert_eq!(&String::from("R1"), keys.next().unwrap());
        assert_eq!(&String::from("#PWR01"), keys.next().unwrap());
        assert_eq!(&String::from("C1"), keys.next().unwrap());
    }

    #[test]
    fn test_nodes() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        assert_eq!(3, netlist.nodes.len());
    }

    #[test]
    fn test_nodes_summe() {
        let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        assert_eq!(118, netlist.nodes.len());
    }

    #[test]
    fn test_nodes_produkt() {
        let schema = Schema::load("files/produkt.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        assert_eq!(75, netlist.nodes.len());
    }

    #[test]
    fn test_circuit() {
        let schema = Schema::load("files/low_pass_filter.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
        netlist.circuit(&mut circuit).unwrap();

        assert_eq!(
            vec![
                String::from(".title auto generated netlist file."),
                String::from("R1 IN OUT 4.7k"),
                String::from("C1 OUT GND 47n")
            ],
            circuit.to_str(false).unwrap()
        );
    }
    /* #[test]
    fn test_4007_vca() {
        let schema = Schema::load("files/4007_vca.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
        netlist.circuit(&mut circuit).unwrap();
        let res = vec![
            String::from(".title auto generated netlist file."),
            String::from(".include files/spice/CD4007.lib\n"),
            String::from("R10 CV 1 100k"),
            String::from("XU1 OUTPUT INPUT 2 NF NF 1 GND 2 NF NF NF NF 2 +5V CMOS4007"),
            String::from("R1 3 4 100k"),
            String::from(".end"),
        ];
        assert_eq!(res, circuit.to_str(true).unwrap());
    } */
    #[test]
    fn test_circuit_summe() {
        let schema = Schema::load("files/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
        netlist.circuit(&mut circuit).unwrap();
        let res = vec![
            String::from(".title auto generated netlist file."),
            String::from(".include files/spice/TL072-dual.lib\n"),
            String::from(".include files/spice/TL072.lib\n"),
            String::from("R5 IN_1 OUTPUT 1k"),
            String::from("R3 1 INPUT 100k"),
            String::from("R4 IN_1 1 100k"),
            String::from("XU1 IN_1 1 GND -15V NC NC NC +15V TL072c"),
            String::from(".end"),
        ];
        assert_eq!(res, circuit.to_str(true).unwrap());
    }
    #[test]
    fn test_circuit_4069() {
        let schema = Schema::load("files/4069.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
        netlist.circuit(&mut circuit).unwrap();
        let res = vec![
            String::from(".title auto generated netlist file."),
            String::from(".include files/spice/4069ub.lib\n"),
            String::from("R1 INPUT 1 100k"),
            String::from("C1 1 2 47n"),
            String::from("XU1 2 3 +5V GND 4069UB"),
            String::from("C2 3 OUTPUT 10u"),
            String::from("R2 3 2 100k"),
            String::from("R3 4 GND 100k"),
            String::from(".end"),
        ];
        assert_eq!(res, circuit.to_str(true).unwrap());
    }
    #[test]
    fn test_circuit_svf() {
        let schema = Schema::load("files/svf.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut count = 0;
        for pos in &netlist.node_positions {
            if let NodePositions::Pin(p, pin, s) = pos.1 {
                if s.get_property("Reference").unwrap() == "R8" && pin.number.0 == "2" {
                    assert_eq!("2", netlist.node_name(&p).unwrap());
                    count += 1;
                } else if s.get_property("Reference").unwrap() == "R7" && pin.number.0 == "2" {
                    assert_eq!("5", netlist.node_name(&p).unwrap());
                    count += 1;
                }
            }
        }
        assert_eq!(2, count);
        let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
        netlist.circuit(&mut circuit).unwrap();
        let res = vec![
            String::from(".title auto generated netlist file."),
            String::from(".include files/spice/4069ub.lib\n"),
            String::from("R1 INPUT 1 100k"),
            String::from("C1 1 2 47n"),
            String::from("XU1 2 3 GND +5V 4069UB"),
            String::from("C2 3 4 47n"),
            String::from("R3 4 5 100k"),
            String::from("XU2 5 HP GND +5V 4069UB"),
            String::from("R5 HP 6 10k"),
            String::from("XU3 6 BP GND +5V 4069UB"),
            String::from("R6 BP 7 10k"),
            String::from("XU4 7 LP GND +5V 4069UB"),
            String::from("R2 3 2 100k"),
            String::from("R4 HP 5 100k"),
            String::from("C3 BP 6 10n"),
            String::from("C4 LP 7 10n"),
            String::from("R7 LP 5 100k"),
            String::from("R8 BP 2 100k"),
            String::from(".end")];
        assert_eq!(res, circuit.to_str(true).unwrap());
    }
}
