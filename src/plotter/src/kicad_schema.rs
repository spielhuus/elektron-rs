use std::io::Write;

use ndarray::{arr1, arr2};
use pathfinder_content::segment::Segment;
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::{rect::RectF, vector::Vector2F};

use log::{trace,debug,info,warn,error};

use sexp::schema::*;

use crate::plotter::{Mirror, Plotter};
use crate::theme::{self, Style, Theme};



pub struct SchemaPlotter<'a, P: Plotter> {
    plotter: P,
    schema: Schema<'a>,
    theme: Theme,
}

impl<'a, P: Plotter> SchemaPlotter<'a, P> {
    pub fn new(schema: Schema<'a>, plotter: P, theme: Theme) -> Self {
        Self { schema, plotter, theme }
    }

    pub fn plot(&mut self) {
        let paper_size = self.schema.paper_size();
        self.plotter.set_view_box(arr2(&[[0.0, 0.0], [paper_size[0], paper_size[1]]]));

        for element in &self.schema {
            match element {
                sexp::schema::Element::Symbol(symbol) => { draw_symbol(&mut self.plotter, symbol, &self.theme); },
                sexp::schema::Element::Wire(wire) => draw_wire(&mut self.plotter, wire, &self.theme),
                sexp::schema::Element::NoConnect(nc) => draw_no_connect(&mut self.plotter, nc, &self.theme),
                sexp::schema::Element::Junction(j) => draw_junction(&mut self.plotter, j, &self.theme),
            }
        }
    }
    pub fn write<W: Write>(self, writer: &mut W) {
        self.plotter.write(writer);
    }
}

//TODO does not show.
fn draw_junction<P: Plotter>(plotter: &mut P, junction: &sexp::Junction, theme: &Theme) {
    //TODO plotter.transform(Transform2F::from_translation(Vector2F::new(0.0, 0.0)));
    plotter.circle(arr1(&[junction.at()[0], 4.0*junction.at()[1]]), 4.0*junction.diameter());
    plotter.stroke(theme.stroke(None, &theme::Style::Wire));
}

fn draw_wire<P: Plotter>(plotter: &mut P, wire: &sexp::Wire, theme: &Theme) {
    plotter.save();
    plotter.move_to(wire.start());
    plotter.line_to(wire.end());
    plotter.transform(Transform2F::from_translation(Vector2F::new(0.0, 0.0)));

    plotter.polyline(arr2(&[wire.start(), wire.end()]));
    plotter.stroke(theme.stroke(None, &theme::Style::Wire));
    plotter.restore();
}

fn draw_no_connect<P: Plotter>(plotter: &mut P, nc: &sexp::NoConnect, theme: &Theme) {
    plotter.save();
    plotter.transform(Transform2F::from_translation(nc.at()));
    plotter.move_to(Vector2F::new(-1.0, -1.0));
    plotter.line_to(Vector2F::new(1.0, 1.0));
    plotter.stroke(theme.stroke(None, &theme::Style::Todo));

    plotter.move_to(Vector2F::new(-1.0, 1.0));
    plotter.line_to(Vector2F::new(1.0, -1.0));
    plotter.stroke(theme.stroke(None, &theme::Style::Todo));
    plotter.restore();
}

fn draw_symbol<P: Plotter>(plotter: &mut P, symbol: &sexp::Symbol, theme: &Theme) {
    debug!("{}", symbol);
    plotter.save();
    for prop in symbol.properties() {
        if prop.effects().unwrap().visible() { 
            trace!("  {}", prop);
            plotter.text(&prop.value(), prop.at(), theme.effects(prop.effects(), &Style::Property));
            //draw the text
        }
    }
    plotter.restore();

    let library = symbol.library();
    for lib_symbol in library.sub_symbols_unit(symbol.unit()) {
        trace!("  {}(symbol({}), subsymbol({}))", symbol.lib_id(), symbol.unit(), lib_symbol.unit());
        plotter.save();
        for g in lib_symbol.graphic() {
            plotter.save();
            plotter.transform(
                Transform2F::from_rotation(symbol.angle().to_radians())
                    .translate(symbol.at())
            );
            plotter.mirror(Mirror::from(symbol.mirror()));
            match g {
                sexp::schema::GraphicType::Polyline(p) => {
                    trace!("    {}", p);
                    polyline(plotter, &p, theme);
                },
                sexp::schema::GraphicType::Rectangle(p) => {
                    trace!("    {}", p);
                    rectangle(plotter, &p, theme);
                }
                sexp::schema::GraphicType::Pin(p) => {
                    trace!("    Pin");
                    pin(plotter, &p, symbol, theme);
                }
            }
            plotter.restore();
        }
        plotter.restore();
    }
}

fn polyline<P: Plotter>(plotter: &mut P, poly: &sexp::Polyline, theme: &Theme) {
    for (i, p) in poly.points().iter().enumerate() {
        if i == 0 {
            plotter.move_to(*p);
        } else {
            plotter.line_to(*p);
        }
    }
    plotter.stroke(theme.stroke(None, &theme::Style::Todo));
}

fn rectangle<P: Plotter>(plotter: &mut P, rect: &sexp::Rectangle, theme: &Theme) {
    println!("RECT: {}", rect);
    plotter.rect(RectF::from_points(rect.start(), rect.end()));
    plotter.stroke(theme.stroke(None, &theme::Style::Todo));
}

fn pin<P: Plotter>(plotter: &mut P, pin: &sexp::Pin, symbol: &Symbol, theme: &Theme) {

    let length = pin.length();
    let angle = pin.angle();

    let to = match angle {
        0.0 => Vector2F::new(pin.at()[0] + length, pin.at()[1]),
        90.0 => Vector2F::new(pin.at()[0], pin.at()[1] + length),
        180.0 => Vector2F::new(pin.at()[0] - length, pin.at()[1]),
        270.0 => Vector2F::new(pin.at()[0], pin.at()[1] - length),
        _ => {
            panic!("pin angle: {}, not supported", angle);
        }
    };
   
    //TODO draw differnt pin graphic types.
    //https://github.com/KiCad/kicad-source-mirror/blob/c36efec4b20a59e306735e5ecbccc4b59c01460e/eeschema/sch_pin.cpp#L245

    plotter.move_to(pin.at());
    plotter.line_to(to);
    plotter.stroke(theme.stroke(None, &theme::Style::Todo));

    if pin.parent().parent().pin_numbers() {

    }
    if pin.parent().parent().pin_names() {
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::{svg_plotter::SvgPlotter, theme::{Theme, Themes}};
    use pathfinder_geometry::transform2d::Transform2F;
    use sexp::{Schema, SexpParser, SexpTree};

    #[test]
    fn test_plot() {
        let doc = SexpParser::load("../sexp/tests/summe.kicad_sch").unwrap();
        let sexp = SexpTree::from(doc.iter()).unwrap();
        let schema = Schema::new(sexp.root().unwrap());
        let plotter = SvgPlotter::new(Transform2F::default());
        let theme = Theme::from(Themes::Kicad2020);
        let mut schema_plotter = super::SchemaPlotter::new(schema, plotter, theme);
        schema_plotter.plot();
        let mut file = File::create("test.svg").unwrap();
        schema_plotter.write(&mut file);
    }
}
