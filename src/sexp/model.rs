use std::collections::HashMap;

use crate::error::Error;

use super::parser::State;

use bit_set::BitSet;
use indexmap::IndexMap;
use ndarray::{arr1, arr2, Array1, Array2, ArrayView};

use lazy_static::lazy_static;
use regex::Regex;

#[macro_export]
macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string()
    };
}
pub use uuid;

#[macro_export]
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
pub use color;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PaperSize {
    A5,
    A4,
    A3,
    A2,
    A1,
    A0,
}

impl std::fmt::Display for PaperSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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

pub enum ERROR_LOC { ERROR_OUTSIDE, ERROR_INSIDE }

impl std::convert::From<PaperSize> for (f64, f64) {
    fn from(size: PaperSize) -> Self {
        if size == PaperSize::A5 {
            (148.0, 210.0)
        } else if size == PaperSize::A4 {
            (297.0, 210.0)
        } else if size == PaperSize::A3 {
            (297.0, 420.0)
        } else if size == PaperSize::A2 {
            (420.0, 594.0)
        } else if size == PaperSize::A1 {
            (594.0, 841.0)
        } else {
            (841.0, 1189.0)
        }
    }
}

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
    pub uuid: Option<String>,
}
impl Polyline {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        let mut stroke: Stroke = Stroke::new();
        let mut fill_type: String = String::new();
        let mut uuid: Option<String> = None;
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
                    } else if name == "uuid" {
                        uuid = Some(iter.next().unwrap().into());
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
pub enum Graph {
    Polyline(Polyline),
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Text(Text),
}

#[derive(Debug, PartialEq, Clone, Default)]
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
            thickness: -1.0,
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
        let mut thickness: f64 = -1.0;
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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Stroke {
    pub width: f64,
    pub linetype: String,
    pub color: (f64, f64, f64, f64),
}
impl Stroke {
    pub fn new() -> Self {
        Self {
            width: 0.0,
            linetype: String::from("default"),
            color: (0.0, 0.0, 0.0, 0.0),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut width: f64 = 0.0;
        let mut linetype: String = String::new();
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
    pub fn new(at: Array1<f64>, uuid: String) -> Self {
        Self { at, uuid }
    }
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
    pub id: u32,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Option<Effects>,
}
impl Property {
    pub fn new(
        key: String,
        value: String,
        id: u32,
        at: Array1<f64>,
        angle: f64,
        effects: Option<Effects>,
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
        let mut id: u32 = 0;
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = None;
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
                        effects = Some(Effects::from(iter));
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
    pub fields_autoplaced: bool,
}
impl Label {
    pub fn new(at: Array1<f64>, angle: f64, text: &str, uuid: String) -> Self {
        Self {
            at,
            angle,
            text: text.to_string(),
            uuid,
            effects: Effects::new(),
            fields_autoplaced: false,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text: String = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = Effects::new();
        let mut uuid = String::new();
        let mut fields_autoplaced = false;
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
                    } else if name == "fields_autoplaced" {
                        fields_autoplaced = true;
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
                            fields_autoplaced,
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
    pub mirror: Option<String>,
    pub unit: u32,
    pub in_bom: bool,
    pub on_board: bool,
    pub do_not_place: bool,
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
        mirror: Option<String>,
        unit: u32,
        in_bom: bool,
        on_board: bool,
        on_schema: bool,
        do_not_place: bool,
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
            do_not_place,
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
        unit: u32,
        reference: String,
        value: String,
    ) -> Self {
        Self {
            lib_id: library.lib_id.clone(),
            at: at.clone(),
            angle,
            mirror: None,
            unit,
            in_bom: true,
            on_board: true,
            on_schema: true,
            do_not_place: false,
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
                            None,
                        ))
                    //set the value
                    } else if p.key == "Value" {
                        Some(Property::new(
                            p.key.clone(),
                            value.clone(),
                            1,
                            at.clone(),
                            0.0,
                            None,
                        ))
                    } else if p.key == "footprint" {
                        Some(Property::new(
                            p.key.clone(),
                            p.value.clone(),
                            p.id,
                            at.clone(),
                            0.0,
                            None,
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
        let mut mirror: Option<String> = None;
        let mut unit: u32 = 0;
        let mut in_bom: bool = false;
        let mut on_board: bool = false;
        let mut do_not_place: bool = false;
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
                            mirror = Some(value.to_string());
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
                    } else if name == "dnp" {
                        let _do_not_place: String = iter.next().unwrap().into();
                        do_not_place = _do_not_place == "yes";
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
                    } else if name == "instances" {
                        let mut in_counter = 0;
                        loop {
                            match iter.next() {
                                Some(State::StartSymbol(name)) => {
                                    in_counter += 1;
                                },
                                Some(State::EndSymbol) => {
                                    in_counter -= 1;
                                }
                                _ => {},
                            }
                            if in_counter == 0 {
                                break;
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
                            do_not_place,
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
    /// Set the property
    pub fn set_property(&mut self, key: &str, value: &str) {
        for prop in &mut self.property {
            if prop.key == key {
                prop.value = value.to_string();
            }
        }
    }

    /// True if Symbol has Property.
    pub fn has_property(&self, key: &str) -> bool {
        for prop in &self.property {
            if prop.key == key {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PinType {
    Input,
    Output,
    Bidirectional,
    TriState,
    Passive,
    Free,
    Unspecified,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    NoConnect,
}

impl std::convert::From<State<'_>> for PinType {
    fn from(state: State<'_>) -> Self {
        if let State::Values(pin_type) = state {
            if pin_type == "input" {
                Self::Input
            } else if pin_type == "output" {
                Self::Output
            } else if pin_type == "bidirectional" {
                Self::Bidirectional
            } else if pin_type == "tri_state" {
                Self::TriState
            } else if pin_type == "passive" {
                Self::Passive
            } else if pin_type == "free" {
                Self::Free
            } else if pin_type == "unspecified" {
                Self::Unspecified
            } else if pin_type == "power_in" {
                Self::PowerIn
            } else if pin_type == "power_out" {
                Self::PowerOut
            } else if pin_type == "open_collector" {
                Self::OpenCollector
            } else if pin_type == "open_emitter" {
                Self::OpenEmitter
            } else if pin_type == "no_connect" {
                Self::NoConnect
            } else {
                todo!("unknown pin type {}", pin_type);
            }
        } else {
            println!("unknown state for pin type {:?}", state);
            Self::Unspecified
        }
    }
}

impl std::fmt::Display for PinType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PinType::Input => write!(f, "input")?,
            PinType::Output => write!(f, "output")?,
            PinType::Bidirectional => write!(f, "bidirectional")?,
            PinType::TriState => write!(f, "tri_state")?,
            PinType::Passive => write!(f, "passive")?,
            PinType::Free => write!(f, "free")?,
            PinType::Unspecified => write!(f, "unspecified")?,
            PinType::PowerIn => write!(f, "power_in")?,
            PinType::PowerOut => write!(f, "power_out")?,
            PinType::OpenCollector => write!(f, "open_collector")?,
            PinType::OpenEmitter => write!(f, "open_emitter")?,
            PinType::NoConnect => write!(f, "no_connect")?,
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PinGraphicalStyle {
    Line,
    Inverted,
    Clock,
    InvertedClock,
    InputLow,
    ClockLow,
    OutputLow,
    EdgeClockHigh,
    NonLogic,
}

impl std::convert::From<State<'_>> for PinGraphicalStyle {
    fn from(state: State<'_>) -> Self {
        if let State::Values(pin_type) = state {
            if pin_type == "line" {
                Self::Line
            } else if pin_type == "inverted" {
                Self::Inverted
            } else if pin_type == "clock" {
                Self::Clock
            } else if pin_type == "inverted_clock" {
                Self::InvertedClock
            } else if pin_type == "input_low" {
                Self::InputLow
            } else if pin_type == "clock_low" {
                Self::ClockLow
            } else if pin_type == "output_low" {
                Self::OutputLow
            } else if pin_type == "edge_clock_high" {
                Self::EdgeClockHigh
            } else if pin_type == "non_logic" {
                Self::NonLogic
            } else {
                println!("unknown pin graphical style {}", pin_type);
                Self::Line
            }
        } else {
            println!("unknown state for pin graphical style {:?}", state);
            Self::Line
        }
    }
}

impl std::fmt::Display for PinGraphicalStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PinGraphicalStyle::Line => write!(f, "line")?,
            PinGraphicalStyle::Inverted => write!(f, "inverted")?,
            PinGraphicalStyle::Clock => write!(f, "clock")?,
            PinGraphicalStyle::InvertedClock => write!(f, "inverted_clock")?,
            PinGraphicalStyle::InputLow => write!(f, "input_low")?,
            PinGraphicalStyle::ClockLow => write!(f, "clock_low")?,
            PinGraphicalStyle::OutputLow => write!(f, "output_low")?,
            PinGraphicalStyle::EdgeClockHigh => write!(f, "edge_clock_high")?,
            PinGraphicalStyle::NonLogic => write!(f, "non_logic")?,
        };
        Ok(())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Pin {
    pub pin_type: PinType,
    pub pin_graphic_style: PinGraphicalStyle,
    pub at: Array1<f64>,
    pub angle: f64,
    pub length: f64,
    pub hide: bool,
    pub name: (String, Effects),
    pub number: (String, Effects),
    pub uuid: String,
}
impl Pin {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let pin_type: PinType = iter.next().unwrap().into();
        let pin_graphic_style: PinGraphicalStyle = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut length: f64 = 0.0;
        let mut hide: bool = false;
        let mut pin_name = (String::new(), Effects::new());
        let mut number = (String::new(), Effects::new());
        let mut uuid = String::new();
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
                        uuid = iter.next().unwrap().into();
                    } else if name == "alternate" {
                        let name: String = iter.next().unwrap().into();
                        let pin_type: String = iter.next().unwrap().into();
                        let pin: String = iter.next().unwrap().into();
                        //TODO:: println!("Pin alternate: {}:{}:{}", name, pin_type, pin);
                    } else if name == "effects" {
                        println!("get effects from root: {}", pin_name.0);
                        pin_name.1 = Effects::from(iter);
                        count -= 1;
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                Some(State::Values(value)) => {
                    if value == "hide" {
                        hide = true;
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
                            hide,
                            name: pin_name,
                            number,
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

lazy_static! {
    pub static ref RE: regex::Regex = Regex::new(r"^.*_(\d*)_(\d*)$").unwrap();
}
#[derive(Debug, PartialEq, Clone)]
pub struct LibrarySymbol {
    pub lib_id: String,
    pub unit: u32,
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
        let unit: u32 = if let Some(line) = RE.captures_iter(&lib_id).next() {
            line[1].parse().unwrap()
        } else {
            0
        };

        let mut pin_numbers_show = true;
        let mut pin_names_show = true;
        let mut pin_names_offset = -1.0;
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

    ///Get pin by number.
    pub fn get_pin(&self, number: String) -> Result<&Pin, Error> {
        for s in &self.symbols {
            for p in &s.pin {
                if p.number.0 == number {
                    return Ok(p);
                }
            }
        }
        Err(Error::PinNotFound(self.lib_id.to_string(), number))
    }

    /// Get all the pins of a library symbol.
    pub fn pins(&self, unit: u32) -> Result<Vec<&Pin>, Error> {
        let mut items: Vec<&Pin> = Vec::new();
        for _unit in &self.symbols {
            if unit == 0 || _unit.unit == 0 || _unit.unit == unit {
                for pin in &_unit.pin {
                    items.push(pin);
                }
            }
        }
        if items.is_empty() {
            Err(Error::NoPinsFound(self.lib_id.clone(), unit))
        } else {
            Ok(items)
        }
    }

    pub fn pin_names(&self) -> Result<HashMap<String, (Pin, u32)>, Error> {
        let mut pins = HashMap::new();
        for symbol in &self.symbols {
            //search the pins
            for pin in &symbol.pin {
                pins.insert(pin.number.0.clone(), (pin.clone(), symbol.unit));
            }
        }
        Ok(pins)
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
pub struct HierarchicalLabel {
    pub text: String,
    pub shape: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub effects: Effects,
    pub uuid: String,
}
impl HierarchicalLabel {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text: String = iter.next().unwrap().into();
        let mut shape: String = String::new();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut effects = Effects::new();
        let mut uuid = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "shape" {
                        shape = iter.next().unwrap().into();
                    } else if name == "at" {
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
                            shape,
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
            None,
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
#[derive(Debug, PartialEq, Clone)]
pub struct BusEntry {
    pub at: Array1<f64>,
    pub angle: f64,
    pub size: Array1<f64>,
    pub stroke: Stroke,
    pub uuid: String,
}
impl BusEntry {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let angle: f64 = 0.0; //TODO: angle is not set.
        let mut size: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut stroke = Stroke::new();
        let mut uuid = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "size" {
                        size = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
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
                        return Self {
                            at,
                            angle,
                            size,
                            stroke,
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
pub struct Bus {
    pub pts: Array2<f64>,
    pub stroke: Stroke,
    pub uuid: String,
}
impl Bus {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        let mut stroke = Stroke::new();
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
                        return Self { pts, stroke, uuid };
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
    pub page: u32,
}
impl SheetInstance {
    pub fn new(path: &str, page: u32) -> Self {
        Self {
            path: path.to_string(),
            page,
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let path: String = iter.next().unwrap().into();
        let mut page: u32 = 0;
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
    pub unit: u32,
    pub value: String,
    pub footprint: String,
}
impl SymbolInstance {
    pub fn new(
        path: String,
        reference: String,
        unit: u32,
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
        let mut unit: u32 = 0;
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

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct TitleBlock {
    pub title: String,
    pub date: String,
    pub rev: String,
    pub company: String,
    pub comment: Vec<(u32, String)>,
}
impl TitleBlock {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            date: String::new(),
            rev: String::new(),
            company: String::new(),
            comment: Vec::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut title: String = String::new();
        let mut date: String = String::new();
        let mut rev: String = String::new();
        let mut company: String = String::new();
        let mut comment: Vec<(u32, String)> = Vec::new();

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
                        comment.push((iter.next().unwrap().into(), iter.next().unwrap().into()));
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

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Sheet {
    pub at: Array1<f64>,
    pub size: Array1<f64>,
    pub stroke: Stroke,
    pub fields_autoplaced: bool,
    pub fill: (f64, f64, f64, f64),
    pub uuid: String,
    pub property: Vec<Property>,
    pub pin: Vec<SheetPin>,
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
                        pin.push(SheetPin::from(iter));
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
    pub fn sheet_name(&self) -> Result<String, Error> {
        for p in &self.property {
            if p.key == "Sheet name" {
                return Ok(p.value.clone());
            }
        }
        Err(Error::ParseError)
    }
    pub fn sheet_filename(&self) -> Result<String, Error> {
        for p in &self.property {
            if p.key == "Sheet file" {
                return Ok(p.value.clone());
            }
        }
        Err(Error::ParseError)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SheetPin {
    pub pin_type: String,
    pub pin_graphic_style: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub length: f64,
    pub hide: bool,
    pub name: (String, Effects),
    pub number: (String, Effects),
    pub uuid: String,
}
impl SheetPin {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let pin_type: String = iter.next().unwrap().into();
        let pin_graphic_style: String = iter.next().unwrap().into();
        let mut at: Array1<f64> = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut length: f64 = 0.0;
        let mut hide: bool = false;
        let mut pin_name = (String::new(), Effects::new());
        let mut number = (String::new(), Effects::new());
        let mut uuid = String::new();
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
                        uuid = iter.next().unwrap().into();
                    } else if name == "effects" {
                        pin_name.1 = Effects::from(iter);
                        count -= 1;
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                Some(State::Values(value)) => {
                    if value == "hide" {
                        hide = true;
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
                            hide,
                            name: pin_name,
                            number,
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
pub enum SchemaElement {
    Symbol(Symbol),
    NoConnect(NoConnect),
    Text(Text),
    Junction(Junction),
    Wire(Wire),
    Label(Label),
    GlobalLabel(GlobalLabel),
    Sheet(Sheet),
    Bus(Bus),
    BusEntry(BusEntry),
    Polyline(Polyline),
    HierarchicalLabel(HierarchicalLabel),
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum LayerId {
    FCu = 0,
    In1Cu,
    In2Cu,
    In3Cu,
    In4Cu,
    In5Cu,
    In6Cu,
    In7Cu,
    In8Cu,
    In9Cu,
    In10Cu,
    In11Cu,
    In12Cu,
    In13Cu,
    In14Cu,
    In15Cu,
    In16Cu,
    In17Cu,
    In18Cu,
    In19Cu,
    In20Cu,
    In21Cu,
    In22Cu,
    In23Cu,
    In24Cu,
    In25Cu,
    In26Cu,
    In27Cu,
    In28Cu,
    In29Cu,
    In30Cu,
    BCu = 31,
    BAdhes,
    FAdhes,
    BPaste,
    FPaste,
    BSilkS,
    FSilkS,
    BMask,
    FMask = 39,
    DwgsUser,
    CmtsUser,
    Eco1User,
    Eco2User,
    EdgeCuts,
    Margin = 45,
    BCrtYd,
    FCrtYd,
    BFab,
    FFab = 49,
    // User definable layers.
    User1,
    User2,
    User3,
    User4,
    User5,
    User6,
    User7,
    User8,
    User9,

    Rescue = 59,
    Undefined,
    Unselected,
}

impl LayerId {
    pub fn is_copper(&self) -> bool {
        *self as u32 >= Self::FCu as u32 && *self as u32 <= Self::BCu as u32
    }
}

impl From<u32> for LayerId {
    fn from(id: u32) -> Self {
        match id {
            0 => Self::FCu,
            1 => Self::In1Cu,
            2 => Self::In2Cu,
            3 => Self::In3Cu,
            4 => Self::In4Cu,
            5 => Self::In5Cu,
            6 => Self::In6Cu,
            7 => Self::In7Cu,
            8 => Self::In8Cu,
            9 => Self::In9Cu,
            10 => Self::In10Cu,
            11 => Self::In11Cu,
            12 => Self::In12Cu,
            13 => Self::In13Cu,
            14 => Self::In14Cu,
            15 => Self::In15Cu,
            16 => Self::In16Cu,
            17 => Self::In17Cu,
            18 => Self::In18Cu,
            19 => Self::In19Cu,
            20 => Self::In20Cu,
            21 => Self::In21Cu,
            22 => Self::In22Cu,
            23 => Self::In23Cu,
            24 => Self::In24Cu,
            25 => Self::In25Cu,
            26 => Self::In26Cu,
            27 => Self::In27Cu,
            28 => Self::In28Cu,
            29 => Self::In29Cu,
            30 => Self::In30Cu,
            31 => Self::BCu,
            32 => Self::BAdhes,
            33 => Self::FAdhes,
            34 => Self::BPaste,
            35 => Self::FPaste,
            36 => Self::BSilkS,
            37 => Self::FSilkS,
            38 => Self::BMask,
            39 => Self::FMask,
            40 => Self::DwgsUser,
            41 => Self::CmtsUser,
            42 => Self::Eco1User,
            43 => Self::Eco2User,
            44 => Self::EdgeCuts,
            45 => Self::Margin,
            46 => Self::BCrtYd,
            47 => Self::FCrtYd,
            48 => Self::BFab,
            49 => Self::FFab,
            // User definable layers.
            50 => Self::User1,
            51 => Self::User2,
            52 => Self::User3,
            53 => Self::User4,
            54 => Self::User5,
            55 => Self::User6,
            56 => Self::User7,
            57 => Self::User8,
            58 => Self::User9,

            59 => Self::Rescue,
            60 => Self::Undefined,
            61 => Self::Unselected,
            _ => {
                todo!("unknown layer id: {}", id)
            }
        }
    }
}

impl From<State<'_>> for LayerId {
    fn from(state: State<'_>) -> Self {
        if let State::Text(value) = state {
            match value {
                "F.Cu" => Self::FCu,
                "In1.Cu" => Self::In1Cu,
                "In2.Cu" => Self::In2Cu,
                "In3.Cu" => Self::In3Cu,
                "In4.Cu" => Self::In4Cu,
                "In5.Cu" => Self::In5Cu,
                "In6.Cu" => Self::In6Cu,
                "In7.Cu" => Self::In7Cu,
                "In8.Cu" => Self::In8Cu,
                "In9.Cu" => Self::In9Cu,
                "In10.Cu" => Self::In10Cu,
                "In11.Cu" => Self::In11Cu,
                "In12.Cu" => Self::In12Cu,
                "In13.Cu" => Self::In13Cu,
                "In14.Cu" => Self::In14Cu,
                "In15.Cu" => Self::In15Cu,
                "In16.Cu" => Self::In16Cu,
                "In17.Cu" => Self::In17Cu,
                "In18.Cu" => Self::In18Cu,
                "In19.Cu" => Self::In19Cu,
                "In20.Cu" => Self::In20Cu,
                "In21.Cu" => Self::In21Cu,
                "In22.Cu" => Self::In22Cu,
                "In23.Cu" => Self::In23Cu,
                "In24.Cu" => Self::In24Cu,
                "In25.Cu" => Self::In25Cu,
                "In26.Cu" => Self::In26Cu,
                "In27.Cu" => Self::In27Cu,
                "In28.Cu" => Self::In28Cu,
                "In29.Cu" => Self::In29Cu,
                "In30.Cu" => Self::In30Cu,
                "B.Cu" => Self::BCu,
                "B.Adhes" => Self::BAdhes,
                "F.Adhes" => Self::FAdhes,
                "B.Paste" => Self::BPaste,
                "F.Paste" => Self::FPaste,
                "B.SilkS" => Self::BSilkS,
                "F.SilkS" => Self::FSilkS,
                "B.Mask" => Self::BMask,
                "F.Mask" => Self::FMask,
                "Dwgs.User" => Self::DwgsUser,
                "Cmts.User" => Self::CmtsUser,
                "Eco1.User" => Self::Eco1User,
                "Eco2.User" => Self::Eco2User,
                "Edge.Cuts" => Self::EdgeCuts,
                "Margin" => Self::Margin,
                "B.CrtYd" => Self::BCrtYd,
                "F.CrtYd" => Self::FCrtYd,
                "B.Fab" => Self::BFab,
                "F.Fab" => Self::FFab,
                // User definable layers.
                "User.1" => Self::User1,
                "User.2" => Self::User2,
                "User.3" => Self::User3,
                "User.4" => Self::User4,
                "User.5" => Self::User5,
                "User.6" => Self::User6,
                "User.7" => Self::User7,
                "User.8" => Self::User8,
                "User.9" => Self::User9,

                "Rescue" => Self::Rescue,
                "Undefined" => Self::Undefined,
                "Unselected" => Self::Unselected,
                _ => {
                    todo!("unknown layer id: {}", value)
                }
            }
        } else {
            panic!("Unknown LayerId: {:?}", state);
        }
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FCu => write!(f, "0"),
            Self::In1Cu => write!(f, "1"),
            Self::In2Cu => write!(f, "2"),
            Self::In3Cu => write!(f, "3"),
            Self::In4Cu => write!(f, "4"),
            Self::In5Cu => write!(f, "5"),
            Self::In6Cu => write!(f, "6"),
            Self::In7Cu => write!(f, "7"),
            Self::In8Cu => write!(f, "8"),
            Self::In9Cu => write!(f, "9"),
            Self::In10Cu => write!(f, "10"),
            Self::In11Cu => write!(f, "11"),
            Self::In12Cu => write!(f, "12"),
            Self::In13Cu => write!(f, "13"),
            Self::In14Cu => write!(f, "14"),
            Self::In15Cu => write!(f, "15"),
            Self::In16Cu => write!(f, "16"),
            Self::In17Cu => write!(f, "17"),
            Self::In18Cu => write!(f, "18"),
            Self::In19Cu => write!(f, "19"),
            Self::In20Cu => write!(f, "20"),
            Self::In21Cu => write!(f, "21"),
            Self::In22Cu => write!(f, "22"),
            Self::In23Cu => write!(f, "23"),
            Self::In24Cu => write!(f, "24"),
            Self::In25Cu => write!(f, "25"),
            Self::In26Cu => write!(f, "26"),
            Self::In27Cu => write!(f, "27"),
            Self::In28Cu => write!(f, "28"),
            Self::In29Cu => write!(f, "29"),
            Self::In30Cu => write!(f, "30"),
            Self::BCu => write!(f, "31"),
            Self::BAdhes => write!(f, "32"),
            Self::FAdhes => write!(f, "33"),
            Self::BPaste => write!(f, "34"),
            Self::FPaste => write!(f, "35"),
            Self::BSilkS => write!(f, "36"),
            Self::FSilkS => write!(f, "37"),
            Self::BMask => write!(f, "38"),
            Self::FMask => write!(f, "39"),
            Self::DwgsUser => write!(f, "40"),
            Self::CmtsUser => write!(f, "41"),
            Self::Eco1User => write!(f, "42"),
            Self::Eco2User => write!(f, "43"),
            Self::EdgeCuts => write!(f, "44"),
            Self::Margin => write!(f, "45"),
            Self::BCrtYd => write!(f, "46"),
            Self::FCrtYd => write!(f, "47"),
            Self::BFab => write!(f, "48"),
            Self::FFab => write!(f, "49"),
            // User definable layers.
            Self::User1 => write!(f, "50"),
            Self::User2 => write!(f, "51"),
            Self::User3 => write!(f, "52"),
            Self::User4 => write!(f, "53"),
            Self::User5 => write!(f, "54"),
            Self::User6 => write!(f, "55"),
            Self::User7 => write!(f, "56"),
            Self::User8 => write!(f, "57"),
            Self::User9 => write!(f, "58"),

            Self::Rescue => write!(f, "59"),
            Self::Undefined => write!(f, "60"),
            Self::Unselected => write!(f, "61"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerSet {
    pub bits: BitSet,
}

impl LayerSet {
    pub fn new() -> Self {
        Self {
            bits: BitSet::with_capacity(LayerId::Unselected as usize)
        }
    }
    pub fn from(id: LayerId) -> Self {
        let mut bits = LayerSet::new();
        bits.set(id);
        bits
    }
    pub fn set(&mut self, id: LayerId) {
        self.bits.insert(id as usize);
    }
    pub fn contains(&self, layer: LayerId) -> bool {
        self.bits.contains(layer as usize)
    }
    pub fn on_solder_mask_layer(&self) -> bool {
        self.contains(LayerId::FMask) || self.contains(LayerId::BMask)
    }
    pub fn on_solder_paste_layer(&self) -> bool {
        self.contains(LayerId::FPaste) || self.contains(LayerId::BPaste)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Layers {
    pub id: LayerId,
    pub canonical_name: String,
    pub layertype: String,
    pub user_name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Footprint {
    pub key: String,
    pub layer: String,
    pub tedit: String,
    pub tstamp: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub descr: String,
    pub tags: Vec<String>,
    pub path: String,
    pub attr: Vec<String>,
    pub graphics: Vec<Graphics>,
    pub pads: Vec<Pad>,
    pub models: Vec<Model>,
    pub locked: bool,
}
impl Footprint {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let key: String = iter.next().unwrap().into();
        let mut layer = String::new();
        let mut tedit = String::new();
        let mut tstamp = String::new();
        let mut at = arr1(&[0.0, 0.0]);
        let mut angle: f64 = 0.0;
        let mut descr = String::new();
        let mut tags = Vec::new();
        let mut path = String::new();
        let mut attr = Vec::new();
        let mut graphics = Vec::new();
        let mut pads = Vec::new();
        let mut models = Vec::new();
        let mut locked = false;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "tedit" {
                        tedit = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        if let Some(State::Values(value)) = iter.next() {
                            angle = value.parse::<f64>().unwrap();
                        } else {
                            count -= 1;
                        }
                    } else if name == "descr" {
                        descr = iter.next().unwrap().into();
                    } else if name == "property" {
                        /* TODO: */
                    } else if name == "tags" {
                        let string_tags: String = iter.next().unwrap().into();
                        tags = string_tags.split(' ').map(|s| s.to_string()).collect();
                    } else if name == "path" {
                        path = iter.next().unwrap().into();
                    } else if name == "attr" {
                        loop {
                            if let Some(State::Values(value)) = iter.next() {
                                attr.push(value.to_string());
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "fp_text" {
                        graphics.push(Graphics::FpText(FpText::from(iter)));
                        count -= 1;
                    } else if name == "fp_line" {
                        graphics.push(Graphics::FpLine(FpLine::from(iter)));
                        count -= 1;
                    } else if name == "fp_circle" {
                        graphics.push(Graphics::FpCircle(FpCircle::from(iter)));
                        count -= 1;
                    } else if name == "fp_arc" {
                        graphics.push(Graphics::FpArc(FpArc::from(iter)));
                        count -= 1;
                    } else if name == "pad" {
                        pads.push(Pad::from(iter));
                        count -= 1;
                    } else if name == "model" {
                        models.push(Model::from(iter));
                        count -= 1;
                    } else if name == "zone_connect" {
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                Some(State::Values(value)) => {
                    if value == "locked" {
                        locked = true;
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
                            layer,
                            tedit,
                            tstamp,
                            at,
                            angle,
                            descr,
                            tags,
                            path,
                            attr,
                            graphics,
                            pads,
                            models,
                            locked,
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
pub struct FpText {
    pub key: String,
    pub value: String,
    pub at: Array1<f64>,
    pub angle: Option<f64>,
    pub layer: String,
    pub effects: Effects,
    pub hidden: bool,
    pub tstamp: String,
}
impl FpText {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let key = iter.next().unwrap().into();
        let value = iter.next().unwrap().into();
        let mut layer = String::new();
        let mut at = arr1(&[0.0, 0.0]);
        let mut angle = None;
        let mut effects = Effects::new();
        let mut hidden = false;
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        if let Some(State::Values(a)) = iter.next() {
                            angle = Some(a.parse::<f64>().unwrap());
                        } else {
                            count -= 1;
                        }
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
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
                Some(State::Values(value)) => {
                    if value == "hide" {
                        hidden = true;
                    } else {
                        todo!("unknown value in FpText: {}", value);
                    }
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            key,
                            value,
                            at,
                            angle,
                            layer,
                            effects,
                            hidden,
                            tstamp,
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
pub struct FpArc {
    pub start: Array1<f64>,
    pub mid: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: String,
    pub width: f64,
    pub fill: String,
    pub tstamp: String,
}
impl FpArc {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut start = arr1(&[0.0, 0.0]);
        let mut mid = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut layer = String::new();
        let mut width: f64 = 0.0;
        let mut fill = String::new();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "mid" {
                        mid = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "fill" {
                        fill = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        let stroke: Stroke = Stroke::from(iter); //TODO
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
                            start,
                            mid,
                            end,
                            layer,
                            width,
                            fill,
                            tstamp,
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
pub struct FpCircle {
    pub center: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: String,
    pub width: f64,
    pub fill: String,
    pub tstamp: String,
}
impl FpCircle {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut center = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut layer = String::new();
        let mut width: f64 = 0.0;
        let mut fill = String::new();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "center" {
                        center = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "fill" {
                        fill = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        let stroke: Stroke = Stroke::from(iter); //TODO
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
                            center,
                            end,
                            layer,
                            width,
                            fill,
                            tstamp,
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
pub struct FpLine {
    pub start: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: String,
    pub width: f64,
    pub tstamp: String,
}
impl FpLine {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut layer = String::new();
        let mut start = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut width: f64 = 0.0;
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        let stroke: Stroke = Stroke::from(iter); //TODO
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
                            start,
                            end,
                            layer,
                            width,
                            tstamp,
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
pub struct Net {
    pub number: u32,
    pub name: String,
    pub tstamp: String,
}
impl Net {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let number = iter.next().unwrap().into();
        let name = iter.next().unwrap().into();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "tstamp" {
                        // let string_layers: String = iter.next().unwrap().into();
                        tstamp = iter.next().unwrap().into();
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
                            number,
                            name,
                            tstamp,
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
pub struct Via {
    pub at: Array1<f64>,
    pub size: f64,
    pub drill: f64,
    pub layers: Vec<String>,
    pub net: String,
    pub tstamp: String,
}
impl Via {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut at = arr1(&[0.0, 0.0]);
        let mut size = 0.0;
        let mut drill = -1.0;
        let mut layers = Vec::new();
        let mut net = String::new();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layers" {
                        loop {
                            let state = iter.next();
                            if let Some(State::Values(value)) = state {
                                layers.push(value.to_string());
                            } else if let Some(State::Text(value)) = state {
                                layers.push(value.to_string());
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "size" {
                        size = iter.next().unwrap().into();
                    } else if name == "drill" {
                        drill = iter.next().unwrap().into();
                    } else if name == "net" {
                        net = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
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
                            layers,
                            drill,
                            net,
                            tstamp,
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
pub enum PadShape {
    Circle,
    Oval,
    Rect,
    Trapezoid,
    RoundRect,
    ChamferedRect,
    Custom,
}

impl From<State<'_>> for PadShape {
    fn from(state: State<'_>) -> Self {
        if let State::Values(shape) = state {
            if shape == "circle" {
                return Self::Circle;
            } else if shape == "oval" {
                return Self::Oval;
            } else if shape == "rect" {
                return Self::Rect;
            } else if shape == "trapezoid" {
                return Self::Trapezoid;
            } else if shape == "roundrect" {
                return Self::RoundRect;
            } else if shape == "chamferedrect" {
                return Self::ChamferedRect;
            } else if shape == "custom" {
                return Self::Custom;
            }
        }
        panic!("Unknown Pad shape: {:?}", state);
    }
}

impl std::fmt::Display for PadShape {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PadShape::Circle => write!(f, "circle")?,
            PadShape::Oval => write!(f, "oval")?,
            PadShape::Rect => write!(f, "rect")?,
            PadShape::Trapezoid => write!(f, "trapezoid")?,
            PadShape::RoundRect => write!(f, "roundrect")?,
            PadShape::ChamferedRect => write!(f, "chamferedrect")?,
            PadShape::Custom => write!(f, "custom")?,
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pad {
    pub number: String,
    pub padtype: String,
    pub padshape: PadShape,
    pub anchor_padshape: PadShape,
    pub locked: bool,
    pub at: Array1<f64>,
    pub angle: f64,
    pub size: Array1<f64>,
    pub layers: Vec<String>,
    pub rratio: f64,
    pub oval: bool,
    pub drill: f64,
    pub net: Option<Net>,
    pub tstamp: String,
}
impl Pad {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let number = iter.next().unwrap().into();
        let padtype = iter.next().unwrap().into();
        let padshape = iter.next().unwrap().into();
        let mut locked = false;
        let mut layers = Vec::new();
        let mut at = arr1(&[0.0, 0.0]);
        let mut angle = 0.0;
        let mut size = arr1(&[0.0, 0.0]);
        let mut rratio = 0.0;
        let mut oval = false;
        let mut drill = -1.0;
        let mut net = None;
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layers" {
                        loop {
                            let state = iter.next();
                            if let Some(State::Values(value)) = state {
                                layers.push(value.to_string());
                            } else if let Some(State::Text(value)) = state {
                                layers.push(value.to_string());
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        if let Some(State::Values(value)) = iter.next() {
                            angle = value.parse::<f64>().unwrap();
                        } else {
                            count -= 1;
                        }
                    } else if name == "size" {
                        size = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "roundrect_rratio" {
                        rratio = iter.next().unwrap().into();
                    } else if name == "drill" {
                        loop {
                            if let Some(State::Values(value)) = iter.next() {
                                if value == "oval" {
                                    oval = true;
                                } else {
                                    drill = value.parse::<f64>().unwrap();
                                }
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "net" {
                        net = Some(Net::from(iter));
                        count -= 1;
                    } else if name == "pinfunction" {
                        /* TODO: */
                    } else if name == "pintype" {
                        /* TODO: */
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "solder_mask_margin" {
                        /* TODO */
                    } else if name == "thermal_bridge_angle" {
                        /* TODO */
                    } else {
                        todo!("unknown: {}", name);
                    }
                }
                Some(State::Values(value)) => {
                    if value == "locked" {
                        locked = true;
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            number,
                            padtype,
                            padshape,
                            anchor_padshape: PadShape::Circle, //TODO: should be none
                            locked,
                            at,
                            angle,
                            size,
                            layers,
                            rratio,
                            oval,
                            drill,
                            net,
                            tstamp,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
    /* pub fn GetSolderPasteMargin(&self) -> Array1<f64> {
        /*TODO: // The pad inherits the margin only to calculate a default shape,
        // therefore only if it is also a copper layer.
        // Pads defined only on mask layers (and perhaps on other tech layers) use the shape
        // defined by the pad settings only
        bool isOnCopperLayer = ( m_layerMask & LSET::AllCuMask() ).any();

        if( !isOnCopperLayer )
            return wxSize( 0, 0 );

        int     margin = m_localSolderPasteMargin;
        double  mratio = m_localSolderPasteMarginRatio;

        FOOTPRINT* parentFootprint = GetParent();

        if( parentFootprint )
        {
            if( margin == 0 )
                margin = parentFootprint->GetLocalSolderPasteMargin();

            auto brd = GetBoard();

            if( margin == 0 && brd )
                margin = brd->GetDesignSettings().m_SolderPasteMargin;

            if( mratio == 0.0 )
                mratio = parentFootprint->GetLocalSolderPasteMarginRatio();

            if( mratio == 0.0 && brd )
            {
                mratio = brd->GetDesignSettings().m_SolderPasteMarginRatio;
            }
        }

        wxSize pad_margin;
        pad_margin.x = margin + KiROUND( m_size.x * mratio );
        pad_margin.y = margin + KiROUND( m_size.y * mratio );

        // ensure mask have a size always >= 0
        if( pad_margin.x < -m_size.x / 2 )
            pad_margin.x = -m_size.x / 2;

        if( pad_margin.y < -m_size.y / 2 )
            pad_margin.y = -m_size.y / 2;

        return pad_margin; */
        arr1(&[0.0, 0.0])
    }
    pub fn GetSolderMaskMargin(&self) -> f64 {
        /*TODO: bool isOnCopperLayer = ( m_layerMask & LSET::AllCuMask() ).any();

        if( !isOnCopperLayer )
            return 0;

        int     margin = m_localSolderMaskMargin;

        FOOTPRINT* parentFootprint = GetParent();

        if( parentFootprint )
        {
            if( margin == 0 )
            {
                if( parentFootprint->GetLocalSolderMaskMargin() )
                    margin = parentFootprint->GetLocalSolderMaskMargin();
            }

            if( margin == 0 )
            {
                const BOARD* brd = GetBoard();

                if( brd )
                    margin = brd->GetDesignSettings().m_SolderMaskMargin;
            }
        }

        // ensure mask have a size always >= 0
        if( margin < 0 )
        {
            int minsize = -std::min( m_size.x, m_size.y ) / 2;

            if( margin < minsize )
                margin = minsize;
        }

        return margin; */
        0.0
    }
    pub fn GetRoundRectCornerRadius(&self) -> f64 {
        //TODO return KiROUND( std::min( m_size.x, m_size.y ) * m_roundedCornerScale );
        0.0
    }

    pub fn SetRoundRectCornerRadius(&mut self, aRadius: f64 ) {
    /* TODO: int min_r = std::min( m_size.x, m_size.y );

    if( min_r > 0 )
        SetRoundRectRadiusRatio( aRadius / min_r ); */
    }


    pub fn SetRoundRectRadiusRatio(&mut self, aRadiusScale: f64) {
        /* m_roundedCornerScale = std::max( 0.0, std::min( aRadiusScale, 0.5 ) );
        SetDirty(); */
    } */
    
    // pub fn MergePrimitivesAsPolygon(&self, aMergedPolygon: &SHAPE_POLY_SET,  aErrorLoc: ERROR_LOC) {
    /* const BOARD* board = GetBoard();
    int          maxError = board ? board->GetDesignSettings().m_MaxError : ARC_HIGH_DEF;

    aMergedPolygon->RemoveAllContours();

    // Add the anchor pad shape in aMergedPolygon, others in aux_polyset:
    // The anchor pad is always at 0,0
    switch( GetAnchorPadShape() )
    {
    case PAD_SHAPE::RECT:
    {
        SHAPE_RECT rect( -GetSize().x / 2, -GetSize().y / 2, GetSize().x, GetSize().y );
        aMergedPolygon->AddOutline( rect.Outline() );
    }
        break;

    default:
    case PAD_SHAPE::CIRCLE:
        TransformCircleToPolygon( *aMergedPolygon, wxPoint( 0, 0 ), GetSize().x / 2, maxError,
                                  aErrorLoc );
        break;
    }

    addPadPrimitivesToPolygon( aMergedPolygon, maxError, aErrorLoc ); */
    // }

    // clear the basic shapes list and associated data
    /* pub fn DeletePrimitivesList(&mut self) {
        /* m_editPrimitives.clear();
        SetDirty(); */
    } */

    /*
 * Has meaning only for free shape pads.
 * add a free shape to the shape list.
 * the shape is a polygon (can be with thick outline), segment, circle or arc
 */

    /* pub fn AddPrimitivePoly(&mut self, aPoly: &SHAPE_POLY_SET, aThickness: u32, aFilled: bool) {
    /* // If aPoly has holes, convert it to a polygon with no holes.
    SHAPE_POLY_SET poly_no_hole;
    poly_no_hole.Append( aPoly );

    if( poly_no_hole.HasHoles() )
        poly_no_hole.Fracture( SHAPE_POLY_SET::PM_STRICTLY_SIMPLE );

    // There should never be multiple shapes, but if there are, we split them into
    // primitives so that we can edit them both.
    for( int ii = 0; ii < poly_no_hole.OutlineCount(); ++ii )
    {
        SHAPE_POLY_SET poly_outline( poly_no_hole.COutline( ii ) );
        PCB_SHAPE* item = new PCB_SHAPE();
        item->SetShape( SHAPE_T::POLY );
        item->SetFilled( aFilled );
        item->SetPolyShape( poly_outline );
        item->SetWidth( aThickness );
        item->SetParent( this );
        m_editPrimitives.emplace_back( item );
    }

    SetDirty(); */
    }
    
    pub fn SetAnchorPadShape(&mut self, shape: PadShape) {
        self.anchor_padshape = if shape ==  PadShape::Rect {
            PadShape::Rect
        } else { PadShape::Circle };
        // SetDirty();
    }

    pub fn TransformShapeWithClearanceToPolygon(&mut self, aCornerBuffer: &SHAPE_POLY_SET,
                                               aLayer: LayerId, aClearanceValue: u32,
                                               aMaxError: u32,  aErrorLoc: ERROR_LOC,
                                               ignoreLineWidth: bool) {
    } */
}

#[derive(Debug, PartialEq, Clone)]
pub struct Model {
    pub path: String,
    pub offset: Array1<f64>,
    pub scale: Array1<f64>,
    pub rotate: Array1<f64>,
}
impl Model {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let path = iter.next().unwrap().into();
        let mut offset = arr1(&[0.0, 0.0]);
        let mut scale = arr1(&[0.0, 0.0]);
        let mut rotate = arr1(&[0.0, 0.0]);
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "offset" {
                        if let Some(State::StartSymbol(_)) = iter.next() {
                            offset = arr1(&[
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                            ]);
                        }
                        count += 1;
                    } else if name == "scale" {
                        if let Some(State::StartSymbol(_)) = iter.next() {
                            scale = arr1(&[
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                            ]);
                        }
                        count += 1;
                    } else if name == "rotate" {
                        if let Some(State::StartSymbol(_)) = iter.next() {
                            rotate = arr1(&[
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                                iter.next().unwrap().into(),
                            ]);
                        }
                        count += 1;
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
                            offset,
                            rotate,
                            scale,
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
pub struct GrText {
    pub text: String,
    pub at: Array1<f64>,
    pub angle: f64,
    pub layer: String,
    pub effects: Effects,
    pub tstamp: String,
}
impl GrText {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let text = iter.next().unwrap().into();
        let mut layer = String::new();
        let mut at = arr1(&[0.0, 0.0]);
        let mut angle = 1.0;
        let mut effects = Effects::new();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "at" {
                        at = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                        if let State::Values(val) = iter.next().unwrap() {
                            angle = val.parse::<f64>().unwrap();
                        } else {
                            count -= 1;
                        }
                    } else if name == "effects" {
                        effects = Effects::from(iter);
                        count -= 1;
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
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
                            layer,
                            effects,
                            tstamp,
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
pub struct GrLine {
    pub start: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: String,
    pub width: f64,
    pub tstamp: String,
}
impl GrLine {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut layer = String::new();
        let mut start = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut width: f64 = 0.0;
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "stroke" {
                        let stroke: Stroke = Stroke::from(iter); //TODO
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
                            start,
                            end,
                            layer,
                            width,
                            tstamp,
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
pub struct GrCircle {
    pub center: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: String,
    pub width: f64,
    pub fill: String,
    pub tstamp: String,
}
impl GrCircle {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut layer = String::new();
        let mut center = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut width: f64 = 0.0;
        let mut fill = String::new();
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "center" {
                        center = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "fill" {
                        fill = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
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
                            end,
                            layer,
                            width,
                            fill,
                            tstamp,
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
pub struct Segment {
    pub start: Array1<f64>,
    pub end: Array1<f64>,
    pub layer: LayerId,
    pub net: String,
    pub width: f64,
    pub tstamp: String,
}
impl Segment {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut layer = LayerId::Undefined;
        let mut net = String::new();
        let mut start = arr1(&[0.0, 0.0]);
        let mut end = arr1(&[0.0, 0.0]);
        let mut width: f64 = 0.0;
        let mut tstamp = String::new();
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "start" {
                        start = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "end" {
                        end = arr1(&[iter.next().unwrap().into(), iter.next().unwrap().into()]);
                    } else if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "net" {
                        net = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
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
                            layer,
                            net,
                            width,
                            tstamp,
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
pub struct Zone {
    pub layer: String,
    pub layers: Vec<LayerId>,
    pub net: u32,
    pub net_name: String,
    pub tstamp: String,
    pub hatch_style: String,
    pub hatch_edge: f64,
    pub pad_clearance: f64,
    pub min_thickness: f64,
    pub filled: bool,
    pub fill_thermal_gap: f64,
    pub fill_thermal_bridge: f64,
    pub polygon: Array2<f64>,
    pub filled_polygon: (String, Array2<f64>),
    pub keepout: Option<Keepout>,
}
impl Zone {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut layer = String::new();
        let mut layers = Vec::new();
        let mut net = 0;
        let mut net_name = String::new();
        let mut tstamp = String::new();
        let mut hatch_style: String = String::new();
        let mut hatch_edge: f64 = 0.0;
        let mut pad_clearance: f64 = 0.0;
        let mut min_thickness: f64 = 0.0;
        let mut filled = false;
        let mut fill_thermal_gap: f64 = 0.0;
        let mut fill_thermal_bridge: f64 = 0.0;
        let mut polygon: Array2<f64> = Array2::zeros((0, 2));
        let mut filled_polygon = (String::new(), Array2::zeros((0, 2)));
        let mut keepout = None;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "layers" {
                        loop {
                            let layer = iter.next();
                            if let Some(State::Values(value)) = layer {
                                if value == "F&B.Cu" {
                                    layers.push(LayerId::FCu);
                                    layers.push(LayerId::BCu);
                                } else {
                                    todo!("Unknown Layer alias: {}", value);
                                }
                            } else if let Some(State::Text(_)) = layer {
                                layers.push(layer.unwrap().into());
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "net" {
                        net = iter.next().unwrap().into();
                    } else if name == "net_name" {
                        net_name = iter.next().unwrap().into();
                    } else if name == "tstamp" {
                        tstamp = iter.next().unwrap().into();
                    } else if name == "hatch" {
                        loop {
                            if let Some(State::Values(value)) = iter.next() {
                                if ["none", "edge", "full"].contains(&value) {
                                    hatch_style = value.to_string();
                                } else {
                                    hatch_edge = value.parse::<f64>().unwrap();
                                }
                            } else {
                                count -= 1;
                                break;
                            }
                        }
                    } else if name == "connect_pads" {
                        if let Some(State::StartSymbol(name)) = iter.next() {
                            if name == "clearance" {
                                pad_clearance = iter.next().unwrap().into();
                            } else {
                                todo!("other start symbol in pad clearance: {}", name);
                            }
                        }
                        count += 1;
                    } else if name == "min_thickness" {
                        min_thickness = iter.next().unwrap().into();
                    } else if name == "fill" {
                        let mut inner_count = 1;
                        loop {
                            let state = iter.next().unwrap();
                            if let State::StartSymbol(name) = state {
                                inner_count += 1;
                                if name == "thermal_gap" {
                                    fill_thermal_gap = iter.next().unwrap().into();
                                } else if name == "thermal_bridge_width" {
                                    fill_thermal_bridge = iter.next().unwrap().into();
                                } else if name == "island_removal_mode" {
                                } else if name == "island_area_min" {
                                    //fill_thermal_bridge = iter.next().unwrap().into();
                                } else {
                                    todo!("other start symbol in fill: {}", name);
                                }
                            } else if let State::Values(val) = state {
                                filled = val == "yes";
                            } else if let State::EndSymbol = state {
                                inner_count -= 1;
                                if inner_count == 0 {
                                    count -= 1;
                                    break;
                                }
                            }
                        }
                    } else if name == "polygon" {
                        let mut index = 1;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(name)) = state {
                                index += 1;
                                if name == "xy" {
                                    polygon
                                        .push_row(ArrayView::from(&[
                                            iter.next().unwrap().into(),
                                            iter.next().unwrap().into(),
                                        ]))
                                        .unwrap();
                                }
                            } else if let Some(State::EndSymbol) = state {
                                index -= 1;
                                if index == 0 {
                                    count -= 1;
                                    break;
                                }
                            }
                        }
                    } else if name == "filled_polygon" {
                        if let Some(State::StartSymbol(_name)) = iter.next() {
                            filled_polygon.0 = iter.next().unwrap().into();
                        }
                        let mut index = 2;
                        loop {
                            let state = iter.next();
                            if let Some(State::StartSymbol(name)) = state {
                                index += 1;
                                if name == "xy" {
                                    filled_polygon
                                        .1
                                        .push_row(ArrayView::from(&[
                                            iter.next().unwrap().into(),
                                            iter.next().unwrap().into(),
                                        ]))
                                        .unwrap();
                                }
                            } else if let Some(State::EndSymbol) = state {
                                index -= 1;
                                if index == 0 {
                                    break;
                                }
                            }
                        }
                        count -= 1;
                    } else if name == "keepout" {
                        keepout = Some(Keepout::from(iter));
                        count -= 1;
                    } else if name == "filled_areas_thickness" {
                        // TODO

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
                            layer,
                            layers,
                            net,
                            net_name,
                            tstamp,
                            hatch_style,
                            hatch_edge,
                            pad_clearance,
                            min_thickness,
                            filled,
                            fill_thermal_gap,
                            fill_thermal_bridge,
                            polygon,
                            filled_polygon,
                            keepout,
                        };
                    }
                }
                _ => {}
            }
        }
        panic!();
    }
    /* pub fn layer_set(&self) -> LayerSet {
        let mut set = LayerSet::new();
        for layer in &self.layers {
            set.set(*layer);
        }
        set
    }
    /**
     * @return a reference to the list of filled polygons.
     */
    pub fn GetFilledPolysList(&self, aLayer: LayerId) -> SHAPE_POLY_SET {
        // return self.m_FilledPolysList.at( aLayer );
        SHAPE_POLY_SET {  } //TODO: implement
    }
    pub fn IsIsland(&self, layer: &LayerId, index: usize) -> bool {
        false //TODO: implemnet
    } */
}

#[derive(Debug, PartialEq, Clone)]
pub struct GrPoly {
    pub pts: Array2<f64>,
    pub width: f64,
    pub layer: String,
    pub fill_type: String,
    pub tstmp: Option<String>,
}
impl GrPoly {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        let mut width = 0.0;
        let mut layer = String::new();
        let mut fill_type = String::new();
        let mut tstmp = None;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "width" {
                        width = iter.next().unwrap().into();
                    } else if name == "layer" {
                        layer = iter.next().unwrap().into();
                    } else if name == "fill_type" {
                        fill_type = iter.next().unwrap().into();
                    } else if name == "tstmp" {
                        tstmp = Some(iter.next().unwrap().into());
                    } else if name == "pts" {
                    } else if name == "xy" {
                        pts.push_row(ArrayView::from(&[
                            iter.next().unwrap().into(),
                            iter.next().unwrap().into(),
                        ]))
                        .unwrap();
                    }
                }
                None => {
                    break;
                }
                Some(State::EndSymbol) => {
                    count -= 1;
                    if count == 0 {
                        return Self {
                            layer,
                            width,
                            fill_type,
                            tstmp,
                            pts,
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
pub struct Keepout {
    pub tracks: bool,
    pub vias: bool,
    pub pads: bool,
    pub copperpour: bool,
    pub footprints: bool,
}
impl Keepout {
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut tracks = false;
        let mut vias = false;
        let mut pads = false;
        let mut copperpour = false;
        let mut footprints = false;
        let mut count = 1;
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    count += 1;
                    if name == "tracks" {
                        let allowed: String = iter.next().unwrap().into();
                        tracks = allowed == "allowed";
                    } else if name == "vias" {
                        let allowed: String = iter.next().unwrap().into();
                        vias = allowed == "allowed";
                    } else if name == "pads" {
                        let allowed: String = iter.next().unwrap().into();
                        pads = allowed == "allowed";
                    } else if name == "copperpour" {
                        let allowed: String = iter.next().unwrap().into();
                        copperpour = allowed == "allowed";
                    } else if name == "footprints" {
                        let allowed: String = iter.next().unwrap().into();
                        footprints = allowed == "allowed";
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
                            tracks,
                            vias,
                            pads,
                            copperpour,
                            footprints,
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
pub struct Setup {
    pub pad_to_mask_clearance: Option<String>,
    pub values: IndexMap<String, String>,
}
impl Setup {
    pub fn new() -> Self {
        Setup {
            pad_to_mask_clearance: None,
            values: IndexMap::new(),
        }
    }
    pub fn from<'a, I: Iterator<Item = State<'a>>>(iter: &mut I) -> Self {
        let mut index = 1;
        let mut pad_to_mask_clearance: Option<String> = None;
        let mut key: Option<String> = None;
        let mut values: IndexMap<String, String> = IndexMap::new();
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    if name == "pad_to_mask_clearance" {
                        pad_to_mask_clearance = Some(iter.next().unwrap().into());
                    } else {
                        key = Some(name.to_string());
                    }
                    index += 1;
                }
                Some(State::EndSymbol) => {
                    index -= 1;
                    if index == 0 {
                        return Setup {
                            pad_to_mask_clearance,
                            values,
                        };
                    }
                }
                Some(State::Values(val)) => {
                    if let Some(key) = key {
                        values.insert(key.to_string(), val.to_string());
                    } else {
                        println!(" no key set for: {}", val);
                    }
                    key = None;
                }
                Some(State::Text(val)) => {
                    if let Some(key) = key {
                        values.insert(key.to_string(), val.to_string());
                    } else {
                        println!(" no key set for: {}", val);
                    }
                    key = None;
                }
                None => {
                    break;
                }
            }
        }
        panic!();
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Graphics {
    FpText(FpText),
    FpLine(FpLine),
    FpCircle(FpCircle),
    FpArc(FpArc),
}
#[derive(Debug, Clone, PartialEq)]
pub enum PcbElements {
    Setup(Setup),
    Footprint(Footprint),
    Text(GrText),
    Line(GrLine),
    GrCircle(GrCircle),
    Segment(Segment),
    Via(Via),
    Zone(Zone),
    GrPoly(GrPoly),
}
