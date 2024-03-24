use sexp_macro::parse_sexp;

#[derive(Default)]
pub struct Builder {
    pub nodes: Vec<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    pub fn push(&mut self, name: &str) {
        self.nodes.push(format!("Node({})", name));
    }
    pub fn end(&mut self) {
        self.nodes.push(String::from("End()"));
    }
    pub fn value(&mut self, name: &str) {
        self.nodes.push(format!("Value({})", name));
    }
    pub fn text(&mut self, name: &str) {
        self.nodes.push(format!("Text({})", name));
    }
}

#[test]
fn it_works() {
    let mut builder = Builder::new();
    parse_sexp!(builder, ("trace" "value1" "value2" ("node3" "value3" "value4") ("node5" "value5" r"RAW TEXT VALUE")));
    assert_eq!(
        vec![
            "Node(trace)",
            "Value(value1)",
            "Value(value2)",
            "Node(node3)",
            "Value(value3)",
            "Value(value4)",
            "End()",
            "Node(node5)",
            "Value(value5)",
            "Text(RAW TEXT VALUE)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
#[test]
fn with_params() {
    let mut builder = Builder::new();
    let value = r"param value";
    parse_sexp!(builder, ("trace" &value "value2" ("node3" "value3" "value4") ("node5" "value5" r"RAW TEXT VALUE")));
    assert_eq!(
        vec![
            "Node(trace)",
            "Value(param value)",
            "Value(value2)",
            "Node(node3)",
            "Value(value3)",
            "Value(value4)",
            "End()",
            "Node(node5)",
            "Value(value5)",
            "Text(RAW TEXT VALUE)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
#[test]
fn with_params_string() {
    let mut builder = Builder::new();
    let value = String::from(r"param value");
    parse_sexp!(builder, ("trace" {value.as_str()} "value2" ("node3" "value3" "value4") ("node5" "value5" r"RAW TEXT VALUE")));
    assert_eq!(
        vec![
            "Node(trace)",
            "Value(param value)",
            "Value(value2)",
            "Node(node3)",
            "Value(value3)",
            "Value(value4)",
            "End()",
            "Node(node5)",
            "Value(value5)",
            "Text(RAW TEXT VALUE)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
#[test]
fn with_params_text() {
    let mut builder = Builder::new();
    let value = String::from(r"param value");
    parse_sexp!(builder, ("trace" !{value.as_str()} "value2" ("node3" "value3" "value4") ("node5" "value5" r"RAW TEXT VALUE")));
    assert_eq!(
        vec![
            "Node(trace)",
            "Text(param value)",
            "Value(value2)",
            "Node(node3)",
            "Value(value3)",
            "Value(value4)",
            "End()",
            "Node(node5)",
            "Value(value5)",
            "Text(RAW TEXT VALUE)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
#[test]
fn with_property() {
    let mut builder = Builder::new();
    let k = String::from(r"param key");
    let v = String::from(r"param value");
    let pos = [0.0, 0.0];
    parse_sexp!(builder, ("property" !{k.as_str()} !{v.as_str()} ("at" {pos[0].to_string().as_str()} {pos[1].to_string().as_str()} "0")
        ("effects" ("font" ("size" "1.27" "1.27")) "hide")));
    assert_eq!(
        vec![
            "Node(property)",
            "Text(param key)",
            "Text(param value)",
            "Node(at)",
            "Value(0)",
            "Value(0)",
            "Value(0)",
            "End()",
            "Node(effects)",
            "Node(font)",
            "Node(size)",
            "Value(1.27)",
            "Value(1.27)",
            "End()",
            "End()",
            "Value(hide)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
#[test]
fn unquoted() {
    let mut builder = Builder::new();
    let k = String::from(r"param key");
    let v = String::from(r"param value");
    let pos = [0.0, 0.0];
    parse_sexp!(builder, (property !{k.as_str()} !{v.as_str()} (at {pos[0].to_string().as_str()} {pos[1].to_string().as_str()} "0")
        ("effects" ("font" ("size" "1.27" "1.27")) "hide")));
    assert_eq!(
        vec![
            "Node(property)",
            "Text(param key)",
            "Text(param value)",
            "Node(at)",
            "Value(0)",
            "Value(0)",
            "Value(0)",
            "End()",
            "Node(effects)",
            "Node(font)",
            "Node(size)",
            "Value(1.27)",
            "Value(1.27)",
            "End()",
            "End()",
            "Value(hide)",
            "End()",
            "End()"
        ],
        builder.nodes
    );
}
