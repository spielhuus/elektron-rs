//!ngspice binding.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//pub use crate::{ngcomplex, ngspice, simulation_types};
use libloading::library_filename;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::{CStr, CString, NulError};
use std::os::raw::{c_char, c_int, c_void};

///ngspice errors.
#[derive(Error, Debug)]
pub enum NgSpiceError {
    #[error("ill-formed matrix can't be decomposed")]
    Badmatrix,
    #[error("matrix is singular")]
    Singular,
    #[error("iteration limit reached,operation aborted")]
    Iterlim,
    #[error("integration order not supported")]
    Order,
    #[error("integration method not supported")]
    Method,
    #[error("timestep too small")]
    TimeStep,
    #[error("transmission line in pz analysis")]
    Xmissionline,
    #[error("pole-zero magnitude too large")]
    Magexceeded,
    #[error("pole-zero input or output shorted")]
    Short,
    #[error("pole-zero input is output")]
    Inisout,
    #[error("ac currents cannot be ASKed")]
    AskCurrent,
    #[error("ac powers cannot be ASKed")]
    AskPower,
    #[error("node not defined in noise anal")]
    Nodundef,
    #[error("no ac input src specified for noise")]
    Noacinput,
    #[error("no source at F2 for IM disto analysis")]
    Nof2src,
    #[error("no distortion analysis - NODISTO defined")]
    NoDisto,
    #[error("no noise analysis - NONOISE defined")]
    NoNoise,
    #[error("can not load the ngspice library.")]
    Init,
    #[error("encoding error.")]
    Encoding,
    #[error("Unknown error: {0}")]
    Unknown(i32),
    #[error("No Results from ngspice.")]
    NoResults,
    #[error("Interior nul byte was found.")]
    NulError,
    #[error("Integral conversion failed.")]
    TryFromIntFailed,
    #[error("Spice Error: {0}\n{1}")]
    Spice(i32, String),
}

///The main struct to use ngspice.
pub struct NgSpice<'a, C> {
    ///the callback to receive messages from ngspice.
    pub callbacks: &'a mut C,
    ngspice: ngspice,
    exited: bool,
}

#[derive(Debug)]
pub enum ComplexSlice<'a> {
    Real(&'a [f64]),
    Complex(&'a [ngcomplex]),
}

///result vector info.
#[derive(Debug)]
pub struct VectorInfo<'a> {
    pub name: String,
    pub dtype: simulation_types,
    pub data: ComplexSlice<'a>,
}

pub struct SimulationResult<'a, C: Callbacks> {
    name: String,
    sim: std::sync::Arc<NgSpice<'a, C>>,
}

impl<'a, C: Callbacks> Drop for SimulationResult<'a, C> {
    fn drop(&mut self) {
        let cmd = format!("destroy {}", self.name.as_str());
        self.sim
            .command(cmd.as_str())
            .expect("Failed to free simulation");
    }
}

unsafe extern "C" fn send_char<C: Callbacks>(
    arg1: *mut c_char,
    _arg2: c_int,
    context: *mut c_void,
) -> c_int {
    let spice = &mut *(context as *mut NgSpice<C>);
    let cb = &mut spice.callbacks;
    let str_res = CStr::from_ptr(arg1).to_str();
    if let Ok(s) = str_res {
        cb.send_char(s);
    }
    0
}

unsafe extern "C" fn controlled_exit<C: Callbacks>(
    status: c_int,
    unload: bool,
    quit: bool,
    _instance: c_int,
    context: *mut c_void,
) -> c_int {
    let spice = &mut *(context as *mut NgSpice<C>);
    let cb = &mut spice.callbacks;
    spice.exited = true;
    cb.controlled_exit(status, unload, quit);
    0
}

impl From<NulError> for NgSpiceError {
    fn from(_e: NulError) -> NgSpiceError {
        NgSpiceError::NulError
    }
}

