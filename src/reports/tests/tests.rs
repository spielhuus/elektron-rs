mod reports {
    mod tests {
        mod bom {
            use reports::{bom, bom::BomItem};
            use sexp::{SexpParser, SexpTree};
            use yaml_rust::{Yaml, YamlLoader};
            #[test]
            fn test_bom() {
                let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
                let tree = SexpTree::from(doc.iter()).unwrap();
                let result = bom::bom(&tree, true, None).unwrap();
                assert_eq!(17, result.0.len());
            }
            #[test]
            fn into_bom_item() {
                let content = std::fs::read_to_string("tests/partlist.yaml").unwrap();
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
                let partlist = bom::get_partlist("tests/partlist.yaml").unwrap();
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
                let partlist = bom::get_partlist("tests/partlist.yaml").unwrap();
                let item = bom::search_part(
                    &partlist,
                    "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder",
                    "0.1u",
                );
                assert!(item.is_some());
            }
            #[test]
            fn search_item_wildcard() {
                let partlist = bom::get_partlist("tests/partlist.yaml").unwrap();
                let item = bom::search_part(
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
                    footprint: String::from(
                        "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder",
                    ),
                    datasheet: String::new(),
                    description: String::new(),
                    mouser_nr: String::new(),
                };
                let partlist = bom::get_partlist("tests/partlist.yaml").unwrap();
                let item = bom::search_part(
                    &partlist,
                    "Capacitor_SMD:C_0805_2012Metric_Pad1.18x1.45mm_HandSolder",
                    "0.1u",
                );
                let item = bom::merge_item(&bom_item, item);
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
        mod erc {
            use std::path::Path;

            use itertools::Itertools;
            use reports::{erc, erc::symbols};
            use sexp::{SexpParser, SexpTree};
            #[test]
            fn collect_symbols() {
                let doc = SexpParser::load("tests/low_pass_filter_unconnected.kicad_sch").unwrap();
                let schema = SexpTree::from(doc.iter()).unwrap();

                let symbols = symbols(&schema);
                let mut keys: Vec<String> = Vec::new();
                for key in symbols.keys().sorted() {
                    keys.push(key.to_string());
                }
                assert_eq!(
                    vec![
                        String::from("C1"),
                        String::from("R1"),
                        String::from("R?"),
                        String::from("U1")
                    ],
                    keys
                );
            }
            #[test]
            fn check_no_errors() {
                let erc = erc::erc(Path::new("tests/summe.kicad_sch")).unwrap();
                assert!(erc.is_empty());
            }
            #[test]
            fn check_with_mounting_holes() {
                let erc = erc::erc(Path::new("tests/produkt.kicad_sch")).unwrap();
                assert!(erc.is_empty());
            }
            #[test]
            fn check_unconnected_pin() {
                let erc = erc::erc(Path::new("tests/low_pass_filter_unconnected.kicad_sch")).unwrap();
                assert_eq!(10, erc.len());
            }
            #[test]
            fn all_units() {
                let erc = erc::erc(Path::new("tests/3280.kicad_sch")).unwrap();
                assert_eq!(0, erc.len());
            }
        }
    }
}
