//! package to run ngspice simulatations.
mod error;
mod circuit;
mod netlist;
mod simulation;

pub use {
    error::Error,
    circuit::Circuit,
    netlist::{Netlist, NodePositions, Point},
    self::simulation::Simulation,
};