impl From<std::str::Utf8Error> for NgSpiceError {
    fn from(_e: std::str::Utf8Error) -> NgSpiceError {
        NgSpiceError::Encoding
    }
}

impl From<std::num::TryFromIntError> for NgSpiceError {
    fn from(_e: std::num::TryFromIntError) -> NgSpiceError {
        NgSpiceError::TryFromIntFailed
    }
}

impl From<libloading::Error> for NgSpiceError {
    fn from(_e: libloading::Error) -> NgSpiceError {
        NgSpiceError::Init
    }
}

impl From<i32> for NgSpiceError {
    fn from(e: i32) -> NgSpiceError {
        if e == 101 {
            NgSpiceError::Badmatrix
        } else if e == 102 {
            NgSpiceError::Singular
        } else if e == 103 {
            NgSpiceError::Iterlim
        } else if e == 104 {
            NgSpiceError::Order
        } else if e == 105 {
            NgSpiceError::Method
        } else if e == 106 {
            NgSpiceError::TimeStep
        } else if e == 107 {
            NgSpiceError::Xmissionline
        } else if e == 108 {
            NgSpiceError::Magexceeded
        } else if e == 109 {
            NgSpiceError::Short
        } else if e == 110 {
            NgSpiceError::Inisout
        } else if e == 111 {
            NgSpiceError::AskCurrent
        } else if e == 112 {
            NgSpiceError::AskPower
        } else if e == 113 {
            NgSpiceError::Nodundef
        } else if e == 114 {
            NgSpiceError::Noacinput
        } else if e == 115 {
            NgSpiceError::Nof2src
        } else if e == 116 {
            NgSpiceError::NoDisto
        } else if e == 117 {
            NgSpiceError::NoNoise
        } else {
            NgSpiceError::Unknown(e)
        }
    }
}

