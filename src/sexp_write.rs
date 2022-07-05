use std::io::Write;
use lazy_static::lazy_static;
use crate::sexp::{Error, SexpNode, SexpConsumer, SexpType, SexpValue, SexpText};


lazy_static! {
    static ref BREAK_NODES: Vec<String> = vec![
        String::from("property"), String::from("stroke"),
        String::from("effects"), String::from("pin"),
        String::from("uuid"), String::from("symbol"),
        String::from("name"), String::from("number"),
        String::from("paper"), String::from("title_block"),
        String::from("wire"), String::from("junction"),
        String::from("polyline"), String::from("reference"),
    ];
}

pub struct SexpResult {
    pub content: String,
    has_break_node: bool,
}

pub trait SexpWriter {
    fn write(&self, pretty: bool, indent: usize) -> SexpResult;
}

impl SexpWriter for SexpNode {
    fn write(&self, pretty: bool, indent: usize) -> SexpResult {
        let mut is_break_node = false;
        let mut has_break_node = false;
        let mut content = String::new();
        if pretty && BREAK_NODES.contains(&self.name) {
            is_break_node = true;
            content += "\n";
            content += &String::from(" ").repeat(indent);
        }
        content += "(";
        content += &self.name;
        for val in &self.values {
            content += " ";
            match val {
                SexpType::ChildSexpNode(node) => {
                    let result = node.write(pretty, indent + 1);
                    if result.has_break_node {
                        has_break_node = true;
                    }
                    content += &result.content;
                }
                SexpType::ChildSexpValue(value) => {
                    content += &value.write(pretty, indent + 1).content;
                }
                SexpType::ChildSexpText(value) => {
                    content += &value.write(pretty, indent + 1).content;
                }
            }
        }
        if has_break_node {
            content += "\n";
            content += &String::from(" ").repeat(indent);
        }
        content += ")";
        SexpResult{ content, has_break_node: (is_break_node || has_break_node) }
    }
}
impl SexpWriter for SexpValue {
    fn write(&self, _: bool, _: usize) -> SexpResult {
        let is_text = self.value.contains(" ");
        let mut content = String::new();
        if is_text {content += "\""};
        content += &self.value;
        if is_text {content += "\""};
        SexpResult{ content, has_break_node: false }
    }
}
impl SexpWriter for SexpText {
    fn write(&self, _: bool, _: usize) -> SexpResult {
        let mut content = String::new();
        content += "\"";
        content += &self.value;
        content += "\"";
        SexpResult{ content, has_break_node: false }
    }
}

pub struct SexpWrite {
    pretty: bool,
    indent: usize,
    writer: Box<dyn Write>,
}
impl SexpWrite {
    pub fn new(writer: Box<dyn Write>, pretty: bool) -> Self {
        SexpWrite { pretty, indent: 0, writer }
    }
    fn start_block(&mut self, name: String)  -> Result<(), Error> {
        if self.pretty {
            let prefix = String::from(" ").repeat(self.indent);
            self.writer.write_all(prefix.as_bytes())?;
        }
        write!(&mut self.writer, "({} ", name)?;
        if self.pretty {
            self.writer.write_all(String::from("\n").as_bytes())?;
        }
        self.indent += 1;
        Ok(())
    }
    fn end_block(&mut self)  -> Result<(), Error> {
        self.indent -= 1;
        if self.pretty {
            let mut prefix = String::from("\n");
            prefix += &String::from(" ").repeat(self.indent);
            self.writer.write_all(prefix.as_bytes())?;
        }
        self.writer.write_all(String::from(")").as_bytes())?;
        if self.pretty {
            self.writer.write_all(String::from("\n").as_bytes())?;
        }
        Ok(())
    }
}
impl SexpConsumer for SexpWrite {
    fn start(&mut self, version: &String, name: &String) -> Result<(), Error> {
        write!(&mut self.writer, "(kicad_sch (version {}) (generator {}) ", version, name)?;
        self.indent += 1;
        Ok(())
    }
    fn visit(&mut self, node: &SexpNode) -> Result<(), Error> {
        if self.pretty {
            let prefix = String::from(" ").repeat(self.indent);
            self.writer.write_all(prefix.as_bytes())?;
        }
        self.writer.write_all(node.write(self.pretty, self.indent).content.as_bytes())?;
        Ok(())
    }
    fn start_library_symbols(&mut self) -> Result<(), Error>  {
        self.start_block(String::from("lib_symbols"))
    }
    fn end_library_symbols(&mut self)  -> Result<(), Error> {
        self.end_block()
    }
    fn start_sheet_instances(&mut self)  -> Result<(), Error> {
        self.start_block(String::from("sheet_instances"))
    }
    fn end_sheet_instances(&mut self)  -> Result<(), Error> {
        self.end_block()
    }
    fn start_symbol_instances(&mut self)  -> Result<(), Error> {
        self.start_block(String::from("symbol_instances"))
    }
    fn end_symbol_instances(&mut self)  -> Result<(), Error> {
        self.end_block()
    }
    fn end(&mut self)  -> Result<(), Error> {
        self.writer.write_all(String::from(")").as_bytes())?;
        Ok(())
    }
}
