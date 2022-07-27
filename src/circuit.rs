use crate::{Error, ngspice::{NgSpice, Callbacks, ComplexSlice}};
use std::{fs::{self, File}, fmt::Display, io::Write, collections::HashMap};
use lazy_static::lazy_static;
use regex::Regex;
use pyo3::prelude::*;

/* extern crate plotly;
use plotly::common::Mode;
use plotly::{Plot, Scatter}; */

lazy_static! {
    pub static ref RE_SUBCKT: regex::Regex =
        Regex::new(r"(?i:\.SUBCKT) ([a-zA-Z0-9]*) .*").unwrap();
    pub static ref RE_MODEL: regex::Regex = Regex::new(r"(?i:\.model) ([a-zA-Z0-9]*) .*").unwrap();
    pub static ref RE_INCLUDE: regex::Regex = Regex::new(r"(?i:\.include) (.*)").unwrap();
}
/* extern "C" fn controlled_exit(_arg1: c_int, _arg2: bool, _arg3: bool, _arg4: c_int, _arg5: *mut c_void) -> c_int {
    return 0;
} */

/* static mut responses: Vec<String> = Vec::new();

unsafe extern "C" fn send_char(arg1: *mut c_char, _arg2: c_int, _arg3: *mut c_void) -> c_int {
    let s = CStr::from_ptr(arg1).to_str().expect("could not make string");
    println!("{}", s);
    responses.push(s.to_string());
    return 0;
} */
struct Cb {
    strs: Vec<String>,
}
impl Callbacks for Cb {
    fn send_char(&mut self, s: &str) {
        print!("{}\n", s);
        self.strs.push(s.to_string())
    }
}
enum CircuitItem {
    R(String, String, String, String),
    C(String, String, String, String),
    D(String, String, String, String),
    Q(String, String, String, String, String),
    X(String, Vec<String>, String),
    V(String, String, String, String),
}

#[pyclass]
pub struct Circuit {
    pathlist: Vec<String>,
    libraries: Vec<String>,
    items: Vec<CircuitItem>,
    ngspice: std::sync::Arc<NgSpice<Cb>>,
}

#[pymethods]
impl Circuit {

    /* fn subcircuit(&mut self, circuit: SubCircuit) -> None:
    """
    Add a subcircuit.
    :param circuit: Circuit to add.
    :type circuit: Circuit
    :return: None
    :rtype: None
    """
    self.subcircuits[circuit.name] = circuit */

