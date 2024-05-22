use std::collections::HashMap;

use sexp::schema::Color;

pub enum Themes {
    Kicad2020,
}

pub enum Theme {
    Kicad2020(Kicad2020),
}

impl Theme {
    pub fn from(theme: Themes) -> Self {
        match theme {
            Themes::Kicad2020 => Self::Kicad2020(Kicad2020::new()),
        }
    }
    pub fn linewidth(&self, style: &Style) -> f32 {
        match self {
            Self::Kicad2020(theme) => theme.linewidth(style),
        }
    }
    pub fn linecolor(&self, style: &Style) -> Color {
        match self {
            Self::Kicad2020(theme) => theme.linecolor(style),
        }
    }
    pub fn fontcolor(&self, style: &Style) -> Color {
        match self {
            Self::Kicad2020(theme) => theme.fontcolor(style),
        }
    }
    pub fn fontsize(&self, style: &Style) -> f32 {
        match self {
            Self::Kicad2020(theme) => theme.fontsize,
        }
    }
    pub fn font_family(&self, style: &Style) -> String {
        match self {
            Self::Kicad2020(theme) => theme.font_family.to_string(),
        }
    }
    pub fn stroke(&self, stroke: Option<sexp::schema::Stroke>, style: &Style) -> Stroke {
        Stroke {
            linewidth: if let Some(ref stroke) = stroke {
                if stroke.width().is_some() && stroke.width().unwrap() > 0.0 {
                    stroke.width().unwrap()
                } else {
                    self.linewidth(style)
                }
            } else {
                self.linewidth(style)
            },
            linecolor: if let Some(stroke) = stroke {
                if stroke.color().is_some() && stroke.color().unwrap() != Color::new(0, 0, 0,  0.0 ) {
                    stroke.color().unwrap()
                } else {
                    self.linecolor(style)
                }
            } else {
                self.linecolor(style)
            },
        }
    }

    pub fn effects(&self, effects: Option<sexp::schema::Effects>, style: &Style) -> Effects {
        //if let Some(effects) = effects {
        //Effects {
        //    color: if let Some(colors) = effects.color() {
        //
        //                   colors
        //            } else {
        //                   Color::default()
        //            },
        //    size: if let Some(effects) = effects {
        //        if effects.size() > 0.0 {
        //            effects.size()
        //        } else {
        //            self.fontsize(style)
        //        }
        //    } else {
        //        self.fontsize(style)
        //    }, 
        //    font: String::new(),
        //}
        //} else {
            Effects {
                color: self.fontcolor(style),
                size: self.fontsize(style),
                font: self.font_family(style),
            }
        //}
    }
}

pub trait ThemeImpl {
    fn linewidth(&self, style: &Style) -> f32;
    fn linecolor(&self, style: &Style) -> Color;
    fn fontcolor(&self, style: &Style) -> Color;
    fn fontsize(&self, style: &Style) -> f32;
    fn font_family(&self, style: &Style) -> String;
}

#[derive(Clone, Debug, Default)]
pub struct Stroke {
    pub linewidth: f32,
    pub linecolor: Color,
    //pub linetype: String,
    //pub linecolor: Color,
    //pub fillcolor: Color,
}

impl Stroke {
    pub fn new() -> Self {
        Self {
            linewidth: 0.25,
            linecolor: Color::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Effects {
    pub color: Color,
    pub size: f32,
    pub font: String,
    //pub linetype: String,
    //pub linecolor: Color,
    //pub fillcolor: Color,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            size: 1.25,
            color: Color::default(),
            font: String::from("osifont"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Style {
    Wire,
    Property,
    Todo,
}

//pub fn get(stroke: Option<sexp::schema::Stroke>,sStyle: Style) -> Stroke {
//    Stroke {
//        linewidth: 0.25,
//        linecolor: Color::new(255, 0, 0, 1.0),
//        //linetype: String,
//        //linecolor: Color,
//        //fillcolor: Color,
//    }
//    //if let Some(stroke) = stroke {
//    //    Stroke {
//    //        linewidth: 1.25,
//    //    }
//    //} else {
//    //}
//}

struct Kicad2020 {
    colors: HashMap<Style, Color>,
    linewidth: f32,
    fontsize: f32,
    font_family: String,
    //linetype: String::from("default"),
    //linecolor: Color::None,
    //fillcolor: Color::None,
}

impl Kicad2020 {
    pub fn new() -> Self {

        let mut colors = HashMap::new();
        colors.insert(Style::Wire, Color::new(0, 255, 0, 1.0));
        colors.insert(Style::Property, Color::new(0, 255, 0, 1.0));

        Self {
            colors,
            linewidth: 0.25,
            fontsize: 1.25,
            font_family: String::from("osifont"),
        }
    }
}

impl ThemeImpl for Kicad2020 {
    fn linewidth(&self, style: &Style) -> f32 {
        0.25
    }
    fn linecolor(&self, style: &Style) -> Color {
        *self.colors.get(&style).unwrap_or(&Color::default())
    }
    fn fontcolor(&self, style: &Style) -> Color {
        *self.colors.get(&style).unwrap_or(&Color::default())
    }
    fn fontsize(&self, style: &Style) -> f32 {
        self.fontsize
    }
    fn font_family(&self, style: &Style) -> String {
        self.font_family.to_string()
    }
}
