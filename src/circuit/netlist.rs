use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use ndarray::Array1;

use crate::{
    sexp::{
        model::{Pin, SchemaElement, Symbol, Wire},
        Schema, Shape, Transform,
    },
    Error,
};

use super::Circuit;

#[derive(Clone, Debug)]
pub enum Erc {
    PinNotConnected(Pin),
    WireNotConnected(Wire),
}

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

#[derive(Clone, Debug)]
pub struct Node<'a> {
    identifier: Option<String>,
    points: Vec<Point>,
    pins: Vec<&'a Pin>,
}

/// create a new node with values.
impl<'a> Node<'a> {
    pub fn from(identifier: Option<String>, points: Vec<Point>, pins: Vec<&'a Pin>) -> Self {
        Self {
            identifier,
            points,
            pins,
        }
    }
}

macro_rules! insert_or_update {
    ($self: expr, $identifier: expr, $next_pos: expr, $pin: expr) => {
                        let mut found_existing_node = false;
                        if let Some(identifier) = $identifier {
                            if $self.has_node(identifier.to_string()) {
                                for n in &mut $self.nodes {
                                    if let Some(node_id) = &n.identifier {
                                        if node_id == identifier {
                                            n.identifier = Some(identifier.to_string());
                                            n.points.append(&mut $next_pos);
                                            n.pins.push($pin);
                                            found_existing_node = true;
                                        }
                                    }
                                }
                            }
                        }
                        if !found_existing_node {
                            $self.nodes.push(Node::from(
                                $identifier.clone(),
                                $next_pos,
                                vec![$pin],
                            ))
                        }
    };
}


/// The Netlist struct
///
/// Create a netlist as a graph.
///
pub struct Netlist<'a> {
    schema: &'a Schema,
    elements: Vec<&'a SchemaElement>,
    used_elements: Vec<&'a Wire>,
    symbols: HashMap<String, Vec<&'a Symbol>>,
    nodes: Vec<Node<'a>>,
}

