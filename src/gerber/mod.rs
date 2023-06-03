/* mod color4d;
mod plot_board_layers;
mod plot_params;
mod plot_items;
mod plotter;
mod pcb_render_settings;
mod gerber_plotter;
mod render_settings;
mod gerber_metadata;
mod gerber_netlist_metadata; */

use pyo3::{prelude::*, py_run, types::PyList};

/* #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlotFormat {
    UNDEFINED,
    HPGL,
    GERBER,
    POST,
    DXF,
    PDF,
    SVG,
} */

fn check_directory(filename: &str) {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
}

pub struct Pcb {
    input: String,
}

impl Pcb {
    pub fn new(input: String) -> Self {
        Self { input }
    }

    pub fn drc(&self, output: String) {
        check_directory(&output);
        Python::with_gil(|py| {
            let list = PyList::new(py, &[self.input.to_string(), output.to_string()]);
            py_run!(py, list, r#"from elektron import Pcb
                board = Pcb(list[0])
                board.drc(list[1])"#);
        });
    }

    pub fn gerber(&self, output: String) {
        check_directory(&output);
        Python::with_gil(|py| {
            let list = PyList::new(py, &[self.input.to_string(), output.to_string()]);
            py_run!(py, list, r#"from elektron import Pcb
                board = Pcb(list[0])
                board.gerber(list[1])"#);
        });
    }
}


/* #[cfg(test)]
mod tests {
    use crate::sexp::{Pcb, LayerId};

    use super::{plot_board_layers::{StartPlotBoard, PlotOneBoardLayer}, plot_params::PCB_PLOT_PARAMS, PlotFormat};


    #[test]
    fn plt_gerber() {
        let mut pcb = Pcb::load("files/empty.kicad_pcb").unwrap();
        let mut plot_params = PCB_PLOT_PARAMS::new();
        plot_params.SetFormat(PlotFormat::GERBER);
        // start plot board
        let plotter = StartPlotBoard(&pcb, &plot_params, LayerId::BMask, String::from("out.gbr"), String::from("SheetDescription")).unwrap();
        // plot_one_board_layer
        PlotOneBoardLayer(&mut pcb, plotter.clone(), LayerId::BMask, &plot_params);
        // end_plot
        plotter.borrow_mut().EndPlot();
        
        // if format == Gerber
        //  create jobfile

    }
} */
