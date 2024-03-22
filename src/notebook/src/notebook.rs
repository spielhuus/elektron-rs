use std::collections::HashMap;
use std::slice::Iter;

use log::trace;

use crate::cells::Cell;
use crate::error::Error;

const INSTR: char = '`';
const COPEN: char = '{';

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lang {
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

impl Lang {
    /// Get the Lang enum from a string or return an error.
    /// * `lang`: the lang string.
    pub fn from(lang: &str) -> Result<Lang, Error> {
        if lang == "audio" {
            Ok(Lang::Audio)
        } else if lang == "python" {
            Ok(Lang::Python)
        } else if lang == "latex" {
            Ok(Lang::Latex)
        } else if lang == "figure" {
            Ok(Lang::Figure)
        } else if lang == "Plot" {
            Ok(Lang::Plot)
        } else if lang == "javascript" {
            Ok(Lang::Javascript)
        } else if lang == "d3" {
            Ok(Lang::D3)
        } else if lang == "elektron" {
            Ok(Lang::Elektron)
        } else {
            trace!("Unknwon language: {}", lang);
            Err(Error::Notebook("Unknwon Lang:".to_string(), lang.to_string())) 
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Content,
    Collect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgType {
    String(String),
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
pub struct Notebook {
    line: usize,

    state: State,
    pub language: Lang,
    pub arguments: HashMap<String, ArgType>,
    pub code: Vec<String>,
    cells: Vec<Cell>,
}

impl Notebook {
    pub fn new() -> Self {
        Self {
            line: 0,

            state: State::Content,
            language: Lang::Unknown(String::from("lang not set")),
            arguments: HashMap::new(),
            code: Vec::new(),
            cells: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.language = Lang::Unknown(String::from("lang not set"));
        self.arguments.clear();
        self.code.clear();
    }

    pub fn from(content: &str) -> Result<Self, Error> {
        let mut notebook = Self::new();

        for line in content.lines() {
            notebook.push(line)?;
        }
        notebook.close();
        Ok(notebook)
    }

    pub fn open(filename: &str) -> Result<Self, Error> {
        Self::from(&std::fs::read_to_string(filename)?)
    }

    ///push a new line to the notebook.
    fn push(&mut self, line: &str) -> Result<(), Error> {

        self.line += 1;

        if self.state == State::Content && line.starts_with("```") {

            if !self.code.is_empty() {
                self.cells.push(Cell::Content(crate::cells::content::ContentCell(self.code.clone())));
                self.clear();
            }

            self.parse(line)?;
            if line.ends_with("```") {
                self.cells.push(Cell::from(&self.language, self.arguments.clone(), self.code.clone()));
                self.clear();
            } else {
                self.state = State::Collect;
            }

        } else if self.state == State::Collect && line.starts_with("```") {
            self.cells.push(Cell::from(&self.language, self.arguments.clone(), self.code.clone()));
            self.state = State::Content;
            self.clear();
        } else {
            self.code.push(line.to_string());
        }

        Ok(())
    }

    fn close(&mut self) {
        if !self.code.is_empty() {
            self.cells.push(Cell::Content(crate::cells::content::ContentCell(self.code.clone())));
            self.code.clear();
        }
    }

    ///parse a markdown scripting instruction.
    fn parse(&mut self, arg: &str) -> Result<(), Error> {
        enum ParserState {
            NotStarted,
            StartInstruction(u8),
            Lang,
            Key,
            Value,
            Quote,
            List,
            ListQuote,
        }
        let mut key = String::new();
        let mut value = String::new();
        let mut lang = String::new();
        let mut list: Vec<String> = Vec::new();
        let mut state = ParserState::NotStarted;
        let mut last_char = '\0';
        for c in arg.chars() {
            use ParserState::*;
            match state {
                ParserState::Quote => {
                    if (c == '"' || c == '\'') && last_char != '\\' {
                        state = ParserState::Value;
                    } else {
                        value.push(c);
                    }
                },
                ParserState::ListQuote => {
                    if (c == '"' || c == '\'') && last_char != '\\' {
                        state = ParserState::List;
                    } else {
                        value.push(c);
                    }
                },
                ParserState::List => {
                    if c == ',' {
                        list.push(value.trim().to_string());
                        value.clear();
                    } else if c == ']' {
                        list.push(value.trim().to_string());
                        value.clear();
                        self.arguments.insert(key.clone(), ArgType::List(list.clone()));
                        key.clear();
                        list.clear();
                        state = Key;
                    } else if c == '"' || c == '\'' {
                        state = ListQuote;
                    } else {
                        value.push(c);
                    }
                },
                ParserState::NotStarted => {
                    if c == INSTR {
                        state = StartInstruction(1);
                    } else {
                        return Err(Error::Notebook(
                                String::from("error parsing instruction"), 
                                String::from("instruction must begin with '")
                        ));
                    }
                },
                ParserState::StartInstruction(n) => {
                    if c == INSTR {
                        match n.cmp(&2) {
                            std::cmp::Ordering::Less => {
                                state = StartInstruction(n+1);
                            },
                            std::cmp::Ordering::Equal => {
                                state = Lang;
                            },
                            std::cmp::Ordering::Greater => {
                                return Err(Error::Notebook(
                                        String::from("error parsing instruction"), 
                                        String::from("to many starting !.")
                                ));
                            },
                        }
                    } else {
                        return Err(Error::Notebook(
                                String::from("error parsing instruction"), 
                                String::from("instruction must begin with !")
                        ));
                    }

                },
                ParserState::Lang => {
                    match c {
                        ',' | COPEN => {}
                        ' ' => {
                            let Ok(lang) = crate::notebook::Lang::from(&lang) else {
                                return Err(Error::Notebook(format!("Unknown lang at line: {}", self.line), format!("lang {} is not supported.", lang)));
                            };
                            self.language = lang;
                            state = Key;
                            continue
                        },
                        c => lang.push(c),
                    }
                },
                ParserState::Key => {
                    match c {
                        '{' | ',' | ' ' => continue,
                        '=' => state = Value,
                        c => key.push(c),
                    }
                },
                ParserState::Value => {
                    match c {
                        '[' => state = ParserState::List,
                        ',' => {},
                        ' ' | '}' => {
                            self.arguments.insert(key.clone(), ArgType::String(value.clone()));
                            key = String::new();
                            value = String::new();
                            state = Key;
                        },
                        '"' | '\'' => state = ParserState::Quote,
                        c => value.push(c),
                    }
                },
            }
            last_char = c;
        }
        if let ParserState::Lang = state {
            if !lang.is_empty() {
                let Ok(lang) = crate::notebook::Lang::from(&lang) else {
                    return Err(Error::Notebook(format!("Unknown lang at line: {}", self.line), format!("lang {} is not supported.", lang)));
                };
                self.language = lang;
            }
        }
        Ok(())
    }
    pub fn iter(&self) -> Iter<'_, Cell> {
        self.cells.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::notebook::{ArgType, Lang, Cell};
    use super::Notebook;

    #[test]
    fn test_unknown_lang() {
        let mut command = Notebook::new();
        let res = command
            .parse("```{suaheli, error=TRUE, echo=FALSE, include=TRUE}");

        if let Err(err) = res {
            assert_eq!("`Unknown lang at line: 0`: lang suaheli is not supported.", err.to_string());
        } else {
            panic!("should fail");
        }
    }
    #[test]
    fn test_parse_lang() {
        let mut command = Notebook::new();
        command
            .parse("```python")
            .unwrap();

        assert_eq!(Lang::Python, command.language);
    }
    #[test]
    fn test_parse_lang_line() {
        let mut command = Notebook::new();
        command
            .parse("```{python, error=TRUE, echo=FALSE, include=TRUE}")
            .unwrap();

        assert_eq!(Lang::Python, command.language);
        assert_eq!(3, command.arguments.len());
        assert_eq!(ArgType::String("TRUE".to_string()), *command.arguments.get("error").unwrap());
        assert_eq!(ArgType::String("FALSE".to_string()), *command.arguments.get("echo").unwrap());
        assert_eq!(ArgType::String("TRUE".to_string()), *command.arguments.get("include").unwrap());
    }
    #[test]
    fn no_arg_quoted() {
        let mut command = Notebook::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', fig.align='center', fig.cap='Linear amplifier'}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
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
    fn no_arg_quoted_and_escaped() {
        let mut command = Notebook::new();
        let res = command
            .push(r#"```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', fig.align='center', fig.cap='Linear \"escaped\" amplifier'}"#);
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
                assert_eq!(6, cell.0.len());
                if let ArgType::String(str) = cell.0.get("fig.cap").unwrap() {
                    assert_eq!(r#"Linear \"escaped\" amplifier"#, str);
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
        let mut command = Notebook::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', number=123}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
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
        let mut command = Notebook::new();
        let res = command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE, results='axis', list=[\"a\", \"b\", \"c\"]}");
        assert!(res.is_ok());
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        for content in command.iter() {
            if let Cell::Python(cell) = content {
                assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
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
        let mut command = Notebook::new();
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
    fn single_schema() {
        let mut command = Notebook::new();
        let res = command
            .push(r#"```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```"#);
        assert!(res.is_ok());
        for content in command.iter() {
        println!("{:?}", content);
            if let Cell::Elektron(cell) = content {
                assert!(cell.1.is_empty());
                assert_eq!(4, cell.0.len());
                if let ArgType::String(str) = cell.0.get("command").unwrap() {
                    assert_eq!("schema", *str);
                } else {
                    panic!("result for command is not a string: {:?}", res)
                }
                if let ArgType::List(str) = cell.0.get("input").unwrap() {
                    assert_eq!(vec!["main", "mount"], *str);
                } else {
                    panic!("result for input is not a list: {:?}", res)
                }
                if let ArgType::String(str) = cell.0.get("border").unwrap() {
                    assert_eq!("TRUE", *str);
                } else {
                    panic!("result for border is not a string: {:?}", res)
                }
            } else {
                panic!("result is not a cell: {:?}", res)
            }
        }
    }
    #[test]
    fn options() {
        let mut command = Notebook::new();
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
    #[test]
    fn test_python() {
        let mut command = Notebook::new();
        command
            .push("```{python, error=TRUE, echo=FALSE, include=TRUE}")
            .unwrap();
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        assert_eq!(1, command.cells.len());
        let cell = command.cells.pop().unwrap();
        if let Cell::Python(cell) = cell {
            if let ArgType::String(value) = cell.0.get("error").unwrap() {
                assert_eq!("TRUE", value);
            }
            assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
        } else {
            assert_eq!(0, 1);
        }
    }
    #[test]
    fn test_python_without_comma() {
        let mut command = Notebook::new();
        command
            .push("```{python error=TRUE echo=FALSE include=TRUE}")
            .unwrap();
        command.push(r#"println("Hello World")"#).unwrap();
        let res = command.push("```");
        assert!(res.is_ok());
        assert_eq!(1, command.cells.len());
        let cell = command.cells.pop().unwrap();
        if let Cell::Python(cell) = cell {
            if let ArgType::String(value) = cell.0.get("error").unwrap() {
                assert_eq!("TRUE", value);
            }
            assert_eq!("println(\"Hello World\")", cell.1.first().unwrap());
        } else {
            assert_eq!(0, 1);
        }
    }
    // #[test]
    // fn no_cell_on_error() {
    //     let mut command = CellParser::new();
    //     let res = command.push("```{scala, error=TRUE, echo=FALSE, include=TRUE}");
    //     assert!(res.is_ok());
    //     command.push(r#"println("Hello World")"#).unwrap();
    //     let res = command.push("```");
    //     assert!(res.is_err());
    //     if let Some(content) = command.iter().next() {
    //         if let NotebookCell::Error(_) = content {
    //         } else {
    //             panic!("result is not an error cell: {:?}", content)
    //         }
    //     }
    // }
 
    #[test]
    fn test_parse_notebook() {
        let content = r#"---
title: "svf circuit"
summary: "draw a filter circuit and do frequency analysis."
---

# Create circuit

This will create the data and parameters to plot datas using D3.

## Usage

First create some datas for example in a python block.

```python
from elektron import Circuit, Draw, Element, Label, Line, Dot, Simulation
import numpy as np

draw = (Draw(["/usr/share/kicad/symbols"])
  + Label("INPUT").rotate(180)
  + Element("R1", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(90)
  + Element("C1", "Device:C", value="220n", Spice_Netlist_Enabled="Y").rotate(90)
  + (u1_dot_in := Dot())
  + Element("U1", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u1_dot_out := Dot())

  + Element("R3", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u2_dot_in := Dot())
  + Element("U2", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u2_dot_out := Dot())

  + Element("R5", "Device:R", value="10k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u3_dot_in := Dot())
  + Element("U3", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u3_dot_out := Dot())

  + Element ("R6", "Device:R", value="10k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u4_dot_in := Dot())
  + Element("U4", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u4_dot_out := Dot())

  + Line().up().length(12.7).at(u1_dot_out)
  + Element("R2", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(u1_dot_in)
  + (r2_out := Dot())
  + Line().down().toy(u1_dot_in)

  + Line().up().length(12.7).at(u2_dot_out)
  + Label("HP")
  + Element("R4", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(u2_dot_in)
  + (r4_out := Dot())
  + Line().down().toy(u2_dot_in)

  + Line().up().length(12.7).at(u3_dot_out)
  + Label("BP")
  + (bp := Dot())
  + Element("C3", "Device:C", value="10n", Spice_Netlist_Enabled="Y").rotate(270).tox(u3_dot_in)
  + Line().down().toy(u3_dot_in)

  + Line().up().length(12.7).at(u4_dot_out)
  + Label("LP")
  + (lp := Dot())
  + Element("C4", "Device:C", value="10n", Spice_Netlist_Enabled="Y").rotate(270).tox(u4_dot_in)
  + Line().down().toy(u4_dot_in)

  + Line().up().length(10.16).at(lp)
  + Element("R7", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(r4_out)
  + Line().down().toy(r4_out)

  + Line().up().length(20.32).at(bp)
  + Element("R8", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(r2_out)
  + Line().down().toy(r2_out)

  + Element("U1", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((50.8, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U1", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U1", "14")

  + Element("U2", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((71.12, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U2", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U2", "14")

  + Element("U3", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((91.44, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U3", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U3", "14")

  + Element("U4", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((111.76, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U4", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U4", "14"))

print("get circuit")
draw.write("svf.kicad_sch")
circuit = draw.circuit(["spice"])
circuit.voltage("1", "+5V", "GND", "DC 15V")
circuit.voltage("2", "INPUT", "GND", "AC 2V SIN(0 2V 1k)")
circuit.control('''
ac dec 10 100 10K

*let r_act = 1k
*let r_step = 2k
*let r_stop = 200k
*while r_act le r_stop
*  alter R7 r_act
*  alter R8 r_act
*  ac dec 10 100 10K
*  let r_act = r_act + r_step
*end
*tran 1us 10ms
''')

print("get simulation")
simulation = Simulation(circuit)
svf_data = simulation.tran("1us", "10ms", "0ms")
print(len(svf_data))
svf = simulation.run()

for key, value in svf.items():
  if key.startswith("ac"):
    for k, v in value.items():
      if k == "frequency":
        svf[key][k] = v[1:]
      else:
        svf[key][k] = 20*np.log10(np.absolute(v))[1:]

draw.plot(scale=6)
```
{{< figure cap="Figure 6: State variable filter" align="center" path="_files/jsoupkvkqaxzetqgteyrjxnnsqxarx.svg">}}

This is the first setup with the 4069 as a voltage follower. C1 and C2 are DC blocking capacitors. When we choose R1 and R2 as 100kOhm we would expect a gain of one.

{{< d3 key="svf_ac" x="frequency" y="bp,hp,lp" yRange="" ySize="0" xDomain="125.89254117941672, 10000.000000000007" yDomain="-48.44932852660952, 6.115253317994574" width="600" height="400" yType="scaleLinear" xType="scaleLog" colors="Red,Green,Blue" xLabel="" yLabel="" range="" align="center" cap="Figure 7: State variable filter simulation">}}
{{< /d3 >}}

```{d3 debug=TRUE include=TRUE}```"#;

        let notebook = Notebook::from(content).unwrap();
        assert_eq!(4, notebook.cells.len());
        
        let mut cells = notebook.iter();

        let first = cells.next().unwrap();
        if let Cell::Content(cell) = first {
            assert_eq!(13, cell.0.len());
        } else {
            panic!("first cell is not of type Content");
        }

        let second = cells.next().unwrap();
        if let Cell::Python(cell) = second {
            assert_eq!(108, cell.1.len());
        } else {
            panic!("first cell is not of type Python");
        }

        let third = cells.next().unwrap();
        if let Cell::Content(cell) = third {
            assert_eq!(7, cell.0.len());
        } else {
            panic!("first cell is not of type Content");
        }

        let fourth = cells.next().unwrap();
        if let Cell::D3(cell) = fourth {
            assert_eq!(2, cell.0.len());
        } else {
            panic!("fourth cell is not of type D3");
        }
    }
}
