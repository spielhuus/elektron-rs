//! Plotter for schema and pcb datas.
//!
//! ### Plot Schema
//!
//!
//! use elektron::sexp::{Schema, Pcb};
//! use elektron::plot::*;
//!
//! let doc = Pcb::load("files/summe.kicad_pcb").unwrap();
//! let mut buffer = std::fs::File::create("target/summe_pcb.pdf").unwrap();
//! Plotter::svg(PlotOptions::new(&doc, &mut buffer).id("summe")).unwrap();
//!
//! ### Plot PCB
//!
//! use elektron::sexp::{Schema, Pcb};
//! use elektron::plot::*;
//!
//! let doc = Pcb::load("files/summe.kicad_pcb").unwrap();
//! let mut buffer = std::fs::File::create("target/summe_pcb.pdf").unwrap();
//! Plotter::pdf(PlotOptions::new(&doc, &mut buffer).theme(Theme::Kicad2020)).unwrap();

mod cairo_plotter;
//mod gerber;
mod gerber_plotter;
mod pcb;
mod plotter;
mod schema;
mod svg_plotter;
mod theme;

use crate::error::Error;
use std::io::Write;

pub use plotter::Theme;

use crate::sexp::{Pcb, Schema};
use {/* plotter::PlotterImpl,*/ svg_plotter::SvgPlotter};
pub use {plotter::PlotterImpl, gerber_plotter::GerberPlotter};

use self::{cairo_plotter::CairoPlotter, plotter::ImageType};

///Create the plotter options.
pub struct PlotOptions<'a, T, W: Write + 'static> {
    doc: &'a T,
    writer: &'a mut W,
    border: bool,
    netlist: bool,
    scale: f64,
    id: Option<&'a str>,
    pages: Option<Vec<usize>>,
    theme: Option<Theme>,
}

impl<'a, T, W: Write + 'static> PlotOptions<'a, T, W> {
    pub fn new(graphic: &'a T, writer: &'a mut W) -> Self {
        Self {
            doc: graphic,
            writer,
            border: true,
            netlist: false,
            scale: 1.0,
            id: None,
            pages: None,
            theme: None,
        }
    }
    ///Draw the document border. When no border is drawn, the image is cropped to the drawing area
    ///(default: true)
    pub fn border(&mut self, border: bool) -> &mut Self {
        self.border = border;
        self
    }
    ///Ouput the netlist (default: false).
    pub fn netlist(&mut self, netlist: bool) -> &mut Self {
        self.netlist = netlist;
        self
    }
    ///Scale the image (default 1.0).
    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.scale = scale;
        self
    }
    ///The if for SVG images (default: None).
    pub fn id(&mut self, id: &'a str) -> &mut Self {
        self.id = Some(id);
        self
    }
    ///The pages to draw (defult: 0).
    pub fn pages(&mut self, pages: Vec<usize>) -> &mut Self {
        self.pages = Some(pages.to_vec());
        self
    }
    ///The selected Theme (Default: Kicad2020).
    pub fn theme(&mut self, theme: Theme) -> &mut Self {
        self.theme = Some(theme);
        self
    }
}

///Plotter struct.
pub struct Plotter;

///Output the Document as an image.
pub trait PlotSelector<'a, T, W: Write + 'static> {
    //Write SVG file.
    fn svg(options: &mut PlotOptions<T, W>) -> Result<(), Error>;
    //Write PNG file.
    fn png(options: &mut PlotOptions<T, W>) -> Result<(), Error>;
    //Write PDF file.
    fn pdf(options: &mut PlotOptions<T, W>) -> Result<(), Error>;
}

