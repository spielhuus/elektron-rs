use super::State;
use std::collections::HashMap;

use ndarray::{arr1, Array1, Array2, ArrayView};

use lazy_static::lazy_static;
use regex::Regex;

macro_rules! color {
    ($iter: expr) => {
        (
            $iter.next().unwrap().into(),
            $iter.next().unwrap().into(),
            $iter.next().unwrap().into(),
            $iter.next().unwrap().into(),
        )
    };
}

pub enum PaperSize {
    A5,
    A4,
    A3,
    A2,
    A1,
    A0,
}

impl std::convert::From<State<'_>> for PaperSize {
    fn from(state: State<'_>) -> Self {
        if let State::Text(size) = state {
            if size == "A5" {
                return Self::A5;
            } else if size == "A4" {
                return Self::A4;
            } else if size == "A3" {
                return Self::A3;
            } else if size == "A2" {
                return Self::A2;
            } else if size == "A1" {
                return Self::A1;
            } else {
                return Self::A0;
            }
        }
        panic!();
    }
}

pub(crate) use color;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Circle {
    pub center: Array1<f64>,
    pub radius: f64,
    pub stroke: Stroke,
    pub fill_type: String,
}
impl Circle {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut center: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut radius: f64 = 0.0;
        let mut stroke: Stroke = Stroke::new();
        let mut fill_type: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "center" {
                        center = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "radius" {
                        radius = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1;
                    } else if name == "fill" {
                    } else if name == "type" {
                        fill_type = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            center,
                            radius,
                            stroke,
                            fill_type,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arc {
    pub start: Array1<f64>,
    pub mid: Array1<f64>,
    pub end: Array1<f64>,
    pub stroke: Stroke,
    pub fill_type: String,
}
impl Arc {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut start: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut mid: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut end: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut stroke: Stroke = Stroke::new();
        let mut fill_type: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "mid" {
                        mid = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1;
                    } else if name == "fill" {
                    } else if name == "type" {
                        fill_type = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            start,
                            mid,
                            end,
                            stroke,
                            fill_type,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rectangle {
    pub start: Array1<f64>,
    pub end: Array1<f64>,
    pub stroke: Stroke,
    pub fill_type: String,
}
impl Rectangle {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut start: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut end: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut stroke: Stroke = Stroke::new();
        let mut fill_type: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1;
                    } else if name == "fill" {
                    } else if name == "type" {
                        fill_type = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            start,
                            end,
                            stroke,
                            fill_type,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Polyline {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub fill_type: String,
}
impl Polyline {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        let mut stroke: Stroke = Stroke::new();
        let mut fill_type: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "pts" {
                    } else if name == "xy" {
                        pts.push_row(ArrayView::from(&[
                            iter.next().unwrap().into(),
                            iter.next().unwrap().into(),
                        ]))
                        .unwrap();
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1;
                    } else if name == "fill" {
                    } else if name == "type" {
                        fill_type = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            pts,
                            stroke,
                            fill_type,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Graph {
    Polyline(Polyline),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Text(Text),
}

#[derive(Debug, PartialEq, Clone)]
/// Text effects
pub struct Effects {
    pub font: String,
    pub font_size: (f64, f64),
    pub thickness: f64,
    pub bold: bool,
    pub italic: bool,
    pub line_spacing: f64,
    pub color: (f64, f64, f64, f64),
    pub justify: Vec<String>,
    pub hide: bool,
}
impl Effects {
    pub fn new() -> Self {
        Self {
            font: String::new(),
            font_size: (1.27, 1.27),
            thickness: 0.0,
            bold: false,
            italic: false,
            line_spacing: 0.0,
            color: (0.0, 0.0, 0.0, 0.0),
            justify: Vec::new(),
            hide: false,
        }
    }
    pub fn hidden() -> Self {
        Self {
            font: String::new(),
            font_size: (1.27, 1.27),
            thickness: 0.0,
            bold: false,
            italic: false,
            line_spacing: 0.0,
            color: (0.0, 0.0, 0.0, 0.0),
            justify: Vec::new(),
            hide: true,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut font: String = String::new();
        let mut font_size: (f64, f64) = (0.0, 0.0);
        let mut thickness: f64 = 0.0;
        let mut bold: bool = false;
        let mut italic: bool = false;
        let mut line_spacing: f64 = 0.0;
        let mut color = (0.0, 0.0, 0.0, 0.0);
        let mut justify: Vec<String> = Vec::new();
        let mut hide = false;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "font" {
                    } else if name == "face" {
                        font = iter.next().unwrap().into();
                    } else if name == "size" {
                        font_size = (iter.next().unwrap().into(), iter.next().unwrap().into());
                    } else if name == "thickness" {
                        thickness = iter.next().unwrap().into();
                    } else if name == "line_spacing" {
                        line_spacing = iter.next().unwrap().into();
                    } else if name == "justify" {
                        while let Some(State::Values(value)) = iter.next() {
                            justify.push(value.to_string());
                        }
                        count -= 1;
                    } else if name == "color" {
                        color = color!(iter);
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::Values(value)) => {
                    if value == "hide" {
                        hide = true;
                    } else if value == "bold" {
                        bold = true;
                    } else if value == "italic" {
                        italic = true;
                    }
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            font,
                            font_size,
                            thickness,
                            bold,
                            italic,
                            line_spacing,
                            color,
                            justify,
                            hide,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Stroke {
    pub width: f64,
    pub linetype: String,
    pub color: (f64, f64, f64, f64),
    pub filltype: String,
}
impl Stroke {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            linetype: String::from("default"),
            color: (0.0, 0.0, 0.0, 0.0),
            filltype: String::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut width: f64 = 0.0;
        let mut linetype: String = String::new();
        let mut filltype: String = String::new();
        let mut color = (0.0, 0.0, 0.0, 0.0);
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "type" {
                        linetype = iter.next().unwrap().into();
                    } else if name == "fill" {
                        /* if let Some(State::StartSymbol(name)) = iter.next() {
                        } */
                    } else if name == "color" {
                        color = color!(iter);
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            width,
                            linetype,
                            filltype,
                            color,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NoConnect {
    pub at: Array1<f64>,
    pub uuid: String,
}
impl NoConnect {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut uuid: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return NoConnect { at, uuid };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Text {
    pub text: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Effects,
    pub uuid: String,
}
impl Text {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text: String = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects: Effects = Effects::new();
        let mut uuid: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "effects" {
                        effects = Effects::from(iter);
                        count -= 1; //the symbol started here and is closed in effects
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Text {
                            text,
                            at,
                            angle,
                            effects,
                            uuid,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Junction {
    pub at: Array1<f64>,
    pub diameter: f64,
    pub color: (f64, f64, f64, f64),
    pub uuid: String,
}
impl Junction {
    pub fn new(at: Array1<f64>, uuid: String) -> Self {
        Self {
            at,
            diameter: 0.0,
            color: (0.0, 0.0, 0.0, 0.0),
            uuid,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut uuid: String = String::new();
        let mut diameter: f64 = 0.0;
        let mut color = (0.0, 0.0, 0.0, 0.0);
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "diameter" {
                        diameter = iter.next().unwrap().into();
                    } else if name == "color" {
                        color = color!(iter);
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            at,
                            uuid,
                            diameter,
                            color,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Property {
    pub key: String,
    pub value: String,
    pub id: i32,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Effects,
}
impl Property {
    pub fn new(
        key: String,
        value: String,
        id: i32,
        at: Array1<f64>,
        angle: f64,
        effects: Effects,
    ) -> Self {
        Self {
            key,
            value,
            id,
            at,
            angle,
            effects,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let key: String = iter.next().unwrap().into();
        let value: String = iter.next().unwrap().into();
        let mut id: i32 = 0;
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = Effects::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "id" {
                        id = iter.next().unwrap().into();
                    } else if name == "effects" {
                        effects = Effects::from(iter);
                        count -= 1; //the symbol started here and is closed in effects
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            key,
                            value,
                            id,
                            at,
                            angle,
                            effects,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Wire {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub uuid: String,
}

impl Wire {
    pub fn new(pts: Array2<f64>, stroke: Stroke, uuid: String) -> Self {
        Self { pts, stroke, uuid }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        let mut stroke: Stroke = Stroke::new();
        let mut uuid = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "pts" {
                    } else if name == "xy" {
                        pts.push_row(ArrayView::from(&[
                            iter.next().unwrap().into(),
                            iter.next().unwrap().into(),
                        ]))
                        .unwrap();
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1; //the symbol started here and is closed in stroke
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self { pts, uuid, stroke };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Label {
    pub text: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Effects,
    pub uuid: String,
}
impl Label {
    pub fn new(at: Array1<f64>, angle: f64, text: &str, uuid: String) -> Self {
        Self {
            at,
            angle,
            text: text.to_string(),
            uuid,
            effects: Effects::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text: String = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = Effects::new();
        let mut uuid = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "effects" {
                        effects = Effects::from(iter);
                        count -= 1; //the symbol started here and is closed in stroke
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            text,
                            at,
                            angle,
                            effects,
                            uuid,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    pub lib_id: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub mirror: Vec<String>,
    pub unit: i32,
    pub in_bom: bool,
    pub on_board: bool,
    pub on_schema: bool,
    pub fields_autoplaced: bool,
    pub uuid: String,
    pub property: Vec<Property>,
    pub pin: Vec<(String, String)>,
}
impl Symbol {
    pub fn new(
        lib_id: String,
        at: Array1<f64>,
        angle: f64,
        mirror: Vec<String>,
        unit: i32,
        in_bom: bool,
        on_board: bool,
        on_schema: bool,
        fields_autoplaced: bool,
        uuid: String,
        property: Vec<Property>,
        pin: Vec<(String, String)>,
    ) -> Self {
        Self {
            lib_id,
            at,
            angle,
            mirror,
            unit,
            in_bom,
            on_board,
            on_schema,
            fields_autoplaced,
            uuid,
            property,
            pin,
        }
    }
    pub fn from_library(
        library: &LibrarySymbol,
        at: Array1<f64>,
        angle: f64,
        unit: i32,
        reference: String,
        value: String,
    ) -> Self {
        Self {
            lib_id: library.lib_id.clone(),
            at: at.clone(),
            angle,
            mirror: Vec::new(),
            unit,
            in_bom: true,
            on_board: true,
            on_schema: true,
            fields_autoplaced: true,
            uuid: Uuid::new_v4().to_string(),
            property: library
                .property
                .iter()
                .filter_map(|p| {
                    //skip properties with ki_
                    if p.key.starts_with("ki_") {
                        None
                    //set the reference
                    } else if p.key == "Reference" {
                        Some(Property::new(
                            p.key.clone(),
                            reference.clone(),
                            0,
                            at.clone(),
                            0.0,
                            Effects::new(),
                        ))
                    //set the value
                    } else if p.key == "Value" {
                        Some(Property::new(
                            p.key.clone(),
                            value.clone(),
                            1,
                            at.clone(),
                            0.0,
                            Effects::new(),
                        ))
                    } else if p.key == "footprint" {
                        Some(Property::new(
                            p.key.clone(),
                            p.value.clone(),
                            p.id,
                            at.clone(),
                            0.0,
                            Effects::new(),
                        ))
                    } else {
                        Some(p.clone())
                    }
                })
                .collect(),
            pin: Vec::new(), //TODO: implement
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut lib_id: String = String::new();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut mirror: Vec<String> = Vec::new();
        let mut unit: i32 = 0;
        let mut in_bom: bool = false;
        let mut on_board: bool = false;
        let mut fields_autoplaced: bool = false;
        let on_schema: bool = true;
        let mut uuid = String::new();
        let mut property: Vec<Property> = Vec::new();
        let mut pin: Vec<(String, String)> = Vec::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "lib_id" {
                        lib_id = iter.next().unwrap().into();
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "mirror" {
                        while let Some(State::Values(value)) = iter.next() {
                            mirror.push(value.to_string());
                        }
                        count -= 1;
                    } else if name == "unit" {
                        unit = iter.next().unwrap().into();
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "in_bom" {
                        let _in_bom: String = iter.next().unwrap().into();
                        in_bom = _in_bom == "yes";
                    } else if name == "on_board" {
                        let _on_board: String = iter.next().unwrap().into();
                        on_board = _on_board == "yes";
                    } else if name == "fields_autoplaced" {
                        fields_autoplaced = true;
                    } else if name == "property" {
                        property.push(Property::from(iter));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "pin" {
                        let pin_number: String = iter.next().unwrap().into();
                        if let Some(State::StartSymbol(name)) = iter.next() {
                            count += 1;
                            if name == "uuid" {
                                pin.push((pin_number, iter.next().unwrap().into()));
                            } else {
                                panic!("other pin element: {}", name);
                            }
                        }
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            lib_id,
                            at,
                            angle,
                            mirror,
                            unit,
                            in_bom,
                            on_board,
                            on_schema,
                            fields_autoplaced,
                            uuid,
                            property,
                            pin,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
    /// Property value by key.
    pub fn get_property(&self, key: &str) -> Option<String> {
        for prop in &self.property {
            if prop.key == key {
                return Some(prop.value.clone());
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pin {
    pub pin_type: String,
    pub pin_graphic_style: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub length: f64,
    pub name: (String, Effects),
    pub number: (String, Effects),
}
impl Pin {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let pin_type: String = iter.next().unwrap().into();
        let pin_graphic_style: String = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut length: f64 = 0.0;
        let mut pin_name = (String::new(), Effects::new());
        let mut number = (String::new(), Effects::new());
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "length" {
                        length = iter.next().unwrap().into();
                    } else if name == "name" {
                        let _name: String = iter.next().unwrap().into();
                        if let Some(State::StartSymbol(_)) = iter.next() {
                            pin_name = (_name, Effects::from(iter));
                        }
                    } else if name == "number" {
                        let _name: String = iter.next().unwrap().into();
                        if let Some(State::StartSymbol(_)) = iter.next() {
                            number = (_name, Effects::from(iter));
                        }
                    } else if name == "uuid" {
                        //TODO: implement
                        let _uuid: String = iter.next().unwrap().into();
                    } else if name == "effects" {
                        //TODO: implement
                        let _effects = Effects::from(iter);
                        count -= 1;
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            pin_type,
                            pin_graphic_style,
                            at,
                            angle,
                            length,
                            name: pin_name,
                            number,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

lazy_static! {
    pub static ref RE: regex::Regex = Regex::new(r"^.*_(\d*)_(\d*)$").unwrap();
}
#[derive(Debug, PartialEq, Clone)]
pub struct LibrarySymbol {
    pub lib_id: String,
    pub unit: i32,
    pub pin_numbers_show: bool,
    pub pin_names_show: bool,
    pub pin_names_offset: f64,
    pub power: bool,
    pub extends: String,
    pub in_bom: bool,
    pub on_board: bool,
    pub property: Vec<Property>,
    pub graph: Vec<Graph>,
    pub pin: Vec<Pin>,
    pub symbols: Vec<LibrarySymbol>,
}
impl LibrarySymbol {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let lib_id: String = iter.next().unwrap().into();
        let unit: i32 = if let Some(line) = RE.captures_iter(&lib_id).next() {
            line[1].parse().unwrap()
        } else {
            0
        };

        let mut pin_numbers_show = true;
        let mut pin_names_show = true;
        let mut pin_names_offset = 0.0;
        let mut power = false;
        let mut extends = String::new();
        let mut in_bom: bool = false;
        let mut on_board: bool = false;
        let mut property: Vec<Property> = Vec::new();
        let mut graph: Vec<Graph> = Vec::new();
        let mut pin: Vec<Pin> = Vec::new();
        let mut symbols: Vec<LibrarySymbol> = Vec::new();

        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "pin_names" {
                        let mut n_count = 1;
                        loop {
                            let item = iter.next();
                            match item {
                                Some(State::StartSymbol(name)) => {
                                    n_count += 1;
                                    if name == "offset" {
                                        pin_names_offset = iter.next().unwrap().into();
                                    } else {
                                        todo!("unexpexted element in pin_names: {}", name);
                                    }
                                }
                                Some(State::Text(_)) => {}
                                Some(State::Values(value)) => {
                                    if value == "hide" {
                                        pin_names_show = false;
                                    }
                                }
                                Some(State::EndSymbol) => {
                                    n_count -= 1;
                                    if n_count == 0 {
                                        break;
                                    }
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                        count -= 1;
                    } else if name == "pin_numbers" {
                        let _show: String = iter.next().unwrap().into();
                        pin_numbers_show = _show != "hide";
                    } else if name == "in_bom" {
                        let _in_bom: String = iter.next().unwrap().into();
                        in_bom = _in_bom == "yes";
                    } else if name == "on_board" {
                        let _on_board: String = iter.next().unwrap().into();
                        on_board = _on_board == "yes";
                    } else if name == "property" {
                        property.push(Property::from(iter));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "power" {
                        power = true;
                    } else if name == "extends" {
                        extends = iter.next().unwrap().into();
                    } else if name == "symbol" {
                        symbols.push(LibrarySymbol::from(iter));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "pin" {
                        pin.push(Pin::from(iter));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "polyline" {
                        graph.push(Graph::Polyline(Polyline::from(iter)));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "rectangle" {
                        graph.push(Graph::Rectangle(Rectangle::from(iter)));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "circle" {
                        graph.push(Graph::Circle(Circle::from(iter)));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "arc" {
                        graph.push(Graph::Arc(Arc::from(iter)));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "text" {
                        graph.push(Graph::Text(Text::from(iter)));
                        count -= 1; //the symbol started here and is closed in stroke
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            lib_id,
                            unit,
                            pin_numbers_show,
                            pin_names_show,
                            pin_names_offset,
                            power,
                            extends,
                            in_bom,
                            on_board,
                            property,
                            graph,
                            pin,
                            symbols,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlobalLabel {
    pub text: String,
    pub shape: String,
    pub fields_autoplaced: bool,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Effects,
    pub uuid: String,
    pub property: Property,
}
impl GlobalLabel {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text: String = iter.next().unwrap().into();
        let mut shape: String = String::new();
        let mut fields_autoplaced = false;
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = Effects::new();
        let mut uuid = String::new();
        let mut property = Property::new(
            String::new(),
            String::new(),
            0,
            arr1(&[0.0, 0.0]),
            0.0,
            Effects::new(),
        );
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "shape" {
                        shape = iter.next().unwrap().into();
                    } else if name == "fields_autoplaced" {
                        fields_autoplaced = true;
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        angle = iter.next().unwrap().into();
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "effects" {
                        effects = Effects::from(iter);
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "property" {
                        property = Property::from(iter);
                        count -= 1; //the symbol started here and is closed in stroke
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            text,
                            shape,
                            fields_autoplaced,
                            at,
                            angle,
                            effects,
                            uuid,
                            property,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SheetInstance {
    pub path: String,
    pub page: i32,
}
impl SheetInstance {
    pub fn new(path: &str, page: i32) -> Self {
        Self {
            path: path.to_string(),
            page,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let path: String = iter.next().unwrap().into();
        let mut page: i32 = 0;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "page" {
                        page = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self { path, page };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolInstance {
    pub path: String,
    pub reference: String,
    pub unit: i32,
    pub value: String,
    pub footprint: String,
}
impl SymbolInstance {
    pub fn new(
        path: String,
        reference: String,
        unit: i32,
        value: String,
        footprint: String,
    ) -> Self {
        Self {
            path,
            reference,
            unit,
            value,
            footprint,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut path: String = iter.next().unwrap().into();
        let mut reference: String = String::new();
        let mut unit: i32 = 0;
        let mut value: String = String::new();
        let mut footprint: String = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "path" {
                        path = iter.next().unwrap().into();
                    } else if name == "reference" {
                        reference = iter.next().unwrap().into();
                    } else if name == "unit" {
                        unit = iter.next().unwrap().into();
                    } else if name == "value" {
                        value = iter.next().unwrap().into();
                    } else if name == "footprint" {
                        footprint = iter.next().unwrap().into();
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            path,
                            reference,
                            unit,
                            value,
                            footprint,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TitleBlock {
    pub title: String,
    pub date: String,
    pub rev: String,
    pub company: String,
    pub comment: HashMap<i32, String>,
}
impl TitleBlock {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            date: String::new(),
            rev: String::new(),
            company: String::new(),
            comment: HashMap::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut title: String = String::new();
        let mut date: String = String::new();
        let mut rev: String = String::new();
        let mut company: String = String::new();
        let mut comment: HashMap<i32, String> = HashMap::new();

        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "title" {
                        title = iter.next().unwrap().into();
                    } else if name == "date" {
                        date = iter.next().unwrap().into();
                    } else if name == "rev" {
                        rev = iter.next().unwrap().into();
                    } else if name == "company" {
                        company = iter.next().unwrap().into();
                    } else if name == "comment" {
                        comment.insert(iter.next().unwrap().into(), iter.next().unwrap().into());
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            title,
                            date,
                            rev,
                            company,
                            comment,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Sheet {
    pub at: Array1<f64>,
    pub size: Array1<f64>,
    pub stroke: Stroke,
    pub fields_autoplaced: bool,
    pub fill: (f64, f64, f64, f64),
    pub uuid: String,
    pub property: Vec<Property>,
    pub pin: Vec<Pin>,
}
impl Sheet {
    pub fn new() -> Self {
        Self {
            at: arr1(&[0.0, 0.0]),
            size: arr1(&[0.0, 0.0]),
            stroke: Stroke::new(),
            fields_autoplaced: false,
            fill: (0.0, 0.0, 0.0, 0.0),
            uuid: String::new(),
            property: Vec::new(),
            pin: Vec::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut at = arr1(&[0.0, 0.0]);
        let mut size = arr1(&[0.0, 0.0]);
        let mut stroke = Stroke::new();
        let mut fields_autoplaced = false;
        let mut fill = (0.0, 0.0, 0.0, 0.0);
        let mut uuid = String::new();
        let mut property = Vec::new();
        let mut pin = Vec::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "size" {
                        size = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "fields_autoplaced" {
                        fields_autoplaced = true;
                    } else if name == "fill" {
                        fields_autoplaced = true;
                        if let Some(State::StartSymbol(n)) = iter.next() {
                            count += 1; //the symbol started here and is closed in stroke
                            if n == "color" {
                                fill = color!(iter);
                            } else {
                                todo!("uknown fill type: {}", n);
                            }
                        }
                    } else if name == "stroke" {
                        stroke = Stroke::from(iter);
                        count -= 1; //the symbol started here and is closed in stroke
                    } else if name == "uuid" {
                        uuid = iter.next().unwrap().into();
                    } else if name == "property" {
                        property.push(Property::from(iter));
                        count -= 1; //the symbol started here and is closed property
                    } else if name == "pin" {
                        pin.push(Pin::from(iter));
                        count -= 1; //the symbol started here and is closed in pin
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            at,
                            size,
                            stroke,
                            fields_autoplaced,
                            fill,
                            uuid,
                            property,
                            pin,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SchemaElement {
    Version(String),
    Generator(String),
    Uuid(String),
    Paper(String),
    TitleBlock(TitleBlock),
    LibrarySymbol(LibrarySymbol),
    Symbol(Symbol),
    NoConnect(NoConnect),
    Text(Text),
    Junction(Junction),
    Wire(Wire),
    Label(Label),
    GlobalLabel(GlobalLabel),
    Sheet(Sheet),
    SheetInstance(Vec<SheetInstance>),
    SymbolInstance(Vec<SymbolInstance>),
}
