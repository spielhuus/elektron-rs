mod test {
    mod plot_svg {
        use sexp::{SexpParser, SexpTree};
        use plotter::{PlotterImpl, themer::Themer, Theme, svg::SvgPlotter};

        #[test]
        fn plt_schema() {
            let doc = SexpParser::load("tests/dco.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let svg_plotter = SvgPlotter::new("test", Some(Themer::new(Theme::Kicad2020)));


            let mut buffer = Vec::<u8>::new();
            svg_plotter
                .plot(&tree, &mut buffer, true, 1.0, None, false)
                .unwrap();

            assert!(!buffer.is_empty());
        }
        #[test]
        fn plt_summe() {
            let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let svg_plotter = SvgPlotter::new("test", Some(Themer::new(Theme::Kicad2020)));

            let mut buffer = Vec::<u8>::new();
            svg_plotter
                .plot(&tree, &mut buffer, true, 1.0, None, false)
                .unwrap();

            assert!(!buffer.is_empty());
        }
        #[test]
        fn plt_jfet() {
            let doc = SexpParser::load("tests/jfet.kicad_sch").unwrap();
            let tree = SexpTree::from(doc.iter()).unwrap();

            let svg_plotter = SvgPlotter::new("test", Some(Themer::new(Theme::Kicad2020)));

            let mut buffer = Vec::<u8>::new();
            svg_plotter
                .plot(&tree, &mut buffer, true, 1.0, None, false)
                .unwrap();

            assert!(!buffer.is_empty());
        }
    }
}