impl<'a, W: Write + 'static> PlotSelector<'a, Schema, W> for Plotter {
    fn svg(options: &mut PlotOptions<Schema, W>) -> Result<(), Error> {
        let svg = SvgPlotter::new(options.id.unwrap(), options.theme.clone());
        svg.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
    fn png(options: &mut PlotOptions<Schema, W>) -> Result<(), Error> {
        let pdf = CairoPlotter::new(
            ImageType::Png,
            options.theme.clone().unwrap_or(Theme::Kicad2020),
        );
        pdf.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
    fn pdf(options: &mut PlotOptions<Schema, W>) -> Result<(), Error> {
        let pdf = CairoPlotter::new(
            ImageType::Pdf,
            options.theme.clone().unwrap_or(Theme::Kicad2020),
        );
        pdf.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
}

impl<'a, W: Write + 'static> PlotSelector<'a, Pcb, W> for Plotter {
    fn svg(options: &mut PlotOptions<Pcb, W>) -> Result<(), Error> {
        let pdf = SvgPlotter::new(options.id.unwrap(), options.theme.clone());
        pdf.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
    fn png(options: &mut PlotOptions<Pcb, W>) -> Result<(), Error> {
        let pdf = CairoPlotter::new(
            ImageType::Png,
            options.theme.clone().unwrap_or(Theme::Kicad2020),
        );
        pdf.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
    fn pdf(options: &mut PlotOptions<Pcb, W>) -> Result<(), Error> {
        let pdf = CairoPlotter::new(
            ImageType::Pdf,
            options.theme.clone().unwrap_or(Theme::Kicad2020),
        );
        pdf.plot(
            options.doc,
            options.writer,
            options.border,
            options.scale,
            options.pages.clone(),
            options.netlist,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{plotter::Theme, PlotOptions};
    use super::{PlotSelector, Plotter};
    use crate::sexp::{Pcb, Schema};
    use std::{fs::File, path::Path};

    #[test]
    fn plt_svg_dco() {
        let doc = Schema::load("files/dco.kicad_sch").unwrap();
        let mut buffer = File::create("target/dco.svg").unwrap();
        Plotter::svg(
            PlotOptions::new(&doc, &mut buffer)
                .id("dco")
                .scale(5.0)
                .border(false)
                .theme(Theme::Kicad2020),
        )
        .unwrap();
        assert!(Path::new("target/dco.svg").exists());
        assert!(Path::new("target/dco.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_svg_summe() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        let mut buffer = File::create("target/summe.svg").unwrap();
        Plotter::svg(
            PlotOptions::new(&doc, &mut buffer)
                .id("summe")
                .theme(Theme::Kicad2020),
        )
        .unwrap();
        assert!(Path::new("target/summe.svg").exists());
        assert!(Path::new("target/summe.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_png_dco() {
        let doc = Schema::load("files/dco.kicad_sch").unwrap();
        let mut buffer = File::create("target/dco.png").unwrap();
        Plotter::png(
            PlotOptions::new(&doc, &mut buffer)
                .scale(5.0)
                .border(false)
                .theme(Theme::Kicad2020),
        )
        .unwrap();
        assert!(Path::new("target/dco.png").exists());
        assert!(Path::new("target/dco.png").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_pdf_summe() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        let mut buffer = File::create("target/summe.pdf").unwrap();
        Plotter::pdf(PlotOptions::new(&doc, &mut buffer).theme(Theme::Kicad2020)).unwrap();
        assert!(Path::new("target/summe.pdf").exists());
        assert!(Path::new("target/summe.pdf").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_svg_header() {
        let doc = Schema::load("files/header.kicad_sch").unwrap();
        let mut buffer = File::create("target/header.svg").unwrap();
        Plotter::svg(PlotOptions::new(&doc, &mut buffer).theme(Theme::Kicad2020).id("header")).unwrap();
        assert!(Path::new("target/header.svg").exists());
        assert!(Path::new("target/header.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_svg_pcb_summe() {
        let doc = Pcb::load("files/summe.kicad_pcb").unwrap();
        let mut buffer = File::create("target/summe_pcb.svg").unwrap();
        Plotter::svg(
            PlotOptions::new(&doc, &mut buffer)
                .id("pcb_summe")
                .theme(Theme::Kicad2020),
        )
        .unwrap();
        assert!(Path::new("target/summe_pcb.svg").exists());
        assert!(Path::new("target/summe_pcb.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_pdf_pcb_summe() {
        let doc = Pcb::load("files/summe.kicad_pcb").unwrap();
        let mut buffer = File::create("target/summe_pcb.pdf").unwrap();
        Plotter::pdf(PlotOptions::new(&doc, &mut buffer).theme(Theme::Kicad2020)).unwrap();
        assert!(Path::new("target/summe_pcb.pdf").exists());
        assert!(Path::new("target/summe_pcb.pdf").metadata().unwrap().len() > 0);
    }
    /* #[test]
    fn plt_dco_mono() {
        let doc = Schema::load("files/dco.kicad_sch").unwrap();
        plot_schema(&doc, Some("/tmp/dco-mono.svg"), 3.0, false, Some(String::from("mono")), None, Some("svg")).unwrap();
        assert!(Path::new("/tmp/dco-mono.svg").exists());
        assert!(Path::new("/tmp/dco-mono.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_summe() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        plot_schema(&doc, Some("/tmp/summe.svg"), 3.0, true, Some(String::from("kicad_2000")), None, Some("svg")).unwrap();
        assert!(Path::new("/tmp/summe.svg").exists());
        assert!(Path::new("/tmp/summe.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_summe_mono() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        plot_schema(&doc, Some("/tmp/summe-mono.svg"), 3.0, true, Some(String::from("mono")), None, Some("svg")).unwrap();
        assert!(Path::new("/tmp/summe-mono.svg").exists());
        assert!(Path::new("/tmp/summe-mono.svg").metadata().unwrap().len() > 0);
    }
    #[test]
    fn plt_summe_netlist() {
        let doc = Schema::load("files/summe.kicad_sch").unwrap();
        let netlist = Netlist::from(&doc).unwrap();
        plot_schema(&doc, Some("/tmp/summe-netlist.svg"), 3.0, true, Some(String::from("mono")), Some(netlist), Some("svg")).unwrap();
        assert!(Path::new("/tmp/summe-netlist.svg").exists());
        assert!(Path::new("/tmp/summe-netlist.svg").metadata().unwrap().len() > 0);
    } */
}
