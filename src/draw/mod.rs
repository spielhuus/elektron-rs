#![allow(clippy::borrow_deref_ref)]
use crate::circuit::{Circuit, Netlist};
use crate::error::Error;
use crate::sexp::model::{
     Effects, Junction, Label, LibrarySymbol, Property, SchemaElement, Stroke, Symbol, Wire,
};
use crate::sexp::{uuid, Bounds, Library, Schema, Shape, Transform};
use itertools::Itertools;
use pyo3::prelude::*;
use std::env::temp_dir;
use std::{fs::{File, self}, io::Read};

use ndarray::{arr1, arr2, Array1};
use rand::Rng;
use uuid::Uuid;

pub mod model;

const LABEL_BORDER: f64 = 2.54;

macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    };
}

fn filter_properties(node: &&mut Property) -> bool {
    if let Some(effects) = &node.effects {
        !effects.hide
    } else {
        true
    }
}

fn sort_properties(a: &&mut Property, b: &&mut Property) -> std::cmp::Ordering {
    a.id.cmp(&b.id)
}

#[pyclass]
pub struct Draw {
    schema: Schema,
    libs: Library,
    last_pos: Array1<f64>,
}

#[pymethods]
impl Draw {
    #[new]
    pub fn new(library_path: Vec<String>) -> Self {
        let mut schema = Schema::new();
        schema.new_page();
        Self {
            schema,
            libs: Library::new(library_path),
            last_pos: arr1(&[10.0, 10.0]),
        }
    }

    fn add(&mut self, item: &'_ PyAny) -> PyResult<()> {
        let line: Result<model::Line, PyErr> = item.extract();
        if let Ok(line) = line {
            self.add_line(line)?;
            return Ok(());
        }
        let dot: PyResult<PyRefMut<model::Dot>> = item.extract();
        if let Ok(mut dot) = dot {
            if dot.pos == vec![0.0, 0.0] {
                dot.pos = vec![self.last_pos[0], self.last_pos[1]];
            }
            self.add_dot(&dot)?;
            return Ok(());
        }
        let label: Result<model::Label, PyErr> = item.extract();
        if let Ok(label) = label {
            self.add_label(label)?;
            return Ok(());
        }
        let element: Result<model::Element, PyErr> = item.extract();
        if let Ok(element) = element {
            self.add_symbol(element)?;
            return Ok(());
        }
        panic!("Item not found {:?}", item);
    }

    pub fn write(&mut self, filename: &str) -> Result<(), Error> {
        self.schema.write(filename)
    }

    pub fn plot(
        &mut self,
        filename: Option<&str>,
        border: bool,
        scale: f64,
        imagetype: &str,
    ) -> Result<Option<Vec<u8>>, Error> {
        let theme = if let Ok(theme) = std::env::var("ELEKTRON_THEME") {
            theme
        } else {
            String::from("kicad_2000")
        };
        if let Some(filename) = filename {
            self.schema.plot(filename, scale, border, theme.as_str())?;
            Ok(None)
        } else {
            let mut rng = rand::thread_rng();
            let num: u32 = rng.gen();
            let filename =
                String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + "." + imagetype;
            self.schema
                .plot(filename.as_str(), scale, border, theme.as_str())?;
            
            let mut f = File::open(&filename).expect("no file found");
            let metadata = fs::metadata(&filename).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read_exact(&mut buffer).expect("buffer overflow");
            Ok(Some(buffer))
            //print_from_file(&filename, &Config::default()).unwrap();
        }
    }

    pub fn circuit(&mut self, pathlist: Vec<String>) -> Circuit {
        let netlist = Netlist::from(&self.schema).unwrap();
        let mut circuit = Circuit::new(String::from("draw circuit"), pathlist);
        netlist.circuit(&mut circuit).unwrap();
        circuit
    }
}

