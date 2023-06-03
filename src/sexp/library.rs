use std::collections::HashMap;
use std::path::Path;

use super::parser::{SexpParser, State};

use super::model::LibrarySymbol;
use crate::error::Error;

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
                    } else if name != "kicad_symbol_lib" && name != "version" && name != "generator"
                    {
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

#[derive(Clone)]
pub struct Library {
    pub cache: HashMap<String, LibrarySymbol>,
    pathlist: Vec<String>,
}

impl Library {
    pub fn new(pathlist: Vec<String>) -> Self {
        Self {
            cache: HashMap::new(),
            pathlist,
        }
    }

    pub fn from(filename: String) -> Self {
        let file = Path::new(&filename).file_name().unwrap().to_str().unwrap().to_string();
        let mut cache: HashMap<String, LibrarySymbol> = HashMap::new();
        if let Ok(doc) = SexpParser::load(filename.as_str()) {
            for symbol in doc.iter().node() {
                let t: Vec<&str> = file.split('.').collect();
                cache.insert(format!("{}:{}", t[0], symbol.lib_id), symbol.clone());
            }
        }
        Self { cache, pathlist: vec!() }
    }
    
    pub fn get(&mut self, name: &str) -> Result<LibrarySymbol, Error> {
        if self.cache.contains_key(name) {
            return Ok(self.cache.get(name).unwrap().clone());
        }
        let t: Vec<&str> = name.split(':').collect();
        for path in &self.pathlist {
            let filename = &format!("{}/{}.kicad_sym", path, t[0]);
            if let Ok(doc) = SexpParser::load(filename) {
                for symbol in doc.iter().node() {
                    self.cache
                        .insert(format!("{}:{}", t[0], symbol.lib_id), symbol.clone());
                }
                if self.cache.contains_key(name) {
                    return Ok(self.cache.get(name).unwrap().clone());
                }
            }
        }
        Err(Error::LibraryNotFound(name.to_string())) //TODO format string
    }
}

#[cfg(test)]
mod tests {
    use super::Library;

    #[test]
    fn load_symbols() {
        let mut library = Library::new(vec![
            String::from("files/symbols"),
            String::from("files/other_symbols"),
        ]);
        assert!(library.get("Amplifier_Operational:AD8015").is_ok());
        assert!(library.get("elektrophon:4007N").is_ok());
    }
}
