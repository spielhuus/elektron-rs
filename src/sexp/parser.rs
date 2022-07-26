use memmap2::MmapOptions;
use std::fs::File;
use core::slice::Iter;
use std::io::Write;

use crate::Error;
use crate::sexp::Sexp;

#[derive(PartialEq)]
enum State {
    Symbol,
    Values,
}

fn parser(iter: &mut Iter<u8>) -> Sexp {
    let mut name = String::new();
    let mut values = Vec::new();
    let mut state = State::Symbol;
    let mut s = String::new();
    while let Some(ch) = iter.next() {
        match *ch as char {
            '(' => {
                values.push(parser(iter));
            }
            ')' => {
                if !s.is_empty() {
                    if state == State::Symbol {
                        name = s.to_string();
                    } else {
                        values.push(Sexp::Value(s.to_string()));
                        s.clear();
                    }
                }
                break;
            }
            '"' => {
                let mut text = String::new();
                let mut last_char = '\0';
                loop {
                    // collect the characters to the next quote
                    if let Some(ch) = iter.next() {
                        if *ch as char == '"' && last_char != '\\' {
                            break;
                        } else {
                            text.push(*ch as char);
                            last_char = *ch as char;
                        }
                    }
                }
                values.push(Sexp::Text(text));
            }
            ' ' | '\n' => {
                if state == State::Symbol {
                    name = s.to_string();
                    s.clear();
                    state = State::Values;
                } else if state == State::Values {
                    if !s.is_empty() {
                        //println!("{} {}", s.to_string(), s.len());
                        values.push(Sexp::Value(s.to_string()));
                        s.clear();
                    }
                }
            }
            c => {
                s.push(c);
            }
        };
    }
    Sexp::Node(name, values)
}

pub struct SexpParser {
    nodes: Sexp,
}

impl SexpParser {
    pub fn new() -> Self {
        Self { nodes: Sexp::Node(String::from("kicad_sch"), Vec::new()) }
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let iter = &mut mmap.iter();
        iter.find(|c| **c as char == '(');
        Ok(Self {
            nodes: parser(iter),
        })
    }
    pub fn save(&self, writer: &mut dyn Write) -> Result<(), Error> {
        self.write_node(&self.nodes, writer, 0)
    }
    fn write_node(&self, node: &Sexp, writer: &mut dyn Write, indent: usize) -> Result<(), Error> {
        let prefix = &String::from("  ").repeat(indent);
        match node {
            Sexp::Node(name, values) => {
                if indent == 0 {
                    write!(writer, "({}", name)?;
                } else {
                    write!(writer, "\n{}({}", prefix, name)?;
                }
                for n in values.iter() {
                    self.write_node(n, writer, indent + 1)?;
                }
                write!(writer, ")")?;
            }
            Sexp::Value(value) => {
                write!(writer, " {}", value)?;
            }
            Sexp::Text(text) => {
                write!(writer, " \"{}\"", text)?;
            }
            Sexp::Empty => {
                return Err(Error::NotLoaded);
            }
        }
        if indent == 0 {
            write!(writer, "\n")?;
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Sexp> {
        if let Sexp::Node(_, values) = &self.nodes {
            values.into_iter()
        } else { panic!("nodes not set."); }
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Sexp> {
        if let Sexp::Node(_, values) = &mut self.nodes {
            values.into_iter()
        } else { panic!("nodes not set."); }
    }
    pub fn push(&mut self, node: Sexp) -> Result<(), Error> {
        if let Sexp::Node(_, ref mut values) = &mut self.nodes {
            values.push(node);
        } else {
            return Err(Error::ParseError);
        }       
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sexp::{get, Get, get_property, Test};
    use ndarray::Array2;

    #[test]
    fn load_and_iterate() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for _ in doc.iter() {
            count += 1;
        }
        assert_eq!(count, 51);
    }
    #[test]
    fn load_and_iterate_wires() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(name, _values) if name == "wire" => {
                    count += 1;
                }
                _ => {}
            }
        }
        assert_eq!(count, 14);
    }
    #[test]
    fn test_get_value() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref _values) if name == "label" => {
                    count += 1;
                    let str: String = n.get(0).unwrap();
                    assert_eq!(String::from("IN_1"), str);
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
    #[test]
    fn test_get_properties() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref _values) if name == "symbol" => {
                    count += 1;
                    let properties: Vec<&Sexp> = n.get("property").unwrap();
                    assert_eq!(properties.len(), 5);
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
    #[test]
    fn test_get_property_hide() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref _values) if name == "symbol" => {
                    let reference = get_property(n, "Reference").unwrap();
                    if reference == "R5" {
                        for val in _values {
                            match val {
                                Sexp::Node(name, _) if name == "property" => {

                                    let property_name: String = get!(val, 0).unwrap();
                                    if property_name == "Footprint" {
                                        let effects: Vec<&Sexp> = val.get("effects").unwrap();
                                        assert_eq!(effects.len(), 1);
                                        assert!(effects.get(0).unwrap().has("hide"));
                                        count += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
    #[test]
    fn test_quoted_string() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref values) if name == "lib_symbols" => {
                    for symbol in values {
                        match &symbol {
                            Sexp::Node(ref name, ref _values) if name == "symbol" => {
                                let symbol_name: String = get!(symbol, 0).unwrap();
                                if symbol_name == String::from("power:+15V") {
                                    count += 1;
                                    let properties: Vec<&Sexp> = symbol.get("property").unwrap();
                                    for prop in properties {
                                        let prop_name: String = get!(prop, 0).unwrap();
                                        if prop_name == "ki_description" {
                                            let prop_value: String = get!(prop, 1).unwrap();
                                            assert_eq!(prop_value, "Power symbol creates a global label with name \\\"+15V\\\"");
                                        }
                                    }
                                    break;
                                }
                            }
                            _ => {}
                        } 
                    }
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
    #[test]
    fn test_get_wire_pts() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref _values) if name == "wire" => {
                    count += 1;
                    let coords: Array2<f64> = n.get("pts").unwrap();
                    assert_eq!(coords.len(), 4);
                    assert_eq!(coords[[0, 0]], 96.52);
                    assert_eq!(coords[[0, 1]], 33.02);
                    assert_eq!(coords[[1, 0]], 96.52);
                    assert_eq!(coords[[1, 1]], 45.72);
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
    #[test]
    fn test_get_macro() {
        let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        let mut count = 0;
        for n in doc.iter() {
            match n {
                Sexp::Node(ref name, ref _values) if name == "symbol" => {
                    count += 1;
                    let lib_id: String = get!(n, "lib_id", 0);
                    assert_eq!(lib_id, "Device:R");
                    break;
                }
                _ => {}
            }
        }
        assert_eq!(count, 1);
    }
}
