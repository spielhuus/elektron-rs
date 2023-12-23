///Model structs exported to python.
use std::collections::HashMap;

use ndarray::{arr1, Array1};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::{distributions::Alphanumeric, Rng};

fn randid() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

///The Line represents a wire.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Line {
    pub direction: Direction,
    pub length: Option<f64>,
    pub atref: Option<(String, String)>,
    pub atdot: Option<Dot>,
    pub pos: Option<(f64, f64)>,
    pub toxref: Option<(String, String)>,
    pub toyref: Option<(String, String)>,
    pub tox: Option<Array1<f64>>,
    pub toy: Option<Array1<f64>>,
    pub toxdot: Option<String>,
    pub toydot: Option<String>,
    pub dot: Option<Vec<String>>,
    pub points: Option<((f64, f64), (f64, f64))>,
}

#[pymethods]
impl Line {
    #[new]
    fn new() -> Self {
        Line {
            direction: Direction::Right,
            length: None,
            atref: None,
            atdot: None,
            pos: None,
            toxref: None,
            toyref: None,
            tox: None,
            toy: None,
            toxdot: None,
            toydot: None,
            dot: None,
            points: None,
        }
    }
    ///Line direction up.
    pub fn up<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Up;
        slf
    }
    ///Line direction down.
    pub fn down<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Down;
        slf
    }
    ///Line direction left.
    pub fn left<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Left;
        slf
    }
    ///Line direction right.
    pub fn right<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Right;
        slf
    }
    pub fn length<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, len: f64) -> PyRefMut<'py, Self> {
        slf.length = Some(len);
        slf
    }
    ///Draw the line from the position.
    ///
    ///The position can either be a Pin or Dot:
    ///
    ///::
    ///  Line().at("REF", "PIN_NUMBER")
    ///
    ///  dot = Dot()
    ///  Line().at(dot)
    ///
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        }
        let dot: Result<(f64, f64), PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.pos = Some(dot);
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    ///Draw the line to the X position.
    ///
    ///The position can either be a Pin or a Dot.
    ///
    ///::
    ///  Line().tox("REF", "PIN_NUMBER")
    ///
    ///  dot = Dot()
    ///  Line().tox(dot)
    ///
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        /* if let Ok(dot) = dot {
            slf.tox = Some(Array1::from_vec(dot.pos));
            return slf;
        } */
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toxref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        let pos: Result<(f64, f64), PyErr> = element.extract();
        if let Ok(pos) = pos {
            slf.tox = Some(arr1(&[pos.0, pos.1]));
            return slf;
        }
        if let Ok(dot) = dot {
            slf.toxdot = Some(dot.id);
            return slf;
        }
        panic!("unknown type for at: {:?}", element);
    }
    ///Draw the line to the Y position.
    ///
    ///The position can either be a Pin or a Dot.
    ///
    ///::
    ///  Line().toy("REF", "PIN_NUMBER")
    ///
    ///  dot = Dot()
    ///  Line().toy(dot)
    ///
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toyref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        let pos: Result<(f64, f64), PyErr> = element.extract();
        if let Ok(pos) = pos {
            slf.toy = Some(arr1(&[pos.0, pos.1]));
            return slf;
        }
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.toydot = Some(dot.id);
            return slf;
        }
        panic!("unknown type for toy: {:?}:{:?}", element, pin);
    }

    ///Draw a dot at the start or end of the line
    ///
    ///::
    ///  Line().dot(["start", "end"])
    ///
    pub fn dot<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        position: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Vec<String>, PyErr> = position.extract();
        if let Ok(dot) = dot {
            slf.dot = Some(dot);
            return slf;
        }
        let dot: Result<String, PyErr> = position.extract();
        if let Ok(dot) = dot {
            slf.dot = Some(vec![dot]);
            return slf;
        }
        panic!("unknown type for dot: {:?}", dot);
    }
    pub fn start(&self) -> (f64, f64) {
        if let Some(points) = &self.points {
            points.0
        } else {
            panic!("line start: points not set!");
        }
    }
    pub fn end(&self) -> (f64, f64) {
        if let Some(points) = &self.points {
            points.1
        } else {
            panic!("line end: points not set!");
        }
    }
    ///Set the points
    #[pyo3(signature = (*args))]
    pub fn set_points(&mut self, args: ((f64, f64), (f64, f64))) {
        self.points = Some(args);
    }
}

