use crate::{sexp::{
    model::{Junction, SheetInstance, SymbolInstance},
    parser::State,
}, Error};

use super::{model::{
    GlobalLabel, Label, LibrarySymbol, NoConnect, SchemaElement, Symbol, Text, TitleBlock, Wire, Sheet, PaperSize,
}, SexpParser};

pub struct Schema {
    uuid: String,
    paper_size: PaperSize,
    title_block: TitleBlock,
    elements: Vec<SchemaElement>,
    libraries: Vec<SchemaElement>,
    sheet_instances: Vec<SheetInstance>,
    symbol_instances: Vec<SymbolInstance>,
    pages: Vec<Schema>,
}

impl Schema {
    fn new() -> Self {
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
        let mut schema =  Self::parse(doc.iter())?;
        Ok(schema)
    }
    fn parse<'a, I>(iter: I) -> Result<Self, Error>
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
                    if name == "version" {
                        //return Some(SchemaElement::Version(iter.next().unwrap().into()));
                    } else if name == "generator" {
                        //return Some(SchemaElement::Generator(iter.next().unwrap().into()));
                    } else if name == "uuid" {
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
                                schema.libraries.push(SchemaElement::LibrarySymbol(symbol));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name == "no_connect" {
                        schema.elements.push(SchemaElement::NoConnect(NoConnect::from(&mut iter)));
                    } else if name == "junction" {
                        schema.elements.push(SchemaElement::Junction(Junction::from(&mut iter)));
                    } else if name == "wire" {
                        schema.elements.push(SchemaElement::Wire(Wire::from(&mut iter)));
                    } else if name == "text" {
                        schema.elements.push(SchemaElement::Text(Text::from(&mut iter)));
                    } else if name == "label" {
                        schema.elements.push(SchemaElement::Label(Label::from(&mut iter)));
                    } else if name == "global_label" {
                        schema.elements.push(SchemaElement::GlobalLabel(GlobalLabel::from(
                            &mut iter,
                        )));
                    } else if name == "symbol" {
                        schema.elements.push(SchemaElement::Symbol(Symbol::from(&mut iter)));
                    } else if name == "sheet" {
                        schema.elements.push(SchemaElement::Sheet(Sheet::from(&mut iter)));
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
                                schema.symbol_instances.push(SymbolInstance::from(&mut iter));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                    } else if name != "kicad_sch" {
                        println!("start symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }
    
    pub fn pages() -> usize {
        0

    }
    pub fn iter(&self, page: usize) -> std::slice::Iter<SchemaElement> {
        return self.elements.iter();
    }
    pub fn iter_all() {

    }
}


/* impl<'a, I> Iterator for Schema<I>
where
    I: Iterator<Item = State<'a>>,
{
    type Item = SchemaElement;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let state = self.iter.next();
            match state {
                None => {
                    return None;
                }
                Some(State::StartSymbol(name)) => {
                    if name == "version" {
                        return Some(SchemaElement::Version(self.iter.next().unwrap().into()));
                    } else if name == "generator" {
                        return Some(SchemaElement::Generator(self.iter.next().unwrap().into()));
                    } else if name == "uuid" {
                        return Some(SchemaElement::Uuid(self.iter.next().unwrap().into()));
                    } else if name == "paper" {
                        return Some(SchemaElement::Paper(self.iter.next().unwrap().into()));
                    } else if name == "title_block" {
                        return Some(SchemaElement::TitleBlock(TitleBlock::from(&mut self.iter)));
                    } else if name == "lib_symbols" {
                        let mut symbols: HashMap<String, LibrarySymbol> = HashMap::new();
                        let mut instance_count = 1;
                        loop {
                            let state = self.iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                let symbol = LibrarySymbol::from(&mut self.iter);
                                symbols.insert(symbol.lib_id.clone(), symbol);
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                        return Some(SchemaElement::LibrarySymbols(symbols));
                    } else if name == "no_connect" {
                        return Some(SchemaElement::NoConnect(NoConnect::from(&mut self.iter)));
                    } else if name == "junction" {
                        return Some(SchemaElement::Junction(Junction::from(&mut self.iter)));
                    } else if name == "wire" {
                        return Some(SchemaElement::Wire(Wire::from(&mut self.iter)));
                    } else if name == "text" {
                        return Some(SchemaElement::Text(Text::from(&mut self.iter)));
                    } else if name == "label" {
                        return Some(SchemaElement::Label(Label::from(&mut self.iter)));
                    } else if name == "global_label" {
                        return Some(SchemaElement::GlobalLabel(GlobalLabel::from(
                            &mut self.iter,
                        )));
                    } else if name == "symbol" {
                        return Some(SchemaElement::Symbol(Symbol::from(&mut self.iter)));
                    } else if name == "sheet" {
                        return Some(SchemaElement::Sheet(Sheet::from(&mut self.iter)));
                    } else if name == "sheet_instances" {
                        let mut sheet_instances: Vec<SheetInstance> = Vec::new();
                        let mut instance_count = 1;
                        loop {
                            let state = self.iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                sheet_instances.push(SheetInstance::from(&mut self.iter));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                        return Some(SchemaElement::SheetInstance(sheet_instances));
                    } else if name == "symbol_instances" {
                        let mut symbol_instances: Vec<SymbolInstance> = Vec::new();
                        let mut instance_count = 1;
                        loop {
                            let state = self.iter.next();
                            if let Some(State::StartSymbol(_)) = state {
                                instance_count += 1;
                                symbol_instances.push(SymbolInstance::from(&mut self.iter));
                                instance_count -= 1;
                            } else if let Some(State::EndSymbol) = state {
                                instance_count -= 1;
                                if instance_count == 0 {
                                    break;
                                }
                            }
                        }
                        return Some(SchemaElement::SymbolInstance(symbol_instances));
                    } else if name != "kicad_sch" {
                        println!("start symbol: {}", name);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<I> Schema<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
} */

/* pub trait SchemaIterator<T>: Iterator<Item = T> + Sized {
    fn node(self) -> Schema<Self> {
        Schema::new(self)
    }
} */
// impl<T, I: Iterator<Item = T>> SchemaIterator<T> for I {}

#[cfg(test)]
mod tests {
    use crate::sexp::{SchemaIterator, schema::Schema};

    #[test]
    fn nodes() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        println!("symbols: {}", doc.elements.len());
    }
    #[test]
    fn nodes_iter() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        assert_eq!(12, doc.iter(0).count());
    }
}