impl<'a, C: Callbacks> NgSpice<'a, C> {
    pub fn new(c: &'a mut C) -> Result<std::sync::Arc<NgSpice<'a, C>>, NgSpiceError> {
        unsafe {
            let spice = NgSpice {
                ngspice: ngspice::new(library_filename("ngspice")).unwrap(),
                callbacks: c,
                exited: false,
            };
            let ptr = std::sync::Arc::new(spice);
            let rawptr = std::sync::Arc::as_ptr(&ptr);
            ptr.ngspice.ngSpice_Init(
                Some(send_char::<C>),
                None,
                Some(controlled_exit::<C>),
                None,
                None,
                None,
                rawptr as _,
            );
            Ok(ptr)
        }
    }

    ///tue if the ngspice library has exited.
    pub fn is_exited(&self) -> bool {
        self.exited
    }

    ///send a command to ngspice.
    pub fn command(&self, s: &str) -> Result<(), NgSpiceError> {
        if self.exited {
            panic!("NgSpice exited")
        }
        let cs = CString::new(s)?;
        let raw = cs.into_raw();
        unsafe {
            let ret = self.ngspice.ngSpice_Command(raw);
            drop(CString::from_raw(raw));
            if ret == 0 {
                Ok(())
            } else {
                println!("Command Error: {}", ret);
                Err(ret.into())
            }
        }
    }

    ///set a spice netlist.
    pub fn circuit(&self, circ: Vec<String>) -> Result<(), NgSpiceError> {
        let mut buf: Vec<*mut i8> = circ
            .iter()
            .map(|s| CString::new(s.as_str()).map(|cs| cs.into_raw()))
            .collect::<Result<Vec<*mut i8>, _>>()?;
        // ngspice wants an empty string and a nullptr
        buf.push(CString::new("").unwrap().into_raw());
        buf.push(std::ptr::null_mut());
        unsafe {
            let res = self.ngspice.ngSpice_Circ(buf.as_mut_ptr());
            for b in buf {
                if !b.is_null() {
                    drop(CString::from_raw(b));
                }
            }
            if res == 0 {
                Ok(())
            } else {
                Err(res.into())
            }
        }
    }

    ///get the name of the current plot.
    pub fn current_plot(&self) -> Result<String, NgSpiceError> {
        unsafe {
            let ret = self.ngspice.ngSpice_CurPlot();
            let ptr = CStr::from_ptr(ret).to_str()?;
            Ok(String::from(ptr))
        }
    }

    ///get the names of all plots.
    pub fn all_plots(&self) -> Result<Vec<String>, NgSpiceError> {
        unsafe {
            let ptrs = self.ngspice.ngSpice_AllPlots();
            let mut strs: Vec<String> = Vec::new();
            let mut i = 0;
            while !(*ptrs.offset(i)).is_null() {
                let ptr = CStr::from_ptr(*ptrs.offset(i)).to_str()?;
                let s = String::from(ptr);
                strs.push(s);
                i += 1;
            }
            Ok(strs)
        }
    }

    ///get all vec names.
    pub fn all_vecs(&self, plot: &str) -> Result<Vec<String>, NgSpiceError> {
        let cs = CString::new(plot)?;
        let raw = cs.into_raw();
        unsafe {
            let ptrs = self.ngspice.ngSpice_AllVecs(raw);
            drop(CString::from_raw(raw));
            let mut strs: Vec<String> = Vec::new();
            let mut i = 0;
            while !(*ptrs.offset(i)).is_null() {
                let ptr = CStr::from_ptr(*ptrs.offset(i)).to_str()?;
                let s = String::from(ptr);
                strs.push(s);
                i += 1;
            }
            Ok(strs)
        }
    }

    ///ge the vector info.
    pub fn vector_info(&self, vec: &str) -> Result<VectorInfo<'a>, NgSpiceError> {
        let cs = CString::new(vec)?;
        let raw = cs.into_raw();
        unsafe {
            let vecinfo = *self.ngspice.ngGet_Vec_Info(raw);
            drop(CString::from_raw(raw));
            let ptr = CStr::from_ptr(vecinfo.v_name).to_str()?;
            let len = vecinfo.v_length.try_into()?;
            let s = String::from(ptr);
            let typ: simulation_types = std::mem::transmute(vecinfo.v_type);
            if !vecinfo.v_realdata.is_null() {
                let real_slice = std::slice::from_raw_parts_mut(vecinfo.v_realdata, len);
                Ok(VectorInfo {
                    name: s,
                    dtype: typ,
                    data: ComplexSlice::Real(real_slice),
                })
            } else if !vecinfo.v_compdata.is_null() {
                let comp_slice = std::slice::from_raw_parts_mut(vecinfo.v_compdata, len);
                Ok(VectorInfo {
                    name: s,
                    dtype: typ,
                    data: ComplexSlice::Complex(comp_slice),
                })
            } else {
                Err(NgSpiceError::NoResults)
            }
        }
    }
}

trait Simulator<'a, C: Callbacks> {
    fn op(&self) -> Result<SimulationResult<'a, C>, NgSpiceError>;
}

impl<'a, C: Callbacks> Simulator<'a, C> for std::sync::Arc<NgSpice<'a, C>> {
    fn op(&self) -> Result<SimulationResult<'a, C>, NgSpiceError> {
        self.command("op")?;
        let plot = self.current_plot()?;
        let vecs = self.all_vecs(&plot)?;
        let mut results = HashMap::new();
        for vec in vecs {
            if let Ok(vecinfo) = self.vector_info(&format!("{}.{}", plot, vec)) {
                results.insert(vec, vecinfo);
            }
        }
        let sim = SimulationResult {
            name: plot,
            sim: self.to_owned(),
        };
        Ok(sim)
    }
}

///type for the ngspice callback.
pub trait Callbacks {
    fn send_char(&mut self, _s: &str) {}
    fn controlled_exit(&mut self, _status: i32, _unload: bool, _quit: bool) {}
}

