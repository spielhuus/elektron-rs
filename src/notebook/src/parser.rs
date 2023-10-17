use super::cells::AudioCell;
use super::cells::ContentCell;
use crate::error::Error;
use std::collections::HashMap;
use std::slice::Iter;

use crate::pest;
use pest::iterators::Pair;
use pest::Parser;

use super::cells::{
    Cell, D3Cell, ElektronCell, FigureCell, JavascriptCell, PlotCell, PythonCell, TikzCell,
};

#[derive(Parser)]
#[grammar = "rule.pest"]
pub struct LangParser;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lang {
    //TODO: Circuit,
    Audio,
    Python,
    Latex,
    Figure,
    Plot,
    Javascript,
    D3,
    Elektron,
    Unknown(String),
}

trait LangDispatch {
    fn cell(&self, arguments: &HashMap<String, ArgType>, code: &Vec<String>)
        -> Result<Cell, Error>;
}

impl LangDispatch for Lang {
    fn cell(
        &self,
        arguments: &HashMap<String, ArgType>,
        code: &Vec<String>,
    ) -> Result<Cell, Error> {
        match self {
            Lang::Audio => Ok(Cell::Audio(AudioCell(arguments.clone(), code.clone()))),
            Lang::Python => Ok(Cell::Python(PythonCell(arguments.clone(), code.clone()))),
            Lang::Latex => Ok(Cell::Tikz(TikzCell(arguments.clone(), code.clone()))),
            Lang::Figure => Ok(Cell::Figure(FigureCell(arguments.clone(), code.clone()))),
            Lang::Plot => Ok(Cell::Plot(PlotCell(arguments.clone(), code.clone()))),
            Lang::Javascript => Ok(Cell::Javascript(JavascriptCell(
                arguments.clone(),
                code.clone(),
            ))),
            Lang::D3 => Ok(Cell::D3(D3Cell(arguments.clone(), code.clone()))),
            Lang::Elektron => Ok(Cell::Elektron(ElektronCell(
                arguments.clone(),
                code.clone(),
            ))),
            //TODO: Lang::Circuit => Ok(Cell::Circuit(CircuitCell(arguments.clone(), code.clone()))),
            Lang::Unknown(lang) => Err(Error::LanguageNotSupported(lang.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Content,
    Collect,
    //TODO Cell(Cell),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    String(String),
    Number(String),
    List(Vec<String>),
    Options(HashMap<String, ArgType>),
}

impl std::fmt::Display for ArgType {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ArgType::String(string) => {
                write!(f, "{}", string)
            }
            ArgType::Number(number) => {
                write!(f, "{}", number)
            }
            ArgType::List(list) => {
                for l in list {
                    write!(f, "{}", l)?;
                }
                Ok(())
            }
            ArgType::Options(map) => {
                for (k, v) in map {
                    write!(f, "{}: {}", k, v)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellParser {
    state: State,
    pub language: Lang,
    pub arguments: HashMap<String, ArgType>,
    pub code: Vec<String>,
    content: Vec<Cell>,
}

impl CellParser {
    pub fn new() -> Self {
        Self {
            state: State::Content,
            language: Lang::Unknown(String::new()),
            arguments: HashMap::new(),
            code: Vec::new(),
            content: Vec::new(),
        }
    }

    fn set_language(&mut self, language: &str) {
        self.language = if language == "python" {
            Lang::Python
        } else if language == "audio" {
            Lang::Audio
        } else if language == "latex" {
            Lang::Latex
        } else if language == "figure" {
            Lang::Figure
        } else if language == "plot" {
            Lang::Plot
        } else if language == "javascript" {
            Lang::Javascript
        } else if language == "d3" {
            Lang::D3
        } else if language == "elektron" {
            Lang::Elektron
        /*TODO: } else if language == "circuit" {
        Lang::Circuit */
        } else {
            Lang::Unknown(language.to_string())
        };
    }

    fn push_code(&mut self, code: &str) {
        self.code.push(code.to_string());
    }

    fn get_property(&self, record: Pair<Rule>) -> Result<(String, ArgType), Error> {
        let mut argument = None;
        let mut name = None;
        for inner in record.into_inner() {
            match inner.as_rule() {
                Rule::name => {
                    name = Some(inner.as_str().to_string());
                }
                Rule::prop_value => {
                    if let Some(name) = &name {
                        for val in inner.into_inner() {
                            match val.as_rule() {
                                Rule::number => {
                                    argument = Some(ArgType::String(val.as_str().to_string()));
                                }
                                Rule::boolean => {
                                    argument = Some(ArgType::String(val.as_str().to_string()));
                                }
                                Rule::sq_value => {
                                    let mut sq = val.into_inner();
                                    sq.next(); //skip quote
                                    argument = Some(ArgType::String(
                                        sq.next().unwrap().as_str().to_string(),
                                    ));
                                }
                                Rule::dq_value => {
                                    let mut sq = val.into_inner();
                                    sq.next(); //skip quote
                                    argument = Some(ArgType::String(
                                        sq.next().unwrap().as_str().to_string(),
                                    ));
                                }
                                Rule::list => {
                                    let mut result = Vec::new();
                                    for item in val.into_inner() {
                                        let mut sq = item.into_inner().next().unwrap().into_inner();
                                        sq.next(); //skip quote
                                        result.push(sq.next().unwrap().as_str().to_string());
                                    }
                                    argument = Some(ArgType::List(result));
                                }
                                Rule::options => {
                                    let mut result = HashMap::new();
                                    for item in val.into_inner() {
                                        if let Rule::property = item.as_rule() {
                                            let opts = self.get_property(item)?;
                                            result.insert(opts.0, opts.1);
                                        }
                                    }
                                    argument = Some(ArgType::Options(result));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if let (Some(name), Some(arg)) = (name, argument) {
            Ok((name, arg))
        } else {
            Err(Error::Notebook(String::new(), String::from("not found")))
        }
    }

    fn parse(&mut self, line: &str) -> Result<(), Error> {
        let parsed = LangParser::parse(Rule::file, line);
        if let Ok(mut parsed) = parsed {
            let file = parsed.next().unwrap(); // get and unwrap the `file` rule; never fails

            for record in file.into_inner() {
                match record.as_rule() {
                    Rule::lang => {
                        self.set_language(record.as_str());
                    }
                    Rule::property => {
                        let prop = self.get_property(record)?;
                        self.arguments.insert(prop.0, prop.1);
                    }
                    _ => {}
                }
            }
        } else {
            return Err(Error::Notebook(
                line.to_string(),
                parsed.err().unwrap().to_string(),
            ));
        }
        Ok(())
    }

    pub fn push(&mut self, line: &str) -> Result<(), Error> {
        if line == "```" {
            if self.state == State::Collect {
                self.content
                    .push(self.language.cell(&self.arguments, &self.code)?);
                self.clear();
            }
        } else if line.starts_with("```") && line.ends_with("```") && self.state == State::Content {
            if let Err(error) = self.parse(line) {
                self.content.push(Cell::Error(error.to_string()));
            } else {
                self.content
                    .push(self.language.cell(&self.arguments, &self.code)?);
            }
            self.clear();
        } else if line.starts_with("```") && self.state == State::Content {
            if let Err(error) = self.parse(line) {
                self.content.push(Cell::Error(error.to_string()));
            }
            self.state = State::Collect;
        } else if self.state == State::Collect {
            self.push_code(line);
        } else {
            self.content.push(Cell::Content(ContentCell(
                HashMap::new(),
                vec![line.to_string()],
            )))
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        self.code = Vec::new();
        self.state = State::Content;
        self.language = Lang::Unknown(String::new());
        self.arguments = HashMap::new();
    }

    pub fn iter(&self) -> Iter<'_, Cell> {
        self.content.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{ArgType, Cell, Lang};

    use super::CellParser;

    #[test]
    fn test_parse() {
        let mut command = CellParser::new();
        command
            .parse("```{python, error=TRUE, echo=FALSE, include=TRUE}")
            .unwrap();
        assert_eq!(Lang::Python, command.language);
        assert_eq!(3, command.arguments.len());
    }
    #[test]
    fn test_python() {
        let mut command = CellParser::new();
        command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE}")
            .unwrap();
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.get(0).unwrap());
                assert_eq!(3, cell.0.len());
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn no_cell_on_error() {
        let mut command = CellParser::new();
        let res = command.push("```{scala, error=TRUE, echo=FALSE, include=TRUE}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_err());
        if let Some(content) = command.iter().next() {
            if let Cell::Error(_) = content {
            } else {
                panic!("result is not an error cell: {:?}", content)
            }
        }
    }
    #[test]
    fn no_arg_quoted() {
        let mut command = CellParser::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', fig.align='center', fig.cap='Linear amplifier'}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.get(0).unwrap());
                assert_eq!(6, cell.0.len());
                if let ArgType::String(str) = cell.0.get("fig.cap").unwrap() {
                    assert_eq!("Linear amplifier", str);
                } else {
                    panic!("result is not a string: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn arg_number() {
        let mut command = CellParser::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', number=123}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.get(0).unwrap());
                assert_eq!(5, cell.0.len());
                if let ArgType::String(str) = cell.0.get("number").unwrap() {
                    assert_eq!("123", str);
                } else {
                    panic!("result is not a string: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn arg_list() {
        let mut command = CellParser::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', list=[\"a\", \"b\", \"c\"]}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.get(0).unwrap());
                assert_eq!(5, cell.0.len());
                if let ArgType::List(str) = cell.0.get("list").unwrap() {
                    assert_eq!(vec!["a", "b", "c"], *str);
                } else {
                    panic!("result is not a list: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn single_line() {
        let mut command = CellParser::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', list=[\"a\", \"b\", \"c\"]}```");
        assert!(res.is_ok());
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert!(cell.1.is_empty());
                assert_eq!(5, cell.0.len());
                if let ArgType::List(str) = cell.0.get("list").unwrap() {
                    assert_eq!(vec!["a", "b", "c"], *str);
                } else {
                    panic!("result is not a list: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn options() {
        let mut command = CellParser::new();
        let res = command.push("```{d3 data=\"test\", options=list(foo=\"bar\")}");
        assert!(res.is_ok());
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert!(cell.1.is_empty());
                assert_eq!(5, cell.0.len());
                if let ArgType::Options(o) = cell.0.get("options").unwrap() {
                    assert_eq!(2, o.len());
                    assert_eq!(ArgType::String(String::from("bar")), *o.get("foo").unwrap());
                } else {
                    panic!("result is not an option: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
}
