use crate::Error;
use crate::sexp::parser::SexpParser;
use crate::sexp::{Sexp, get_unit};
use crate::sexp::get::{Get, get};

use std::collections::HashMap;

pub fn libraries<'a>(sexp_parser: &'a SexpParser) -> Result<HashMap<String, &'a Sexp>, Error> {
   let mut libraries: std::collections::HashMap<String, &Sexp> = std::collections::HashMap::new();
   for element in sexp_parser.values() {
       if let Sexp::Node(name, values) = element {
           if name == "lib_symbols" {
               for value in values {
                   let name: String = value.get(0).unwrap();
                   libraries.insert(name, value);
               }
           }
       }
   }
   Ok(libraries)
}

/* pub fn iterate_units(node: &Sexp, libraries: HashMap<String, Sexp>, f: &dyn Fn(&Sexp) -> Result<(), Error>) {

    let lib_id: String = get!(node, "lib_id", 0);
    let unit: usize = get_unit(node).unwrap();
    let library = libraries.get(&lib_id).unwrap();
    let syms: Vec<&Sexp> = library.get("symbol").unwrap();
    for _unit in syms {
        let unit_number = get_unit(_unit).unwrap();
        if unit_number == 0 || unit_number == unit {
            f(_unit).unwrap();
        }
    }
} */

pub fn iterate_unit_pins<'a>(node: &'a Sexp, libraries: &HashMap<String, &'a Sexp>) -> Vec<&'a Sexp> {

    let mut items: Vec<&Sexp> = Vec::new();
    let lib_id: String = get!(node, "lib_id", 0);
    let unit: usize = get_unit(node).unwrap();
    let library = libraries.get(&lib_id).unwrap();
    let syms: Vec<&Sexp> = library.get("symbol").unwrap();
    for _unit in syms {
        let unit_number = get_unit(_unit).unwrap();
        if unit_number == 0 || unit_number == unit {
            if let Sexp::Node(_, values) = _unit {
                for el in values {
                    if let Sexp::Node(name, _) = el {
                        if name == "pin" {
                            items.push(el);
                        }
                    }
                }
            }
        }
    }
    items
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_iterate() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let libraries = libraries(&doc).unwrap();
        let mut count = 0;
        doc.values().for_each(|node|{
            if let Sexp::Node(name, _) = node {
                if name == "symbol" {
                    count = iterate_unit_pins(node, &libraries).iter().map(|node| -> Result<&Sexp, Error> {
                       Ok(node)
                    }).count();
                }
            }
        });
        assert_eq!(count, 3);
    }
}
