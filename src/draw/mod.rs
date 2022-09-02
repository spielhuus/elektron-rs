use crate::circuit::Circuit;
use crate::plot::Theme;
use crate::plot::{PlotIterator, Plotter};
use crate::sexp::model::{
    Effects, Junction, Label, LibrarySymbol, Pin, Property, SchemaElement, SheetInstance, Stroke,
    Symbol, SymbolInstance, TitleBlock, Wire,
};
use crate::sexp::{self, Bounds, Library, Shape, Transform};
use crate::Error;
use crate::{CairoPlotter, ImageType};
use itertools::Itertools;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::env::temp_dir;
use std::fs::{self, File};
use std::io::{Read, Write};

use ndarray::{arr1, arr2, Array1};
use rand::Rng;
use uuid::Uuid;

mod model;

const LABEL_BORDER: f64 = 2.54;
const LABEL_MIN_BORDER: f64 = 5.06;

macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string()
    };
}

macro_rules! round {
    ($val: expr) => {
        $val.iter()
            .map(|v| format!("{:.2}", v).parse::<f64>().unwrap())
            .collect()
    };
}

pub fn get_pin(symbol: &LibrarySymbol, number: i32) -> Result<&Pin, Error> {
    for s in &symbol.symbols {
        for p in &s.pin {
            if p.number.0 == number.to_string() {
                return Ok(p);
            }
        }
    }
    Err(Error::PinNotFound(number as usize)) //TODO: remove cast
}
/// Get all the pins of a library symbol.
pub fn get_pins(symbol: &LibrarySymbol, number: i32) -> Result<Vec<&Pin>, Error> {
    let mut result = Vec::new();
    for s in &symbol.symbols {
        if s.unit == number {
            for p in &s.pin {
                result.push(p);
            }
        }
    }
    Ok(result)
}

fn filter_properties(node: &&mut Property) -> bool {
    !node.effects.hide
}

fn sort_properties(a: &&mut Property, b: &&mut Property) -> std::cmp::Ordering {
    a.id.cmp(&b.id)
}

#[pyclass]
pub struct Draw {
    version: String,
    generator: String,
    uuid: String,
    paper: String,
    title_block: TitleBlock,
    pub elements: Vec<SchemaElement>,
    libraries: HashMap<String, LibrarySymbol>,
    sheet_instance: Vec<SheetInstance>,
    symbol_instance: Vec<SymbolInstance>,
    libs: Library,
}

#[pymethods]
impl Draw {
    #[new]
    pub fn new(library_path: Vec<String>) -> Self {
        Self {
            version: String::from("20211123"),
            generator: String::from("elektron"),
            uuid: uuid!(),
            paper: String::from("A4"),
            title_block: TitleBlock::new(),
            elements: Vec::<SchemaElement>::new(),
            libraries: HashMap::<String, LibrarySymbol>::new(),
            sheet_instance: vec![SheetInstance::new("/", 1)],
            symbol_instance: Vec::<SymbolInstance>::new(),
            libs: Library::new(library_path),
        }
    }

