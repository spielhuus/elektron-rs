use crate::Error;
use super::sexp::{Sexp, get, Get, SexpParser};
use std::fs;


pub struct Libraries {
    pathlist: Vec<String>,
}
impl Libraries {
    pub fn new(pathlist: Vec<String>) -> Self {
        Self {
            pathlist,
        }
    }

    pub fn search(&mut self, name: &str) -> Result<String, Error> {
        let pathlist = self.pathlist.clone();
        for path in pathlist {
            for entry in fs::read_dir(path).unwrap() {
                let dir = entry.unwrap();
                if dir.path().is_file() {
                    let parser = SexpParser::load(dir.path().to_str().unwrap()).unwrap();
                    //get the Libraries
                    for node in parser.values() {
                        match node {
                            Sexp::Node(node_name, _) => {
                                if node_name == "symbol" {
                                    let lib_id: String = get!(node, 0).unwrap();
                                    if lib_id == name.to_string() {
                                        let lib_name = dir.path().file_stem().unwrap().to_str().unwrap().to_string();
                                        println!("found: {}:{}", lib_name, lib_id);
                                    }
                                }
                            }
                            _ => { return Err(Error::ExpectSexpNode); }
                        }
                    }
                }
            }
        }
        Err(Error::ParseError)
    }

    pub fn get(&mut self, name: &str) -> Result<Sexp, Error> {
//        let t: Vec<&str> = name.split(':').collect();
//        if !self.libraries.contains_key(t[0]) {
//            let pathlist = self.pathlist.clone();
//            for path in pathlist {
//                let filename = &format!("{}/{}.kicad_sym", path, t[0]);
//                let path = Path::new(filename);
//                self.load_file(path)?;
//            }
//        }
//
//        println!("search library for: {}", name);
//        let libs: &Vec<Sexp> = self.libraries.get(t[0]).unwrap();
//        for lib in libs {
//            let symbol: String = lib.get(0).unwrap();
//            if symbol == t[1] {
//                return Ok(lib.clone());
//            }
//        }
//        Err(Error::LibraryNotFound(name.to_string())) //TODO format string
        Err(Error::ParseError)
    }

//    pub fn load(&mut self) -> Result<(), Error> {
//        let pathlist = self.pathlist.clone();
//        for path in pathlist {
//            for entry in fs::read_dir(path).unwrap() {
//                let dir = entry.unwrap();
//                if dir.path().is_file() {
//                    let path = dir.path();
//                    let key = path.file_stem().unwrap();
//                    let key = match key.to_str() {
//                        Some(k) => k,
//                        _ => "",
//                    };
//                    let lib = self.load_file(&path)?;
//                    self.libraries.insert(key.to_string(), lib);
//                }
//            }
//        }
//        Ok(())
//    }
//    pub fn load_file(&mut self, path: &Path) -> Result<Vec<&Sexp>, Error> {
//        let parser = SexpParser::load(path.to_str().unwrap()).unwrap();
//        //get the Libraries
//        let mut results: Vec<&Sexp> = Vec::new();
//        for node in parser.values() {
//            match node {
//                Sexp::Node(name, _) => {
//                    if name == "symbol" {
//                        results.push(node);
//                    }
//                }
//                _ => { return Err(Error::ExpectSexpNode); }
//            }
//        }
//        if results.is_empty() {
//            Err(Error::LibraryNotFound(path.to_str().unwrap().to_string()))
//        } else {
//            Ok(results)
//        }
//    }
}
