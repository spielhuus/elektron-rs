use crate::Error;
use crate::sexp::parser::SexpParser;
use crate::sexp::{Sexp, get_unit};
use crate::sexp::get::{Get, get};

use std::collections::HashMap;
use std::slice::{Iter, IterMut};

pub fn libraries<'a>(sexp_parser: &'a SexpParser) -> Result<HashMap<String, &'a Sexp>, Error> {
   let mut libraries: std::collections::HashMap<String, &Sexp> = std::collections::HashMap::new();
   for element in sexp_parser.iter() {
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

pub fn nodes(node: &mut Sexp, key: String, f: fn(&mut Sexp)) {
    if let Sexp::Node(_, ref mut node_elements) = node {
        for child in node_elements {
            if let Sexp::Node(node_name, _) = child {
                if key == *node_name {
                    f(child);
                }
            }
        }
    }
}
/* struct SexpIterator<'a> {
    items: Vec<&'a mut Sexp>,
    position: usize,
}

impl<'a> SexpIterator<'a> {
    pub fn from(node: &'a mut Sexp, name: &str) -> Self {
        let mut items: Vec<&'a mut Sexp> = Vec::new();
        if let Sexp::Node(_, ref mut node_elements) = node {
            for child in node_elements {
                if let Sexp::Node(node_name, _) = child {
                    if name == node_name {
                        items.push(child);
                    }
                }
            }
        }
        Self { items, position: 0 }
    }
}*/

/* impl Iterator for SexpParser {
    type Item = &mut Sexp;
    fn next(&mut self) -> Option<Self::Item> {
        println!("get next pos:{}", self.position);
        if let Some(node) = self.items.get(self.position) {
            self.position += 1;
            // Option::from(node)
        } else {
        }
    None
    }
} */

/* pub fn iterate_properties<'a>(node: &'a mut Sexp) -> Vec<&'a mut Sexp> {

    let mut items: Vec<&mut Sexp> = Vec::new();
    if let Sexp::Node(_, ref mut node_elements) = node {
        for child in node_elements {
            if let Sexp::Node(name, _) = child {
                if name == "property" {
                    items.push(child);
                }
            }
        }
    }
    items
} */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_and_iterate() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let libraries = libraries(&doc).unwrap();
        let mut count = 0;
        doc.iter().for_each(|node|{
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

    /* #[test]
    fn load_and_iterate_properties() {
        let mut doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        doc.iter_mut().for_each(|mut node|{
            if let Sexp::Node(name, _) = node {
                if name == "symbol" {
                    count = SexpIterator::from(&mut node, "property").map(|node| -> Result<&Sexp, Error> {
                       Ok(node)
                    }).count();
                }
            }
        });
        assert_eq!(count, 7);
    } */
}
