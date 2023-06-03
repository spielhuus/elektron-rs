use std::{fs::File, io::Write, path::Path, iter::FilterMap};

use super::{
    model::{Footprint, GrLine, GrText, PaperSize, Segment, Setup, TitleBlock, Via, Zone},
    parser::{SexpParser, State},
    write::SexpWriter, GrPoly, GrCircle,
};
use crate::error::Error;

use super::model::{Layers, PcbElements};

pub struct Pcb {
    filename: Option<String>,
    generator: String,
    setup: Setup,
    general: Vec<(String, f64)>,
    layers: Vec<Layers>,
    elements: Vec<PcbElements>,

    nets: Vec<(u32, String)>,
    pub paper_size: PaperSize,
    pub title_block: TitleBlock,
}

impl Pcb {
    pub fn new() -> Self {
        Self {
            filename: None,
            generator: String::new(),
            setup: Setup::new(),
            general: Vec::new(),
            layers: Vec::new(),
            nets: Vec::new(),
            elements: Vec::new(),
            paper_size: PaperSize::A4,
            title_block: TitleBlock::new(),
        }
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let doc = SexpParser::load(filename)?;
        Self::parse(filename.to_string(), doc.iter())
    }
    fn parse<'a, I>(path: String, mut iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = State<'a>>,
    {
        let mut pcb = Self::new();
        pcb.filename = Some(path);
        loop {
            let state = iter.next();
            match state {
                None => {
                    return Ok(pcb);
                }
                Some(State::StartSymbol(name)) => {
                    if name == "uuid" {
                        // schema.uuid = iter.next().unwrap().into();
                    } else if name == "generator" {
                        pcb.generator = iter.next().unwrap().into();
                    } else if name == "paper" {
                        pcb.paper_size = iter.next().unwrap().into();
                    } else if name == "title_block" {
                        pcb.title_block = TitleBlock::from(&mut iter);
                    } else if name == "setup" {
                        pcb.setup = Setup::from(&mut iter);
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
                                    id: ordinal.parse::<u32>().unwrap().into(),
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
                    } else if name == "gr_circle" {
                        pcb.elements
                            .push(PcbElements::GrCircle(GrCircle::from(&mut iter)));
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
                    } else if name == "gr_poly" {
                        pcb.elements.push(PcbElements::GrPoly(GrPoly::from(&mut iter)));
                    } else if name != "kicad_pcb" && name != "version" && name != "host" {
                        println!("unknown symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }

    ///iterate over the layers.
    /* pub fn drawings(&self) -> Result<Box<dyn Iterator<Item = PcbElements>>, Error> {
        Ok(Box::new(self.elements.iter().filter(|item| {
            if let PcbElements::Text(_) = item {
                true
            } else { false }
        })))
    } */

    ///iterate over the layers.
    pub fn layers(&self) -> Result<std::slice::Iter<Layers>, Error> {
        Ok(self.layers.iter())
    }
    
    ///iterate over the layers.
    //TODO do not collect the items.
    pub fn elements(&self) -> Result<std::slice::Iter<PcbElements>, Error> {
        Ok(self.elements.iter())
    }

    pub fn segment(item: &PcbElements) -> Option<&Segment> {
        if let PcbElements::Segment(t) = item {
            Some(t)
        } else { None }
    }

    ///iterate over the elements of the pcb.
    pub fn iter(&self) -> Result<std::slice::Iter<PcbElements>, Error> {
        Ok(self.elements.iter())
    }

    ///iterate over the footprints of the pcb.
    pub fn footprints(&self) -> impl Iterator<Item = &Footprint> {
        self.elements.iter().filter_map(|item| {
            if let PcbElements::Footprint(fp) = item {
                Some(fp)
            } else { None }
        })
    }
    
    ///iterate over the segments of the pcb.
    pub fn segements(&self) -> impl Iterator<Item = &Segment> {
        self.elements.iter().filter_map(|item| {
            if let PcbElements::Segment(s) = item {
                Some(s)
            } else { None }
        })
    }

    ///iterate over the segments of the pcb.
    pub fn zones(&self) -> impl Iterator<Item = &Zone> {
        self.elements.iter().filter_map(|item| {
            if let PcbElements::Zone(s) = item {
                Some(s)
            } else { None }
        })
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
                PcbElements::Setup(setup) => { /* TODO: */ },
                PcbElements::Footprint(footprint) => footprint.write(&mut out, 1)?,
                PcbElements::Text(text) => text.write(&mut out, 1)?,
                PcbElements::Line(line) => line.write(&mut out, 1)?,
                PcbElements::Segment(segment) => segment.write(&mut out, 1)?,
                PcbElements::Via(via) => via.write(&mut out, 1)?,
                PcbElements::Zone(zone) => zone.write(&mut out, 1)?,
                PcbElements::GrPoly(poly) => { /* TODO: */},
                PcbElements::GrCircle(circle) => { /* TODO: */},
            }
        }

        out.write_all(b")\n")?;
        Ok(())
    }
    pub fn path(&self) -> Option<String> {
        self.filename.as_ref().map(|path| path.to_string())
    }
    pub fn filename(&self) -> Option<String> {
        if let Some(path) = &self.filename {
            Some(Path::new(&path).file_name().unwrap().to_str()?.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sexp::Pcb;


    #[test]
    fn summe() {
        let doc = Pcb::load("files/summe/summe.kicad_pcb").unwrap();
        assert_eq!(31, doc.setup.values.len());
    }
    #[test]
    fn iter_layers() {
        let doc = Pcb::load("files/summe/summe.kicad_pcb").unwrap();
        assert_eq!(20, doc.layers().unwrap().count());
    }
    #[test]
    fn visit_footprint() {
        let doc = Pcb::load("files/summe/summe.kicad_pcb").unwrap();
        let counter = doc.footprints().count();
        assert_eq!(79, counter);
    }
}
