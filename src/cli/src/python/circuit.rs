#![allow(clippy::borrow_deref_ref)]
use std::collections::HashMap;

use simulation::Circuit as SpiceCircuit;
use simulation::Simulation as SpiceSimulation;
use pyo3::{exceptions::PyOSError, prelude::*};

use crate::error::Error;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct Circuit {
    pub circuit: SpiceCircuit,
}

#[pymethods]
impl Circuit {
    #[new]
    pub fn new(name: String, pathlist: Vec<String>) -> Self {
        Self {
            circuit: SpiceCircuit::new(name, pathlist),
        }
    }

    fn __str__(&self) -> String {
        self.circuit.to_str(true).unwrap().join("\n")
    }

    pub fn resistor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.circuit.resistor(reference, n0, n1, value);
    }

    pub fn capacitor(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.circuit.capacitor(reference, n0, n1, value);
    }

    pub fn diode(&mut self, reference: String, n0: String, n1: String, value: String) {
        self.circuit.diode(reference, n0, n1, value);
    }

    pub fn bjt(&mut self, reference: String, n0: String, n1: String, n2: String, value: String) {
        self.circuit.bjt(reference, n0, n1, n2, value);
    }

    pub fn circuit(
        &mut self,
        reference: String,
        n: Vec<String>,
        value: String,
    ) -> Result<(), Error> {
        self.circuit.circuit(reference, n, value).unwrap();
        Ok(())
    }
    pub fn subcircuit(
        &mut self,
        name: String,
        n: Vec<String>,
        circuit: Circuit,
    ) -> Result<(), Error> {
        self.circuit.subcircuit(name, n, circuit.circuit).unwrap();
        Ok(())
    }
    pub fn voltage(&mut self, reference: String, n1: String, n2: String, value: String) {
        self.circuit.voltage(reference, n1, n2, value);
    }
    pub fn option(&mut self, option: String, value: String) {
        self.circuit.option(option, value);
    }
    pub fn control(&mut self, control: String) {
        self.circuit.control(control);
    }
    pub fn save(&self, filename: Option<String>) -> Result<(), Error> {
        self.circuit.save(filename).unwrap();
        Ok(())
    }
    pub fn set_value(&mut self, reference: &str, value: &str) -> Result<(), Error> {
        self.circuit.set_value(reference, value).unwrap();
        Ok(())
    }
}

#[pyclass]
pub struct Simulation {
    simulation: SpiceSimulation,
}

/// simulate the circuit with ngspice
#[pymethods]
impl Simulation {
    #[new]
    pub fn new(circuit: Circuit) -> Self {
        Self {
            simulation: SpiceSimulation::new(circuit.circuit),
        }
    }

    pub fn run(&self) -> PyResult<HashMap<String, HashMap<String, Vec<f64>>>> {
        match self.simulation.run() {
            Ok(buffer) => Ok(buffer),
            Err(err) => Err(PyOSError::new_err(err.to_string())),
        }
    }

    pub fn op(&mut self, py: Python) -> PyResult<HashMap<String, Vec<f64>>> {
        let res = self.simulation.op();
        if let Ok(res) = res {
            if let Some(buffer) = &self.simulation.buffer {
                let mut res_string = Vec::new();
                for line in buffer {
                    let line = line.replace('\r', "\\n");
                    let line = line.replace('\"', "\\\"");
                    res_string.push(line.replace('\'', "\\\'"));
                }
                py.eval(
                    format!("print('{}')", res_string.join("\\n")).as_str(),
                    None,
                    None,
                )
                .unwrap();
                Ok(res)
            } else {
                Err(PyOSError::new_err(String::from("no data found.")))
            }
        } else if let Err(err) = res {
            Err(PyOSError::new_err(err.to_string()))
        } else {
            Err(PyOSError::new_err(String::from("unknown error")))
        }
    }

    pub fn tran(
        &mut self,
        py: Python,
        step: &str,
        stop: &str,
        start: &str,
    ) -> PyResult<HashMap<String, Vec<f64>>> {
        let res = self.simulation.tran(step, stop, start);
        if let Ok(res) = res {
            if let Some(buffer) = &self.simulation.buffer {
                let mut res_string = Vec::new();
                for line in buffer {
                    let line = line.replace('\r', "\\n");
                    let line = line.replace('\"', "\\\"");
                    res_string.push(line.replace('\'', "\\\'"));
                }
                py.eval(
                    format!("print('{}')", res_string.join("\\n")).as_str(),
                    None,
                    None,
                )
                .unwrap();
                Ok(res)
            } else {
                Err(PyOSError::new_err(String::from("no data found.")))
            }
        } else if let Err(err) = res {
            Err(PyOSError::new_err(err.to_string()))
        } else {
            Err(PyOSError::new_err(String::from("unknown error")))
        }
    }

    pub fn ac(
        &mut self,
        py: Python,
        start_frequency: &str,
        stop_frequency: &str,
        number_of_points: u32,
        variation: &str,
    ) -> PyResult<HashMap<String, Vec<f64>>> {
        let res = self
            .simulation
            .ac(start_frequency, stop_frequency, number_of_points, variation);
        if let Ok(res) = res {
            if let Some(buffer) = &self.simulation.buffer {
                let mut res_string = Vec::new();
                for line in buffer {
                    let line = line.replace('\r', "");
                    let line = line.replace('\"', "\\\"");
                    res_string.push(line.replace('\'', "\\\'"));
                }
                py.eval(
                    format!("print('{}')", res_string.join("\\n")).as_str(),
                    None,
                    None,
                )
                .unwrap();
                Ok(res)
            } else {
                Err(PyOSError::new_err(String::from("no data found.")))
            }
        } else if let Err(err) = res {
            Err(PyOSError::new_err(err.to_string()))
        } else {
            Err(PyOSError::new_err(String::from("unknown error")))
        }
    }
}