    fn pin_pos(&mut self, reference: &str, number: usize) -> Vec<f64> {
        let symbol = self.get_symbol(reference, 1).unwrap();
        let library = self.get_library(&symbol.lib_id).unwrap();
        for subsymbol in library.symbols {
            for pin in subsymbol.pin {
                if pin.number.0 == number.to_string() {
                    //TODO: Type
                    let real_symbol = self.get_symbol(reference, subsymbol.unit).unwrap();
                    let at: Array1<f64> = Shape::transform(&real_symbol, &pin.at);
                    return vec![at[0], at[1]];
                }
            }
        }
        panic!("pin not found {}:{}", reference, number); //TODO return error
    }
    fn wire(&mut self, start: Vec<f64>, end: Vec<f64>) {
        let pts: Vec<f64> = round!(start);
        let end: Vec<f64> = round!(end);
        self.elements.push(SchemaElement::Wire(Wire::new(
            arr2(&[[pts[0], pts[1]], [end[0], end[1]]]),
            Stroke::new(),
            uuid!(),
        )));
    }
    fn junction(&mut self, pos: Vec<f64>) {
        let pos: Vec<f64> = round!(pos);
        self.elements.push(SchemaElement::Junction(Junction::new(
            arr1(&[pos[0], pos[1]]),
            uuid!(),
        )));
    }
    fn label(&mut self, name: &str, pos: Vec<f64>, angle: f64) {
        let pos: Vec<f64> = round!(pos);
        self.elements.push(SchemaElement::Label(Label::new(
            arr1(&[pos[0], pos[1]]),
            angle,
            name,
            uuid!(),
        )));
    }
    fn symbol(
        &mut self,
        reference: &str,
        value: &str,
        library: &str,
        unit: i32,
        pos: Vec<f64>,
        pin: usize,
        angle: f64,
        mirror: String,
        end_pos: Option<f64>,
        properties: HashMap<String, String>,
    ) {
        let pos: Vec<f64> = round!(pos);
        let lib_symbol = self.get_library(library).unwrap();
        let sym_pin = get_pin(&lib_symbol, pin as i32).unwrap(); //TODO: remove cast

        // transform pin pos
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = sym_pin.at.dot(&rot);
        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
        verts = arr1(&[pos[0], pos[1]]) - &verts;

        if let Some(end_pos) = end_pos {
            let pins = get_pins(&lib_symbol, unit).unwrap();
            if pins.len() == 2 {
                for p in pins {
                    if p.number.0 != unit.to_string() {
                        //TODO: What is that?
                        let mut verts2: Array1<f64> = p.at.dot(&rot);
                        verts2 = verts2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        //TODO verts = verts.dot(sexp::MIRROR.get(mirror.as_str()).unwrap());
                        verts2 = arr1(&[pos[0], pos[1]]) - &verts2;
                        let sym_len = verts[0] - verts2[0];
                        let wire_len = ((end_pos - pos[0]) - sym_len) / 2.0;
                        verts = verts + arr1(&[wire_len, 0.0]);
                        let mut wire1 = arr2(&[[pos[0], pos[1]], [pos[0] + wire_len, pos[1]]]);
                        wire1 = wire1.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        let mut wire2 = arr2(&[
                            [pos[0] + wire_len + sym_len, pos[1]],
                            [pos[0] + 2.0 * wire_len + sym_len, pos[1]],
                        ]);
                        wire2 = wire2.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
                        self.elements.push(SchemaElement::Wire(Wire::new(
                            wire1,
                            Stroke::new(),
                            uuid!(),
                        )));
                        self.elements.push(SchemaElement::Wire(Wire::new(
                            wire2,
                            Stroke::new(),
                            uuid!(),
                        )));
                    }
                }
            }
        }

        verts = verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap());
        let mut symbol = Symbol::from_library(
            &lib_symbol,
            verts.clone(),
            angle,
            unit,
            reference.to_string(),
            value.to_string(),
        );
        symbol.on_schema = if let Some(on_schema) = properties.get("on_schema") {
            on_schema == "yes"
        } else {
            true
        };
        // add the extra properties
        for (k, v) in properties.into_iter() {
            if k != "on_schema" {
                symbol
                    .property
                    .push(Property::new(k, v, 0, verts.clone(), 0.0, Effects::hidden()));
            }
        }
        self.place_property(&mut symbol).unwrap();
        self.symbol_instance.push(SymbolInstance::new(
            symbol.uuid.clone(),
            reference.to_string(),
            unit,
            value.to_string(),
            String::new(), /* TODO footprint.to_string()*/
        ));
        self.elements.push(SchemaElement::Symbol(symbol));
    }

    pub fn write(&mut self, filename: Option<String>) -> Result<(), Error> {
        match self._write() {
            Ok(doc) => {
                if let Some(output) = filename {
                    //check_directory(&output)?;
                    let mut out = File::create(output)?;
                    let iter = doc.into_iter();
                    sexp::write(&mut out, iter)?;
                } else {
                    let iter = doc.into_iter();
                    sexp::write(&mut std::io::stdout(), iter)?;
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
        Ok(())
    }

    pub fn plot(
        &mut self,
        filename: Option<&str>,
        border: bool,
        scale: f64,
    ) -> Result<Vec<u8>, Error> {
        let doc = self._write().unwrap();

        let theme = Theme::kicad_2000();
        let iter = doc.into_iter().plot(theme, border).flatten().collect();

        if let Some(filename) = filename {
            let image_type = if filename.ends_with(".svg") {
                ImageType::Svg
            } else if filename.ends_with(".png") {
                ImageType::Png
            } else {
                ImageType::Pdf
            };

            let mut cairo = CairoPlotter::new(&iter);
            let out: Box<dyn Write> = Box::new(File::create(filename)?);
            cairo.plot(out, border, scale, image_type)?;
            Ok(Vec::new())
        } else {
            let mut rng = rand::thread_rng();
            let num: u32 = rng.gen();
            let filename =
                String::new() + temp_dir().to_str().unwrap() + "/" + &num.to_string() + ".png";
            let mut cairo = CairoPlotter::new(&iter);
            let out: Box<dyn Write> = Box::new(File::create(filename.clone())?);
            cairo.plot(out, border, scale, ImageType::Png)?;
            let mut f = File::open(&filename).expect("no file found");
            let metadata = fs::metadata(&filename).expect("unable to read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            f.read_exact(&mut buffer).expect("buffer overflow");
            Ok(buffer)
            // print_from_file(&filename, &Config::default()).expect("Image printing failed.");
        }
        /* let theme = if let Some(theme) = theme {
            if theme == "kicad_2000" {
                Theme::kicad_2000() //TODO:
            } else {
                Theme::kicad_2000()
            }
        } else {
            Theme::kicad_2000()
        }; */
    }
    pub fn circuit(&mut self, pathlist: Vec<String>) -> Circuit {
        /* let mut circuit: Circuit = Circuit::new("circuit from draw".to_string(), pathlist);
        match self._write() {
            Ok(doc) => {
                let mut netlist = Box::new(Netlist::from(&doc));
                netlist.dump(&mut circuit).unwrap();
                return circuit;
            }
            Err(err) => {
                println!("{:?}", err);
            }
        } */
        panic!();
    }
}

impl Draw {
    /// return a library symbol when it exists or load it from the libraries.
    fn get_library(&mut self, name: &str) -> Result<LibrarySymbol, Error> {
        Ok(self
            .libraries
            .entry(name.to_string())
            .or_insert_with(|| {
                let mut lib = self.libs.get(name).unwrap();
                lib.lib_id = name.to_string();
                lib
            })
            .clone())
    }

    /// get the symbol by reference and unit from this schema.
    fn get_symbol(&mut self, reference: &str, unit: i32) -> Result<Symbol, Error> {
        for l in &self.elements {
            if let SchemaElement::Symbol(symbol) = l {
                let _ref = symbol.get_property("Reference").unwrap();
                if reference == _ref && unit == symbol.unit {
                    return Ok(symbol.clone());
                }
            }
        }
        Err(Error::SymbolNotFound(reference.to_string()))
    }

    fn _write(&mut self) -> Result<Vec<SchemaElement>, Error> {
        let mut doc = Vec::new();
        doc.push(SchemaElement::Version(self.version.clone()));
        doc.push(SchemaElement::Generator(self.generator.clone()));
        doc.push(SchemaElement::Uuid(self.uuid.clone()));
        doc.push(SchemaElement::Paper(self.paper.clone()));
        doc.push(SchemaElement::TitleBlock(self.title_block.clone()));
        doc.push(SchemaElement::LibrarySymbols(self.libraries.clone()));
        doc.append(&mut self.elements.clone());
        doc.push(SchemaElement::SheetInstance(self.sheet_instance.clone()));
        doc.push(SchemaElement::SymbolInstance(self.symbol_instance.clone()));
        Ok(doc)
    }

    fn place_property(&mut self, symbol: &mut Symbol) -> Result<(), Error> {
        let vis_field = symbol
            .property
            .iter()
            .filter_map(|node| {
                if !node.effects.hide {
                    Option::from(node)
                } else {
                    None
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
        let pins = get_pins(&lib, symbol.unit).unwrap().len();
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
                        p.effects.justify.clear();
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
                        p.effects.justify.clear();
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
                        p.effects.justify.clear();
                        p.effects.justify.push(String::from("left"));
                        p.at = arr1(&[_size[[1, 0]] + LABEL_BORDER / 2.0, top_pos - offset]);
                        p.angle = 360.0 - symbol.angle;
                        offset -= LABEL_BORDER;
                    });
                return Ok(());
            } else if positions[0] == 0 {
                //west
                todo!();
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

        for pin in get_pins(self.libraries.get(&symbol.lib_id).unwrap(), symbol.unit).unwrap() {
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