impl<'a> Netlist<'a> {
    pub fn from(schema: &'a Schema) -> Result<Self, Error> {
        let mut elements: Vec<&'a SchemaElement> = Vec::new();
        let mut symbols: HashMap<String, Vec<&Symbol>> = HashMap::new();

        for page in 0..schema.pages() {
            for node in schema.iter(page)? {
                match node {
                    SchemaElement::Symbol(symbol) => {
                        if let Some(reference) = symbol.get_property("Reference") {
                            symbols.entry(reference).or_insert(Vec::new()).push(symbol);
                        } else {
                            return Err(Error::PropertyNotFound(
                                symbol.lib_id.to_string(),
                                String::from("Reference"),
                            ));
                        }
                    }
                    SchemaElement::NoConnect(_) => elements.push(node),
                    SchemaElement::Junction(_) => elements.push(node),
                    SchemaElement::Wire(_) => elements.push(node),
                    SchemaElement::Label(_) => elements.push(node),
                    SchemaElement::GlobalLabel(_) => elements.push(node),
                    SchemaElement::Bus(_) => todo!(),
                    SchemaElement::BusEntry(_) => todo!(),
                    SchemaElement::HierarchicalLabel(_) => todo!(),
                    SchemaElement::Text(_) => {}
                    SchemaElement::Polyline(_) => {}
                    SchemaElement::Sheet(_) => {}
                }
            }
        }

        let mut netlist = Self {
            schema,
            elements,
            used_elements: Vec::new(),
            symbols,
            nodes: Vec::new(),
        };

        for (reference, symbols) in &netlist.symbols {
            for symbol in symbols.iter() {
                if let Some(lib_symbol) = schema.get_library(symbol.lib_id.as_str()) {
                    let identifier: Option<String> = if lib_symbol.power {
                        symbol.get_property("Value")
                    } else {
                        None
                    };
                    for pin in lib_symbol.pins(symbol.unit)? {
                        let point: Point = Shape::transform(*symbol, &pin.at).into();

                        //search the netlist if we have already found this pin position
                        let mut found_node: Option<&mut Node> = None;
                        for node in &mut netlist.nodes {
                            if node.points.contains(&point) {
                                found_node = Some(node);
                                break;
                            }
                        }

                        // use the existing netlist or create a new one.
                        if let Some(found_node) = found_node {
                            found_node.pins.push(pin);
                            if identifier.is_some() {
                                found_node.identifier = identifier.clone();
                            }
                        } else {
                            let next_pos = netlist.next_pos(point, netlist.used_elements.clone());
                            if let Some(mut next_pos) = next_pos {
                                netlist.used_elements = next_pos.1;
                                next_pos.0.push(point);
                                let mut found_existing_node = false;
                                if let Some(identifier) = &identifier {
                                    if netlist.has_node(identifier.to_string()) {
                                        for n in &mut netlist.nodes {
                                            if let Some(node_id) = &n.identifier {
                                                if node_id == identifier {
                                                    n.identifier = Some(identifier.to_string());
                                                    n.points.append(&mut next_pos.0);
                                                    n.pins.push(pin);
                                                    found_existing_node = true;
                                                }
                                            }
                                        }
                                    }
                                }
                                if !found_existing_node {
                                    netlist.nodes.push(Node::from(
                                        identifier.clone(),
                                        next_pos.0,
                                        vec![pin],
                                    ))
                                }
                            } else {
                                let mut found = false;
                                for (subref, subsymbol) in &netlist.symbols {
                                    if subref != reference {
                                        for sym in subsymbol {
                                            if let Some(sub_lib_symbol) =
                                                schema.get_library(sym.lib_id.as_str())
                                            {
                                                for pin in sub_lib_symbol.pins(sym.unit)? {
                                                    let subpts = Shape::transform(*sym, &pin.at);
                                                    let subpoint = Point::new(subpts[0], subpts[1]);
                                                    if subpoint == point {
                                                        if lib_symbol.power {
                                                            insert_or_update!(netlist, &symbol.get_property("Value"), vec![point], pin);
                                                        } else if sub_lib_symbol.power {
                                                            insert_or_update!(netlist, &sym.get_property("Value"), vec![point], pin);
                                                        } else {
                                                            insert_or_update!(netlist, &identifier, vec![point], pin);
                                                        }
                                                        found = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                for el in &netlist.elements {
                                    if let SchemaElement::NoConnect(nc) = el {
                                        let ncpoint = Point::new(nc.at[0], nc.at[1]);
                                        if ncpoint == point {
                                            let mut found_existing_node = false;
                                            if netlist.has_node(String::from("NC")) {
                                                for n in &mut netlist.nodes {
                                                    if let Some(node_id) = &n.identifier {
                                                        if node_id == "NC" {
                                                            n.points.push(point);
                                                            n.pins.push(pin);
                                                            found_existing_node = true;
                                                        }
                                                    }
                                                }
                                            }
                                            if !found_existing_node {
                                                netlist.nodes.push(Node::from(
                                                    Some(String::from("NC")),
                                                    vec![point],
                                                    vec![pin],
                                                ));
                                            }
                                            found = true;
                                        }
                                    }
                                }
                                if !found {
                                    let mut found_existing_node = false;
                                    if netlist.has_node(String::from("UNCONNECTED")) {
                                        for n in &mut netlist.nodes {
                                            if let Some(node_id) = &n.identifier {
                                                if node_id == "UNCONNECTED" {
                                                    n.points.push(point);
                                                    n.pins.push(pin);
                                                    found_existing_node = true;
                                                }
                                            }
                                        }
                                    }
                                    if !found_existing_node {
                                        netlist.nodes.push(Node::from(
                                            Some(String::from("UNCONNECTED")),
                                            vec![point],
                                            vec![pin],
                                        ));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    return Err(Error::LibraryNotFound(symbol.lib_id.clone()));
                }
            }
        }

        //search the labels
        for element in &netlist.elements {
            if let SchemaElement::Label(label) = element {
                let point = Point::new(label.at[0], label.at[1]);
                for node in &mut netlist.nodes {
                    if node.points.contains(&point) {
                        node.identifier = Some(label.text.clone());
                    }
                }
            } else if let SchemaElement::GlobalLabel(label) = element {
                let point = Point::new(label.at[0], label.at[1]);
                for node in &mut netlist.nodes {
                    if node.points.contains(&point) {
                        node.identifier = Some(label.text.clone());
                    }
                }
            }
        }

        //create the netlist numbers
        let mut index: u32 = 1;
        for node in &mut netlist.nodes {
            if node.identifier.is_none() {
                node.identifier = Some(index.to_string());
                index += 1;
            }
        }

        Ok(netlist)
    }
    fn has_node(&self, identifier: String) -> bool {
        for n in &self.nodes {
            if let Some(node_id) = &n.identifier {
                if *node_id == identifier {
                    return true;
                }
            }
        }
        false
    }
    fn node_name(&self, point: &Point) -> Option<String> {
        for n in &self.nodes {
            if n.points.contains(point) {
                return n.identifier.clone();
            }
        }
        None
    }
    fn next_pos(
        &self,
        pos: Point,
        mut used_wires: Vec<&'a Wire>,
    ) -> Option<(Vec<Point>, Vec<&'a Wire>)> {
        let mut points: Vec<Point> = Vec::new();
        for element in &self.elements {
            if let SchemaElement::Wire(wire) = element {
                if !used_wires.contains(&wire) {
                    let p0 = Point::new(wire.pts.row(0)[0], wire.pts.row(0)[1]);
                    let p1 = Point::new(wire.pts.row(1)[0], wire.pts.row(1)[1]);
                    if let Some(nextpos) = if p0 == pos {
                        used_wires.push(wire);
                        Some(p1)
                    } else if p1 == pos {
                        used_wires.push(wire);
                        Some(p0)
                    } else {
                        None
                    } {
                        if let Some(mut final_pos) = self.next_pos(nextpos, used_wires.clone()) {
                            points.append(&mut final_pos.0);
                            used_wires = final_pos.1;
                        } else {
                            points.push(nextpos);
                        }
                    }
                }
            } else if let SchemaElement::Junction(junction) = element {
                if Point::new(junction.at[0], junction.at[1]) == pos {
                    points.push(pos);
                }
            }
        }
        if points.is_empty() {
            None
        } else {
            Some((points, used_wires))
        }
    }
    pub fn circuit(&self, circuit: &mut Circuit) -> Result<(), Error> {
        //Create a spice entry for each referenca
        for (reference, symbols) in &self.symbols {
            //but not for the power symbols
            if reference.starts_with('#') {
                continue;
            }

            let first_symbol = &symbols[0];

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
            let mut pin_sequence: Vec<usize> = (0..my_pins.len()).collect();

            //when Node_Sequence is defined, use it
            let netlist_sequence = first_symbol.get_property("Spice_Node_Sequence");
            if let Some(sequence) = netlist_sequence {
                pin_sequence.clear();
                let splits: Vec<&str> = sequence.split(' ').collect();
                for s in splits {
                    pin_sequence.push(s.parse::<usize>().unwrap());
                }
            }

            //write the spice netlist item
            let spice_primitive = first_symbol.get_property("Spice_Primitive");
            let spice_model = first_symbol.get_property("Spice_Model");
            let spice_value = first_symbol.get_property("Value");
            match spice_primitive {
                Some(primitive) => {
                    if primitive == "X" {
                        let mut seq_string = String::new();
                        for seq in pin_sequence {
                            let real_pin = (&seq + 1).to_string();
                            let pin = my_pins.get(&real_pin).unwrap();
                            //get the symbol from the unit number
                            for s in symbols.iter() {
                                if s.unit == pin.1 {
                                    let pts = Shape::transform(*s, &pin.0.at);
                                    let p0 = Point::new(pts[0], pts[1]);

                                    if let Some(node_name) = self.node_name(&p0) {
                                        seq_string += &node_name;
                                    } else {
                                        seq_string += "NaN"
                                    }
                                    seq_string += " ";
                                }
                            }
                        }
                        if primitive == "X" {
                            let nodes = seq_string.split(' ').map(|s| s.to_string()).collect();
                            circuit.circuit(reference.to_string(), nodes, spice_model.unwrap())?;
                        } else {
                            println!(
                                "=> {}{} - {}- {}",
                                primitive,
                                reference,
                                seq_string,
                                spice_model.unwrap()
                            );
                        }
                    } else {
                        println!("-> {}{} - - {}", primitive, reference, spice_value.unwrap());
                    }
                }
                None => {
                    let mut seq_string = String::new();
                    for seq in pin_sequence {
                        let real_pin = (&seq + 1).to_string();
                        let pin = my_pins.get(&real_pin).unwrap();
                        //get the symbol from the unit number
                        for s in symbols.iter() {
                            if s.unit == pin.1 {
                                let pts = Shape::transform(*s, &pin.0.at);
                                let p0 = Point::new(pts[0], pts[1]);
                                if let Some(node_name) = self.node_name(&p0) {
                                    seq_string += &node_name;
                                } else {
                                    seq_string += "NaN"
                                }
                                seq_string += " ";
                            }
                        }
                    }
                    if reference.starts_with('R') {
                        let nodes: Vec<String> =
                            seq_string.split(' ').map(|s| s.to_string()).collect();
                        circuit.resistor(
                            reference.clone(),
                            nodes[0].clone(),
                            nodes[1].clone(),
                            spice_value.unwrap(),
                        );
                    } else if reference.starts_with('C') {
                        let nodes: Vec<String> =
                            seq_string.split(' ').map(|s| s.to_string()).collect();
                        circuit.capacitor(
                            reference.clone(),
                            nodes[0].clone(),
                            nodes[1].clone(),
                            spice_value.unwrap(),
                        );
                    } else if std::env::var("ELEKTRON_DEBUG").is_ok() {
                        println!(
                            "Unkknwon Reference: {} {}{}",
                            reference,
                            seq_string,
                            spice_value.unwrap()
                        );
                    }
                }
            }
        }

        Ok(())
    }
    pub fn erc(&self) -> Vec<Erc> {
        let mut result = Vec::new();
        if self.has_node(String::from("UNCONNECTED")) {
            for n in &self.nodes {
                if let Some(node_id) = &n.identifier {
                    if node_id == "UNCONNECTED" {
                        for p in &n.pins {
                            let pin = *p;
                            result.push(Erc::PinNotConnected(pin.clone()));
                        }
                    }
                }
            }
        }
        for element in &self.elements {
            if let SchemaElement::Wire(wire) = element {
                let mut found = false;
                for used_elements in &self.used_elements {
                    if *used_elements == wire {
                        found = true;
                        break;
                    }
                }
                if !found {
                    result.push(Erc::WireNotConnected(wire.clone()));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{Netlist, Node};
    use crate::{
        circuit::{netlist::Point, Circuit},
        sexp::{Schema, Shape, Transform},
    };

    #[test]
    fn test_next_pos() {
        let schema = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut nodes: Vec<Node> = Vec::new();
        for (reference, symbols) in &netlist.symbols {
            if reference == "R4" {
                for symbol in symbols.iter() {
                    if let Some(lib_symbol) = schema.get_library(symbol.lib_id.as_str()) {
                        for pin in lib_symbol.pins(symbol.unit).unwrap() {
                            let pts = Shape::transform(*symbol, &pin.at);
                            let next_pos = netlist.next_pos(Point::new(pts[0], pts[1]), Vec::new());
                            if let Some(next_pos) = next_pos {
                                nodes.push(Node::from(None, next_pos.0, vec![pin]))
                            } else {
                                panic!("no pos found for: {}", reference);
                            }
                        }
                    } else {
                        panic!("library symbol not found!");
                    }
                }
            }
        }
        assert_eq!(2, nodes.len());
        assert_eq!(3, nodes[0].points.len());
        assert_eq!(
            vec![
                Point::new(96.52, 45.72),
                Point::new(95.25, 45.72),
                Point::new(101.6, 45.72)
            ],
            nodes[0].points
        );
        assert_eq!(2, nodes[1].points.len());
        assert_eq!(
            vec![Point::new(80.01, 43.18), Point::new(77.47, 43.18)],
            nodes[1].points
        );
    }
    #[test]
    fn used_elements() {
        let schema = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        assert_eq!(14, netlist.used_elements.len());
    }
    #[test]
    fn test_unconnected() {
        let schema = Schema::load("samples/files/summe/summe_unconnected.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let res = netlist.erc();
        assert_eq!(1, res.len());
    }
    #[test]
    fn test_check() {
        let schema = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        assert_eq!(0, netlist.erc().len());
    }
    #[test]
    fn test_circuit() {
        let schema = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&schema).unwrap();
        let mut circuit = Circuit::new(
            String::from("summe"),
            vec![String::from("samples/files/spice/")],
        );
        netlist.circuit(&mut circuit).unwrap();
        circuit.save(None).unwrap();
    }
}
