
macro_rules! node {
    ($key:expr, $($value:expr),*) => {
        SexpType::ChildSexpNode(SexpNode { name: $key.to_string(), values: vec![
            $(SexpType::ChildSexpValue(SexpValue { value: $value.to_string() }),)*]
        })
    }
}

macro_rules! uuid {
    () => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("uuid"),
            values: vec![SexpType::ChildSexpValue(SexpValue {
                value: Uuid::new_v4().to_string(),
            })],
        })
    };
}

macro_rules! pos {
    ($pos:expr) => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("at"),
            values: vec![
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[0].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[1].to_string(),
                }),
            ],
        })
    };
    ($pos:expr, $angle:expr) => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("at"),
            values: vec![
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[0].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $pos[1].to_string(),
                }),
                SexpType::ChildSexpValue(SexpValue {
                    value: $angle.to_string(),
                }),
            ],
        })
    };
}

macro_rules! stroke {
    () => {
        SexpType::ChildSexpNode(SexpNode {
            name: String::from("stroke"),
            values: vec![
                node!("width", 0),
                node!("type", "default"),
                node!("color", 0, 0, 0, 0),
            ],
        })
    };
}

macro_rules! effects {
    () => {
        SexpType::ChildSexpNode(SexpNode { name: String::from("effects"), values: vec![
            SexpType::ChildSexpNode(SexpNode { name: String::from("font"), values: vec![
                SexpType::ChildSexpNode(SexpNode { name: String::from("size"), values: vec![
                    SexpType::ChildSexpValue(SexpValue { value: String::from("1.27") }),
                    SexpType::ChildSexpValue(SexpValue { value: String::from("1.27") })]
                    })]
                }),
            SexpType::ChildSexpNode(SexpNode { name: String::from("justify"), values: vec![
                SexpType::ChildSexpValue(SexpValue { value: String::from("left") }),
                SexpType::ChildSexpValue(SexpValue { value: String::from("bottom") })]
            })]
        })
    };
    ($font_width:expr, $font_height:expr, $($align:expr),+) => {
        SexpType::ChildSexpNode(SexpNode { name: String::from("effects"), values: vec![
            SexpType::ChildSexpNode(SexpNode { name: String::from("font"), values: vec![
                SexpType::ChildSexpNode(SexpNode { name: String::from("size"), values: vec![
                    SexpType::ChildSexpValue(SexpValue { value: String::from($font_width.to_string()) }),
                    SexpType::ChildSexpValue(SexpValue { value: String::from($font_width.to_string()) })]
                    })]
                }),
            SexpType::ChildSexpNode(SexpNode { name: String::from("justify"), values: vec![
                $(SexpType::ChildSexpValue(SexpValue { value: String::from($align.to_string()) }),)* ]
            })]
        })
    }
}

macro_rules! pts {
    ($($pt:expr),+) => {
        SexpType::ChildSexpNode(SexpNode {name: String::from("pts"), values: vec![
            $(SexpType::ChildSexpNode(SexpNode { name: String::from("xy"), values: vec![
                SexpType::ChildSexpValue( SexpValue {
                    value: String::from($pt[0].to_string()),
                }),
                SexpType::ChildSexpValue( SexpValue {
                    value: String::from($pt[1].to_string()),
                }),
            ]}),)*
        ]})
    }
}

macro_rules! property {
    ($pos:expr, $angle:expr, $key:expr, $value:expr, $id:expr) => {
        SexpType::ChildSexpNode(SexpNode { name: "property".to_string(), values: vec![
            SexpType::ChildSexpText(SexpText { value: $key.to_string() }),
            SexpType::ChildSexpText(SexpText { value: $value.to_string() }),
            node!("id", $id),
            pos!($pos, $angle),
            effects!(),
        ]})
    }
}

macro_rules! junction {
    ($pos:expr) => {
        SexpNode {
            name: String::from("junction"),
            values: vec![
                pos!($pos),
                node!("diameter", "0"),
                node!("color", 0, 0, 0, 0),
                uuid!(),
            ],
        }
    };
}

macro_rules! label {
    ($pos:expr, $angle:expr, $name:expr) => {
        SexpNode {
            name: String::from("label"),
            values: vec![
                SexpType::ChildSexpText(SexpText { value: $name }),
                pos!($pos, $angle),
                effects!(
                    "1.27",
                    "1.27",
                    if vec![0.0, 90.0].contains($angle) {
                        "left"
                    } else {
                        "right"
                    }
                ),
                uuid!(),
            ],
        }
    };
}

macro_rules! wire {
    ($pts:expr) => {
        SexpNode {
            name: String::from("wire"),
            values: vec![pts!($pts.row(0), $pts.row(1)), stroke!(), uuid!()],
        }
    };
}

macro_rules! symbol {
    ($pos:expr, $angle:expr, $reference:expr, $library:expr, $unit:expr, $uuid:expr) => {
        SexpNode {
            name: String::from("symbol"),
            values: vec![
                node!("lib_id", $library),
                pos!($pos, $angle),
                node!("unit", $unit),
                node!("in_bom", "yes"),
                node!("on_board", "yes"),
                node!("uuid", $uuid),
            ],
        }
    };
}

macro_rules! sheet {
    ($path:expr, $page:expr) => {
        SexpNode {
            name: String::from("path"),
            values: vec![
                SexpType::ChildSexpText(SexpText {
                    value: $path.to_string(),
                }),
                SexpType::ChildSexpNode(SexpNode {
                    name: String::from("page"),
                    values: vec![SexpType::ChildSexpText(SexpText {
                        value: $page.to_string(),
                    })],
                }),
            ],
        }
    };
}

macro_rules! symbol_instance {
    ($uuid:expr, $reference:expr, $value:expr, $unit:expr, $footprint:expr) => {
        SexpNode {
            name: String::from("path"),
            values: vec![
                SexpType::ChildSexpText(SexpText {
                    value: $uuid.to_string(),
                }),
                node!("reference", $reference),
                node!("unit", $unit),
                node!("value", $value),
                node!("footprint", $footprint.unwrap_or(String::from("~"))),
            ],
        }
    };
}

pub(crate) use {node, uuid, pos, stroke, effects, pts, property, junction, label, wire, symbol, symbol_instance, sheet };
