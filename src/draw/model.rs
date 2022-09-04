use std::collections::HashMap;

use ndarray::{arr1, Array1};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[pyclass(subclass)]
#[derive(Debug, Clone)]
pub struct DrawBase {
    pub atref: Option<String>,
    pub atpin: Option<String>,
}
#[pymethods]
impl DrawBase {
    #[new]
    fn new() -> Self {
        DrawBase {
            atref: None,
            atpin: None,
        }
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        py: Python,
        reference: &'_ PyAny,
        pin: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = reference.extract();
        if let Ok(dot) = dot {
            return slf;
        }
        let reference: Result<String, PyErr> = reference.extract();
        let pin: Result<String, PyErr> = pin.extract();
        if let (Ok(reference), Ok(pin)) = (&reference, pin) {
            slf.atref = Some(reference.to_string());
            slf.atpin = Some(pin);
            return slf;
        }
        panic!("unknown type for at: {:?}", reference);
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Line {
    pub direction: Direction,
    pub length: f64,
    pub atref: Option<String>,
    pub atpin: Option<String>,
    pub atdot: Option<Dot>,
    pub tox: Option<Array1<f64>>,
    pub toy: Option<Array1<f64>>,
}
#[pymethods]
impl Line {
    #[new]
    fn new() -> Self {
        //(Self, DrawBase) {
        // (Line { direction: String::from("left"), length: 2.54 }, DrawBase::new())
        Line {
            direction: Direction::Right,
            length: 2.54,
            atref: None,
            atpin: None,
            atdot: None,
            tox: None,
            toy: None,
        }
    }
    pub fn up<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Up;
        slf
    }
    pub fn down<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Down;
        slf
    }
    pub fn left<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Left;
        slf
    }
    pub fn right<'py>(mut slf: PyRefMut<'py, Self>, _py: Python) -> PyRefMut<'py, Self> {
        slf.direction = Direction::Right;
        slf
    }
    pub fn length<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, len: f64) -> PyRefMut<'py, Self> {
        slf.length = len;
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
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.tox = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        /* if let Some(pin) = pin {
        let reference: Result<String, PyErr> = reference.extract();
        let pin: Result<String, PyErr> = pin.extract();
        if let (Ok(reference), Ok(pin)) = (&reference, pin) {
         slf.atref = Some(reference.to_string());
         slf.atpin = Some(pin);
         return slf;
        }
        } */
        panic!("unknown type for at: {:?}", element);
    }
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.toy = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        /* if let Some(pin) = pin {
        let reference: Result<String, PyErr> = reference.extract();
        let pin: Result<String, PyErr> = pin.extract();
        if let (Ok(reference), Ok(pin)) = (&reference, pin) {
         slf.atref = Some(reference.to_string());
         slf.atpin = Some(pin);
         return slf;
        }
        } */
        panic!("unknown type for at: {:?}", element);
    }
    /* pub fn tox<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, element: f64) -> PyRefMut<'py, Self> {
         slf.x_pos = element;
         slf
    }
    pub fn toy<'py>(mut slf: PyRefMut<'py, Self>, _py: Python, element: f64) -> PyRefMut<'py, Self> {
         slf.x_pos = element;
         slf
    } */
}

#[pyclass(extends=DrawBase)]
#[derive(Debug, Clone)]
pub struct Dot {
    pub pos: Vec<f64>,
}
#[pymethods]
impl Dot {
    #[new]
    fn new() -> (Self, DrawBase) {
        (
            Dot {
                pos: vec![0.0, 0.0],
            },
            DrawBase::new(),
        )
    }
    pub fn at<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        pos: Vec<f64>,
    ) -> PyRefMut<'py, Self> {
        slf.pos = pos;
        slf
    }
}
#[pyclass(extends=DrawBase)]
#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub angle: f64,
}
#[pymethods]
impl Label {
    #[new]
    pub fn new(name: String) -> (Self, DrawBase) {
        (Label { name, angle: 0.0 }, DrawBase::new())
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

#[pyclass(extends=DrawBase)]
#[derive(Debug, Clone)]
pub struct Element {
    pub reference: String,
    pub library: String,
    pub value: String,
    pub unit: u32,
    pub args: Option<HashMap<String, String>>,
    pub angle: f64,
    pub pin: u32,
    pub atref: Option<String>,
    pub atpin: Option<String>,
    pub atdot: Option<Dot>,
    pub endpos: Option<Array1<f64>>,
}
#[pymethods]
impl Element {
    #[new]
    #[args(kwargs = "**")]
    pub fn new(
        reference: String,
        library: String,
        value: String,
        unit: u32,
        kwargs: Option<&PyDict>,
    ) -> (Self, DrawBase) {
        let args = if let Some(args) = kwargs {
            let mut myargs: HashMap<String, String> = HashMap::new();
            for (k, v) in args {
                myargs.insert(k.to_string(), v.to_string());
            }
            Some(myargs)
        } else {
            None
        };
        (
            Element {
                reference,
                library,
                value,
                unit,
                args,
                angle: 0.0,
                pin: 1,
                atref: None,
                atpin: None,
                atdot: None,
                endpos: None,
            },
            DrawBase::new(),
        )
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
        if let Some(pin) = pin {
            let reference: Result<String, PyErr> = reference.extract();
            let pin: Result<String, PyErr> = pin.extract();
            if let (Ok(reference), Ok(pin)) = (&reference, pin) {
                slf.atref = Some(reference.to_string());
                slf.atpin = Some(pin);
                return slf;
            }
        }
        panic!("unknown type for at: {:?}", reference);
    }
    pub fn tox<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.endpos = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        /* if let Some(pin) = pin {
        let reference: Result<String, PyErr> = reference.extract();
        let pin: Result<String, PyErr> = pin.extract();
        if let (Ok(reference), Ok(pin)) = (&reference, pin) {
         slf.atref = Some(reference.to_string());
         slf.atpin = Some(pin);
         return slf;
        }
        } */
        panic!("unknown type for at: {:?}", element);
    }
    pub fn toy<'py>(
        mut slf: PyRefMut<'py, Self>,
        _py: Python,
        element: &'_ PyAny,
    ) -> PyRefMut<'py, Self> {
        let dot: Result<Dot, PyErr> = element.extract();
        if let Ok(dot) = dot {
            slf.endpos = Some(Array1::from_vec(dot.pos));
            return slf;
        }
        /* if let Some(pin) = pin {
        let reference: Result<String, PyErr> = reference.extract();
        let pin: Result<String, PyErr> = pin.extract();
        if let (Ok(reference), Ok(pin)) = (&reference, pin) {
         slf.atref = Some(reference.to_string());
         slf.atpin = Some(pin);
         return slf;
        }
        } */
        panic!("unknown type for at: {:?}", element);
    }
}
