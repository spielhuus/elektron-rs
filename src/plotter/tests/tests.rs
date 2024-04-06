mod test {
    mod plot_svg {
        use std::fs::File;

        use plotter::{schema::SchemaPlot, svg::SvgPlotter, themer::Themer, PlotterImpl, Theme};
        use sexp::{SexpParser, SexpTree};

        #[test]
        fn plt_schema() {
            let mut plotter = SchemaPlot::new()
                .border(false).theme(Theme::Kicad2020).scale(2.0);

            //plotter.open("tests/dco.kicad_sch");
            plotter.open("/home/etienne/github/elektrophon/src/hall/main/main.kicad_sch");
            //plotter.open("/home/etienne/github/elektrophon/src/resonanz/main/main.kicad_sch");
            for page in plotter.iter() {
                println!("{:?}", page);
                //let mut buffer = Vec::<u8>::new();
                let mut file = File::create("out.svg").unwrap();
                let mut svg_plotter = SvgPlotter::new(&mut file);
                plotter.write(page.0, &mut svg_plotter).unwrap();
            }
        }
        /* #[test]
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
        } */
    }
}
