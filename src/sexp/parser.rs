
use crate::sexp::{Error, SexpText, SexpValue, SexpType, SexpNode, SexpConsumer};

enum NodeState {
    Root,
    LibrarySymbols,
    SheetInstances,
    SymbolInstances,
}

enum State {
    NotStarted,
    Delimiter,
    Value,
    ParenthesisOpen,
    SingleQuoted,
    DoubleQuoted,
    Backslash,
}

pub struct SexpParser<'a> {
    source: std::str::Chars<'a>,
    state: State,
    node_state: NodeState,
    index: u8,
}

impl<'a> SexpParser<'a> {
    pub fn new(source: &str) -> SexpParser {
        SexpParser {
            source: source.chars(),
            state: State::NotStarted,
            node_state: NodeState::Root,
            index: 0,
        }
    }

    pub fn parse(&mut self, consumer: &mut dyn SexpConsumer) -> Result<SexpNode, Error> {
        use State::*;

        let mut version = String::with_capacity(1024);
        //let mut generator = String::with_capacity(1024);
        let mut name = String::with_capacity(1024);
        let mut word = String::with_capacity(1024);
        let mut values: Vec<SexpType> = Vec::new();

        loop {
            let c = self.source.next();
            self.state = match self.state {
                NotStarted => match c {
                    None => break,
                    Some('(') => ParenthesisOpen,
                    Some(_c) => NotStarted,
                },
                ParenthesisOpen => match c {
                    None => break,
                    Some(' ') | Some('\n') => {
                        if self.index == 0 {
                            if let Some(c) = c {
                                if !c.is_whitespace() {
                                    println!("No other root level element expected: {:?}", c);
                                }
                            }
                        } else if self.index == 1 && name == "lib_symbols" {
                            self.node_state = NodeState::LibrarySymbols;
                            consumer.start_library_symbols()?;
                        } else if self.index == 1 && name == "sheet_instances" {
                            self.node_state = NodeState::SheetInstances;
                            consumer.start_sheet_instances()?;
                        } else if self.index == 1 && name == "symbol_instances" {
                            self.node_state = NodeState::SymbolInstances;
                            consumer.start_symbol_instances()?;
                        }
                        Delimiter
                    }
                    Some(')') => {
                        break;
                    }
                    Some(c) => {
                        name.push(c);
                        ParenthesisOpen
                    }
                },
                Delimiter => match c {
                    None => break,
                    Some('\'') => SingleQuoted,
                    Some('\"') => DoubleQuoted,
                    Some('(') => {
                        self.state = ParenthesisOpen;
                        self.index += 1;
                        let node = self.parse(consumer)?;
                        if self.index == 1 && node.name == "version" {
                            version = node.values.get(0).unwrap().to_string();
                        } else if self.index == 1 && node.name == "generator" {
                            let generator = node.values.get(0).unwrap().to_string();
                            consumer.start(&version, &generator).unwrap();
                        } else {
                            values.push(SexpType::ChildSexpNode(node));
                        }
                        self.index -= 1;
                        Delimiter
                    }
                    Some(')') => {
                        break;
                    }
                    Some(' ') | Some('\n') => Delimiter,
                    Some(c) => {
                        word.push(c);
                        Value
                    }
                },
                Value => match c {
                    None => break,
                    Some('\'') => SingleQuoted,
                    Some('\"') => DoubleQuoted,
                    Some(' ') | Some('\n') => {
                        values.push(SexpType::ChildSexpValue(SexpValue::new(word.clone())));
                        word.clear();
                        Delimiter
                    }
                    Some(')') => {
                        values.push(SexpType::ChildSexpValue(SexpValue::new(word.clone())));
                        word.clear();
                        break;
                    }
                    Some(c) => {
                        word.push(c);
                        Value
                    }
                },
                SingleQuoted => match c {
                    None => break, //return Err(ParseError),
                    Some('\'') => {
                        values.push(SexpType::ChildSexpText(SexpText::new(word.clone())));
                        word.clear();
                        Delimiter
                    }
                    Some(c) => {
                        word.push(c);
                        SingleQuoted
                    }
                },
                DoubleQuoted => match c {
                    None => break, //return Err(ParseError),
                    Some('\\') => Backslash,
                    Some('\"') => {
                        values.push(SexpType::ChildSexpText(SexpText::new(word.clone())));
                        word.clear();
                        Delimiter
                    }
                    Some(c) => {
                        word.push(c);
                        DoubleQuoted
                    }
                },
                Backslash => match c {
                    None => break, //return Err(ParseError),
                    Some(c) => {
                        word.push('\\');
                        word.push(c);
                        DoubleQuoted
                    }
                },
            }
        }

        let mut node = SexpNode::from(name, values);
        match self.node_state {
            NodeState::Root => {
                if self.index == 1 {
                    consumer.visit(&mut node)?;
                }
            }
            NodeState::LibrarySymbols => {
                if self.index == 1 {
                    consumer.end_library_symbols()?;
                    self.node_state = NodeState::Root;
                } else if self.index == 2 {
                    consumer.visit(&mut node)?;
                }
            }
            NodeState::SheetInstances => {
                if self.index == 1 {
                    consumer.end_sheet_instances()?;
                    self.node_state = NodeState::Root;
                } else if self.index == 2 {
                    consumer.visit(&mut node)?;
                }
            }
            NodeState::SymbolInstances => {
                if self.index == 1 {
                    consumer.end_symbol_instances()?;
                    self.node_state = NodeState::Root;
                } else if self.index == 2 {
                    consumer.visit(&mut node)?;
                }
            }
        }
        if self.index == 0 {
            consumer.end()?;
        }
        Ok(node)
    }
}

