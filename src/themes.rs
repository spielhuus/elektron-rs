use crate::sexp::{Sexp, Color, Effects, Justify, LineType, Stroke, FillType};
use crate::sexp::get::{get, Get};
use crate::sexp::test::Test;
use crate::Error;

pub enum StyleContext {
    SchemaSymbol,
    SchemaWire,
    SchemaProperty,
    SchemaPin,
    SchemaPinNumber,
    SchemaJunction,
}

pub struct Style {
    property_effects: Effects,
    label_effects: Effects,
    schema_wire: Stroke,
    schema_symbol: Stroke,
    graphic: Stroke,
    fill_outline: Color,
    fill_background: Color,
    schema_border: Stroke,
    schema_title_effects: Effects,
    schema_subtitle_effects: Effects,
    schema_effects: Effects,
    pin: Stroke,
    pin_number_effects: Effects,
    junction: Stroke,
}

/// Access the nodes and values.
pub trait StyleTypes<S, T> {
    fn style(&self, node: &Sexp, index: S, context: StyleContext) -> Result<T, Error>;
}

impl StyleTypes<&str, Effects> for Style {
    fn style(&self, node: &Sexp, key: &str, context: StyleContext) -> Result<Effects, Error> {
    
        let overwrite = true;

        let style_effects = self.effects(&context);
        if !node.contains(key) {
            return Ok(style_effects);
        }
        let effect: Effects = get!(node, key).unwrap();

        let font = if effect.font != "" && !overwrite {
            effect.font.clone()
        } else {
            style_effects.font.clone()
        };
        let size = if effect.size != 0.0 && !overwrite {
            effect.size.clone()
        } else {
            style_effects.size.clone()
        };
        let thickness = if effect.thickness != 0.0 && !overwrite {
            effect.thickness.clone()
        } else {
            style_effects.thickness.clone()
        };
        let line_spacing = if effect.line_spacing != 0.0 && !overwrite {
            effect.line_spacing
        } else {
            style_effects.line_spacing
        };
        let justify = if !effect.justify.is_empty() {
            effect.justify
        } else {
            style_effects.justify.clone()
        };
        Ok(Effects::new(
            font,
            effect.color.clone(),
            size,
            thickness,
            effect.bold,
            effect.italic,
            line_spacing,
            justify,
            effect.hide,
        ))
    }
}

impl StyleTypes<&str, Stroke> for Style {
    fn style(
        &self,
        node: &Sexp,
        key: &str,
        context: StyleContext,
    ) -> Result<Stroke, Error> {

        let stroke: Result<Stroke, _> = get!(node, key);
        let style_stroke = self.stroke(&context);

        match stroke {
            Ok(stroke) => Ok(Stroke {
                width: if stroke.width != 0.0 {
                    stroke.width * 1.4
                } else {
                    style_stroke.width
                },
                line_type: stroke.line_type,
                color: if stroke.color
                    != (Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }) {
                    stroke.color.clone()
                } else {
                    style_stroke.color
                },
                fill: stroke.fill,
            }),
            _ => Ok(style_stroke.clone()),
        }
    }
}

impl Style {
    pub fn new() -> Style {
        Style {
            property_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                2.0,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Center],
                false,
            ),
            label_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                2.0,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Center],
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
            junction: Stroke {
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
                    r: 132.0 / 255.0,
                    g: 0.0,
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
            schema_border: Stroke {
                width: 0.18,
                line_type: LineType::Default,
                color: Color {
                    r: 132.0 / 255.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                fill: FillType::None,
            },
            schema_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 132.0 / 255.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                2.5,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Left],
                false,
            ),
            schema_title_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                5.0,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Left],
                false,
            ),
            schema_subtitle_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                2.5,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Left],
                false,
            ),
            pin: Stroke {
                width: 0.25,
                line_type: LineType::Default,
                color: Color {
                    r: 132.0 / 255.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                fill: FillType::None,
            },
            pin_number_effects: Effects::new(
                "osifont".to_string(),
                Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
                1.0,
                1.0,
                false,
                false,
                1.0,
                vec![Justify::Center],
                false,
            ),
        }
    }

    pub fn stroke(&self, context: &StyleContext) -> Stroke {
        match context {
            StyleContext::SchemaWire => self.schema_wire.clone(),
            StyleContext::SchemaSymbol => self.schema_symbol.clone(),
            StyleContext::SchemaPin => self.pin.clone(),
            StyleContext::SchemaJunction => self.junction.clone(),
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
    pub fn schema_border(&self) -> Stroke {
        return self.schema_border.clone();
    }
    pub fn schema_effects(&self) -> Effects {
        return self.schema_effects.clone();
    }
    pub fn schema_title_effects(&self) -> Effects {
        return self.schema_title_effects.clone();
    }
    pub fn schema_subtitle_effects(&self) -> Effects {
        return self.schema_subtitle_effects.clone();
    }
}
