use super::sexp::{Error, SexpConsumer, SexpNode};
use crate::sexp::get::SexpGet;
use crate::sexp::parser::SexpParser;
use ::std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibrariesError {
    value: String,
}
impl LibrariesError {
    pub fn new(msg: &str) -> LibrariesError {
        LibrariesError {
            value: msg.to_string(),
        }
    }
}
impl Display for LibrariesError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str("missing closing quote")
    }
}
impl std::error::Error for LibrariesError {}

struct LibraryConsumer {
    libraries: Vec<SexpNode>,
}
impl SexpConsumer for LibraryConsumer {
    fn visit(&mut self, node: &SexpNode) -> Result<(), Error> {
        if node.name == "symbol" {
            self.libraries.push(node.clone());
        }
        Ok(())
    }
    fn start(&mut self, _: &String, _: &String) -> Result<(), Error> {
        Ok(())
    }
    fn end(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn start_library_symbols(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_library_symbols(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn start_sheet_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_sheet_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn start_symbol_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_symbol_instances(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
impl LibraryConsumer {
    fn new() -> LibraryConsumer {
        LibraryConsumer {
            libraries: Vec::new(),
        }
    }
}

pub struct Libraries {
    pathlist: Vec<String>,
    libraries: HashMap<String, Vec<SexpNode>>,
}
impl Libraries {
    pub fn new(pathlist: Vec<String>) -> Self {
        Self {
            pathlist,
            libraries: HashMap::new(),
        }
    }

    pub fn search(&mut self, name: &str) -> Option<String> {
        self.load();
        println!("search for library: {}", name);
        Option::from(String::from("not implemented"))
    }

    pub fn get(&mut self, name: &str) -> Result<SexpNode, LibrariesError> {
        let t: Vec<&str> = name.split(':').collect();
        if !self.libraries.contains_key(t[0]) {
            let pathlist = self.pathlist.clone();
            for path in pathlist {
                let filename = &format!("{}/{}.kicad_sym", path, t[0]);
                println!("search for file: {} {}", t[0], filename);
                let path = Path::new(filename);
                self.load_file(path);
            }
        }

        println!("search library for: {}", name);
        let libs: &Vec<SexpNode> = self.libraries.get(t[0]).unwrap();
        for lib in libs {
            let symbol: String = lib.get(0).unwrap();
            if symbol == t[1] {
                return Ok(lib.clone());
            }
        }
        Err(LibrariesError::new("library not found {}")) //TODO format string
    }

    pub fn load(&mut self) -> Result<(), Error> {
        for path in &self.pathlist {
            for entry in fs::read_dir(path).unwrap() {
                let dir = entry.unwrap();

                if dir.path().is_file() {
                    let mut file = File::open(dir.path())?;
                    let mut content = String::new();
                    file.read_to_string(&mut content).unwrap();
                    let mut parser = SexpParser::new(&content);
                    let mut consumer = LibraryConsumer::new();
                    parser.parse(&mut consumer)?;
                    self.libraries.insert(
                        String::from(String::from("XX")),
                        consumer.libraries.clone(),
                    );
                }
            }
        }
        Ok(())
    }
    pub fn load_file(&mut self, path: &Path) {
        let key = path.file_stem().unwrap();
        let key = match key.to_str() {
            Some(k) => k,
            _ => "",
        };
        match File::open(path) {
            Ok(mut f) => {
                println!("Load file: {}", key);
                let mut content = String::new();
                f.read_to_string(&mut content).unwrap();
                let mut parser = SexpParser::new(&content);
                let mut consumer = LibraryConsumer::new();
                parser.parse(&mut consumer).unwrap(); //TODO return error
                self.libraries.insert(
                    String::from(key), /*path.file_name()*/
                    consumer.libraries.clone(),
                );
            }
            Err(error) => {
                println!("Error opening file {:?}: {}", path, error);
            }
        }
    }
}