///The Dot for wire junctions.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Dot {
    pub pos: Vec<f64>,
    pub id: String,
    pub atref: Option<(String, String)>,
    pub pushed: bool,
}
#[pymethods]
impl Dot {
    #[new]
    fn new() -> Self {
        Dot {
            pos: vec![0.0, 0.0],
            id: randid(),
            atref: None,
            pushed: false,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn push(mut slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf.pushed = true;
        slf
    }
}

///No Connect, used to stisfy the ERC check.
#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct Nc {
    pub pos: Vec<f64>,
    pub atref: Option<(String, String)>,
}
#[pymethods]
impl Nc {
    #[new]
    pub fn new() -> Self {
        Self {
            pos: vec![0.0, 0.0],
            atref: None,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        /* let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        } */
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
}

///Local label.
#[pyclass]
#[derive(Debug, Clone)]
pub struct Label {
    pub pos: Vec<f64>,
    pub name: String,
    pub angle: f64,
    pub atdot: Option<Dot>,
    pub atref: Option<(String, String)>,
}
#[pymethods]
impl Label {
    #[new]
    pub fn new(name: String) -> Self {
        Label {
            pos: vec![0.0, 0.0],
            name,
            angle: 0.0,
            atdot: None,
            atref: None,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        }
        /* let dot: Result<(f64, f64), PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.pos = Some(dot);
            return slf;
        } */
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn rotate<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        angle: f64,
    ) -> PyRefMut<'py, Self> {
        slf.angle = angle;
        slf
    }
}

///Symbols to draw elements for the Kicad symbol library.
#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct Element {
    pub reference: String,
    pub library: String,
    pub value: String,
    pub unit: u32,
    pub args: Option<HashMap<String, String>>,
    pub angle: f64,
    pub anchor: String,
    pub pos: Option<(f64, f64)>,
    pub atref: Option<(String, String)>,
    pub atdot: Option<Dot>,
    pub toxref: Option<(String, String)>,
    pub toyref: Option<(String, String)>,
    pub tox: Option<Array1<f64>>,
    pub toy: Option<Array1<f64>>,
    pub toxdot: Option<String>,
    pub toydot: Option<String>,
    pub length: Option<f64>,
    pub mirror: Option<String>,
    pub label: Option<String>,
}
#[pymethods]
impl Element {
    #[new]
    #[pyo3(signature = (reference, library, value, unit=1, **kwargs))]
    pub fn new(
        reference: String,
        library: String,
        value: String,
        unit: u32,
        kwargs: Option<&PyDict>,
    ) -> Self {
        let args = if let Some(args) = kwargs {
            let mut myargs: HashMap<String, String> = HashMap::new();
            for (k, v) in args {
                myargs.insert(k.to_string(), v.to_string());
            }
            Some(myargs)
        } else {
            None
        };
        Element {
            reference,
            library,
            value,
            unit,
            args,
            angle: 0.0,
            anchor: String::from("1"),
            pos: None,
            atref: None,
            atdot: None,
            tox: None,
            toy: None,
            toxref: None,
            toyref: None,
            toxdot: None,
            toydot: None,
            length: None,
            mirror: None,
            label: None,
        }
    }

    pub fn anchor<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        pin: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let str_pin: Result<String, PyErr> = pin.extract();
        if let Ok(pin) = str_pin {
            slf.anchor = pin;
            return slf;
        }
        let pin: Result<u32, PyErr> = pin.extract();
        if let Ok(pin) = pin {
            slf.anchor = pin.to_string();
            return slf;
        }
        panic!("unknown type for at: {:?}", pin);
    }

