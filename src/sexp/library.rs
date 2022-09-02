use std::collections::HashMap;

use crate::error::Error;
use crate::sexp::{SexpParser, State};

use super::model::LibrarySymbol;

pub struct LibraryParser<I> {
    iter: I,
}

impl<'a, I> Iterator for LibraryParser<I>
where
    I: Iterator<Item = State<'a>>,
{
    type Item = LibrarySymbol;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let state = self.iter.next();
            match state {
                None => {
                    return None;
                }
                Some(State::StartSymbol(name)) => {
                    if name == "symbol" {
                        return Some(LibrarySymbol::from(&mut self.iter));
                    } else if name != "kicad_sch" {
                        println!("start symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<I> LibraryParser<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

pub trait LibraryIterator<T>: Iterator<Item = T> + Sized {
    fn node(self) -> LibraryParser<Self> {
        LibraryParser::new(self)
    }
}

impl<T, I: Iterator<Item = T>> LibraryIterator<T> for I {}

pub struct Library {
    cache: HashMap<String, LibrarySymbol>,
    pathlist: Vec<String>,
}

impl Library {
    pub fn new(pathlist: Vec<String>) -> Self {
        Self {
            cache: HashMap::new(),
            pathlist,
        }
    }
    pub fn get(&mut self, name: &str) -> Result<LibrarySymbol, Error> {
        if self.cache.contains_key(name) {
            return Ok(self.cache.get(name).unwrap().clone());
        }
        let t: Vec<&str> = name.split(':').collect();
        for path in &self.pathlist {
            let filename = &format!("{}/{}.kicad_sym", path, t[0]);
            println!("load library: {}->{}", name, filename);
            let doc = SexpParser::load(filename).unwrap();

            for symbol in doc.iter().node() {
                self.cache
                    .insert(format!("{}:{}", t[0], symbol.lib_id), symbol.clone());
            }
            if self.cache.contains_key(name) {
                return Ok(self.cache.get(name).unwrap().clone());
            }
        }
        Err(Error::LibraryNotFound(name.to_string())) //TODO format string
    }
}
