///Python bindings for the schema drawer.
use ndarray::Array1;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::fs::File;
use log::debug;

pub mod circuit;
pub mod model;

use crate::error::Error;

use draw::{At, Attribute, Direction, Dot, DotPosition, Draw, Drawer, Label, Nc, Symbol, To};
use plotter::{schema::SchemaPlot, svg::SvgPlotter, Theme};
use sexp::{el, utils, Sexp, SexpValuesQuery};

macro_rules! at {
    ($drawing:expr, $element:expr) => {
        if let Some(pos) = &$element.pos {
            $drawing.draw(&At::Pos(*pos))?;
        }
    };
}

macro_rules! atref {
    ($drawing:expr, $element:expr) => {
        if let Some((reference, pin)) = &$element.atref {
            $drawing.draw(&At::Pin(reference.to_string(), pin.to_string()))?;
        }
    };
}
macro_rules! atdot {
    ($drawing:expr, $element:expr) => {
        if let Some(dot) = &$element.atdot {
            $drawing.draw(&At::Dot(dot.id.to_string()))?;
        }
    };
}
macro_rules! property {
    ($attributes:expr, $element:expr) => {
        if let Some(property) = $element.label {
            $attributes.push(Attribute::Property(property));
        }
    };
}
macro_rules! anchor {
    ($attributes:expr, $element:expr) => {
        $attributes.push(Attribute::Anchor($element.anchor));
    };
}
macro_rules! dot {
    ($attributes:expr, $element:expr) => {
        if let Some(dot) = $element.dot {
            let dot: Vec<DotPosition> = dot.iter().map(|d| d.as_str().into()).collect();
            $attributes.push(Attribute::Dot(dot));
        }
    };
}
macro_rules! toxref {
    ($attributes:expr, $element:expr) => {
        if let Some((reference, pin)) = $element.toxref {
            $attributes.push(Attribute::Tox(At::Pin(reference, pin)));
        }
    };
}
macro_rules! toyref {
    ($attributes:expr, $element:expr) => {
        if let Some((reference, pin)) = $element.toyref {
            $attributes.push(Attribute::Toy(At::Pin(reference, pin)));
        }
    };
}
macro_rules! toxdot {
    ($attributes:expr, $element:expr) => {
        if let Some(dot) = $element.toxdot {
            $attributes.push(Attribute::Tox(At::Dot(dot)));
        }
    };
}
macro_rules! toydot {
    ($attributes:expr, $element:expr) => {
        if let Some(dot) = $element.toydot {
            $attributes.push(Attribute::Toy(At::Dot(dot)));
        }
    };
}
macro_rules! tox {
    ($attributes:expr, $element:expr) => {
        if let Some(tox) = $element.tox {
            $attributes.push(Attribute::Tox(At::Pos((tox[0], tox[1]))));
        }
    };
}
macro_rules! toy {
    ($attributes:expr, $element:expr) => {
        if let Some(toy) = $element.toy {
            $attributes.push(Attribute::Toy(At::Pos((toy[0], toy[1]))));
        }
    };
}
macro_rules! mirror {
    ($attributes:expr, $element:expr) => {
        if let Some(mirror) = $element.mirror {
            $attributes.push(Attribute::Mirror(mirror));
        }
    };
}
macro_rules! id {
    ($attributes:expr, $element:expr) => {
        $attributes.push(Attribute::Id($element.id.to_string()));
    };
}
macro_rules! direction {
    ($attributes:expr, $element:expr) => {
        //get the direction
        match $element.direction {
            model::Direction::Up => $attributes.push(Attribute::Direction(Direction::Up)),
            model::Direction::Down => $attributes.push(Attribute::Direction(Direction::Down)),
            model::Direction::Left => $attributes.push(Attribute::Direction(Direction::Left)),
            model::Direction::Right => $attributes.push(Attribute::Direction(Direction::Right)),
        }
    };
}
macro_rules! length {
    ($attributes:expr, $element:expr) => {
        if let Some(length) = $element.length {
            $attributes.push(Attribute::Length(length));
        }
    };
}

macro_rules! rotate {
    ($attributes:expr, $element:expr) => {
        $attributes.push(Attribute::Rotate($element.angle));
    };
}

macro_rules! attributes {
    ($attributes:expr, $element:expr, $( $keys:tt ),* ) => {
        $(
            $keys!($attributes, $element);
        )*
    };
}

#[pyclass]
pub struct PyDraw {
    draw: Draw,
    positions: Vec<(f64, f64)>,
}

#[pymethods]
impl PyDraw {
    #[new]
    #[pyo3(signature = (library_path, **kwargs))]
    pub fn new(library_path: Vec<String>, kwargs: Option<&Bound<PyDict>>) -> Self {
        let dict = if let Some(kwargs) = kwargs {
            let mut dict: HashMap<String, String> = HashMap::new();
            for (k, v) in kwargs.iter() {
                let value: Result<&str, PyErr> = v.extract();
                if let Ok(value) = value {
                    dict.insert(k.to_string(), value.to_string());
                } //TODO: handle comment
            }
            Some(dict)
        } else {
            None
        };
        Self {
            draw: Draw::new(library_path, dict),
            positions: Vec::new(),
        }
    }

