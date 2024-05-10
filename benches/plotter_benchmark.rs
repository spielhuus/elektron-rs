use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use plotter::{schema::SchemaPlot, svg::SvgPlotter, Theme};

fn plt_schema() {

    let mut plotter = SchemaPlot::new()
        .border(false).theme(Theme::Kicad2020).scale(2.0);

    //plotter.open("tests/dco.kicad_sch");
    plotter.open(Path::new("src/plotter/tests/hall.kicad_sch")).unwrap();
    //plotter.open("/home/etienne/github/elektrophon/src/resonanz/main/main.kicad_sch");
    for page in plotter.iter() {
        let mut buffer = Vec::<u8>::new();
        let mut svg_plotter = SvgPlotter::new(&mut buffer);
        plotter.write(page.0, &mut svg_plotter).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("plt_schema", |b| b.iter( plt_schema));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