    pub fn rotate<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        angle: f64,
    ) -> PyRefMut<'py, Self> {
        slf.angle = angle;
        slf
    }

    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.atdot = Some(dot);
            return slf;
        }
        let dot: Result<(f64, f64), PyErr> = reference.extract();
        if let Ok(dot) = dot {
            slf.pos = Some(dot);
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn length<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, len: f64) -> PyRefMut<'py, Self> {
        slf.length = Some(len);
        slf
    }
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let pos: Result<(f64, f64), PyErr> = element.extract();
        if let Ok(pos) = pos {
            slf.tox = Some(arr1(&[pos.0, pos.1]));
            return slf;
        }
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toxref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.toxdot = Some(dot.id);
            return slf;
        }
        panic!("unknown type for tox: {:?}", element);
    }
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        /* if let Ok(dot) = dot {
            slf.tox = Some(Array1::from_vec(dot.pos));
            return slf;
        } */
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = element.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toyref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        if let Ok(dot) = dot {
            slf.toydot = Some(dot.id);
            return slf;
        }
        panic!("unknown type for toy: {:?}", element);
    }
    ///mirror the symbol, possible values are x and y.
    pub fn mirror<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        mirror: String,
    ) -> PyRefMut<'py, Self> {
        slf.mirror = Some(mirror);
        slf
    }
    ///place property, possible values are offset tuple or position by name: north, n, northeast, ne...
    pub fn label<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        pos: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {

        let name: Result<String, PyErr> = pos.extract();
        if let Ok(name) = name {
            slf.label = Some(name);
            return slf;
        }
        let offset: Result<(f64, f64), PyErr> = pos.extract();
        if let Ok(offset) = offset {
            slf.label = Some(format!("{},{}", offset.0, offset.1));
            return slf;
        }
        panic!("unknown type for label postion: {:?}", pos);
    }
}

#[pyclass(extends=Element, subclass)]
pub struct R {}

#[pymethods]
impl R {
    #[new]
    pub fn new(reference: String, value: String) -> (Self, Element) {
        (
            R {},
            Element::new(reference, String::from("Device:R"), value, 1, None),
        )
    }
}

#[pyclass(extends=Element, subclass)]
pub struct C {}

#[pymethods]
impl C {
    #[new]
    pub fn new(reference: String, value: String) -> (Self, Element) {
        (
            C {},
            Element::new(reference, String::from("Device:C"), value, 1, None),
        )
    }
}

#[pyclass(extends=Element, subclass)]
pub struct Gnd {}

#[pymethods]
impl Gnd {
    #[new]
    pub fn new() -> (Self, Element) {
        (
            Gnd {},
            Element::new(
                String::from("GND"),
                String::from("power:GND"),
                String::from("GND"),
                1,
                None,
            ),
        )
    }
}

#[pyclass(extends=Element, subclass)]
pub struct Power {}

#[pymethods]
impl Power {
    #[new]
    pub fn new(reference: String) -> (Self, Element) {
        (
            Power {},
            Element::new(
                reference.to_string(),
                format!("power:{}", &reference),
                reference,
                1,
                None,
            ),
        )
    }
}

///Feedback
#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct Feedback {
    pub atref: Option<(String, String)>,
    pub toref: Option<(String, String)>,
    pub with: Option<Element>,
    pub height: f64,
    pub dot: Option<Vec<String>>,
}

#[pymethods]
impl Feedback {
    #[new]
    pub fn new() -> Self {
        Self {
            atref: None,
            toref: None,
            with: None,
            height: 5.0 * 2.54,
            dot: None,
        }
    }
    pub fn start<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for from: {:?}", reference);
    }
    pub fn end<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        reference: &'_ PyAny,
        pin: Option<&'_ PyAny>,
    ) -> PyRefMut<'py, Self> {
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.toref = Some((reference.to_string(), pin));
                return slf;
            }
        }
        panic!("unknown type for to: {:?}", reference);
    }
    pub fn height<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        h: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let h: Result<f64, PyErr> = h.extract();
        if let Ok(h) = h {
            slf.height = h;
            return slf;
        }
        panic!("unknown type for height: {:?}", h);
    }

    ///Draw a dot at the start or end of the line
    ///
    ///::
    ///  Line().dot(["start", "end"])
    ///
    pub fn dot<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        position: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Vec<String>, PyErr> = position.extract();
        if let Ok(dot) = dot {
            slf.dot = Some(dot);
            return slf;
        }
        let dot: Result<String, PyErr> = position.extract();
        if let Ok(dot) = dot {
            slf.dot = Some(vec![dot]);
            return slf;
        }
        panic!("unknown type for dot: {:?}", dot);
    }
}
