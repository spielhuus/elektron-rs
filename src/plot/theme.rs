use std::collections::HashMap;

use crate::error::Error;
use crate::sexp::model::{color, Effects, Stroke};
use crate::sexp::{SexpParser, State};

#[derive(Debug)]
enum ThemeItems {
    Effects(Effects),
    Stroke(Stroke),
    Color((f64, f64, f64, f64)),
}

pub struct Themer;

pub trait ThemerMerge<T> {
    fn get(a: &T, b: &T) -> T;
}
impl ThemerMerge<Stroke> for Themer {
    fn get(a: &Stroke, b: &Stroke) -> Stroke {
        Stroke {
            width: if a.width != 0.0 {
                a.width * 1.4
            } else {
                b.width
            },
            linetype: a.linetype.clone(),
            color: if a.color != (0.0, 0.0, 0.0, 0.0) {
                a.color
            } else {
                b.color
            },
            filltype: a.filltype.clone(),
        }
    }
}
impl ThemerMerge<Effects> for Themer {
    fn get(a: &Effects, b: &Effects) -> Effects {
        let font = if !a.font.is_empty() {
            a.font.clone()
        } else {
            b.font.clone()
        };
        let font_size = if a.font_size != (0.0, 0.0) {
            a.font_size
        } else {
            b.font_size
        };
        let thickness = if a.thickness != 0.0 {
            a.thickness
        } else {
            b.thickness
        };
        let line_spacing = if a.line_spacing != 0.0 {
            a.line_spacing
        } else {
            b.line_spacing
        };
        let justify = if !a.justify.is_empty() {
            a.justify.clone()
        } else {
            b.justify.clone()
        };
        Effects {
            font,
            color: a.color,
            font_size,
            thickness,
            bold: a.bold,
            italic: a.italic,
            line_spacing,
            justify,
            hide: a.hide,
        }
    }
}

pub struct Theme {
    items: HashMap<String, ThemeItems>,
}

impl Theme {
    fn new(content: String) -> Self {
        let mut items: HashMap<String, ThemeItems> = HashMap::new();
        let doc = SexpParser::from(content);
        let mut iter = doc.iter();
        loop {
            match iter.next() {
                Some(State::StartSymbol(name)) => {
                    if name != "theme" {
                        let next = iter.next();
                        if let Some(State::StartSymbol(element)) = next {
                            if element == "stroke" {
                                items.insert(
                                    name.to_string(),
                                    ThemeItems::Stroke(Stroke::from(&mut iter)),
                                );
                            } else if element == "effects" {
                                items.insert(
                                    name.to_string(),
                                    ThemeItems::Effects(Effects::from(&mut iter)),
                                );
                            } else if element == "color" {
                                items.insert(name.to_string(), ThemeItems::Color(color!(iter)));
                            } else {
                                todo!("symbol item not implemented: {}", name);
                            }
                        }
                    }
                }
                None => {
                    break;
                }
                _ => {}
            }
        }
        Theme { items }
    }
    pub fn kicad_2000() -> Theme {
        let content = r#"(theme
            (wire (stroke (width 0.254) (type default) (color 1 0 0 1)))
            (junction (stroke (width 0.254) (type default) (color 1 0 0 1)))
            (no_connect (stroke (width 0.254) (type default) (color 1 0 0 1)))
            (symbol (stroke (width 0.254) (type default) (color 1 0 0 1)))
            (pin (stroke (width 0.254) (type default) (color 1 0 0 1)))
            (border_stroke (stroke (width 0.254) (type default) (color 0 0 0 1)))
            (label (effects (font (size 1.27 1.27))))
            (property (effects (font (size 1.27 1.27))))
            (pin_number (effects (font (size 1.27 1.27))))
            (text (effects (font (size 5.0 5.0))))
            (border_effects (effects (font (size 2.54 2.54))))
            (subtitle_effects (effects (font (size 2.54 2.54))))
            (title_effects (effects (font (size 5.0 5.0))))
            "#;
        Theme::new(content.to_string())
    }
    pub fn stroke(&self, name: &str) -> Result<Stroke, Error> {
        if let Some(ThemeItems::Stroke(stroke)) = &self.items.get(name) {
            Ok(stroke.clone())
        } else {
            Err(Error::Theme("stroke".to_string(), name.to_string()))
        }
    }
    pub fn effects(&self, name: &str) -> Result<Effects, Error> {
        if let Some(ThemeItems::Effects(effects)) = &self.items.get(name) {
            Ok(effects.clone())
        } else {
            Err(Error::Theme("effects".to_string(), name.to_string()))
        }
    }
    pub fn color(&self, name: &str) -> Option<(f64, f64, f64, f64)> {
        if let Some(ThemeItems::Color(color)) = &self.items.get(name) {
            Some(*color)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::Theme;

    #[test]
    fn themes() {
        let theme = Theme::new(String::from(
            r#"(theme
            (no_connect (stroke (width 0.254) (type default) (color 0 0 0 0)))

            )"#,
        ));

        assert_eq!(0.254, theme.stroke("no_connect").unwrap().width);
        assert_eq!(
            String::from("default"),
            theme.stroke("no_connect").unwrap().linetype
        );
    }
}
