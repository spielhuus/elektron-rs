//! Run the ngspice simulations
//!
//!

use log::{debug, log_enabled, Level};
use std::collections::HashMap;

use ngspice::{Callbacks, ComplexSlice, NgSpice, NgSpiceError};

use crate::{circuit::Circuit, error::Error};

macro_rules! handle_error {
    ($cmd:expr, $cb:expr) => {
        match $cmd {
            Ok(_) => {}
            Err(error) => match error {
                ngspice::NgSpiceError::Unknown(code) => {
                    return Err(NgSpiceError::Spice(code, $cb.strs.join("\n")).into());
                }
                _ => {
                    return Err(error.into());
                }
            },
        }
    };
}

/// The callback message buffer
pub struct Cb {
    ///The string buffer
    strs: Vec<String>,
    ///last status
    status: i32,
    ///unloaded
    unload: bool,
    ///quited
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

///# The Simulation struct
///
/// ## Examples
///
/// Load a Kicad schema:
///
/// ```
/// use sexp::{SexpParser, SexpTree};
///
/// let doc = SexpParser::load("tests/summe.kicad_sch").unwrap();
/// let tree = SexpTree::from(doc.iter()).unwrap();
/// let root = tree.root().unwrap();
///
/// assert_eq!("kicad_sch", root.name);
/// ```
pub struct Simulation {
    pub circuit: Circuit,
    pub buffer: Option<Vec<String>>,
}

impl Simulation {
    ///### Create new simulation from circuit.
    pub fn new(circuit: Circuit) -> Self {
        Self {
            circuit,
            buffer: None,
        }
    }

    ///Run the stored commands.
    ///
    ///the commands can be added with xxx.
    pub fn run(&self) -> Result<HashMap<String, HashMap<String, Vec<f64>>>, Error> {
        if log_enabled!(Level::Debug) {
            debug!("run commands:\n{}", self.circuit.controls.join("\n"));
        }
        let mut cb = Cb::new();
        let ng = NgSpice::new(&mut cb)?;

        handle_error!(ng.circuit(self.circuit.to_str(true).unwrap()), cb);
        for c in &self.circuit.controls {
            handle_error!(ng.command(c), cb);
        }
        let mut plot_result: HashMap<String, HashMap<String, Vec<f64>>> = HashMap::new();
        for plot in ng.all_plots()? {
            let vecs = ng.all_vecs(&plot)?;
            let mut vec_values: HashMap<String, Vec<f64>> = HashMap::new();
            for v in vecs {
                let vals = ng.vector_info(format!("{}.{}", plot, &v).as_str())?;
                let data1 = match vals.data {
                    ComplexSlice::Real(list) => list.to_vec(),
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

    ///Operating Point Analysis
    ///
    /// Compute the DC operating point of the circuit with inductors
    /// shorted and capacitorsopened.
    pub fn op(&mut self) -> Result<HashMap<String, Vec<f64>>, Error> {
        if log_enabled!(Level::Debug) {
            debug!("run operating point:\n{}", self.circuit.controls.join("\n"));
        }
        let mut cb = Cb::new();
        let ngspice = NgSpice::new(&mut cb)?;
        let circ = self.circuit.to_str(true)?;
        handle_error!(ngspice.circuit(circ), cb);
        handle_error!(ngspice.command("op"), cb);
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str())?;
            let name = re.name;
            let data1 = match re.data {
                ComplexSlice::Real(list) => list.to_vec(),
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
        self.buffer = Some(cb.strs.clone());
        Ok(map)
    }

    ///Transient analysis.
    ///
    /// ## Arguments
    /// * `step`   - the starting frequency.
    /// * `stop`    - the final frequency.
    /// * `start`  - number of points per decade.
    ///
    ///Reference in the [ngspice Documentation](https://ngspice.sourceforge.io/docs/ngspice-41-manual.pdf) in chapter 15.3.10.
    pub fn tran(
        &mut self,
        step: &str,
        stop: &str,
        start: &str,
    ) -> Result<HashMap<String, Vec<f64>>, Error> {
        if log_enabled!(Level::Debug) {
            debug!(
                "run transient analysis: step:{}, stop={}, start={}",
                step, stop, start
            );
        }
        let mut cb = Cb::new();
        let ngspice = NgSpice::new(&mut cb)?;
        let circ = self.circuit.to_str(true)?;
        handle_error!(ngspice.circuit(circ), cb);
        handle_error!(
            ngspice.command(format!("tran {} {} {}", step, stop, start).as_str()),
            cb
        );
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str())?;
            let name = re.name;
            let data1 = match re.data {
                ComplexSlice::Real(list) => list.to_vec(),
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
        self.buffer = Some(cb.strs.clone());
        Ok(map)
    }

    ///Small-Signal AC Analysis
    ///
    /// ## Arguments
    /// * `start_frequency`   - the starting frequency.
    /// * `stop_frequency`    - the final frequency.
    /// * `number_of_points`  - number of points per decade.
    /// * `variation`         - type [dec, oct, lin]
    ///
    ///Reference in the [ngspice Documentation](https://ngspice.sourceforge.io/docs/ngspice-41-manual.pdf) in chapter 15.3.1.
    pub fn ac(
        &mut self,
        start_frequency: &str,
        stop_frequency: &str,
        number_of_points: u32,
        variation: &str,
    ) -> Result<HashMap<String, Vec<f64>>, Error> {
        if log_enabled!(Level::Debug) {
            debug!(
                "run ac analysis: start frequency:{}, stop frequency={}, points={}, variation={}",
                start_frequency, stop_frequency, number_of_points, variation
            );
        }
        let mut cb = Cb::new();
        let ngspice = NgSpice::new(&mut cb)?;
        let circ = self.circuit.to_str(true)?;
        handle_error!(ngspice.circuit(circ), cb);
        handle_error!(
            ngspice
                //DEC ND FSTART FSTOP
                .command(
                    format!(
                        "ac {} {} {} {}",
                        variation, number_of_points, start_frequency, stop_frequency
                    )
                    .as_str(),
                ),
            cb
        );
        let plot = ngspice.current_plot()?;
        let res = ngspice.all_vecs(plot.as_str())?;
        let mut map: HashMap<String, Vec<f64>> = HashMap::new();
        for name in res {
            let re = ngspice.vector_info(name.as_str());
            if let Ok(r) = re {
                let name = r.name;
                let data1 = match r.data {
                    ComplexSlice::Real(list) => list.to_vec(),
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
        self.buffer = Some(cb.strs.clone());
        Ok(map)
    }
}
