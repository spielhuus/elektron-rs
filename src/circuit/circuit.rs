#![allow(clippy::borrow_deref_ref)]
use crate::{
    error::Error,
    spice::{Callbacks, ComplexSlice, NgSpice},
};
use lazy_static::lazy_static;
use pyo3::prelude::*;
use regex::Regex;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
};

use plotters::prelude::*;

lazy_static! {
    pub static ref RE_SUBCKT: regex::Regex =
        Regex::new(r"(?i:\.SUBCKT) ([a-zA-Z0-9]*) .*").unwrap();
    pub static ref RE_MODEL: regex::Regex = Regex::new(r"(?i:\.model) ([a-zA-Z0-9]*) .*").unwrap();
    pub static ref RE_INCLUDE: regex::Regex = Regex::new(r"(?i:\.include) (.*)").unwrap();
}

struct Cb {
    strs: Vec<String>,
}
impl Callbacks for Cb {
    fn send_char(&mut self, s: &str) {
        if std::env::var("ELEKTRON_DEBUG").is_ok() {
            println!("{}", s);
        }
        self.strs.push(s.to_string())
    }
}
#[derive(Debug, Clone, PartialEq)]
enum CircuitItem {
    R(String, String, String, String),
    C(String, String, String, String),
    D(String, String, String, String),
    Q(String, String, String, String, String),
    X(String, Vec<String>, String),
    V(String, String, String, String),
}

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct Circuit {
    name: String,
    pathlist: Vec<String>,
    items: Vec<CircuitItem>,
    subcircuits: HashMap<String, (Vec<String>, Circuit)>,
}

#[pymethods]
impl Circuit {
    #[new]
    pub fn new(name: String, pathlist: Vec<String>) -> Self {
        Self {
            name,
            pathlist,
            items: Vec::new(),
            subcircuits: HashMap::new(),
        }
    }

    pub fn resistor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.items.push(CircuitItem::R(reference, n0, n1, value));
    }

    pub fn capacitor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.items.push(CircuitItem::C(reference, n0, n1, value));
    }

    pub fn diode(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.items.push(CircuitItem::D(reference, n0, n1, value));
    }

    pub fn bjt(&mut self, reference: String, n0: String, n1: String, n2: String, value: String) {
        self.items
            .push(CircuitItem::Q(reference, n0, n1, n2, value));
    }

    pub fn circuit(
        &mut self,
        reference: String,
        n: Vec<String>,
        value: String,
    ) -> Result<(), Error> {
        //TODO self.get_includes(&value)?;
        self.items.push(CircuitItem::X(reference, n, value));
        Ok(())
    }
    pub fn subcircuit(
        &mut self,
        name: String,
        n: Vec<String>,
        circuit: Circuit,
    ) -> Result<(), Error> {
        self.subcircuits.insert(name, (n, circuit));
        Ok(())
    }
    pub fn voltage(&mut self, reference: String, n1: String, n2: String, value: String) {
        self.items.push(CircuitItem::V(reference, n1, n2, value));
    }
    pub fn save(&self, filename: Option<String>) -> Result<(), Error> {
        let mut out: Box<dyn Write> = if let Some(filename) = filename {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        for s in self.to_str(true).unwrap() {
            writeln!(out, "{}", s)?;
        }
        out.flush()?;
        Ok(())
    }
    pub fn set_value(&mut self, reference: &str, value: &str) -> Result<(), Error> {
        for item in &mut self.items.iter_mut() {
            match item {
                CircuitItem::R(r, _, _, ref mut v) => {
                    if reference == r {
                        *v = value.to_string();
                        return Ok(());
                    }
                }
                CircuitItem::C(r, _, _, ref mut v) => {
                    if reference == r {
                        *v = value.to_string();
                        return Ok(());
                    }
                }
                CircuitItem::D(r, _, _, ref mut v) => {
                    if reference == r {
                        *v = value.to_string();
                        return Ok(());
                    }
                }
                CircuitItem::Q(_, _, _, _, _) => {}
                CircuitItem::X(_, _, _) => {}
                CircuitItem::V(r, _, _, ref mut v) => {
                    if reference == r {
                        *v = value.to_string();
                        return Ok(());
                    }
                }
            }
        }
        Err(Error::UnknownCircuitElement(reference.to_string()))
    }
}

