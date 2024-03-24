//! package to run ngspice simulatations.
mod circuit;
mod error;
mod netlist;
mod simulation;

pub use {
    self::simulation::Simulation,
    circuit::Circuit,
    error::Error,
    netlist::{Netlist, NodePositions, Point},
};
