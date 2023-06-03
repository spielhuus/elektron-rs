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
use crate::sexp::{Schema, SchemaElement};
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

fn get_partlist(partlist: &str) -> Result<Vec<BomItem>, Error> {
    let content = std::fs::read_to_string(partlist)?;
    let partlist = YamlLoader::load_from_str(&content).map_err(|_| Error::FileNotFound(content))?;

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

fn search_part<'a>(partlist: &'a [BomItem], footprint: &str, value: &str) -> Option<&'a BomItem> {
    partlist
        .iter()
        .find(|item| item.footprint == footprint && (item.value == value || item.value == "*"))
}

fn merge_item(item: &BomItem, part: Option<&BomItem>) -> BomItem {
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
    document: &Schema,
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
    for item in document.iter_all() {
        if let SchemaElement::Symbol(symbol) = item {
            if symbol.unit == 1
                && symbol.on_board != false 
                && symbol.in_bom != false
                && !symbol.lib_id.starts_with("power:")
                && !symbol.lib_id.starts_with("Mechanical:")
            {
                let item = BomItem {
                    amount: 1,
                    references: vec![symbol.get_property("Reference").unwrap()],
                    value: symbol.get_property("Value").unwrap(),
                    footprint: symbol.get_property("Footprint").unwrap(),
                    datasheet: symbol.get_property("Datasheet").unwrap(),
                    description: if let Some(description) = symbol.get_property("Description") {
                        description
                    } else {
                        String::new()
                    },
                    mouser_nr: String::new(),
                };
                if let Some(partlist) = &partlist {
                    let part = search_part(partlist, &item.footprint, &item.value);
                    if part.is_none() {
                        missing_items.push(item.clone());
                        bom_items.push(item);
                    } else {
                        bom_items.push(merge_item(&item, part));
                    }
                } else {
                    bom_items.push(item);
                }
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
    use yaml_rust::{Yaml, YamlLoader};

    use super::{bom, BomItem};
    use crate::sexp::Schema;

    #[test]
    fn test_bom() {
        let schema = Schema::load("files/summe_min.kicad_sch").unwrap();
        let result = bom(&schema, true, None).unwrap();
        assert_eq!(4, result.0.len());
    }
    #[test]
    fn into_bom_item() {
        let content = std::fs::read_to_string("files/partlist.yaml").unwrap();
        let doc = YamlLoader::load_from_str(&content).unwrap();
        let items = doc.first().unwrap();
        if let Yaml::Array(items) = items {
            let item = items.first();
            if let Some(item) = item {
                /* let item = item.clone(); */
                let item: BomItem = item.clone().into();
                assert_eq!(
                    BomItem {
                        amount: 0,
                        references: vec![],
                        value: String::from("0.1u"),
                        footprint: String::from(
                            "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder"
                        ),
                        datasheet: String::from("http://datasheet.com/capacitor.pdf"),
                        description: String::from("Multilayer Ceramic Capacitors MLCC"),
                        mouser_nr: String::from("asd")
                    },
                    item
                );
            } else {
                panic!("item not found")
            }
        } else {
            panic!("items not found")
        }
    }
    #[test]
    fn partlist() {
        let partlist = super::get_partlist("files/partlist.yaml").unwrap();
        assert_eq!(4, partlist.len());
        assert_eq!(
            &BomItem {
                amount: 0,
                references: vec![],
                value: String::from("0.1u"),
                footprint: String::from(
                    "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder"
                ),
                datasheet: String::from("http://datasheet.com/capacitor.pdf"),
                description: String::from("Multilayer Ceramic Capacitors MLCC"),
                mouser_nr: String::from("asd")
            },
            partlist.first().unwrap()
        );
    }
    #[test]
    fn search_item() {
        let partlist = super::get_partlist("files/partlist.yaml").unwrap();
        let item = super::search_part(
            &partlist,
            "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder",
            "0.1u",
        );
        assert!(item.is_some());
    }
    #[test]
    fn search_item_wildcard() {
        let partlist = super::get_partlist("files/partlist.yaml").unwrap();
        let item = super::search_part(
            &partlist,
            "elektrophon:Jack_3.5mm_WQP-PJ398SM_Vertical",
            "*",
        );
        assert_eq!(
            BomItem {
                amount: 0,
                references: vec![],
                value: String::from("*"),
                footprint: String::from("elektrophon:Jack_3.5mm_WQP-PJ398SM_Vertical"),
                datasheet: String::new(),
                description: String::from("Audio Jack"),
                mouser_nr: String::new(),
            },
            *item.unwrap()
        );
        assert!(item.is_some());
    }
    #[test]
    fn merge() {
        let bom_item = BomItem {
            amount: 2,
            references: vec![String::from("C1")],
            value: String::from("0.1u"),
            footprint: String::from("Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder"),
            datasheet: String::new(),
            description: String::new(),
            mouser_nr: String::new(),
        };
        let partlist = super::get_partlist("files/partlist.yaml").unwrap();
        let item = super::search_part(
            &partlist,
            "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder",
            "0.1u",
        );
        let item = super::merge_item(&bom_item, item);
        assert_eq!(
            BomItem {
                amount: 2,
                references: vec![String::from("C1")],
                value: String::from("0.1u"),
                footprint: String::from(
                    "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder"
                ),
                datasheet: String::from("http://datasheet.com/capacitor.pdf"),
                description: String::from("Multilayer Ceramic Capacitors MLCC"),
                mouser_nr: String::from("asd")
            },
            item
        );
    }
}