impl Circuit {
    fn get_includes(&self, key: String) -> Result<HashMap<String, String>, Error> {
        let mut result: HashMap<String, String> = HashMap::new();
        for path in &self.pathlist {
            for entry in fs::read_dir(path).unwrap() {
                let dir = entry.unwrap();
                if dir.path().is_file() {
                    let content = fs::read_to_string(dir.path())?;
                    let captures = RE_SUBCKT.captures(&content);
                    if let Some(caps) = captures {
                        let text1 = caps.get(1).map_or("", |m| m.as_str());
                        if text1 == key {
                            result.insert(key, dir.path().to_str().unwrap().to_string());
                            let captures = RE_INCLUDE.captures(&content);
                            if let Some(caps) = captures {
                                for cap in caps.iter().skip(1) {
                                    let text1 = cap.map_or("", |m| m.as_str());
                                    if !text1.contains('/') {
                                        //when there is no slash i could be
                                        //a relative path.
                                        let mut parent = dir
                                            .path()
                                            .parent()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string();
                                        parent += "/";
                                        parent += text1;
                                        result.insert(text1.to_string(), parent.to_string());
                                    } else {
                                        result.insert(text1.to_string(), text1.to_string());
                                    }
                                }
                            }
                            return Ok(result);
                        }
                    }
                }
            }
        }
        Err(Error::SpiceModelNotFound(key))
    }

    fn includes(&self) -> Vec<String> {
        let mut includes: HashMap<String, String> = HashMap::new();
        for item in &self.items {
            if let CircuitItem::X(_, _, value) = item {
                if !includes.contains_key(value) && !self.subcircuits.contains_key(value) {
                    let incs = self.get_includes(value.to_string()).unwrap();
                    for (key, value) in incs {
                        includes.entry(key).or_insert(value);
                    }
                }
            }
        }
        let mut result = Vec::new();
        for (_, v) in includes {
            result.push(format!(".include {}\n", v).to_string());
        }
        result
    }

    fn to_str(&self, close: bool) -> Result<Vec<String>, Error> {
        let mut res = Vec::new();
        res.append(&mut self.includes());
        for (key, value) in &self.subcircuits {
            let nodes = value.0.join(" ");
            res.push(format!(".subckt {} {}", key, nodes));
            res.append(&mut value.1.to_str(false).unwrap());
            res.push(".ends".to_string());
        }
        for item in &self.items {
            match item {
                CircuitItem::R(reference, n0, n1, value) => {
                    if reference.starts_with('R') {
                        res.push(format!("{} {} {} {}", reference, n0, n1, value));
                    } else {
                        res.push(format!("R{} {} {} {}", reference, n0, n1, value));
                    }
                }
                CircuitItem::C(reference, n0, n1, value) => {
                    if reference.starts_with('C') {
                        res.push(format!("{} {} {} {}", reference, n0, n1, value));
                    } else {
                        res.push(format!("C{} {} {} {}", reference, n0, n1, value));
                    }
                }
                CircuitItem::D(reference, n0, n1, value) => {
                    if reference.starts_with('D') {
                        res.push(format!("{} {} {} {}", reference, n0, n1, value));
                    } else {
                        res.push(format!("D{} {} {} {}", reference, n0, n1, value));
                    }
                }
                CircuitItem::Q(reference, n0, n1, n2, value) => {
                    res.push(format!("Q{} {} {} {} {}", reference, n0, n1, n2, value));
                }
                CircuitItem::X(reference, n, value) => {
                    let mut nodes: String = String::new();
                    for _n in n {
                        nodes += _n;
                        nodes += " ";
                    }
                    res.push(format!("X{} {}{}", reference, nodes, value));
                }
                CircuitItem::V(reference, n0, n1, value) => {
                    res.push(format!("V{} {} {} {}", reference, n0, n1, value));
                }
            }
        }
        //TODO add options
        if close {
            res.push(String::from(".end"));
        }
        Ok(res)
    }
}

