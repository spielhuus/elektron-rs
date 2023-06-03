use std::{fs::File, io::Write, path::Path};

use super::{
    model::{
        GlobalLabel, Junction, Label, LibrarySymbol, NoConnect, PaperSize, SchemaElement, Sheet,
        SheetInstance, Symbol, SymbolInstance, Text, TitleBlock, Wire,
    },
    parser::{SexpParser, State},
    uuid,
    write::SexpWriter,
};
use crate::error::Error;

use super::model::{Bus, BusEntry, HierarchicalLabel, Polyline};

use uuid::Uuid;

#[derive(Default, Clone)]
pub struct Schema {
    pub pages: Vec<Page>,
}
impl Schema {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }
    ///load the schema from a file.
    pub fn load(filename: &str) -> Result<Self, Error> {
        let doc = Page::load(filename, "root")?;
        let mut sheets = Vec::new();
        for sheet in &doc.elements {
            if let SchemaElement::Sheet(sheet) = sheet {
                let path = Path::new(&filename).parent().unwrap();
                let filename = format!(
                    "{}/{}",
                    path.to_str().unwrap(),
                    sheet.sheet_filename().unwrap()
                );
                sheets.push(Page::load(
                    filename.as_str(),
                    sheet.sheet_filename().unwrap().as_str(),
                )?);
            }
        }
        let mut pages = vec![doc];
        pages.extend(sheets);
        Ok(Self { pages })
    }
    pub fn new_page(&mut self) {
        self.pages.push(Page::new(String::new()));
    }
    ///push element to page. will also create the SymbolInstace if required.
    pub fn push(&mut self, page: usize, element: SchemaElement) -> Result<(), Error> {
        if let Some(page) = self.pages.get_mut(page) {
            page.elements.push(element);
            Ok(())
        } else {
            Err(Error::ParseError)
        }
    }
    ///get the library symbol from a page.
    pub fn get_library(&self, key: &str) -> Option<&LibrarySymbol> {
        for page in &self.pages {
            for lib in &page.libraries {
                if lib.lib_id == key {
                    return Some(lib);
                }
            }
        }
        None
    }
    ///search symbol from all pages
    pub fn get_symbol(&self, reference: &str, unit: u32) -> Option<&Symbol> {
        for page in &self.pages {
            for lib in &page.elements {
                if let SchemaElement::Symbol(symbol) = lib {
                    if let Some(r) = symbol.get_property("Reference") {
                        if (unit == 0 || symbol.unit == unit) && reference == r {
                            return Some(symbol);
                        }
                    }
                }
            }
        }
        None
    }
    /// return the number of pages.
    pub fn pages(&self) -> usize {
        self.pages.len()
    }
    /// return the number of pages.
    pub fn page(&self, page: usize) -> Option<&Page> {
        self.pages.get(page)
    }
    /// return the number of pages.
    pub fn page_mut(&mut self, page: usize) -> Option<&mut Page> {
        self.pages.get_mut(page)
    }
    ///iterate over the elements in a page.
    pub fn iter(&self, page: usize) -> Result<std::slice::Iter<SchemaElement>, Error> {
        if let Some(page) = self.pages.get(page) {
            Ok(page.elements.iter())
        } else {
            Err(Error::ParseError) //TODO: meaningfull error
        }
    }

    ///iterate the elements in all pages.
    pub fn iter_all(&self) -> impl Iterator<Item = &SchemaElement> {
        self.pages.iter().flat_map(|el| el.elements.iter())
    }

    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut SchemaElement> {
        self.pages.iter_mut().flat_map(|el| el.elements.iter_mut())
    }

    pub fn write(&self, filename: &str) -> Result<(), Error> {
        let mut out = File::create(filename)?;
        self.pages.first().unwrap().write(&mut out)?;
        for page in self.pages.iter().skip(1) {
            let path = Path::new(&filename).parent().unwrap();
            let sheetname = if path.to_str().unwrap() == "" {
                page.filename.clone()
            } else {
                format!("{}/{}", path.to_str().unwrap(), page.filename)
            };
            let mut out = File::create(sheetname)?;
            page.write(&mut out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    filename: String,
    uuid: String,
    pub paper_size: PaperSize,
    pub title_block: Option<TitleBlock>,
    elements: Vec<SchemaElement>,
    pub libraries: Vec<LibrarySymbol>,
    sheet_instances: Vec<SheetInstance>,
    symbol_instances: Vec<SymbolInstance>,
}

impl Page {
    pub fn new(filename: String) -> Self {
        Self {
            filename,
            uuid: uuid!(),
            paper_size: PaperSize::A4,
            title_block: None,
            elements: Vec::new(),
            libraries: Vec::new(),
            sheet_instances: Vec::new(),
            symbol_instances: Vec::new(),
        }
    }
    pub fn load(filename: &str, name: &str) -> Result<Self, Error> {
        let doc = SexpParser::load(filename)?;
        Self::parse(doc.iter(), name.to_string())
    }
    fn parse<'a, I>(mut iter: I, filename: String) -> Result<Self, Error>
    where
        I: Iterator<Item = State<'a>>,
    {
        let mut schema = Self::new(filename);
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
                        schema.title_block = Some(TitleBlock::from(&mut iter));
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
                    } else if name == "polyline" {
                        schema
                            .elements
                            .push(SchemaElement::Polyline(Polyline::from(&mut iter)));
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
                    } else if name == "bus" {
                        schema
                            .elements
                            .push(SchemaElement::Bus(Bus::from(&mut iter)));
                    } else if name == "bus_entry" {
                        schema
                            .elements
                            .push(SchemaElement::BusEntry(BusEntry::from(&mut iter)));
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
                    } else if name == "hierarchical_label" {
                        schema.elements.push(SchemaElement::HierarchicalLabel(
                            HierarchicalLabel::from(&mut iter),
                        ));
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
        if let Some(title_block) = &self.title_block {
            title_block.write(out, 1)?;
        }
        out.write_all(b"  (lib_symbols\n")?;
        for lib in &self.libraries {
            lib.write(out, 2)?;
        }
        out.write_all(b"  )\n")?;

        for item in self.elements.iter() {
            match item {
                SchemaElement::Symbol(symbol) => {
                    symbol.write(out, 1)?;
                }
                SchemaElement::Polyline(line) => {
                    line.write(out, 1)?;
                }
                SchemaElement::Bus(bus) => {
                    bus.write(out, 1)?;
                }
                SchemaElement::BusEntry(bus) => {
                    bus.write(out, 1)?;
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
                    label.write(out, 1)?;
                }
                SchemaElement::GlobalLabel(global_label) => {
                    global_label.write(out, 1)?;
                }
                SchemaElement::HierarchicalLabel(hierarchical_label) => {
                    hierarchical_label.write(out, 1)?;
                }
                SchemaElement::Text(text) => {
                    text.write(out, 1)?;
                }
                SchemaElement::Sheet(sheet) => {
                    sheet.write(out, 1)?;
                }
            }
        }

        if !self.sheet_instances.is_empty() {
            out.write_all(b"  (sheet_instances\n")?;
            for instance in &self.sheet_instances {
                instance.write(out, 2)?;
            }
            out.write_all(b"  )\n")?;
        }

        if !self.symbol_instances.is_empty() {
            out.write_all(b"  (symbol_instances\n")?;
            for instance in &self.symbol_instances {
                instance.write(out, 2)?;
            }
            out.write_all(b"  )\n")?;
        }

        out.write_all(b")\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ndarray::arr1;

    use crate::sexp::{
        model::{SchemaElement, Sheet},
        schema::Schema,
    };

    #[test]
    fn nodes_iter() {
        let doc = Schema::load("files/summe/summe.kicad_sch").unwrap();
        assert_eq!(398, doc.iter(0).unwrap().count());
    }
    #[test]
    fn get_library() {
        let doc = Schema::load("files/summe/summe.kicad_sch").unwrap();
        let library = doc.get_library("Amplifier_Operational:TL072").unwrap();
        assert_eq!("Amplifier_Operational:TL072", library.lib_id);
        assert_eq!(arr1(&[0.0, 5.08]), library.property[0].at);
    }
    #[test]
    fn sheet_names() {
        let doc = Schema::load("files/multipage/multipage.kicad_sch").unwrap();
        let res: Vec<&Sheet> = doc
            .iter(0)
            .unwrap()
            .filter_map(|el| {
                if let SchemaElement::Sheet(sheet) = el {
                    Some(sheet)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(1, res.len());
        assert_eq!("subsheet", res[0].sheet_name().unwrap());
        assert_eq!("subsheet.kicad_sch", res[0].sheet_filename().unwrap());
    }
    #[test]
    fn parse_multipage() {
        let doc = Schema::load("files/multipage/multipage.kicad_sch").unwrap();
        assert_eq!(2, doc.pages());
    }
    #[test]
    fn iter_multipage() {
        let doc = Schema::load("files/multipage/multipage.kicad_sch").unwrap();
        let count = doc.iter_all().count();
        assert_eq!(27, count);
    }
    /* #[test]
    fn read_write() {
        let path = Path::new("/tmp/multipage");
        if path.exists() {
            std::fs::remove_dir_all("/tmp/multipage").unwrap();
        }
        let doc = Schema::load("files/multipage/multipage.kicad_sch").unwrap();
        std::fs::create_dir("/tmp/multipage/").unwrap();
        doc.write("/tmp/multipage/multipage.kicad_sch").unwrap();

        let left = std::fs::read_to_string("files/multipage/multipage.kicad_sch").unwrap();
        let right = std::fs::read_to_string("/tmp/multipage/multipage.kicad_sch").unwrap();
        for diff in diff::lines(left.as_str(), right.as_str()) {
            match diff {
                diff::Result::Left(l) => {
                    if !l.is_empty() {
                        assert_eq!("(kicad_sch (version 20211123) (generator eeschema)", l);
                    }
                }
                diff::Result::Both(_, _) => {}
                diff::Result::Right(r) => {
                    assert_eq!("(kicad_sch (version 20211123) (generator elektron)", r);
                }
            }
        }

        let left = std::fs::read_to_string("files/multipage/subsheet.kicad_sch").unwrap();
        let right = std::fs::read_to_string("/tmp/multipage/subsheet.kicad_sch").unwrap();
        for diff in diff::lines(left.as_str(), right.as_str()) {
            match diff {
                diff::Result::Left(l) => {
                    if !l.is_empty() {
                        assert_eq!("(kicad_sch (version 20211123) (generator eeschema)", l);
                    }
                }
                diff::Result::Both(_, _) => {}
                diff::Result::Right(r) => {
                    if !r.is_empty() {
                        assert_eq!("(kicad_sch (version 20211123) (generator elektron)", r);
                    }
                }
            }
        }
    } */
}
