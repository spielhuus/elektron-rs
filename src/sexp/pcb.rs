use std::{fs::File, io::Write};

use crate::{
    check_directory,
    error::Error,
    plot::{CairoPlotter, ImageType, Theme},
    sexp::{
        model::{Footprint, GrLine, GrText, PaperSize, Segment, TitleBlock, Via, Zone},
        parser::State,
        write::SexpWriter,
        SexpParser,
    },
};

use super::model::{Layers, PcbElements};

pub struct Pcb {
    general: Vec<(String, f64)>,
    layers: Vec<Layers>,
    elements: Vec<PcbElements>,

    nets: Vec<(u32, String)>,
    paper_size: PaperSize,
    title_block: TitleBlock,
}

impl Pcb {
    pub fn new() -> Self {
        Self {
            general: Vec::new(),
            layers: Vec::new(),
            nets: Vec::new(),
            elements: Vec::new(),
            paper_size: PaperSize::A4,
            title_block: TitleBlock::new(),
        }
    }
    ///plot the pcb.
    pub fn plot(&self, filename: &str, scale: f64, border: bool, theme: &str) -> Result<(), Error> {
        let image_type = if filename.ends_with(".svg") {
            ImageType::Svg
        } else if filename.ends_with(".png") {
            ImageType::Png
        } else {
            ImageType::Pdf
        };
        let theme = if theme == "mono" {
            Theme::mono()
        } else {
            Theme::kicad_2000()
        };

        use crate::plot::{PcbPlotIterator, Plotter};
        let iter = self.iter()?.plot(self, theme, border).flatten().collect(); //TODO: plot all
                                                                               //pages
        let mut cairo = CairoPlotter::new(&iter);

        check_directory(filename)?;
        let out: Box<dyn Write> = Box::new(File::create(filename)?);
        cairo.plot(out, border, scale, image_type)?;
        Ok(())
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let doc = SexpParser::load(filename)?;
        Self::parse(doc.iter())
    }
    fn parse<'a, I>(mut iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = State<'a>>,
    {
        let mut pcb = Self::new();
        loop {
            let state = iter.next();
            match state {
                None => {
                    return Ok(pcb);
                }
                Some(State::StartSymbol(name)) => {
                    if name == "uuid" {
                        // schema.uuid = iter.next().unwrap().into();
                    } else if name == "paper" {
                        pcb.paper_size = iter.next().unwrap().into();
                    } else if name == "title_block" {
                        pcb.title_block = TitleBlock::from(&mut iter);
                    } else if name == "general" {
                        let mut index = 1;
                        loop {
                            match iter.next() {
                                Some(State::StartSymbol(name)) => {
                                    pcb.general
                                        .push((name.to_string(), iter.next().unwrap().into()));
                                    index += 1;
                                }
                                Some(State::EndSymbol) => {
                                    index -= 1;
                                    if index == 0 {
                                        break;
                                    }
                                }
                                Some(State::Values(_)) => {}
                                Some(State::Text(_)) => {}
                                None => {
                                    break;
                                }
                            }
                        }
                    } else if name == "layers" {
                        let mut count = 1;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(ordinal)) = state {
                                count += 1;
                                let canonical_name = iter.next().unwrap().into();
                                let layertype = iter.next().unwrap().into();
                                let user_name = if let Some(State::Text(value)) = iter.next() {
                                    Some(value.to_string())
                                } else {
                                    count -= 1;
                                    None
                                };
                                pcb.layers.push(Layers {
                                    ordinal: ordinal.parse::<u32>().unwrap(),
                                    canonical_name,
                                    layertype,
                                    user_name,
                                });
                            } else if let Some(State::EndSymbol) = state {
                                count -= 1;
                                if count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name == "net" {
                        pcb.nets
                            .push((iter.next().unwrap().into(), iter.next().unwrap().into()));
                    } else if name == "footprint" {
                        pcb.elements
                            .push(PcbElements::Footprint(Footprint::from(&mut iter)));
                    } else if name == "gr_line" {
                        pcb.elements
                            .push(PcbElements::Line(GrLine::from(&mut iter)));
                    } else if name == "gr_text" {
                        pcb.elements
                            .push(PcbElements::Text(GrText::from(&mut iter)));
                    } else if name == "segment" {
                        pcb.elements
                            .push(PcbElements::Segment(Segment::from(&mut iter)));
                    } else if name == "via" {
                        pcb.elements.push(PcbElements::Via(Via::from(&mut iter)));
                    } else if name == "zone" {
                        pcb.elements.push(PcbElements::Zone(Zone::from(&mut iter)));
                    } else if name != "kicad_pcb" && name != "version" && name != "host" {
                        println!("unknown symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }
    ///iterate over the elements of the pcb.
    pub fn iter(&self) -> Result<std::slice::Iter<PcbElements>, Error> {
        Ok(self.elements.iter())
    }
    pub fn write(&self, filename: &str) -> Result<(), Error> {
        let mut out = File::create(filename)?;
        out.write_all(b"(kicad_sch ")?;

        out.write_all(b"(version ")?;
        out.write_all("20211123".as_bytes())?;
        out.write_all(b") ")?;
        out.write_all(b"(generator ")?;
        out.write_all("elektron".as_bytes())?;
        out.write_all(b")\n\n")?;

        out.write_all(b"  (general\n")?;
        for general in &self.general {
            out.write_all(b"    (")?;
            out.write_all(general.0.as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(general.1.to_string().as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all(b"  )\n")?;

        out.write_all(b"  (paper \"")?;
        out.write_all(self.paper_size.to_string().as_bytes())?;
        out.write_all(b"\")\n\n")?;
        self.title_block.write(&mut out, 1)?;

        out.write_all(b"  (layers\n")?;
        for layer in &self.layers {
            layer.write(&mut out, 2)?;
        }
        out.write_all(b"  )\n")?;

        //setup
        //
        //

        for net in &self.nets {
            out.write_all(b"  (net ")?;
            out.write_all(net.0.to_string().as_bytes())?;
            out.write_all(b" \"")?;
            out.write_all(net.1.as_bytes())?;
            out.write_all(b"\")\n")?;
        }

        for element in &self.elements {
            match element {
                PcbElements::Footprint(footprint) => footprint.write(&mut out, 1)?,
                PcbElements::Text(text) => text.write(&mut out, 1)?,
                PcbElements::Line(line) => line.write(&mut out, 1)?,
                PcbElements::Segment(segment) => segment.write(&mut out, 1)?,
                PcbElements::Via(via) => via.write(&mut out, 1)?,
                PcbElements::Zone(zone) => zone.write(&mut out, 1)?,
            }
        }

        out.write_all(b")\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
