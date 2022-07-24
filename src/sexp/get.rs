use ndarray::{arr1, Array1, Array2, ArrayView};

use crate::Error;
use crate::sexp::{Color, Effects, Justify, Sexp, Stroke, LineType, FillType};
use crate::sexp::test::Test;

macro_rules! get {
    ($node:expr, $key:expr) => {
        $node.get($key)
    };
    ($node:expr, $key:expr, $index:expr) => {
        Get::<_, Vec<&Sexp>>::get($node, $key)
            .unwrap().get(0).unwrap()
            .get($index).unwrap()
    };
}
pub(crate) use get;


/// Access the nodes and values.
pub trait Get<'a, S, T> {
    fn get(&'a self, index: S) -> Result<T, Error>;
}
/// Get the value as String by index.
impl Get<'_, usize, String> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<String, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.to_string())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.to_string())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as float by index.
impl Get<'_, usize, f64> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<f64, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as usize by index.
impl Get<'_, usize, usize> for Sexp {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<usize, Error> {
        if let Sexp::Node(_, values) = &self {
            if let Some(Sexp::Value(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else if let Some(Sexp::Text(value)) = values.get(index) {
                Ok(value.parse().unwrap())
            } else { Err(Error::ParseError) }
        } else { Err(Error::ExpectValueNode) }
    }
}
/// Get the value as Array1 by index.
impl Get<'_, &str, Array1<f64>> for Sexp {
    fn get(&self, key: &str) -> Result<Array1<f64>, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        let pos: &Sexp = nodes.get(0).unwrap();
        let x: f64 = pos.get(0).unwrap();
        let y: f64 = pos.get(1).unwrap();
        Ok(arr1(&[x, y]))
    }
}
/// Get the value as Array2 by index.
impl Get<'_, &str, Array2<f64>> for Sexp {
    /// Get the value as String by index.
    fn get(&self, key: &str) -> Result<Array2<f64>, Error> {
        let mut array: Array2<f64> = Array2::zeros((0, 2));
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        let xy: Vec<&Sexp> = nodes.get(0).unwrap().get("xy").unwrap();

        for _xy in xy {
            let x: f64 = _xy.get(0).unwrap();
            let y: f64 = _xy.get(1).unwrap();
            array.push_row(ArrayView::from(&[x, y])).unwrap();
        }
        Ok(array)
    }
}
/// Get the value as String by index.
impl<'a> Get<'a, &str, Vec<&'a Sexp>> for Sexp {
    /// Get the value as String by index.
    fn get(&'a self, key: &str) -> Result<Vec<&'a Sexp>, Error> {
        if let Sexp::Node(_, values) = &self {
            Ok(values.into_iter().filter(|n| {
                if let Sexp::Node(name, _) = n {
                    name == key
                } else { false }
            }).collect())
        } else { Err(Error::ExpectValueNode) }
    }
}

/// Get the value as Effects by index.
impl<'a> Get<'a, &str, Effects> for Sexp {
    fn get(&'a self, key: &str) -> Result<Effects, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        if nodes.len() == 1 {
            let node = nodes.get(0).unwrap();
                let fonts: Vec<&Sexp> = node.get("font").unwrap();
                if fonts.len() == 1 {
                    let font = fonts.get(0).unwrap();
                    // get face 0
                    /* let face_list: Vec<&Sexp> = font.get("face").unwrap();
                    let face_item: &Sexp = face_list.get(0).unwrap();
                    let face: String = face_item.get(0).unwrap(); */

                    let face: String = if font.contains("face") {
                        get!(*font, "face", 0)
                    } else {
                        "default".to_string()
                    };
                    let size: f64 = if font.contains("size") {
                        get!(*font, "size", 0)
                    } else {
                        0.0
                    };
                    let thickness: f64 = if font.contains("thickess") {
                        get!(*font, "thickness", 0)
                    } else {
                        0.0
                    };
                    let line_spacing: f64 = if font.contains("line_spacing") {
                        get!(*font, "line_spacing", 0)
                    } else {
                        0.0
                    };
                    let justify: Vec<Justify> = if node.contains("justify") {
                        get!(*node, "justify").unwrap()
                    } else {
                        vec![Justify::Center]
                    };

                    let effects = Effects::new(
                        face,
                        Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        },
                        size,
                        thickness,
                        font.has("bold"),
                        font.has("italic"),
                        line_spacing,
                        justify,
                        node.has("hide"),
                    );
                    return Ok(effects);
                } else {
                    Err(Error::ParseError)
                }
        } else {
            Err(Error::ParseError)
        }
    }
}

