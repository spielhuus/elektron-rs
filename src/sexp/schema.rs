use std::{fs::File, io::Write};

use crate::{
    check_directory,
    error::Error,
    plot::{CairoPlotter, ImageType, Theme},
    sexp::{
        model::{
            GlobalLabel, Junction, Label, LibrarySymbol, NoConnect, PaperSize, SchemaElement,
            Sheet, SheetInstance, Symbol, SymbolInstance, Text, TitleBlock, Wire,
        },
        parser::State,
        write::SexpWriter,
        SexpParser,
    },
};

pub struct Schema {
    uuid: String,
    paper_size: PaperSize,
    title_block: TitleBlock,
    elements: Vec<SchemaElement>,
    pub libraries: Vec<LibrarySymbol>,
    sheet_instances: Vec<SheetInstance>,
    symbol_instances: Vec<SymbolInstance>,
    pages: Vec<Schema>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            uuid: String::new(),
            paper_size: PaperSize::A4,
            title_block: TitleBlock::new(),
            elements: Vec::new(),
            libraries: Vec::new(),
            sheet_instances: Vec::new(),
            symbol_instances: Vec::new(),
            pages: Vec::new(),
        }
    }
    pub fn load(filename: &str) -> Result<Self, Error> {
        let doc = SexpParser::load(filename)?;
        Self::parse(doc.iter())
    }
    fn parse<'a, I>(mut iter: I) -> Result<Self, Error>
    where
        I: Iterator<Item = State<'a>>,
    {
        let mut schema = Self::new();
        loop {
            let state = iter.next();
            match state {
                None => {
                    return Ok(schema);
                }
                Some(State::StartSymbol(name)) => {
                    if name == "uuid" {
                        schema.uuid = iter.next().unwrap().into();
                    } else if name == "paper" {
                        schema.paper_size = iter.next().unwrap().into();
                    } else if name == "title_block" {
                        schema.title_block = TitleBlock::from(&mut iter);
                    } else if name == "lib_symbols" {
                        let mut instance_count = 1;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                let symbol = LibrarySymbol::from(&mut iter);
                                schema.libraries.push(symbol);
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name == "no_connect" {
                        schema
                            .elements
                            .push(SchemaElement::NoConnect(NoConnect::from(&mut iter)));
                    } else if name == "junction" {
                        schema
                            .elements
                            .push(SchemaElement::Junction(Junction::from(&mut iter)));
                    } else if name == "wire" {
                        schema
                            .elements
                            .push(SchemaElement::Wire(Wire::from(&mut iter)));
                    } else if name == "text" {
                        schema
                            .elements
                            .push(SchemaElement::Text(Text::from(&mut iter)));
                    } else if name == "label" {
                        schema
                            .elements
                            .push(SchemaElement::Label(Label::from(&mut iter)));
                    } else if name == "global_label" {
                        schema
                            .elements
                            .push(SchemaElement::GlobalLabel(GlobalLabel::from(&mut iter)));
                    } else if name == "symbol" {
                        schema
                            .elements
                            .push(SchemaElement::Symbol(Symbol::from(&mut iter)));
                    } else if name == "sheet" {
                        schema
                            .elements
                            .push(SchemaElement::Sheet(Sheet::from(&mut iter)));
                    } else if name == "sheet_instances" {
                        let mut instance_count = 1;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                schema.sheet_instances.push(SheetInstance::from(&mut iter));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name == "symbol_instances" {
                        let mut instance_count = 1;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                schema
                                    .symbol_instances
                                    .push(SymbolInstance::from(&mut iter));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name != "kicad_sch" && name != "version" && name != "generator" {
                        println!("start symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_library(&self, key: &str) -> Option<&LibrarySymbol> {
        for lib in &self.libraries {
            if lib.lib_id == key {
                return Some(lib);
            }
        }
        None
    }
    pub fn get_symbol(&self, reference: &str, unit: u32) -> Option<&Symbol> {
        for lib in &self.elements {
            if let SchemaElement::Symbol(symbol) = lib {
                if let Some(r) = symbol.get_property("Reference") {
                    if symbol.unit as u32 == unit && reference == r {
                        return Some(symbol);
                    }
                }
            }
        }
        None
    }

    pub fn pages() -> usize {
        0
    }
    pub fn iter(&self, page: usize) -> std::slice::Iter<SchemaElement> {
        self.elements.iter()
    }
    pub fn iter_all(&self) -> std::slice::Iter<SchemaElement> {
        self.elements.iter()
    }
    pub fn plot(&self, filename: &str, scale: f64, border: bool, theme: &str) -> Result<(), Error> {
        let image_type = if filename.ends_with(".svg") {
            ImageType::Svg
        } else if filename.ends_with(".png") {
            ImageType::Png
        } else {
            ImageType::Pdf
        };
        let theme = if theme == "kicad_2000" {
            Theme::kicad_2000() //TODO:
        } else {
            Theme::kicad_2000()
        };

        use crate::plot::{PlotIterator, Plotter};
        let iter = self.iter(0).plot(self, theme, border).flatten().collect();
        let mut cairo = CairoPlotter::new(&iter);

        check_directory(filename)?;
        let out: Box<dyn Write> = Box::new(File::create(filename)?);
        cairo.plot(out, border, scale, image_type)?;
        Ok(())
    }
    pub fn write(&self, out: &mut dyn Write) -> Result<(), Error> {
        out.write_all(b"(kicad_sch ")?;

        out.write_all(b"(version ")?;
        out.write_all("20211123".as_bytes())?;
        out.write_all(b") ")?;
        out.write_all(b"(generator ")?;
        out.write_all("elektron".as_bytes())?;
        out.write_all(b")\n\n")?;

        out.write_all(b"  (uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n\n")?;
        out.write_all(b"  (paper \"")?;
        out.write_all(self.paper_size.to_string().as_bytes())?;
        out.write_all(b"\")\n\n")?;
        self.title_block.write(out, 1)?;

        out.write_all(b"  (lib_symbols\n")?;
        for lib in &self.libraries {
            lib.write(out, 2)?;
        }
        out.write_all(b"  )\n")?;

        for item in self.iter(0) {
            match item {
                SchemaElement::Symbol(symbol) => {
                    symbol.write(out, 1)?;
                }
                SchemaElement::NoConnect(no_connect) => {
                    no_connect.write(out, 1)?;
                }
                SchemaElement::Junction(junction) => {
                    junction.write(out, 1)?;
                }
                SchemaElement::Wire(wire) => {
                    wire.write(out, 1)?;
                }
                SchemaElement::Label(label) => {
                    println!("found label {:?}", label);
                    label.write(out, 1)?;
                }
                SchemaElement::GlobalLabel(global_label) => {
                    global_label.write(out, 1)?;
                }
                SchemaElement::Text(text) => {
                    text.write(out, 1)?;
                }
                SchemaElement::Sheet(sheet) => {
                    sheet.write(out, 1)?;
                }
                SchemaElement::SheetInstance(sheet_instances) => {
                    out.write_all(b"  (sheet_instances\n")?;
                    for instance in sheet_instances {
                        instance.write(out, 2)?;
                    }
                    out.write_all(b"  )\n")?;
                }
                SchemaElement::SymbolInstance(symbol_instances) => {
                    out.write_all(b"  (symbol_instances\n")?;
                    for instance in symbol_instances {
                        instance.write(out, 2)?;
                    }
                    out.write_all(b"  )\n")?;
                }
            }
        }
        out.write_all(b")\n")?;
        Ok(())
    }
    pub fn push(&mut self, element: SchemaElement) {
        self.elements.push(element);
    }
}

#[cfg(test)]
mod tests {
    use ndarray::arr1;

    use crate::sexp::Schema;

    #[test]
    fn nodes_iter() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        assert_eq!(43, doc.iter(0).count());
    }
    #[test]
    fn get_library() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        let library = doc.get_library("Amplifier_Operational:TL072").unwrap();
        assert_eq!("Amplifier_Operational:TL072", library.lib_id);
        assert_eq!(arr1(&[0.0, 5.08]), library.property[0].at);
    }
}
