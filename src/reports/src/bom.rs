//! Create a BOM for the Schema.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::bom::bom;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let result = bom(&schema, true, Some(String::from("files/partlist.yaml"))).unwrap();
//! println!("Items not found {:#?}", result.1);
//!
use crate::error::Error;
use sexp::{SexpProperty, SexpTree, SexpValueQuery};
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

/// BOM Item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BomItem {
    pub amount: usize,
    pub references: Vec<String>,
    pub value: String,
    pub footprint: String,
    pub datasheet: String,
    pub description: String,
    pub mouser_nr: String,
}

impl From<Yaml> for BomItem {
    fn from(yaml: Yaml) -> Self {
        let mut value = String::new();
        let mut description = String::new();
        let mut footprint = String::new();
        let mut datasheet = String::new();
        let mut mouser_nr = String::new();
        if let Yaml::Hash(hash) = yaml {
            let key = hash.keys().next().unwrap();
            if let Yaml::String(key) = key {
                description = key.to_string();
            } else {
                panic!("part key is not a String.");
            }
            if let Yaml::Array(items) = hash.get(key).unwrap() {
                for item in items {
                    if let Yaml::Hash(value_hash) = item {
                        let k = value_hash.keys().next().unwrap();
                        let v = value_hash.get(k).unwrap();
                        if let (Yaml::String(k), Yaml::String(v)) = (k, v) {
                            if k == "value" {
                                value = v.to_string();
                            } else if k == "footprint" {
                                footprint = v.to_string();
                            } else if k == "datasheet" {
                                datasheet = v.to_string();
                            } else if k == "mouser" {
                                mouser_nr = v.to_string();
                            }
                        }
                    }
                }
            }
        }
        BomItem {
            amount: 0,
            references: vec![],
            value,
            footprint,
            datasheet,
            description,
            mouser_nr,
        }
    }
}

pub fn get_partlist(partlist: &str) -> Result<Vec<BomItem>, Error> {

    let content = match std::fs::read_to_string(partlist) {
        Ok(content) => content,
        Err(err) => return Err(Error::PartlistError(partlist.to_string(), err.to_string())),
    };
    let partlist = match YamlLoader::load_from_str(&content) {
        Ok(content) => content,
        Err(err) => return Err(Error::YamlError(partlist.to_string(), err.to_string())),
    };

    let mut bom: Vec<BomItem> = Vec::new();
    for items in partlist {
        if let Yaml::Array(items) = items {
            for item in items {
                bom.push(item.into());
            }
        }
    }
    Ok(bom)
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

pub fn search_part<'a>(partlist: &'a [BomItem], footprint: &str, value: &str) -> Option<&'a BomItem> {
    partlist
        .iter()
        .find(|item| item.footprint == footprint && (item.value == value || item.value == "*"))
}

pub fn merge_item(item: &BomItem, part: Option<&BomItem>) -> BomItem {
    let datasheet = if let Some(part) = &part {
        part.datasheet.to_string()
    } else {
        item.datasheet.to_string()
    };
    let description = if let Some(part) = &part {
        part.description.to_string()
    } else {
        item.description.to_string()
    };
    let mouser_nr = if let Some(part) = &part {
        part.mouser_nr.to_string()
    } else {
        item.mouser_nr.to_string()
    };

    BomItem {
        amount: item.amount,
        references: item.references.clone(),
        value: item.value.to_string(),
        footprint: item.footprint.to_string(),
        datasheet,
        description,
        mouser_nr,
    }
}

/// Create the BOM for a Schema.
///
/// # Arguments
///
/// * `document` - A Schema struct.
/// * `group`    - group equal items.
/// * `partlist` - A YAML file with the parts description.
/// * `return`   - Tuple with a Vec<BomItem> and the items not found
///                in the partlist, when provided.
pub fn bom(
    document: &SexpTree,
    group: bool,
    partlist: Option<String>,
) -> Result<(Vec<BomItem>, Option<Vec<BomItem>>), Error> {
    let partlist = if let Some(partlist) = partlist {
        Some(get_partlist(&partlist)?)
    } else {
        None
    };
    let mut bom_items: Vec<BomItem> = Vec::new();
    let mut missing_items: Vec<BomItem> = Vec::new();

    for item in document.root()?.query(&"symbol") {
        let unit: usize = item.value("unit").unwrap_or_else(|| 1);
        let lib_id: String = item.value("lib_id").unwrap();
        let on_board: bool = item.value("on_board").unwrap();
        let in_bom: bool = item.value("in_bom").unwrap();
        if unit == 1
            && on_board
            && in_bom
            && !lib_id.starts_with("power:")
            && !lib_id.starts_with("Mechanical:")
        {
            let bom_item = BomItem {
                amount: 1,
                references: vec![item.property("Reference").unwrap()],
                value: item.property("Value").unwrap(),
                footprint: item.property("Footprint").unwrap(),
                datasheet: item.property("Datasheet").unwrap(),
                description: if let Some(description) = item.property("Description") {
                    description
                } else {
                    String::new()
                },
                mouser_nr: String::new(),
            };
            if let Some(partlist) = &partlist {
                let part = search_part(partlist, &bom_item.footprint, &bom_item.value);
                if part.is_none() {
                    missing_items.push(bom_item.clone());
                    bom_items.push(bom_item);
                } else {
                    bom_items.push(merge_item(&bom_item, part));
                }
            } else {
                bom_items.push(bom_item);
            }
        }
    }

    if group {
        let mut map: HashMap<String, Vec<&BomItem>> = HashMap::new();
        for item in &bom_items {
            let key = format!("{}:{}", item.value, item.footprint);
            map.entry(key).or_default().push(item);
        }
        bom_items = map
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
                    mouser_nr: value[0].mouser_nr.to_string(),
                }
            })
            .collect();
    }

    bom_items.sort_by(|a, b| {
        let ref_a = reference(&a.references[0]);
        let ref_b = reference(&b.references[0]);
        ref_a.partial_cmp(&ref_b).unwrap()
    });

    Ok((
        bom_items,
        if missing_items.is_empty() {
            None
        } else {
            Some(missing_items)
        },
    ))
}

#[cfg(test)]
mod tests {

}
