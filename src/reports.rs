use json;

use crate::Error;
use crate::sexp::{Get, Sexp, parser::SexpParser, get_unit};

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone)]
struct BomItem {
    amount: usize,
    references: Vec<String>,
    value: String,
    footprint: String,
    datasheet: String,
    description: String,
}

fn reference(value: &str) -> String {
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

fn read_entry(node: &Sexp) -> Result<Option<BomItem>, Error> {
    let mut references = Vec::new();
    let mut value = String::new();
    let mut footprint = String::new();
    let mut datasheet = String::new();
    let mut description = String::new();

    if let Sexp::Node(name, _values) = node {
        if name == "symbol" {
            let unit = get_unit(node)?;
            let lib_id: Vec<&Sexp> = node.get("lib_id")?;
            let lib_name: String = lib_id.get(0).unwrap().get(0)?; //TODO remove unerap
            if unit == 1 && !lib_name.starts_with("power:") && !lib_name.starts_with("Mechanical:") {
                let properties: Vec<&Sexp> = node.get("property")?;
                for prop in properties {
                    let prop_name: String = prop.get(0)?;
                    let prop_value = prop.get(1)?;
                    if prop_name == "Reference" {
                        references.push(prop_value);
                    } else if prop_name == "Value" {
                        value = prop_value;
                    } else if prop_name == "Datascheet" {
                        datasheet = prop_value;
                    } else if prop_name == "Footprint" {
                        footprint = prop_value;
                    } else if prop_name == "Description" {
                        description = prop_value;
                    }
                }
            } else { return Ok(None); }
        } else { return Ok(None); }
    } else { return Err(Error::ExpectSexpNode); }
    Ok(Some(BomItem {
        amount: 1,
        references,
        value,
        footprint,
        datasheet,
        description,
    }))
}

pub fn bom(filename: Option<&str>, sexp_parser: &SexpParser, group: bool) -> Result<(), Error> {

    let mut items: Vec<BomItem> = sexp_parser
        .values()
        .filter_map(|n| read_entry(n).transpose()).collect::<Result<Vec<BomItem>, Error>>()?;

    if group {
        let mut map: HashMap<String, Vec<&BomItem>> = HashMap::new();
        for item in &items {
            let key = format!("{}:{}", item.value, item.footprint);
            if map.contains_key(&key) {
                map.get_mut(&key).unwrap().push(item);
            } else {
                map.insert(key, vec![item]);
            }
        }
        items = map
            .iter()
            .map(|(_, value)| {
                let mut refs: Vec<String> = Vec::new();
                for v in value {
                    refs.push(v.references.get(0).unwrap().to_string());
                }
                BomItem {
                    amount: value.len(),
                    references: refs,
                    value: value[0].value.to_string(),
                    footprint: value[0].footprint.to_string(),
                    datasheet: value[0].datasheet.to_string(),
                    description: value[0].description.to_string(),
                }
            })
            .collect();
    }

    items.sort_by(|a, b| {
        let ref_a = reference(&a.references[0]);
        let ref_b = reference(&b.references[0]);
        ref_a.partial_cmp(&ref_b).unwrap()
    });
    
    //creeate the json stream
    let mut data = json::JsonValue::new_array();
    for item in &items {
        data.push(json::object!{
            amount: item.amount,
            reference: item.references.clone(),
            value: item.value.clone(),
            footprint: item.footprint.clone(),
            datasheet: item.datasheet.clone(),
            description: item.description.clone()
        }).unwrap();
    }

    let mut out: Box<dyn Write> = if let Some(filename) = filename {
        Box::new(File::create(filename).unwrap())
    } else {
        Box::new(std::io::stdout())
    };
    data.write(&mut out)?;
    out.flush()?;
    Ok(())
}