impl Draw {
    fn add_dot(&mut self, dot: &model::Dot) -> Result<(), Error> {
        self.schema.push(
            0,
            SchemaElement::Junction(Junction::new(
                round!(arr1(&[dot.pos[0], dot.pos[1]])),
                uuid!(),
            )),
        )?;
        Ok(())
    }
    fn add_label(&mut self, label: model::Label) -> Result<(), Error> {
        let pos = self.last_pos.clone();
        let mut new_label = Label::new(
            round!(arr1(&[pos[0], pos[1]])),
            label.angle,
            label.name.as_str(),
            uuid!(),
        );
        if label.angle == 180.0 {
            new_label.effects.justify.push("right".to_string());
        } else {
            new_label.effects.justify.push("left".to_string());
        }
        self.schema.push(0, SchemaElement::Label(new_label))?;
        Ok(())
    }
    fn add_line(&mut self, line: model::Line) -> Result<(), Error> {
        let start_pos = if let Some(atdot) = line.atdot {
            arr1(&[atdot.pos[0], atdot.pos[1]])
        } else if let (Some(atpin), Some(atref)) = (line.atpin, line.atref) {
            self.pin_pos(atref, atpin)
        } else {
            self.last_pos.clone()
        };
        let end_pos = if let Some(end) = line.tox {
            arr1(&[end[0], start_pos[1]])
        } else if let Some(end) = line.toy {
            arr1(&[start_pos[0], end[1]])
        } else {
            match line.direction {
                model::Direction::Up => arr1(&[start_pos[0], start_pos[1] - line.length]),
                model::Direction::Down => arr1(&[start_pos[0], start_pos[1] + line.length]),
                model::Direction::Left => arr1(&[start_pos[0] - line.length, start_pos[1]]),
                model::Direction::Right => arr1(&[start_pos[0] + line.length, start_pos[1]]),
            }
        };
        self.schema.push(
            0,
            SchemaElement::Wire(Wire::new(
                round!(arr2(&[
                    [start_pos[0], start_pos[1]],
                    [end_pos[0], end_pos[1]]
                ])),
                Stroke::new(),
                uuid!(),
            )),
        )?;
        self.last_pos = end_pos;
        Ok(())
    }
    fn add_symbol(&mut self, element: model::Element) -> Result<(), Error> {
        let lib_symbol = self.get_library(element.library.as_str())?;
        let sym_pin = lib_symbol.get_pin(element.pin)?;

        let pos = if let (Some(atref), Some(atpin)) = (element.atref, element.atpin) {
            self.pin_pos(atref, atpin)
        } else if let Some(dot) = element.atdot {
            arr1(&[dot.pos[0], dot.pos[1]])
        } else {
            self.last_pos.clone()
        };
        // transform pin pos
        let theta = -element.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = sym_pin.at.dot(&rot);
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
        verts = arr1(&[pos[0], pos[1]]) - &verts;

        if let Some(end_pos) = &element.endpos {
            let pins = lib_symbol.pins(element.unit)?;
            if pins.len() == 2 {
                let mut verts2: Array1<f64> = pins.get(element.pin as usize).unwrap().at.dot(&rot);
                verts2 = verts2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
                verts2 = arr1(&[pos[0], pos[1]]) - &verts2;
                let sym_len = verts[0] - verts2[0];
                let wire_len = ((end_pos[0] - pos[0]) - sym_len) / 2.0;
                verts = verts + arr1(&[wire_len, 0.0]);
                let mut wire1 = arr2(&[[pos[0], pos[1]], [pos[0] + wire_len, pos[1]]]);
                wire1 = wire1.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                let mut wire2 = arr2(&[
                    [pos[0] + wire_len + sym_len, pos[1]],
                    [pos[0] + 2.0 * wire_len + sym_len, pos[1]],
                ]);
                wire2 = wire2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                self.schema.push(
                    0,
                    SchemaElement::Wire(Wire::new(wire1, Stroke::new(), uuid!())),
                )?;
                self.schema.push(
                    0,
                    SchemaElement::Wire(Wire::new(wire2, Stroke::new(), uuid!())),
                )?;
                self.last_pos = arr1(&[pos[0] + 2.0 * wire_len + sym_len, pos[1]]);
            } else {
                panic!("tox and toy can only be used on symbols with two pins.")
            }
        }

        let mut symbol = Symbol::from_library(
            &lib_symbol,
            round!(verts.clone()),
            element.angle,
            element.unit,
            element.reference.to_string(),
            element.value.to_string(),
        );
        if let Some(mirror) = element.mirror {
            symbol.mirror = vec![mirror];
        }
        if let Some(properties) = element.args {
            symbol.on_schema = if let Some(on_schema) = properties.get("on_schema") {
                on_schema == "yes"
            } else {
                true
            };
            // add the extra properties
            for (k, v) in properties.into_iter() {
                if k != "on_schema" {
                    symbol.property.push(Property::new(
                        k,
                        v,
                        0,
                        round!(verts.clone()),
                        0.0,
                        Some(Effects::hidden()),
                    ));
                }
            }
        }
        self.place_property(&mut symbol).unwrap();
        /* self.symbol_instance.push(SymbolInstance::new(
            symbol.uuid.clone(),
            reference.to_string(),
            unit,
            value.to_string(),
            String::new(), /* TODO footprint.to_string()*/
        )); */
        self.schema.push(0, SchemaElement::Symbol(symbol))?;
        self.get_library(&element.library)?;

        Ok(())
    }

    fn pin_pos(&self, reference: String, number: String) -> Array1<f64> {
        let symbol = self.schema.get_symbol(reference.as_str(), 1).unwrap();
        let library = self.schema.get_library(symbol.lib_id.as_str()).unwrap();
        for subsymbol in &library.symbols {
            for pin in &subsymbol.pin {
                if pin.number.0 == number {
                    //TODO: Type
                    let real_symbol = self
                        .get_symbol(reference.as_str(), subsymbol.unit as u32)
                        .unwrap();
                    return Shape::transform(real_symbol, &pin.at);
                }
            }
        }
        panic!("pin not found {}:{}", reference, number); //TODO return error
    }
    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<LibrarySymbol, Error> {
        if let Some(lib) = self.schema.get_library(name) {
            Ok(lib.clone())
        } else {
            let mut lib = self.libs.get(name).unwrap();
            lib.lib_id = name.to_string();
            self.schema.page(0).unwrap().libraries.push(lib.clone());
            Ok(lib)
        }
    }

    /// get the symbol by reference and unit from this schema.
    fn get_symbol(&self, reference: &str, unit: u32) -> Option<&Symbol> {
        self.schema.get_symbol(reference, unit)
    }

    fn place_property(&mut self, symbol: &mut Symbol) -> Result<(), Error> {
        let vis_field = symbol
            .property
            .iter()
            .filter_map(|node| {
                if let Some(effects) = &node.effects {
                    if !effects.hide {
                        Option::from(node)
                    } else {
                        None
                    }
                } else {
                    Option::from(node)
                }
            })
            .count();
        let lib = self.get_library(&symbol.lib_id).unwrap();

        //get and sort the shape size
        let _size = Shape::transform(symbol, &symbol.bounds(&lib).unwrap());
        let _size = if _size[[0, 0]] > _size[[1, 0]] {
            arr2(&[
                [_size[[1, 0]], _size[[0, 1]]],
                [_size[[0, 0]], _size[[1, 1]]],
            ])
        } else {
            _size
        };
        let _size = if _size[[0, 1]] > _size[[1, 1]] {
            arr2(&[
                [_size[[0, 0]], _size[[1, 1]]],
                [_size[[1, 0]], _size[[0, 1]]],
            ])
        } else {
            _size
        };
        let positions = self.pin_position(symbol, &lib);
        let mut offset = 0.0;
        let pins = lib.pins(symbol.unit)?.len();
        if pins == 1 {
            //PINS!
            if positions[0] == 1 {
                //west
                /* vis_fields[0].pos = (_size[1][0]+1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.LEFT]
                vis_fields[0].angle = 360 - symbol.angle */

                return Ok(());
            } else if positions[3] == 1 {
                //south
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], _size[[1, 1]] + LABEL_BORDER]);
                        p.angle = 0.0 - symbol.angle;
                    });
                return Ok(());
            } else if positions[2] == 1 {
                //east
                todo!();
                /* vis_fields[0].pos = (_size[0][0]-1.28, symbol.pos[1])
                assert vis_fields[0].text_effects, "pin has no text_effects"
                vis_fields[0].text_effects.justify = [Justify.RIGHT]
                vis_fields[0].angle = 360 - symbol.angle */
            } else if positions[1] == 1 {
                //south
                let top_pos = if _size[[0, 1]] > _size[[1, 1]] {
                    _size[[0, 1]] - ((vis_field as f64 - 1.0) * 2.0) + 0.64
                } else {
                    _size[[1, 1]] - ((vis_field as f64 - 1.0) * 2.0) + 0.64
                };
                //symbol.nodes_mut("property")?.iter_mut().for_each(|node| {
                // let props: Vec<&Sexp> = symbol.get("property")?;
                /* props.iter_mut().for_each(|node| {
                    let effects: Vec<&Sexp> = get!(node, "effects").unwrap();
                    if !effects.has("hide") {
                        let mut field_pos: Array1<f64> = get!(node, "at");
                        field_pos[1] = top_pos;
                        node.set("at", field_pos).unwrap();
                        for n in &mut node.values {
                            if let Sexp::Node(name, effects) = n {
                                if name == "effects" {
                                    let index: usize = 0;
                                    for val in effects {
                                        if let Sexp::Node(name, nodes) = val {
                                            if name == "justify" {
                                                effects.remove(index);                                           }
                                        }
                                        index += 1;
                                    }
                                }
                            }
                        }
                        //TODO set!(node, "at", 2, 360.0 - angle);
                        offset += 2.0;
                    }
                }); */
                //vis_fields[0].text_effects.justify = [Justify.CENTER]
                return Ok(());
            }
        } else {
            let top_pos = if _size[[0, 1]] < _size[[1, 1]] {
                _size[[0, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
            } else {
                _size[[1, 1]] - ((vis_field as f64 - 1.0) * LABEL_BORDER) - LABEL_BORDER
            };
            if positions[3] == 0 {
                //north
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                        }
                        p.at = arr1(&[symbol.at[0], top_pos - offset]);
                        p.angle = 0.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });

                return Ok(());
            } else if positions[2] == 0 {
                //east
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                    - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("left"));
                        } else {
                            let mut effects = Effects::new();
                            effects.justify.push(String::from("left"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[0] == 0 {
                //west
                let top_pos = _size[[0, 1]] + ((_size[[1, 1]] - _size[[0, 1]]) / 2.0)
                    - ((vis_field as f64 - 1.0) * LABEL_BORDER) / 2.0;
                symbol
                    .property
                    .iter_mut()
                    .filter(filter_properties)
                    .sorted_by(sort_properties)
                    .for_each(|p| {
                        if let Some(effects) = &mut p.effects {
                            effects.justify.clear();
                            effects.justify.push(String::from("right"));
                        } else {
                            let mut effects = Effects::new();
                            effects.justify.push(String::from("right"));
                            p.effects = Some(effects);
                        }
                        p.at = arr1(&[_size[[1, 0]] - LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[1] == 0 {
                //south
                return Ok(());
            } else {
                return Ok(()); //todo!();
            }
        }
        Err(Error::ParseError)
    }

    /// get the pin position
    /// returns an array containing the number of pins:
    ///   3
    /// 2   0
    ///   1
    fn pin_position(&self, symbol: &Symbol, lib: &LibrarySymbol) -> Vec<usize> {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_shift: usize = (symbol.angle / 90.0).round() as usize;

        for pin in lib.pins(symbol.unit).unwrap() {
            let lib_pos: usize = (pin.angle / 90.0).round() as usize;
            position[lib_pos] += 1;
        }
        position.rotate_right(symbol_shift);
        if symbol.mirror.contains(&String::from('x')) {
            position = vec![position[0], position[3], position[2], position[1]];
        } else if symbol.mirror.contains(&String::from("y")) {
            position = vec![position[2], position[1], position[0], position[3]];
        }
        position
    }
}
