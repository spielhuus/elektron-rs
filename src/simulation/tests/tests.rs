mod tests {
    mod netlist {
        extern crate sexp;
        extern crate simulation;
        use self::sexp::{el, SexpParser, SexpProperty, SexpTree, SexpValueQuery};
        use self::simulation::{Netlist, NodePositions, Point};
        use std::{cell::RefCell, rc::Rc};
        #[test]
        fn test_positions() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();
            assert_eq!(11, positions.len());
            let mut iter = positions.iter();
            assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 41.91, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 52.07, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 41.91, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 55.88, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 44.45, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 52.07, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 54.61, y: 91.44 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 54.61, y: 83.82 }, iter.next().unwrap().0);
            assert_eq!(Point { x: 54.61, y: 91.44 }, iter.next().unwrap().0);
            assert!(iter.next().is_none());
        }
        #[test]
        fn test_positions_summe() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();
            assert_eq!(521, positions.len());
        }
        #[test]
        fn test_next_node() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();

            let mut found = false;
            for pos in &positions {
                if let NodePositions::Pin(_, p, s) = pos.1 {
                    let number: String = p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                    let reference: String = s.property("Reference").unwrap();
                    if reference == "R1" && number == "1" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        assert_eq!(1, node.len());
                        if let NodePositions::Label(_, label) = node[0] {
                            let text: String = label.get(0).unwrap();
                            assert_eq!("IN", text);
                            found = true;
                        } else {
                            panic!("found node is not a label");
                        }
                    }
                }
            }
            assert!(found);
        }
        #[test]
        fn test_next_node_with_junction() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();

            let mut found = 0;
            for pos in &positions {
                if let NodePositions::Pin(_, p, s) = pos.1 {
                    let number: String = p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                    let reference: String = s.property("Reference").unwrap();
                    if reference == "R1" && number == "2" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        assert_eq!(2, node.len());
                        if let NodePositions::Label(_, label) = node[0] {
                            let text: String = label.get(0).unwrap();
                            assert_eq!("OUT", text);
                            found += 1;
                        } else {
                            panic!("found node is not a label");
                        }
                        if let NodePositions::Pin(_, p, s) = node[1] {
                            let number: String =
                                p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                            let reference: String = s.property("Reference").unwrap();
                            assert_eq!("C1", reference);
                            assert_eq!("1", number);
                            found += 1;
                        } else {
                            panic!("found node is not a label");
                        }
                    }
                }
            }
            assert_eq!(2, found);
        }
        #[test]
        fn test_next_node_gnd() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();

            let mut found = 0;
            for pos in &positions {
                if let NodePositions::Pin(_, p, s) = pos.1 {
                    let number: String = p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                    let reference: String = s.property("Reference").unwrap();
                    if reference == "#PWR01" && number == "1" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        assert_eq!(1, node.len());
                        if let NodePositions::Pin(_, _, _) = node[0] {
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                    }
                }
            }
            assert_eq!(1, found);
        }
        #[test]
        fn test_next_node_summe() {
            let doc = SexpParser::load("tests/summe1.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();

            let mut found = 0;
            for pos in &positions {
                if let NodePositions::Pin(_np, p, s) = pos.1 {
                    let number: String = p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                    let reference: String = s.property("Reference").unwrap();
                    if reference == "R3" && number == "1" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        assert_eq!(2, node.len());
                        if let NodePositions::Pin(pos, _, _) = node[0] {
                            assert_eq!(Point::new(87.63, 33.02), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Pin(pos, _, _) = node[1] {
                            assert_eq!(Point::new(82.55, 43.18), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                    }
                }
            }
            assert_eq!(2, found);
        }
        #[test]
        fn test_next_node_svf() {
            let doc = SexpParser::load("tests/svf.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let positions = Netlist::positions(tree.root().unwrap()).unwrap();

            let mut found = 0;
            for pos in &positions {
                if let NodePositions::Pin(_np, p, s) = pos.1 {
                    let number: String = p.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                    let reference: String = s.property("Reference").unwrap();
                    if reference == "C1" && number == "2" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        //TODO assert_eq!(3, node.len());
                        if let NodePositions::Pin(pos, _, _) = node[0] {
                            assert_eq!(Point::new(40.64, 25.4), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Pin(pos, _, _) = node[1] {
                            assert_eq!(Point::new(44.45, 12.7), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Pin(pos, _, _) = node[2] {
                            assert_eq!(Point::new(71.12, -7.62), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                    } else if reference == "U3" && number == "2" {
                        let used = &mut vec![&pos.1];
                        let node =
                            Netlist::next_node(&pos.0, &positions, &Rc::new(RefCell::new(used)))
                                .unwrap();
                        assert_eq!(4, node.len());
                        if let NodePositions::Pin(pos, _, _) = node[0] {
                            assert_eq!(Point::new(109.22, 25.4), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Label(pos, _) = node[1] {
                            assert_eq!(Point::new(109.22, 12.7), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Pin(pos, _, _) = node[2] {
                            assert_eq!(Point::new(105.41, 12.7), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                        if let NodePositions::Pin(pos, _, _) = node[3] {
                            assert_eq!(Point::new(78.74, -7.62), *pos);
                            found += 1;
                        } else {
                            panic!("found node is not a pin");
                        }
                    }
                }
            }
            assert_eq!(7, found);
        }

        #[test]
        fn test_get_symbols() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbols = Netlist::get_symbols(tree.root().unwrap()).unwrap();
            let mut keys = symbols.keys();

            assert_eq!(&String::from("R1"), keys.next().unwrap());
            assert_eq!(&String::from("#PWR01"), keys.next().unwrap());
            assert_eq!(&String::from("C1"), keys.next().unwrap());
            assert!(keys.next().is_none());
        }
        #[test]
        fn test_get_symbols_summe() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbols = Netlist::get_symbols(tree.root().unwrap()).unwrap();
            let l = symbols.len();
            assert_eq!(139, l);
        }
        #[test]
        fn test_get_symbols_produkt() {
            let doc = SexpParser::load("tests/produkt.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let symbols = Netlist::get_symbols(tree.root().unwrap()).unwrap();
            let l = symbols.len();
            assert_eq!(85, l);
        }
        #[test]
        fn test_get_positions_summe() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let positions = Netlist::positions(tree.root().unwrap()).unwrap();
            let l = positions.len();
            assert_eq!(521, l);
        }
        #[test]
        fn test_nodes() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            assert_eq!(3, netlist.nodes.len());
        }
        #[test]
        fn test_nodes_summe() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            assert_eq!(118, netlist.nodes.len());
        }
        #[test]
        fn test_nodes_produkt() {
            let doc = SexpParser::load("tests/produkt.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            assert_eq!(75, netlist.nodes.len());
        }
    }

    mod circuit {
        extern crate sexp;
        extern crate simulation;
        use self::sexp::{SexpParser, SexpTree};
        use self::simulation::{Circuit, Netlist};
        #[test]
        fn test_circuit() {
            let doc = SexpParser::load("tests/low_pass_filter.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            let mut circuit =
                Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
            netlist.circuit(&mut circuit).unwrap();

            assert_eq!(
                vec![
                    String::from(".title auto generated netlist file."),
                    String::from("R1 IN OUT 4.7k"),
                    String::from("C1 OUT GND 47n")
                ],
                circuit.to_str(false).unwrap()
            );
        }
        /* #[test]
        fn test_4007_vca() {
            let doc = SexpParser::load("files/4007_vca.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            let mut circuit = Circuit::new(String::from("summe"), vec![String::from("files/spice/")]);
            netlist.circuit(&mut circuit).unwrap();
            let res = vec![
                String::from(".title auto generated netlist file."),
                String::from(".include files/spice/CD4007.lib\n"),
                String::from("R10 CV 1 100k"),
                String::from("XU1 OUTPUT INPUT 2 NF NF 1 GND 2 NF NF NF NF 2 +5V CMOS4007"),
                String::from("R1 3 4 100k"),
                String::from(".end"),
            ];
            assert_eq!(res, circuit.to_str(true).unwrap());
        } */
        #[test]
        fn test_circuit_summe() {
            let doc = SexpParser::load("tests/summe1.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            let mut circuit =
                Circuit::new(String::from("summe"), vec![String::from("tests/spice/")]);
            netlist.circuit(&mut circuit).unwrap();
            let res = vec![
                String::from(".title auto generated netlist file."),
                String::from(".include tests/spice/TL072-dual.lib\n"),
                String::from(".include tests/spice/TL072.lib\n"),
                String::from("R5 IN_1 OUTPUT 1k"),
                String::from("R3 1 INPUT 100k"),
                String::from("R4 IN_1 1 100k"),
                String::from("XU1 IN_1 1 GND -15V NC NC NC +15V TL072c"),
                String::from(".end"),
            ];
            assert_eq!(res, circuit.to_str(true).unwrap());
        }
        #[test]
        fn test_circuit_4069() {
            let doc = SexpParser::load("tests/4069.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            let mut circuit =
                Circuit::new(String::from("summe"), vec![String::from("tests/spice/")]);
            netlist.circuit(&mut circuit).unwrap();
            let res = vec![
                String::from(".title auto generated netlist file."),
                String::from(".include tests/spice/4069ub.lib\n"),
                String::from("R1 INPUT 1 100k"),
                String::from("C1 1 2 47n"),
                String::from("XU1 2 3 +5V GND 4069UB"),
                String::from("C2 3 OUTPUT 10u"),
                String::from("R2 3 2 100k"),
                String::from("R3 4 GND 100k"),
                String::from(".end"),
            ];
            assert_eq!(res, circuit.to_str(true).unwrap());
        }
        #[test]
        fn test_circuit_svf() {
            let doc = SexpParser::load("tests/svf.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();
            let netlist = Netlist::from(&tree).unwrap();
            let mut circuit =
                Circuit::new(String::from("summe"), vec![String::from("tests/spice/")]);
            netlist.circuit(&mut circuit).unwrap();
            let res = vec![
                String::from(".title auto generated netlist file."),
                String::from(".include tests/spice/4069ub.lib\n"),
                String::from("R1 INPUT 1 100k"),
                String::from("C1 1 2 47n"),
                String::from("XU1 2 3 GND +5V 4069UB"),
                String::from("C2 3 4 47n"),
                String::from("R3 4 5 100k"),
                String::from("XU2 5 HP GND +5V 4069UB"),
                String::from("R5 HP 6 10k"),
                String::from("XU3 6 BP GND +5V 4069UB"),
                String::from("R6 BP 7 10k"),
                String::from("XU4 7 LP GND +5V 4069UB"),
                String::from("R2 3 2 100k"),
                String::from("R4 HP 5 100k"),
                String::from("C3 BP 6 10n"),
                String::from("C4 LP 7 10n"),
                String::from("R7 LP 5 100k"),
                String::from("R8 BP 2 100k"),
                String::from(".end"),
            ];
            assert_eq!(res, circuit.to_str(true).unwrap());
        }

        // Test the Circuit struct.
        #[test]
        fn load_model() {
            let circuit = Circuit::new(String::from("test"), vec![String::from("tests/spice/")]);
            let include = circuit.get_includes(String::from("TL072")).unwrap();
            assert_eq!("tests/spice/TL072.lib", include.get("TL072").unwrap());
            let include = circuit.get_includes(String::from("BC547B")).unwrap();
            assert_eq!("tests/spice/BC547.mod", include.get("BC547B").unwrap());
            let include = circuit.get_includes(String::from("BC556B")).unwrap();
            assert_eq!("tests/spice/bc5x7.lib", include.get("BC556B").unwrap());
        }
        #[test]
        fn load_4007() {
            let circuit = Circuit::new(String::from("test"), vec![String::from("tests/spice/")]);
            let include = circuit.get_includes(String::from("CMOS4007")).unwrap();
            assert_eq!("tests/spice/CD4007.lib", include.get("CMOS4007").unwrap());
        }
    }
}