#[pyclass]
pub struct Simulation {
    circuit: Circuit,
    ngspice: std::sync::Arc<NgSpice<Cb>>,
}

/// simulate the circuit with ngspice
/// TODO circuit models are imported twice
/// TODO create simulatio file
#[pymethods]
impl Simulation {
    /* fn subcircuit(&mut self, circuit: SubCircuit) -> None:
    """
    Add a subcircuit.
    :param circuit: Circuit to add.
    :type circuit: Circuit
    :return: None
    :rtype: None
    """
    self.subcircuits[circuit.name] = circuit */

    /* pub fn add_subcircuit(&mut self, name: &str, circuit: Circuit) {
        self.subcircuit.insert(name.to_string(), circuit);
    } */

    #[new]
    pub fn new(circuit: Circuit) -> Self {
        let c = Cb { strs: Vec::new() };
        Self {
            circuit,
            ngspice: NgSpice::new(c).unwrap(),
        }
    }
    fn tran(&self, step: &str, stop: &str, start: &str) -> HashMap<String, Vec<f64>> {
        let circ = self.circuit.to_str(true).unwrap();
        self.ngspice.circuit(circ).unwrap();
        self.ngspice
            .command(format!("tran {} {} {}", step, stop, start).as_str())
            .unwrap(); //TODO
        let plot = self.ngspice.current_plot().unwrap();
        let res = self.ngspice.all_vecs(plot.as_str()).unwrap();
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = self.ngspice.vector_info(name.as_str());
            if let Ok(r) = re {
                let name = r.name;
                let data1 = match r.data {
                    ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                    ComplexSlice::Complex(_list) => {
                        //list.into_iter().map(|f| f.parse::<f64>()).collect()
                        println!("found complex list"); //TODO use this result
                        vec![0.0]
                    }
                };
                map.insert(name, data1);
            } else {
                panic!("Can not run tran with schema.");
            }
        }
        map
    }

    pub fn plot(&self, name: &str, filename: Option<&str>) -> Result<(), Error> {

        let plot = self.ngspice.current_plot().unwrap();
        let vecs = self.ngspice.all_vecs(&plot).unwrap();
        let re = self.ngspice.vector_info("time").unwrap();
        let data1 = match re.data {
            ComplexSlice::Real(list) => {
                list
            },
            ComplexSlice::Complex(list) => {
                //list.into_iter().map(|f| f.parse::<f64>()).collect()
                &[0.0]
            }
        };
        let re = self.ngspice.vector_info("input").unwrap();
        let data2 = match re.data {
            ComplexSlice::Real(list) => {
                list
            },
            ComplexSlice::Complex(list) => {
                //list.into_iter().map(|f| f.parse::<f64>()).collect()
                &[0.0]
            }
        };
        let re = self.ngspice.vector_info("output").unwrap();
        let data3 = match re.data {
            ComplexSlice::Real(list) => {
                list
            },
            ComplexSlice::Complex(list) => {
                //list.into_iter().map(|f| f.parse::<f64>()).collect()
                &[0.0]
            }
        };


    let root = BitMapBackend::new("0.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("y=x^2", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..0.01f32, -5f32..5f32).unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            data1.iter().zip(data2.iter()).map(|(x, y)| (*x as f32, *y as f32)),
            &RED,
        )).unwrap()
        .label("y = x^2");

    chart
        .draw_series(LineSeries::new(
            data1.iter().zip(data3.iter()).map(|(x, y)| (*x as f32, *y as f32)),
            &BLUE,
        )).unwrap()
        .label("y = x^2");
        // .legend(|(x, y)| LineSeries::new(data2.iter().map(|x| *x as f32), &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw().unwrap();

    root.present().unwrap();

    Ok(())
    }
}