/// Get the value as Stroke by index.
impl<'a> Get<'a, &str, Stroke> for Sexp {
    fn get(&'a self, key: &str) -> Result<Stroke, Error> {
        let nodes: Vec<&Sexp> = self.get(key).unwrap();
        if nodes.len() == 1 {
            let stroke = nodes.get(0).unwrap();

            let width: f64 = if stroke.contains("width") {
                get!(*stroke, "width", 0)
            } else {
                0.0
            };
            let line_type: LineType = if stroke.contains("type") {
                stroke.get("type").unwrap()
            } else {
                LineType::Default
            };
            let color: Color = if stroke.contains("color") {
                let nodes: Vec<&Sexp> = stroke.get("color").unwrap();
                let color: &Sexp = nodes.get(0).unwrap();
                Color::from(color)
            } else {
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }
            };
            let fill: FillType = if self.contains("fill") {
                get!(self, "fill").unwrap()
            } else {
                FillType::None
            };

            Ok(Stroke {
                width,
                line_type,
                color,
                fill,
            })
        } else {
            Err(Error::ParseError)
        }
    }
}

impl<'a> Get<'a, &str, Vec<Justify>> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<Vec<Justify>, Error> {
        let mut justify = Vec::new();
        let node: Vec<&Sexp> = self.get(key).unwrap();
        if node.len() == 1 {
            if let Sexp::Node(_, j) = node.get(0).unwrap() {
                for _j in j {
                    if let Sexp::Value(_j) = _j {
                        if _j == "right" {
                            justify.push(Justify::Right);
                        } else if _j == "left" {
                            justify.push(Justify::Left);
                        } else if _j == "top" {
                            justify.push(Justify::Top);
                        } else if _j == "bottom" {
                            justify.push(Justify::Bottom);
                        } else if _j == "mirror" {
                            justify.push(Justify::Mirror);
                        } else {
                            return Err(Error::JustifyValueError);
                        }
                    } else {
                        return Err(Error::ExpectValueNode);
                    }
                }
            } else {
                return Err(Error::ExpectSexpNode);
            }
        } else {
            return Err(Error::JustifyValueError);
        }
        Ok(justify)
    }
}

impl<'a> Get<'a, &str, FillType> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<FillType, Error> {
        let nodes: Vec<&Sexp> = get!(self, key).unwrap();
        let myfill: &Sexp = nodes.get(0).unwrap();
        let mytype: String = get!(myfill, "type", 0);
        if mytype == "none" {
            Ok(FillType::None)
        } else if mytype == "outline" {
            Ok(FillType::Outline)
        } else if mytype == "background" {
            Ok(FillType::Background)
        } else {
            Ok(FillType::None)
        }
    }
}

impl<'a> Get<'a, &str, LineType> for Sexp {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<LineType, Error> {
        let mytype: String = get!(self, key, 0);
        if mytype == "dash" {
            Ok(LineType::Dash)
        } else if mytype == "dash_dot" {
            Ok(LineType::DashDot)
        } else if mytype == "dash_dot_dot" {
            Ok(LineType::DashDotDot)
        } else if mytype == "dot" {
            Ok(LineType::Dot)
        } else if mytype == "default" {
            Ok(LineType::Default)
        } else if mytype == "solid" {
            Ok(LineType::Solid)
        } else {
            Err(Error::LineTypeValueError)
        }
    }
}
