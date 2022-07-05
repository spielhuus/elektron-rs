use crate::sexp::{Error, SexpNode, SexpType, Justify, FillType, Color, Effects, LineType, Stroke};
use ndarray::{arr1, Array, Array1, Array2, ArrayView};

macro_rules! get {
    ($node:expr, $key:expr) => {
        $node.get($key).unwrap()
    };
    ($node:expr, $key:expr, $index:expr) => {
        SexpGet::<_, SexpNode>::get($node, $key)
            .unwrap()
            .get($index)
            .unwrap()
    };
}
pub(crate) use get;

/// Access the nodes and values.
pub trait SexpGet<S, T> {
    fn get(&self, index: S) -> Result<T, Error>;
}
/// Get the value as String by index.
impl SexpGet<usize, String> for SexpNode {
    /// Get the value as String by index.
    fn get(&self, index: usize) -> Result<String, Error> {
        match &self.values[index] {
            SexpType::ChildSexpValue(n) => Ok(n.value.to_string()),
            SexpType::ChildSexpText(n) => Ok(n.value.to_string()),
            SexpType::ChildSexpNode(_) => Err(Error::ExpectValueNode),
        }
    }
}
impl SexpGet<usize, SexpNode> for SexpNode {
    fn get(&self, index: usize) -> Result<SexpNode, Error> {
        match &self.values[index] {
            SexpType::ChildSexpNode(n) => Ok(n.clone()),
            _ => Err(Error::ExpectSexpNode),
        }
    }
}
/// get the first child Node by name.
impl SexpGet<&str, SexpNode> for SexpNode {
    fn get(&self, index: &str) -> Result<SexpNode, Error> {
        for v in &self.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == index {
                    return Ok(n.clone());
                }
            }
        }
        Err(Error::SymbolNotFound(index.to_string()))
    }
}
impl SexpGet<&str, Justify> for SexpNode {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<Justify, Error> {
        let mytype: String = get!(self, key, 0);
        if mytype == "right" {
            Ok(Justify::Right)
        } else if mytype == "left" {
            Ok(Justify::Left)
        } else if mytype == "top" {
            Ok(Justify::Top)
        } else if mytype == "bottom" {
            Ok(Justify::Bottom)
        } else if mytype == "mirror" {
            Ok(Justify::Mirror)
        } else {
            Err(Error::JustifyValueError)
        }
    }
}
impl SexpGet<&str, FillType> for SexpNode {
    /// Get the LineType
    fn get(&self, key: &str) -> Result<FillType, Error> {
        let myfill: SexpNode = get!(self, key);
        let mytype: String = get!(&myfill, "type", 0);
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
impl SexpGet<&str, LineType> for SexpNode {
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
impl SexpGet<usize, usize> for SexpNode {
    fn get(&self, index: usize) -> Result<usize, Error> {
        match &self.values[index] {
            SexpType::ChildSexpValue(n) => {
                let f: usize = n.value.parse().unwrap();
                Ok(f)
            }
            SexpType::ChildSexpText(n) => {
                let f: usize = n.value.parse().unwrap();
                Ok(f)
            }
            _ => Err(Error::ExpectValueNode),
        }
    }
}
impl SexpGet<usize, i32> for SexpNode {
    fn get(&self, index: usize) -> Result<i32, Error> {
        match &self.values[index] {
            SexpType::ChildSexpText(n) => {
                let f: i32 = n.value.parse().unwrap();
                Ok(f)
            }
            SexpType::ChildSexpValue(n) => {
                let f: i32 = n.value.parse().unwrap();
                Ok(f)
            }
            _ => Err(Error::ExpectValueNode),
        }
    }
}
impl SexpGet<usize, f64> for SexpNode {
    fn get(&self, index: usize) -> Result<f64, Error> {
        match &self.values[index] {
            SexpType::ChildSexpValue(n) => {
                let f: f64 = n.value.parse().unwrap();
                Ok(f)
            }
            SexpType::ChildSexpText(n) => {
                let f: f64 = n.value.parse().unwrap();
                Ok(f)
            }
            _ => Err(Error::ExpectValueNode),
        }
    }
}
impl SexpGet<&str, Array1<f64>> for SexpNode {
    fn get(&self, key: &str) -> Result<Array1<f64>, Error> {
        for v in &self.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == key {
                    let x: f64 = n.get(0).unwrap();
                    let y: f64 = n.get(1).unwrap();
                    return Ok(arr1(&[x, y]));
                }
            }
        }
        Err(Error::SymbolNotFound(key.to_string()))
    }
}
impl SexpGet<&str, Array2<f64>> for SexpNode {
    fn get(&self, key: &str) -> Result<Array2<f64>, Error> {
        for v in &self.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == key {
                    //let array: Array2<f64> = arr2(&[]);
                    let mut array: Array2<f64> = Array::zeros((0, 2));
                    for _xy in &n.values {
                        if let SexpType::ChildSexpNode(xy_value) = _xy {
                            if xy_value.name == "xy" {
                                let x: f64 = xy_value.get(0).unwrap();
                                let y: f64 = xy_value.get(1).unwrap();
                                array.push_row(ArrayView::from(&[x, y])).unwrap();
                            }
                        }
                    }
                    return Ok(array);
                }
            }
        }
        Err(Error::SymbolNotFound(key.to_string()))
    }
}
/* impl SexpGet<&str, Color> for SexpNode {
    fn get(&self, key: &str) -> Result<Color, ParseError> {
        for v in &self.values {
            match v {
                SexpType::ChildSexpNode(n) => {
                    if n.name == key {
                        let color = Color {
                            r: n.get(0).unwrap(),
                            g: n.get(0).unwrap(),
                            b: n.get(0).unwrap(),
                            a: n.get(0).unwrap(),
                        };
                    return Ok(color);
                    }
                }
                _ => {}
            }
        }
        Err(ParseError::new("node not found"))
    }
} */
impl SexpGet<&str, Stroke> for SexpNode {
    fn get(&self, key: &str) -> Result<Stroke, Error> {
        let stroke: SexpNode = self.get(key)?;
        let width: f64 = if stroke.contains("width") {
            get!(&stroke, "width", 0)
        } else {
            0.0
        };
        let line_type: LineType = if stroke.contains("type") {
            stroke.get("type").unwrap()
        } else {
            LineType::Default
        };
        let color: Color = if stroke.contains("color") {
            Color::from(stroke.nodes("color").unwrap()[0])
        } else {
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }
        };
        let fill: FillType = if self.contains("fill") {
            get!(self, "fill")
        } else {
            FillType::None
        };

        Ok(Stroke {
            width,
            line_type,
            color,
            fill,
        })
    }
}
impl SexpGet<&str, Effects> for SexpNode {
    fn get(&self, key: &str) -> Result<Effects, Error> {
        for v in &self.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == key {
                    let font: SexpNode = get!(n, "font");
                    let face: String = if font.contains("face") {
                        get!(&font, "face", 0)
                    } else {
                        "default".to_string()
                    };
                    let size: f64 = if font.contains("size") {
                        get!(&font, "size", 0)
                    } else {
                        0.0
                    };
                    let thickness: f64 = if font.contains("thickess") {
                        get!(&font, "thickness", 0)
                    } else {
                        0.0
                    };
                    let line_spacing: f64 = if font.contains("line_spacing") {
                        get!(&font, "line_spacing", 0)
                    } else {
                        0.0
                    };
                    let justify: Justify = if n.contains("justify") {
                        get!(n, "justify")
                    } else {
                        Justify::Center
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
                        n.has("hide"),
                    );
                    return Ok(effects);
                }
            }
        }
        Err(Error::SymbolNotFound(key.to_string()))
    }
}
