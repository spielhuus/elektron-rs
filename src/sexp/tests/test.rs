
mod tests {
    mod parser {
        use sexp::{
            Sexp, SexpParser, SexpTree, State, SexpValueQuery, SexpValuesQuery, utils, el,
            SexpProperty, math::{Shape, Transform}, 
        };
        use ndarray::{arr1, Array1, s};
        #[test]
        fn next_siebling() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let mut count = 0;
            let mut iter = doc.iter();

            if let Some(State::StartSymbol(name)) = &iter.next() {
                count += 1;
                assert_eq!("kicad_sch", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next() {
                count += 1;
                assert_eq!("version", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("generator", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("uuid", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("paper", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("title_block", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("lib_symbols", *name);
            }
            if let Some(State::StartSymbol(name)) = &iter.next_siebling() {
                count += 1;
                assert_eq!("junction", *name);
            }
            assert_eq!(count, 8);
        }

        #[test]
        fn search() {
            let doc = SexpParser::load("tests/Amplifier_Operational.kicad_sym").unwrap();
            let mut count = 0;
            let mut iter = doc.iter();

            if let Some(State::StartSymbol(name)) = &iter.next() {
                if *name == "kicad_symbol_lib" {
                    iter.next(); //take first symbol
                    while let Some(state) = iter.next_siebling() {
                        if let State::StartSymbol(name) = state {
                            if name == el::SYMBOL {
                                if let Some(State::Text(id)) = iter.next() {
                                    if id == "TL072" {
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    panic!("file is not a symbol library")
                }
            }
            assert_eq!(count, 1);
        }
        #[test]
        fn parse_tree() {
            let doc = SexpParser::from(String::from("(node value1\n(node2 \"value 2\"\n)\n)"));
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();
            assert_eq!("node", root.name);
            assert_eq!(
                vec!["value1"],
                <Sexp as SexpValuesQuery<Vec<String>>>::values(root)
            );
            assert_eq!("node2", root.nodes().next().unwrap().name);
            assert_eq!(
                vec!["value 2"],
                <Sexp as SexpValuesQuery<Vec<String>>>::values(root.nodes().next().unwrap())
            );
        }
        #[test]
        fn query_tree() {
            let doc = SexpParser::from(String::from("(node value1\n(node2 \"value 2\"\n)\n)"));
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();

            let node = root.query("node2").collect::<Vec<&Sexp>>();
            assert_eq!("node2", node[0].name);
            assert_eq!(
                vec!["value 2"],
                <Sexp as SexpValuesQuery<Vec<String>>>::values(node[0])
            );
        }
        #[test]
        fn tree_get_string() {
            let doc = SexpParser::from(String::from("(node value1\n(node2 \"value 2\"\n)\n)"));
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();
            let node: String = root.value("node2").unwrap();
            assert_eq!("value 2".to_string(), node);
        }
        #[test]
        fn tree_iter_symbols() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();
            let res = root.query(el::SYMBOL).collect::<Vec<&Sexp>>();
            assert_eq!(151, res.len());
        }
        #[test]
        fn symbol_at() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();
            let res = root.query(el::SYMBOL).collect::<Vec<&Sexp>>();
            assert_eq!(
                arr1(&[48.26, 43.18]),
                utils::at(res.get(0).unwrap()).unwrap()
            );
        }
        #[test]
        fn symbol_angle() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let root = tree.root().unwrap();
            let res = root.query(el::SYMBOL).collect::<Vec<&Sexp>>();
            assert_eq!(0.0, utils::angle(res.get(0).unwrap()).unwrap());
        }
        #[test]
        fn pin_pos_r1() {
            let doc = SexpParser::load("tests/pinpos.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R1"
                })
                .unwrap();

            let lib = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let pin1 = utils::pin(lib, "1").unwrap();
            let pin1_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin1, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin1_at);
            assert_eq!(arr1(&[48.26, 38.1]), pos);

            let pin2 = utils::pin(lib, "2").unwrap();
            let pin2_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin2, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin2_at);
            assert_eq!(arr1(&[48.26, 45.72]), pos);
        }
        #[test]
        fn pin_pos_r2() {
            let doc = SexpParser::load("tests/pinpos.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R2"
                })
                .unwrap();

            let lib = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let pin1 = utils::pin(lib, "1").unwrap();
            let pin1_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin1, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin1_at);
            assert_eq!(arr1(&[58.42, 41.91]), pos);

            let pin2 = utils::pin(lib, "2").unwrap();
            let pin2_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin2, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin2_at);
            assert_eq!(arr1(&[66.04, 41.91]), pos);
        }
        #[test]
        fn pin_pos_r3() {
            let doc = SexpParser::load("tests/pinpos.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R3"
                })
                .unwrap();

            let lib = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let pin1 = utils::pin(lib, "1").unwrap();
            let pin1_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin1, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin1_at);
            assert_eq!(arr1(&[76.2, 45.72]), pos);

            let pin2 = utils::pin(lib, "2").unwrap();
            let pin2_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin2, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin2_at);
            assert_eq!(arr1(&[76.2, 38.1]), pos);
        }
        #[test]
        fn pin_pos_r4() {
            let doc = SexpParser::load("tests/pinpos.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R4"
                })
                .unwrap();

