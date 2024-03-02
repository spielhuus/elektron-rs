///Model for the schema drawings.
use std::collections::HashMap;

///Dot position
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DotPosition {
    Start,
    End,
}

impl From<&str> for DotPosition {
    fn from(position: &str) -> Self {
        let position = position.to_lowercase();
        if position == "start" {
            Self::Start
        } else {
            Self::End
        }
    }
}

///Label position
#[derive(Debug, Clone, PartialEq)]
pub enum LabelPosition {
    North,
    South,
    West,
    East,
    Offset(f64, f64),
}

impl From<&str> for LabelPosition {
    fn from(position: &str) -> Self {
        let position = position.to_lowercase();
        if position == "north" || position == "n" {
            Self::North
        } else if position == "south" || position == "s" {
            Self::South
        } else if position == "west" || position == "w" {
            Self::West
        } else if position == "east" || position == "e" {
            Self::East
        } else if position.contains(',') {
            let mut tokens = position.split(',');
            let x = tokens.next().unwrap();
            let y = tokens.next().unwrap();
            Self::Offset(x.parse::<f64>().unwrap(), y.parse::<f64>().unwrap())
        } else {
            Self::North
        }
    }
}

///Direction enum
#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl From<&str> for Direction {
    fn from(direction: &str) -> Self {
        if direction == "right" {
            Direction::Right
        } else if direction == "up" {
            Direction::Up
        } else if direction == "down" {
            Direction::Down
        } else {
            Direction::Left
        }
    }
}

///At enum, can be an absolute position, Pin or a Dot.
#[derive(Debug, Clone, PartialEq)]
pub enum At {
    Pos((f64, f64)),
    Pin(String, String),
    Dot(String),
}

///Attributes for the elements.
#[derive(Debug, Clone)]
pub enum Attribute {
    Anchor(String),
    Direction(Direction),
    Id(String),
    Mirror(String),
    Length(f64),
    Rotate(f64),
    Tox(At),
    Toy(At),
    Property(String),
    Dot(Vec<DotPosition>),
}

///Attributes trait
pub trait Attributes {
    ///Add a new attribute to an element.
    fn push(&mut self, attr: Attribute);
}

///Properties trait
pub trait Properties {
    ///Add a new property to an element
    fn insert(&mut self, key: &str, value: &str);
    ///Get the property
    fn get_property(&self, key: &str) -> Option<&String>;
}

///Label Element
#[derive(Debug)]
pub struct Label {
    ///The Label name.
    name: Option<String>,
    ///The Label Attributes.
    pub attributes: Vec<Attribute>,
}

impl Label {
    ///Create a new empty label.
    pub fn new() -> Self {
        Self {
            name: None,
            attributes: Vec::new(),
        }
    }
    ///Add the name to the Label.
    pub fn add_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }
    ///Get the label name.
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
    ///Set the Label angle.
    pub fn angle(&self) -> f64 {
        for i in &self.attributes {
            if let Attribute::Rotate(angle) = i {
                return *angle;
            }
        }
        0.0
    }
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes for Label {
    fn push(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }
}

///NoConnect Element
#[derive(Debug)]
pub struct Nc {
    ///The NoConnect Attributes.
    pub attributes: Vec<Attribute>,
}

impl Nc {
    ///Create a new empty label.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }
}

