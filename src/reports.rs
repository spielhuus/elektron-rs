use sexp::{SexpNode, Error};
use json;

use super::sexp;
use crate::sexp::get::SexpGet;

use std::io::Write;

pub struct Bom {
    index: u8,
    content: std::collections::HashMap<String, BomItem>,
    reflist: Vec<String>,
    grouped: bool,
    writer: Box<dyn Write>, 
}

#[derive(Debug, Clone)]
struct BomItem {
    amount: u8,
    references: Vec<String>,
    value: String,
    footprint: String,
    datasheet: String,
    description: String
}

impl sexp::SexpConsumer for Bom {
    fn visit(&mut self, node: &SexpNode) -> Result<(), Error> {
        if self.index == 0 && node.name == "symbol" {
            let lib_node: Vec<&SexpNode> = node.nodes("lib_id")?;
            let lib_name: String = lib_node[0].get(0)?;
            if !lib_name.starts_with("power:") && !lib_name.starts_with("Mechanical:") {
                let mut reference = String::from("");
                let mut value = String::from("");
                let mut footprint = String::from("");
                let mut datasheet = String::from("");
                let mut description = String::from("");
                let nodes: Vec<&SexpNode> = node.nodes("property").unwrap();
                for _n in nodes {
                    let key: String = _n.get(0).unwrap();
                    let property_value = _n.get(1).unwrap();
                    if key == "Reference" {
                        reference = property_value;
                    } else if key == "Value" {
                        value = property_value;
                    } else if key == "Datasheet" {
                        datasheet = property_value;
                    } else if key == "Footprint" {
                        footprint = property_value;
                    } else if key == "Description" {
                        description = property_value;
                    }
                }
                let item = BomItem{
                        amount: 1,
                        references: vec![reference.clone()],
                        value,
                        footprint,
                        datasheet,
                        description,
                        };

                self.content.insert(self.reference(&reference), item);
                if !self.reflist.contains(&reference) {
                    self.reflist.push(reference);
                }
            }
        }
        Ok(())
    }
    fn start_library_symbols(&mut self) -> Result<(), Error>  {
        self.index+=1;
        Ok(())
    }
    fn end_library_symbols(&mut self)  -> Result<(), Error> {
        self.index-=1;
        Ok(())
    }
    fn start(&mut self, _: &String, _: &String)  -> Result<(), Error> { Ok(()) }
    fn start_sheet_instances(&mut self)  -> Result<(), Error> { Ok(()) }
    fn end_sheet_instances(&mut self)  -> Result<(), Error> { Ok(()) }
    fn start_symbol_instances(&mut self)  -> Result<(), Error> { Ok(()) }
    fn end_symbol_instances(&mut self)  -> Result<(), Error> { Ok(()) }
    fn end(&mut self)  -> Result<(), Error> { 
        let mut data = json::JsonValue::new_array();
        let references = self.sort_reference(&self.reflist);
        if self.grouped {
            let
                mut groups: std::collections::HashMap<String, BomItem> =
                std::collections::HashMap::new();
            let mut keys: Vec<String> = std::vec![];
            for key in &references {
                let item = self.content.get(key).unwrap();
                let group_key = String::from(&item.value) + &item.footprint;
                if groups.contains_key(&group_key) {
                    let mut reference = groups.get_mut(&group_key).unwrap();
                    reference.amount += 1;
                    reference.references.push(item.references[0].clone());
                } else {
                    keys.push(group_key.clone());
                    groups.insert(group_key, item.clone());
                }
            }
            for key in &keys {
                data.push(json::object!{
                    amount: groups.get(key).unwrap().amount,
                    reference: groups.get(key).unwrap().references.clone(),
                    value: groups.get(key).unwrap().value.clone(),
                    footprint: groups.get(key).unwrap().footprint.clone(),
                    datasheet: groups.get(key).unwrap().datasheet.clone(),
                    description: groups.get(key).unwrap().description.clone()
                }).unwrap();
            }
        } else {
            for key in &references {
                let item = self.content.get(key).unwrap();
                data.push(json::object!{
                    amount: item.amount,
                    reference: item.references.clone(),
                    value: item.value.clone(),
                    footprint: item.footprint.clone(),
                    datasheet: item.datasheet.clone(),
                    description: item.description.clone()
                }).unwrap();
            }
        }
        data.write(&mut self.writer);
        Ok(())
    }
}

impl Bom {
    pub fn new(writer: Box<dyn Write>, grouped: bool) -> Self {
        Bom {
            index: 0,
            content: std::collections::HashMap::new(),
            reflist: Vec::new(),
            grouped,
            writer,
        }
    }
    pub fn reference(&self, value: &str) -> String {
        let mut reference_characters = String::new();
        let mut reference_numbers = String::new();
        for c in value.chars() {
            if c.is_numeric() {
                reference_numbers.push(c);
            } else {
                reference_characters.push(c);
            }
        }
        format!("{}{:0>4}", reference_characters, reference_numbers)
    }

    fn sort_reference(&self, values: &Vec<String>) -> Vec<String> {
        let mut references = Vec::new();
        for v in values {
            let full_reference = self.reference(v);
            references.push(full_reference);
        }
        let mut result = Vec::new();
        for i in itertools::sorted(references) {
            result.push(i)
        }
        result
    }
}
