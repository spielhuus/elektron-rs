use std::collections::HashMap;

use elektron_ngspice::{NgSpice, ComplexSlice, Callbacks};

use crate::error::Error;

use super::Circuit;

pub struct Cb {
    strs: Vec<String>,
    status: i32,
    unload: bool,
    quit: bool,
}

impl Cb {
    /// Creates a new Callback struct.
    pub fn new() -> Self {
        Self {
            strs: Vec::new(),
            status: 0,
            unload: false,
            quit: false,
        }
    }
}

impl Default for Cb {
    fn default() -> Self {
        Self::new()
    }
}

impl Callbacks for Cb {
    fn send_char(&mut self, s: &str) {
        if std::env::var("ELEKTRON_DEBUG").is_ok() {
            println!("{}", s);
        }
        self.strs.push(s.to_string())
    }
    fn controlled_exit(&mut self, status: i32, unload: bool, quit: bool) {
        self.status = status;
        self.unload = unload;
        self.quit = quit;
    }
}

pub struct Simulation {
    pub circuit: Circuit,
    pub buffer: Option<Vec<String>>,
}

/// simulate the circuit with ngspice
/// TODO circuit models are imported twice
/// TODO create simulatio file
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

    pub fn new(circuit: Circuit) -> Self {
        Self {
            circuit,
            buffer: None,
        }
    }

    pub fn run(&self) -> Result<HashMap<String, HashMap<String, Vec<f64>>>, Error> {
        let mut cb = Cb::new();
        let ng = NgSpice::new(&mut cb)?;

        ng.circuit(self.circuit.to_str(true).unwrap())?;
        for c in &self.circuit.controls {
            ng.command(c)?;
        }
        let mut plot_result: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        for plot in ng.all_plots()? {
            let vecs = ng.all_vecs(&plot)?;
            let mut vec_values: HashMap<String, Vec<f64>> = HashMap::new();
            for v in vecs {
                let vals = ng.vector_info(format!("{}.{}", plot, &v).as_str())?;
                let data1 = match vals.data {
                    ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                    ComplexSlice::Complex(list) => list
                        .iter()
                        .map(|f| {
                            if !f.cx_real.is_nan() {
                                f.cx_real
                            } else if !f.cx_imag.is_nan() {
                                f.cx_imag
                            } else {
                                todo!("can not get value from complex: {:?}", f);
                            }
                        })
                        .collect(),
                };
                vec_values.insert(v, data1);
            }
            plot_result.insert(plot, vec_values);
        }
        Ok(plot_result)
    }

    pub fn op(
        &mut self,
    ) -> Result<HashMap<String, Vec<f64>>, Error> {
        let mut c = Cb::new();
        let ngspice = NgSpice::new(&mut c)?;
        let circ = self.circuit.to_str(true)?;
        ngspice.circuit(circ)?;
        ngspice.command("op")?;
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str())?;
            let name = re.name;
            let data1 = match re.data {
                ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                ComplexSlice::Complex(list) => list
                    .iter()
                    .map(|f| {
                        if !f.cx_real.is_nan() {
                            f.cx_real
                        } else if !f.cx_imag.is_nan() {
                            f.cx_imag
                        } else {
                            todo!("can not get value from complex: {:?}", f);
                        }
                    })
                    .collect(),
            };
            map.insert(name, data1);
        }
        self.buffer = Some(c.strs.clone());
        Ok(map)
    }

    pub fn tran(
        &mut self,
        step: &str,
        stop: &str,
        start: &str,
    ) -> Result<HashMap<String, Vec<f64>>, Error> {
        let mut c = Cb::new();
        let ngspice = NgSpice::new(&mut c)?;
        let circ = self.circuit.to_str(true)?;
        ngspice.circuit(circ)?;
        ngspice.command(format!("tran {} {} {}", step, stop, start).as_str())?;
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str())?;
            let name = re.name;
            let data1 = match re.data {
                ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                ComplexSlice::Complex(list) => list
                    .iter()
                    .map(|f| {
                        if !f.cx_real.is_nan() {
                            f.cx_real
                        } else if !f.cx_imag.is_nan() {
                            f.cx_imag
                        } else {
                            todo!("can not get value from complex: {:?}", f);
                        }
                    })
                    .collect(),
            };
            map.insert(name, data1);
        }
        self.buffer = Some(c.strs.clone());
        Ok(map)
    }

    pub fn ac(
        &mut self,
        start_frequency: &str,
        stop_frequency: &str,
        number_of_points: u32,
        variation: &str,
    ) -> Result<HashMap<String, Vec<f64>>, Error> {
        let mut c = Cb::new();
        let ngspice = NgSpice::new(&mut c)?;
        let circ = self.circuit.to_str(true)?;
        ngspice.circuit(circ)?;
        ngspice
            //DEC ND FSTART FSTOP
            .command(
                format!(
                    "ac {} {} {} {}",
                    variation, number_of_points, start_frequency, stop_frequency
                )
                .as_str(),
            )?;
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str());
            if let Ok(r) = re {
                let name = r.name;
                let data1 = match r.data {
                    ComplexSlice::Real(list) => list.iter().map(|i| *i).collect(),
                    ComplexSlice::Complex(list) => list
                        .iter()
                        .map(|f| {
                            if !f.cx_real.is_nan() {
                                f.cx_real
                            } else if !f.cx_imag.is_nan() {
                                f.cx_imag
                            } else {
                                todo!("can not get value from complex: {:?}", f);
                            }
                        })
                        .collect(),
                };
                map.insert(name, data1);
            } else {
                panic!("Can not run ac with schema.");
            }
        }
        self.buffer = Some(c.strs.clone());
        Ok(map)
    }
}