    pub fn pos<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &Bound<PyAny>,
        pin: Option<&Bound<PyAny>>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<model::Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.draw.set(At::Dot(dot.id));
            return slf;
        }
        let dot: Result<(f64, f64), PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.draw.set(At::Pos(dot));
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.draw.set(At::Pin(reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }

    fn add(&mut self, item: &Bound<PyAny>) -> PyResult<()> {
        let feedback: Result<model::Feedback, PyErr> = item.extract();
        if let Ok(feedback) = feedback {
            if let Some((reference, pin)) = feedback.atref {
                //draw the first line
                self.draw
                    .draw(&At::Pin(reference.to_string(), pin.to_string()))?;
                let mut to = To::new();
                to.attributes.push(Attribute::Direction(Direction::Up));
                to.attributes.push(Attribute::Length(feedback.height));
                self.draw.draw(&to)?;

                //draw the second line
                if let Some((to_reference, to_pin)) = feedback.toref {
                    let mut to = To::new();
                    to.attributes.push(Attribute::Tox(At::Pin(
                        to_reference.to_string(),
                        to_pin.to_string(),
                    )));
                    self.draw.draw(&to)?;

                    //and close it
                    let mut to = To::new();
                    to.attributes.push(Attribute::Toy(At::Pin(
                        to_reference.to_string(),
                        to_pin.to_string(),
                    )));
                    self.draw.draw(&to)?;

                    if let Some(dots) = feedback.dot {
                        let dot: Vec<DotPosition> =
                            dots.iter().map(|d| d.as_str().into()).collect();
                        for d in dot {
                            match d {
                                DotPosition::Start => {
                                    let d = Dot::new();
                                    self.draw
                                        .draw(&At::Pin(reference.to_string(), pin.to_string()))?;
                                    self.draw.draw(&d)?;
                                }
                                DotPosition::End => {
                                    let d = Dot::new();
                                    self.draw.draw(&At::Pin(
                                        to_reference.to_string(),
                                        to_pin.to_string(),
                                    ))?;
                                    self.draw.draw(&d)?;
                                }
                            }
                        }
                    }
                }
            }
            return Ok(());
        }

        let line: Result<model::Line, PyErr> = item.extract();
        if let Ok(line) = line {
            let mut to = To::new();
            at!(self.draw, line);
            atref!(self.draw, line);
            atdot!(self.draw, line);
            attributes!(
                to.attributes,
                line,
                length,
                direction,
                tox,
                toy,
                toxref,
                toyref,
                toxdot,
                toydot,
                dot
            );
            let wire = self.draw.draw(&to)?;
            if let Some(wire) = wire {
                let pts = wire.query(el::PTS).next().unwrap();
                let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
                let xy1: Array1<f64> = xy.first().unwrap().values();
                let xy2: Array1<f64> = xy.get(1).unwrap().values();

                item.call_method1("set_points", ((xy1[0], xy1[1]), (xy2[0], xy2[1])))?;
            } else {
                //TODO create error
                println!("no wire returned");
            }

            return Ok(());
        }

        let dot: PyResult<PyRefMut<model::Dot>> = item.extract();
        if let Ok(dot) = dot {
            let mut d = Dot::new();
            atref!(self.draw, dot);
            attributes!(d.attributes, dot, id);
            let junction = self.draw.draw(&d)?;
            if let Some(junction) = junction {
                if dot.pushed {
                    let at: Array1<f64> = utils::at(&junction).unwrap();
                    self.positions.push((at[0], at[1]));
                }
            }
            return Ok(());
        }

        let nc: PyResult<PyRefMut<model::Nc>> = item.extract();
        if let Ok(nc) = nc {
            let n = Nc::new();
            atref!(self.draw, nc);
            self.draw.draw(&n)?;
            return Ok(());
        }

        let label: Result<model::Label, PyErr> = item.extract();
        if let Ok(label) = label {
            let mut l = Label::new();
            l.add_name(label.name);
            atref!(self.draw, label);
            attributes!(l.attributes, label, rotate);
            self.draw.draw(&l)?;
            return Ok(());
        }

        let element: Result<model::Element, PyErr> = item.extract();
        if let Ok(element) = element {
            let mut symbol = Symbol::new();
            symbol.set_reference(element.reference);
            symbol.set_lib_id(element.library);
            let mut props = HashMap::new();
            props.insert("value".to_string(), element.value);
            props.insert("unit".to_string(), element.unit.to_string());
            if let Some(args) = element.args {
                for (k, v) in args {
                    props.insert(k, v);
                }
            }
            symbol.properties = props;

            at!(self.draw, element);
            atref!(self.draw, element);
            atdot!(self.draw, element);
            attributes!(
                symbol.attributes,
                element,
                anchor,
                mirror,
                length,
                rotate,
                tox,
                toy,
                toxref,
                toyref,
                toxdot,
                toydot,
                property
            );
            self.draw.draw(&symbol)?;
            return Ok(());
        }
        panic!("Item not found {:?}", item);
    }

    pub fn pop(&mut self) -> Option<(f64, f64)> {
        self.positions.pop()
    }

    pub fn peek(&mut self) -> Option<(f64, f64)> {
        self.positions.last().copied()
    }

    pub fn next(&mut self, key: String) -> String {
        self.draw.next(key)
    }

    pub fn counter(&mut self, key: String, count: u32) {
        self.draw.counter(key, count)
    }

    pub fn last(&mut self, key: String) -> Result<String, Error> {
        Ok(self.draw.last(key)?)
    }

    ///Set a property value for symbols.
    pub fn property(&mut self, regex: String, key: String, value: String) -> Result<(), Error> {
        Ok(self.draw.property(regex, key, value)?)
    }

    pub fn erc(&mut self) -> Result<String, Error> {
        Ok(self.draw.erc())
    }

    pub fn write(&mut self, filename: &str) -> Result<(), Error> {
        self.draw.write(filename);
        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    pub fn plot(&mut self, kwargs: Option<Bound<PyDict>>) -> Result<Option<Vec<Vec<u8>>>, Error> {
        let mut filename: Option<String> = None;
        let mut id = "not_set";
        let mut border = false;
        let mut scale = 1.0;
        let mut pages: Option<Vec<usize>> = None;
        let mut netlist = false;
        let mut theme = Theme::default();

        if let Some(kwargs) = kwargs {
            if let Ok(Some(raw_item)) = kwargs.get_item("filename") {
                let item: Result<&str, PyErr> = raw_item.extract();
                if let Ok(item) = item {
                    filename = Some(item.to_string());
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("id") {
                let item: Result<&str, PyErr> = item.extract();
                if let Ok(item) = item {
                    id = item;
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("border") {
                let item: Result<bool, PyErr> = item.extract();
                if let Ok(item) = item {
                    border = item;
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("scale") {
                let item: Result<f64, PyErr> = item.extract();
                if let Ok(item) = item {
                    scale = item;
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("pages") {
                let item: Result<Vec<usize>, PyErr> = item.extract();
                if let Ok(item) = item {
                    pages = Some(item);
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("netlist") {
                let item: Result<bool, PyErr> = item.extract();
                if let Ok(item) = item {
                    netlist = item;
                }
            }
            if let Ok(Some(item)) = kwargs.get_item("theme") {
                let item: Result<&str, PyErr> = item.extract();
                if let Ok(item) = item {
                    theme = item.into();
                }
            }
        }

        if let Some(filename) = filename {
            let mut buffer = File::create(filename).unwrap();

            let mut plotter = SchemaPlot::new()
                .border(border)
                .theme(theme)
                .scale(scale)
                .netlist(netlist);

            plotter.open_buffer(self.draw.schema.clone());
            for page in plotter.iter() {

                //TODO check page with pages.

                let mut svg_plotter = SvgPlotter::new(&mut buffer);
                plotter.write(page.0, &mut svg_plotter).unwrap();
            }
            return Ok(None);
        } else {
            debug!("Plotting to notebook: theme: {:?}, border: {}, scale: {}, netlist: {}", theme, border, scale, netlist);
            let nb = if let Ok(nb) = std::env::var("ELEKTRON_NOTEBOOK") {
                nb == "true"
            } else {
                false
            };

            if nb {
                let mut buffer = Vec::new();
                let mut plotter = SchemaPlot::new()
                    .border(border)
                    .theme(theme)
                    .scale(scale)
                    .netlist(netlist);

                plotter.open_buffer(self.draw.schema.clone());
                for page in plotter.iter() {

                    //TODO check page with pages.

                    let mut svg_plotter = SvgPlotter::new(&mut buffer);
                    plotter.write(page.0, &mut svg_plotter).unwrap();
                }

                return Ok(Some(vec![buffer]));

                /* } else {
                let mut rng = rand::thread_rng();
                let num: u32 = rng.gen();
                let filename = String::new()
                    + temp_dir().to_str().unwrap()
                    + "/"
                    + &num.to_string()
                    + ".png";
                let mut buffer = File::create(&filename)?;
                Plotter::png(
                    PlotOptions::new(&self.draw.schema, &mut buffer)
                        .border(border)
                        .theme(theme),
                )?;
                print_from_file(&filename, &Config::default()).expect("Image printing failed.");
                Ok(Some(vec![])) */
            }
        }
        todo!();
    }

    pub fn circuit(&mut self, pathlist: Vec<String>) -> circuit::Circuit {
        let netlist = simulation::Netlist::from(&self.draw.schema).unwrap();
        let mut circuit = circuit::Circuit::new(String::from("draw circuit"), pathlist);
        netlist.circuit(&mut circuit.circuit).unwrap();
        circuit
    }
}
