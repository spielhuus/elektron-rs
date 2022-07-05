use ::std::fmt::{Display, Formatter, Result as FmtResult};
use super::sexp::{Color, Effects, Justify, LineType, Stroke};
use crate::sexp::FillType;

pub enum StyleContext {
    SchemaSymbol,
    SchemaWire,
    SchemaProperty,
}

pub struct Style {
    property_effects: Effects,
    label_effects: Effects,
    schema_wire: Stroke,
    schema_symbol: Stroke,
    graphic: Stroke,
    fill_outline: Color,
    fill_background: Color,
}


impl Display for Style {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&format!("Style: {:?} {:?} {:?} {:?} {:?} {:?} {:?}", self.property_effects, self.label_effects, self.schema_wire, self.schema_symbol, self.graphic, self.fill_outline, self.fill_background))
    }
}

impl Style {
    pub fn new() -> Style {
        Style {
            property_effects: Effects::new(
                "monospace".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                1.27,
                1.0,
                false,
                false,
                1.0,
                Justify::Center,
                false,
            ),
            label_effects: Effects::new(
                "monospace".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                1.27,
                1.0,
                false,
                false,
                1.0,
                Justify::Center,
                false,
            ),
            schema_wire: Stroke {
                width: 0.25,
                line_type: LineType::Default,
                color: Color {
                    r: 0.0,
                    g: 150.0 / 255.0,
                    b: 0.0,
                    a: 1.0,
                },
                fill: FillType::None,
            },
            schema_symbol: Stroke {
                width: 0.25,
                line_type: LineType::Default,
                color: Color {
                    r: 0.0,
                    g: 150.0 / 255.0,
                    b: 0.0,
                    a: 1.0,
                },
                fill: FillType::None,
            },
            graphic: Stroke {
                width: 0.2,
                line_type: LineType::Default,
                color: Color {
                    r: 132.0 / 255.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                fill: FillType::None,
            },
            fill_outline: Color {
                r: 132.0 / 255.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fill_background: Color {
                r: 1.0,
                g: 1.0,
                b: 194.0 / 255.0,
                a: 1.0,
            },
        }
    }

    pub fn stroke(&self, context: &StyleContext) -> Stroke {
        match context {
            StyleContext::SchemaWire => self.schema_wire.clone(),
            StyleContext::SchemaSymbol => self.schema_symbol.clone(),
            _ => self.schema_symbol.clone(), //TODO
        }
    }
    pub fn effects(&self, context: &StyleContext) -> Effects {
        match context {
            StyleContext::SchemaWire => self.label_effects.clone(),
            StyleContext::SchemaSymbol => self.label_effects.clone(),
            _ => self.label_effects.clone(),
        }
    }
    pub fn color(&self, fill: &FillType) -> Option<Color> {
        match fill {
            FillType::Outline => Option::from(self.fill_outline.clone()),
            FillType::Background => Option::from(self.fill_background.clone()),
            _ => None,
        }
    }
}