impl Default for Nc {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes for Nc {
    fn push(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }
}
///The Symbol element.
#[derive(Debug)]
pub struct Symbol {
    ///Symbol reference.
    reference: Option<String>,
    ///Symbol library id.
    lib_id: Option<String>,
    ///Symbol properties.
    pub properties: HashMap<String, String>,
    ///Symbol attributes.
    pub attributes: Vec<Attribute>,
    ///Symbol label position.
    pub label: Option<LabelPosition>,
}

impl Symbol {
    ///Create a new empty Symbol.
    pub fn new() -> Self {
        Self {
            reference: None,
            lib_id: None,
            properties: HashMap::new(),
            attributes: Vec::new(),
            label: None,
        }
    }
    ///Set the Symbol reference.
    pub fn set_reference(&mut self, reference: String) {
        self.reference = Some(reference);
    }
    ///Set the Symbol library id.
    pub fn set_lib_id(&mut self, lib_id: String) {
        self.lib_id = Some(lib_id);
    }
    ///Get the reference.
    pub fn get_reference(&self) -> Option<String> {
        self.reference.clone()
    }
    ///Get the library id.
    pub fn get_lib_id(&self) -> Option<String> {
        self.lib_id.clone()
    }
    ///Set the Symbol angle.
    pub fn angle(&self) -> f64 {
        for i in &self.attributes {
            if let Attribute::Rotate(angle) = i {
                return *angle;
            }
        }
        0.0
    }
    ///Get the length.
    pub fn length(&self) -> Option<f64> {
        for i in &self.attributes {
            if let Attribute::Length(len) = i {
                return Some(*len);
            }
        }
        None
    }
    ///Get tox.
    pub fn tox(&self) -> Option<&At> {
        for i in &self.attributes {
            if let Attribute::Tox(at) = i {
                return Some(at);
            }
        }
        None
    }
    ///Get toy.
    pub fn toy(&self) -> Option<&At> {
        for i in &self.attributes {
            if let Attribute::Toy(at) = i {
                return Some(at);
            }
        }
        None
    }
    ///Get the anchor pin.
    pub fn anchor(&self) -> Option<String> {
        for i in &self.attributes {
            if let Attribute::Anchor(a) = i {
                return Some(a.clone());
            }
        }
        None
    }
    ///Get symbol mirror, None if not set.
    pub fn mirror(&self) -> Option<String> {
        for i in &self.attributes {
            if let Attribute::Mirror(m) = i {
                return Some(m.clone());
            }
        }
        None
    }
    ///Set the label position.
    pub fn label(&self) -> Option<LabelPosition> {
        for i in &self.attributes {
            if let Attribute::Property(m) = i {
                return Some(LabelPosition::from(m.as_str()));
            }
        }
        None
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self::new()
    }
}

impl Properties for Symbol {
    fn insert(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }
    fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

impl Attributes for Symbol {
    fn push(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }
}

///Junction.
#[derive(Debug, Clone)]
pub struct Dot {
    ///Dot attributes.
    pub attributes: Vec<Attribute>,
}

impl Dot {
    ///Create a new empty dot.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }
    ///Get the ID, the ID is used as key to find a Dot.
    pub fn id(&self) -> Option<String> {
        for i in &self.attributes {
            if let Attribute::Id(id) = i {
                return Some(id.clone());
            }
        }
        None
    }
}

impl Default for Dot {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes for Dot {
    fn push(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }
}

///Draw a Wire from the actual posistion to position.
#[derive(Debug, Clone)]
pub struct To {
    ///The Attributes.
    pub attributes: Vec<Attribute>,
}

impl To {
    ///Create a new empty To.
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }
    ///Get the Wire length.
    pub fn length(&self) -> Option<f64> {
        for i in &self.attributes {
            if let Attribute::Length(length) = i {
                return Some(*length);
            }
        }
        None
    }
    ///Get the direction.
    pub fn direction(&self) -> &Direction {
        for i in &self.attributes {
            if let Attribute::Direction(direction) = i {
                return direction;
            }
        }
        &Direction::Left
    }
    ///Get the tox position.
    pub fn tox(&self) -> Option<&At> {
        for i in &self.attributes {
            if let Attribute::Tox(at) = i {
                return Some(at);
            }
        }
        None
    }
    ///Get the toy position.
    pub fn toy(&self) -> Option<&At> {
        for i in &self.attributes {
            if let Attribute::Toy(at) = i {
                return Some(at);
            }
        }
        None
    }
    //Get the dot positions.
    pub fn dot(&self) -> Option<&Vec<DotPosition>> {
        for i in &self.attributes {
            if let Attribute::Dot(dot) = i {
                return Some(dot);
            }
        }
        None
    }
}

impl Default for To {
    fn default() -> Self {
        Self::new()
    }
}

impl Attributes for To {
    fn push(&mut self, attr: Attribute) {
        self.attributes.push(attr);
    }
}