// TODO test fail on github
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     struct Cb {
//         strs: Vec<String>,
//     }
//
//     impl Callbacks for Cb {
//         fn send_char(&mut self, s: &str) {
//             println!("{}", s);
//             self.strs.push(s.to_string())
//         }
//     }
//     #[test]
//     fn it_works() {
//         let mut c = Cb { strs: Vec::new() };
//         let spice = NgSpice::new(&mut c).unwrap();
//         // assert!(NgSpice::new(Cb { strs: Vec::new() }).is_err());
//         spice.command("echo hello").expect("echo failed");
//         assert_eq!(
//             spice.callbacks.strs.last().unwrap_or(&String::new()),
//             "stdout hello"
//         );
//         spice
//             .circuit(vec![
//                 ".title KiCad schematic".to_string(),
//                 ".MODEL FAKE_NMOS NMOS (LEVEL=3 VTO=0.75)".to_string(),
//                 ".save all @m1[gm] @m1[id] @m1[vgs] @m1[vds] @m1[vto]".to_string(),
//                 "R1 /vdd /drain 10k".to_string(),
//                 "M1 /drain /gate GND GND FAKE_NMOS W=10u L=1u".to_string(),
//                 "V1 /vdd GND dc(5)".to_string(),
//                 "V2 /gate GND dc(2)".to_string(),
//                 ".end".to_string(),
//             ])
//             .expect("circuit failed");
//         {
//             let _sim1 = spice.op().expect("op failed");
//             // println!("{}: {:?}", sim1.name, sim1.data);
//             spice.command("alter m1 W=20u").expect("op failed");
//             let _sim2 = spice.op().expect("op failed");
//             // println!("{}: {:?}", sim2.name, sim2.data);
//             let plots = spice.all_plots().expect("plots failed");
//             println!("{:?}", plots);
//             assert_eq!(plots[0], "op2");
//             let curplot = spice.current_plot().expect("curplot failed");
//             assert_eq!(curplot, "op2");
//         }
//         let plots = spice.all_plots().expect("plots failed");
//         println!("{:?}", plots);
//         assert_eq!(plots.len(), 1);
//
//         //test loop
//         spice.command("echo hello").expect("echo failed");
//         spice.command("let topi = 0.1").expect("topi failed");
//         spice.command("while topi < 0.9").unwrap();
//         spice.command(" echo $&topi").unwrap();
//         spice.command(" let topi = topi + 0.1").unwrap();
//         spice.command("end").unwrap();
//
//         println!("-> {:?}", spice.callbacks.strs.join("\n"));
//         /* assert_eq!(
//             21,
//             spice.callbacks.strs.len(),
//         ); */
//
//         //test circuit loop
//         spice
//             .circuit(vec![
//                 "* name".to_string(),
//                 "Vcc cc 0 2".to_string(),
//                 "".to_string(),
//                 "Vin in 0 dc 0 pulse (0 2 95n 2n 2n 90n 180n)".to_string(),
//                 "".to_string(),
//                 "Mn1 out in 0 0 nm W=2u L=0.18u".to_string(),
//                 "".to_string(),
//                 "Mp1 out in cc cc pm W=4u L=0.18u".to_string(),
//                 "".to_string(),
//                 ".model nm nmos level=14 version=4.8.1".to_string(),
//                 ".model pm pmos level=14 version=4.8.1".to_string(),
//                 ".end".to_string(),
//             ])
//             .expect("circuit failed");
//
//             spice.command("let vccc = 1.2").unwrap();
//             spice.command("repeat 5").unwrap();
//             spice.command("  alter Vcc $&vccc").unwrap();
//             spice.command("  dc vin 0 2 0.01").unwrap();
//             spice.command("  let vccc = vccc + 0.2").unwrap();
//             spice.command("end").unwrap();
//             spice.command("save all").unwrap();
//
//         assert_eq!(6, spice.all_plots().expect("plots failed").len());
//     }
// }
