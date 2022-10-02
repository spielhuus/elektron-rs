use crate::{circuit::{Netlist, Point}, sexp::{Schema, model::SchemaElement, Shape, Transform}, Error};

use petgraph::{graph::{NodeIndex, UnGraph}, Graph};


pub struct ErcError {

}

pub struct Erc<'a> {
    errors: Vec<ErcError>,
    netlist: Netlist<'a>,
    schema: &'a Schema,
}

impl<'a> Erc<'a> {
    pub fn from(schema: &'a Schema) -> Result<Self, Error> {
        Ok(Self { errors: Vec::new(), netlist: Netlist::from(schema)?, schema })
    }
    pub fn get_errors(&'a self) -> &'a Vec<ErcError>{
        for symbol in self.schema.iter(0).unwrap() {
            if let SchemaElement::Symbol(symbol) = symbol {
                println!("Symbol: {:?}", symbol.get_property("Reference"));
                for _unit in &self.schema.get_library(&symbol.lib_id).unwrap().symbols {
                    if _unit.unit == 0 || _unit.unit == symbol.unit {
                        for pin in &_unit.pin {
                            println!("\tPin: #:{:?}, type: {:?}, at:{:?}", pin.number.0, pin.pin_type, pin.at);
                            let pts = Shape::transform(symbol, &pin.at);
                            let point = Point::new(pts[0], pts[1]);
                            if self.netlist.nodes.contains_key(&point) {
                                let node = self.netlist.netlists.get(*self.netlist.nodes.get(&point).unwrap()).unwrap();
                                println!("\t\t == found pin pos: {:?}", node);
                            } else {
                                println!("\t\t != pin pos not found");
                            }
                        }
                    }
                }
            }
        }
        /* for nl in &self.netlist.netlists {
            println!("netlist: {:?}", nl);
        } */
        &self.errors

    }
}

/* #[cfg(test)]
mod tests {
    use crate::{sexp::{SexpParser, Schema}, circuit::Netlist};

    use super::Erc;

    #[test]
    fn check_no_errors() {
        let schema = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let erc = Erc::from(&schema).unwrap();
        erc.get_errors();
        // assert_eq!(state, Some(State::StartSymbol("node")));
    }
    #[test]
    fn check_unconnected_pin() {
        let schema = Schema::load("samples/files/summe/summe_unconnected.kicad_sch").unwrap();
        let erc = Erc::from(&schema).unwrap();
        erc.get_errors();
        // assert_eq!(state, Some(State::StartSymbol("node")));
    }
}
 */