    pub fn resistor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.items.push(CircuitItem::R(reference, n0, n1, value));
    }

    pub fn capacitor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.items.push(CircuitItem::C(reference, n0, n1, value));
    }

    pub fn diode(&mut self, reference: String, n0: String, n1: String, value: String) {
        /* if value in self.subcircuits:
            pass
        else:
            get_includes(value, self.includes, self.spice_models) */
        self.items.push(CircuitItem::D(reference, n0, n1, value));
    }

    pub fn bjt(&mut self, reference: String, n0: String, n1: String, n2: String, value: String) {
        /* if value in self.subcircuits:
            pass
        else:
            get_includes(value, self.includes, self.spice_models) */
        self.items
            .push(CircuitItem::Q(reference, n0, n1, n2, value));
    }

    pub fn circuit(&mut self, reference: String, n: Vec<String>, value: String) -> Result<(), Error>{
        /* if x.value not in self.subcircuits:
        get_includes(x.value, self.includes, self.spice_models) */
        self.get_includes(&value)?;
        self.items.push(CircuitItem::X(reference, n, value));
        Ok(())
    }

    pub fn voltage(&mut self, reference: String, n1: String, n2: String, value: String) {
        self.items.push(CircuitItem::V(reference, n1, n2, value));
    }

    fn tran(&self) -> HashMap<String, Vec<f64>> {
        let circ = self.to_str().unwrap();
        self.ngspice.circuit(circ).unwrap();
        self.ngspice.command("tran 10u 10ms").unwrap(); //TODO
        let plot = self.ngspice.current_plot().unwrap();
        let res = self.ngspice.all_vecs(plot.as_str()).unwrap();
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = self.ngspice.vector_info(name.as_str());
            for r in re {
                let name = r.name;
                let data1 = match r.data {
                    ComplexSlice::Real(list) => {
                        list.iter().map(|i| i.clone()).collect()
                    },
                    ComplexSlice::Complex(_list) => {
                        //list.into_iter().map(|f| f.parse::<f64>()).collect()
                        println!("found complex list"); //TODO use this result
                        vec![0.0]
                    }
                };
                map.insert(name, data1);
            }
        }
        map
    }

    fn save(&self, filename: Option<String>) -> PyResult<()> {

        let mut out: Box<dyn Write> = if let Some(filename) = filename {
            Box::new(File::create(filename).unwrap())
        } else {
            Box::new(std::io::stdout())
        };
        for lib in &self.libraries {
            writeln!(out, ".include {}", lib)?;
        }
        for item in &self.items {
            match item {
                CircuitItem::R(reference, n0, n1, value) => {
                    writeln!(out, "R{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::C(reference, n0, n1, value) => {
                    writeln!(out, "C{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::D(reference, n0, n1, value) => {
                    writeln!(out, "{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::Q(reference, n0, n1, n2, value) => {
                    writeln!(out, "Q{} {} {} {} {}", reference, n0, n1, n2, value)?;
                },
                CircuitItem::X(reference, n, value) => {
                    let mut nodes: String = String::new();
                    for _n in n {
                        nodes += _n;
                        nodes += " ";
                    };
                    writeln!(out, "X{} {}{}", reference, nodes, value)?;
                },
                CircuitItem::V(reference, n0, n1, value) => {
                    writeln!(out, "V{} {} {} {}", reference, n0, n1, value)?;
                },
            }
        }
        out.flush()?;
        Ok(())
    }
    /* fn plot(&self, name: &str, filename: Option<&str>) {

        let plot = self.ngspice.current_plot().unwrap();
        let vecs = self.ngspice.all_vecs(&plot).unwrap();
        for vec in vecs {
            if let Ok(vecinfo) = self.ngspice.vector_info(&format!("{}.{}", plot, vec)) {
                println!("{} {:?}", vec, vecinfo);
            }
        }
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
        let trace1 = Scatter::new(data1, data2)
            .name("trace1")
            .mode(Mode::Markers);

        let mut plot = Plot::new();
        plot.add_trace(trace1);
        /* plot.add_trace(trace2);
        plot.add_trace(trace3); */
        plot.show();
    } */
}

impl Circuit {
    pub fn new(pathlist: Vec<String>) -> Self {

        let c =  Cb { strs: Vec::new() };
        Self {
            pathlist,
            libraries: Vec::new(),
            items: Vec::new(),
            ngspice: NgSpice::new(c).unwrap(),
        }
    }
    fn get_includes(&mut self, key: &String) -> Result<(), Error> {
        for path in &self.pathlist {
            for entry in fs::read_dir(path).unwrap() {
                let dir = entry.unwrap();
                if dir.path().is_file() {
                    let content = fs::read_to_string(dir.path())?;
                    let captures = RE_SUBCKT.captures(&content);
                    if let Some(caps) = captures {
                        let text1 = caps.get(1).map_or("", |m| m.as_str());
                        if text1 == key {
                            self.libraries.push(dir.path().to_str().unwrap().to_string());
                            let captures = RE_INCLUDE.captures(&content);
                            if let Some(caps) = captures {
                                for cap in caps.iter().skip(1) {
                                    let text1 = cap.map_or("", |m| m.as_str());
                                    if !text1.contains("/") { //when there is no slash i could be
                                                              //a relative path.
                                        let mut parent = dir.path().parent().unwrap().to_str().unwrap().to_string();
                                        parent += "/";
                                        parent += &text1.to_string();
                                        self.libraries.push(parent.to_string());
                                    } else {
                                        self.libraries.push(text1.to_string());
                                    }
                                }
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(Error::SpiceModelNotFound(key.to_string()))
    }

    fn to_str(&self) -> Result<Vec<String>, Error> {
        let mut res = Vec::new();
        for lib in &self.libraries {
            res.push(format!(".include {}", lib));
        }
        for item in &self.items {
            match item {
                CircuitItem::R(reference, n0, n1, value) => {
                    res.push(format!("R{} {} {} {}", reference, n0, n1, value));
                },
                CircuitItem::C(reference, n0, n1, value) => {
                    res.push(format!("C{} {} {} {}", reference, n0, n1, value));
                },
                CircuitItem::D(reference, n0, n1, value) => {
                    res.push(format!("{} {} {} {}", reference, n0, n1, value));
                },
                CircuitItem::Q(reference, n0, n1, n2, value) => {
                    res.push(format!("Q{} {} {} {} {}", reference, n0, n1, n2, value));
                },
                CircuitItem::X(reference, n, value) => {
                    let mut nodes: String = String::new();
                    for _n in n {
                        nodes += _n;
                        nodes += " ";
                    };
                    res.push(format!("X{} {}{}", reference, nodes, value));
                },
                CircuitItem::V(reference, n0, n1, value) => {
                    res.push(format!("V{} {} {} {}", reference, n0, n1, value));
                },
            }
        }
        res.push(String::from(".end"));
        /* res.push(CString::new("").unwrap().into_raw());
        res.push(std::ptr::null_mut()); */
        Ok(res)
    }
}

impl Display for Circuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for lib in &self.libraries {
            writeln!(f, ".include {}", lib)?;
        }
        for item in &self.items {
            match item {
                CircuitItem::R(reference, n0, n1, value) => {
                    writeln!(f, "R{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::C(reference, n0, n1, value) => {
                    writeln!(f, "C{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::D(reference, n0, n1, value) => {
                    writeln!(f, "{} {} {} {}", reference, n0, n1, value)?;
                },
                CircuitItem::Q(reference, n0, n1, n2, value) => {
                    writeln!(f, "Q{} {} {} {} {}", reference, n0, n1, n2, value)?;
                },
                CircuitItem::X(reference, n, value) => {
                    let mut nodes: String = String::new();
                    for _n in n {
                        nodes += _n;
                        nodes += " ";
                    };
                    writeln!(f, "X{} {}{}", reference, nodes, value)?;
                },
                CircuitItem::V(reference, n0, n1, value) => {
                    writeln!(f, "V{} {} {} {}", reference, n0, n1, value)?;
                },
            }
        }
        Ok(())
    }
}
