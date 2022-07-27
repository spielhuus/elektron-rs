

macro_rules! node {
    ($key:expr, $($value:expr),*) => {
        Sexp::Node($key.to_string(), vec![
            $(Sexp::Value($value.to_string()),)*]
        )
    }
}

macro_rules! uuid {
    () => {
        Sexp::Node(
            String::from("uuid"),
            vec![Sexp::Value(
                Uuid::new_v4().to_string(),
            )]
        )
    };
}

macro_rules! pos {
    ($pos:expr) => {
        Sexp::Node(
            String::from("at"),
            vec![
                Sexp::Value($pos[0].to_string()),
                Sexp::Value($pos[1].to_string()),
            ],
        )
    };
    ($pos:expr, $angle:expr) => {
        Sexp::Node(
            String::from("at"),
            vec![
                Sexp::Value($pos[0].to_string()),
                Sexp::Value($pos[1].to_string()),
                Sexp::Value($angle.to_string()),
            ],
        )
    };
}

macro_rules! stroke {
    () => {
        Sexp::Node(
            String::from("stroke"),
            vec![
                node!("width", 0),
                node!("type", "default"),
                node!("color", 0, 0, 0, 0),
            ],
        )
    };
}

macro_rules! effects {
    () => {
        Sexp::Node(String::from("effects"), vec![
            Sexp::Node(String::from("font"), vec![
                Sexp::Node(String::from("size"), vec![
                    Sexp::Value(String::from("1.27")),
                    Sexp::Value(String::from("1.27"))]
                    )]
                ),
            Sexp::Node(String::from("justify"), vec![
                Sexp::Value(String::from("left")),
                Sexp::Value(String::from("bottom"))]
            ),
        ])
    };
    ($hide:expr) => {
        Sexp::Node(String::from("effects"), vec![
            Sexp::Node(String::from("font"), vec![
                Sexp::Node(String::from("size"), vec![
                    Sexp::Value(String::from("1.27")),
                    Sexp::Value(String::from("1.27"))]
                    )]
                ),
            Sexp::Node(String::from("justify"), vec![
                Sexp::Value(String::from("left")),
                Sexp::Value(String::from("bottom"))]
            ),
        Sexp::Value(String::from("hide"))])
    };
    ($font_width:expr, $font_height:expr, $($align:expr),+) => {
        Sexp::Node(String::from("effects"), vec![
            Sexp::Node(String::from("font"), vec![
                Sexp::Node(String::from("size"), vec![
                    Sexp::Value(String::from($font_width.to_string())),
                    Sexp::Value(String::from($font_width.to_string()))]
                    )]
                ),
            Sexp::Node(String::from("justify"), vec![
                $(Sexp::Value(String::from($align.to_string())),)* ]
            )]
        )
    }
}

macro_rules! pts {
    ($($pt:expr),+) => {
        Sexp::Node(String::from("pts"), vec![
            $(Sexp::Node(String::from("xy"), vec![
                    Sexp::Value(String::from($pt[0].to_string())),
                    Sexp::Value(String::from($pt[1].to_string())),
            ]),)*
        ])
    }
}

macro_rules! property {
    ($pos:expr, $angle:expr, $key:expr, $value:expr, $id:expr, $hide:expr) => {
        Sexp::Node("property".to_string(), vec![
            Sexp::Value($key.to_string()),
            Sexp::Value($value.to_string()),
            node!("id", $id),
            pos!($pos, $angle),
            if $hide {effects!(hide)} else {effects!()},
        ])
    }
}

macro_rules! junction {
    ($pos:expr) => {
        Sexp::Node(
            String::from("junction"),
            vec![
                pos!($pos),
                node!("diameter", "0"),
                node!("color", 0, 0, 0, 0),
                uuid!(),
            ],
        )
    };
}

macro_rules! label {
    ($pos:expr, $angle:expr, $name:expr) => {
        Sexp::Node(
            String::from("label"),
            vec![
                Sexp::Text($name),
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
        )
    };
}

macro_rules! wire {
    ($pts:expr) => {
        Sexp::Node(
            String::from("wire"),
            vec![pts!($pts.row(0), $pts.row(1)), stroke!(), uuid!()],
        )
    };
}

macro_rules! symbol {
    ($pos:expr, $angle:expr, $mirror:expr, $reference:expr, $library:expr, $unit:expr, $uuid:expr, $on_schema:expr) => {
        if !$mirror.is_empty() {
            Sexp::Node(
                String::from("symbol"),
                vec![
                    Sexp::Node("lib_id".to_string(), vec![Sexp::Text($library.to_string())]),
                    pos!($pos, $angle),
                    node!("mirror", $mirror),
                    node!("unit", $unit),
                    node!("in_bom", "yes"),
                    node!("on_board", "yes"),
                    node!("on_schema", $on_schema),
                    node!("uuid", $uuid),
                ],
            )
        } else {
            Sexp::Node(
                String::from("symbol"),
                vec![
                    Sexp::Node("lib_id".to_string(), vec![Sexp::Text($library.to_string())]),
                    pos!($pos, $angle),
                    node!("unit", $unit),
                    node!("in_bom", "yes"),
                    node!("on_board", "yes"),
                    node!("on_schema", $on_schema),
                    node!("uuid", $uuid),
                ],
            )
        }
    };
}

macro_rules! sheet {
    ($path:expr, $page:expr) => {
        Sexp::Node(
            String::from("path"),
            vec![
                Sexp::Text($path.to_string()),
                Sexp::Node(
                    String::from("page"),
                    vec![Sexp::Text(
                        $page.to_string(),
                    )],
                ),
            ],
        )
    };
}

macro_rules! symbol_instance {
    ($uuid:expr, $reference:expr, $value:expr, $unit:expr, $footprint:expr) => {
        Sexp::Node(
            String::from("path"),
            vec![
                Sexp::Text(
                    $uuid.to_string(),
                ),
                node!("reference", $reference),
                node!("unit", $unit),
                node!("value", $value),
                node!("footprint", $footprint.unwrap_or(String::from("~"))),
            ],
        )
    };
}

pub(crate) use {node, uuid, pos, stroke, effects, pts, property, junction, label, wire, symbol, symbol_instance, sheet };