            let lib = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let pin1 = utils::pin(lib, "1").unwrap();
            let pin1_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin1, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin1_at);
            assert_eq!(arr1(&[93.98, 41.91]), pos);

            let pin2 = utils::pin(lib, "2").unwrap();
            let pin2_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin2, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin2_at);
            assert_eq!(arr1(&[86.36, 41.91]), pos);
        }
        #[test]
        fn pin_pos_4069() {
            let doc = SexpParser::load("tests/pinpos_2.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R3"
                })
                .unwrap();

            let lib = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let pin1 = utils::pin(lib, "1").unwrap();
            let pin1_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin1, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin1_at);
            assert_eq!(arr1(&[63.5, 33.02]), pos);

            let pin2 = utils::pin(lib, "2").unwrap();
            let pin2_at = <Sexp as SexpValueQuery<Array1<f64>>>::value(pin2, el::AT)
                .unwrap()
                .slice_move(s![0..2]);
            let pos = Shape::transform(symbol, &pin2_at);
            assert_eq!(arr1(&[63.5, 25.4]), pos);
        }
    }
    mod math {
        use ndarray::{arr2, arr1};
        use sexp::{Builder, SexpParser, SexpTree, SexpValueQuery, SexpProperty, utils, math::{Bounds, CalcArc, MathUtils, normalize_angle}, el};

        #[test]
        fn shape_opamp_a() {
            let doc = SexpParser::load("tests/opamp.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let unit: usize = s.value(el::SYMBOL_UNIT).unwrap();
                    let name: String = s.property("Reference").unwrap();
                    name == "U1" && unit == 1
                })
                .unwrap();
            let lib_symbol = utils::get_library(tree.root().unwrap(), "Amplifier_Operational:TL072").unwrap();
            let size = symbol.bounds(lib_symbol).unwrap();
            assert_eq!(arr2(&[[-7.62, -5.08], [7.62, 5.08]]), size)
        }
        #[test]
        fn shape_opamp_c() {
            let doc = SexpParser::load("tests/opamp.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let unit: usize = s.value(el::SYMBOL_UNIT).unwrap();
                    let name: String = s.property("Reference").unwrap();
                    name == "U1" && unit == 3
                })
                .unwrap();
            let lib_symbol = utils::get_library(tree.root().unwrap(), "Amplifier_Operational:TL072").unwrap();

            let size = symbol.bounds(lib_symbol).unwrap();
            assert_eq!(arr2(&[[-2.54, -7.62], [-2.54, 7.62]]), size)
        }
        #[test]
        fn shape_r() {
            let doc = SexpParser::load("tests/opamp.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let symbol = tree
                .root()
                .unwrap()
                .query(el::SYMBOL)
                .find(|s| {
                    let name: String = s.property("Reference").unwrap();
                    name == "R1"
                })
                .unwrap();
            let lib_symbol = utils::get_library(tree.root().unwrap(), "Device:R").unwrap();

            let size = symbol.bounds(lib_symbol).unwrap();
            assert_eq!(arr2(&[[-1.016, -3.81], [1.016, 3.81]]), size)
        }
        #[test]
        fn calc_arc() {
            let arc = sexp::sexp!((arc (start "0" "0.508") (mid "-0.508" "0") (end "0" "-0.508")
                (stroke (width "0.1524") (type "default") (color "0" "0" "0" "0"))
                (fill (type "none"))
            ));

            let arc = arc.root().unwrap();
            assert_eq!(0.508, arc.radius());
            assert_eq!(arr1(&[0.0, 0.0]), arc.center());
            assert_eq!(90.0, arc.start_angle());
            assert_eq!(270.0, arc.end_angle());
        }
        #[test]
        fn calc_arc_center1() {
            let arc = sexp::sexp!((arc (start "38.1" "-69.85") (mid "31.75" "-63.5") (end "25.4" "-69.85")
                (stroke (width "0.1524") (type "default") (color "0" "0" "0" "0"))
                (fill (type "none"))
            ));
            let arc = arc.root().unwrap();
            assert_eq!(arr1(&[31.75, -69.85]), arc.center());
        }
        #[test]
        fn calc_arc_center2() {
            let arc = sexp::sexp!((arc (start "-44196.0" "-38100.0") (mid "-32033.0" "0.0") (end "-44196.0" "38100.0")
                (stroke (width "0.1524") (type "default") (color "0" "0" "0" "0"))
                (fill (type "none"))
            ));
            let arc = arc.root().unwrap();
            assert_eq!(arr1(&[-97787.6891803009, 0.0]), arc.center());
        }
        #[test]
        fn test_normalize_angle() {
            assert_eq!(270.0, normalize_angle(-90.0));
            assert_eq!(90.0, normalize_angle(450.0));
            assert_eq!(180.0, normalize_angle(180.0));
        }
        #[test]
        fn test_vector_distance() {
            assert_eq!(arr1(&[10.0, 0.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 0.0, 10.0));
            assert_eq!(arr1(&[0.0, 10.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 90.0, 10.0));
            assert_eq!(arr1(&[-10.0, 0.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 180.0, 10.0));
        } 
    }
}
