use simplecss::{AttributeOperator, PseudoClass, StyleSheet};

use crate::{error::Error, Color, Effects, Stroke, Style, Theme};
use sexp::el;

static BEHAVE_DARK: &str = include_str!("css/behave-dark.css");
static BLACK_WHITE: &str = include_str!("css/black-white.css");
static BLUE_GREEN_DARK: &str = include_str!("css/blue-green-dark.css");
static BLUE_TONE: &str = include_str!("css/blue-tone.css");
static EAGLE_DARK: &str = include_str!("css/eagle-dark.css");
static NORD: &str = include_str!("css/nord.css");
static SOLARIZED_DARK: &str = include_str!("css/solarized-dark.css");
static SOLARIZED_LIGHT: &str = include_str!("css/solarized-light.css");
static WDARK: &str = include_str!("css/wdark.css");
static WLIGHT: &str = include_str!("css/wlight.css");
static KICAD2020: &str = include_str!("css/kicad_2020.css");

pub struct Themer<'a> {
    theme: StyleSheet<'a>,
    css: &'a str,
}

impl<'a> Themer<'a> {
    pub fn new(theme: Theme) -> Self {
        match theme {
            Theme::BehaveDark => Self {
                theme: StyleSheet::parse(BEHAVE_DARK),
                css: BEHAVE_DARK,
            },
            Theme::BlackWhite => Self {
                theme: StyleSheet::parse(BLACK_WHITE),
                css: BLACK_WHITE,
            },
            Theme::BlueGreenDark => Self {
                theme: StyleSheet::parse(BLUE_GREEN_DARK),
                css: BLUE_GREEN_DARK,
            },
            Theme::BlueTone => Self {
                theme: StyleSheet::parse(BLUE_TONE),
                css: BLUE_TONE,
            },
            Theme::EagleDark => Self {
                theme: StyleSheet::parse(EAGLE_DARK),
                css: EAGLE_DARK,
            },
            Theme::Nord => Self {
                theme: StyleSheet::parse(NORD),
                css: NORD,
            },
            Theme::SolarizedDark => Self {
                theme: StyleSheet::parse(SOLARIZED_DARK),
                css: SOLARIZED_DARK,
            },
            Theme::SolarizedLight => Self {
                theme: StyleSheet::parse(SOLARIZED_LIGHT),
                css: SOLARIZED_LIGHT,
            },
            Theme::WDark => Self {
                theme: StyleSheet::parse(WDARK),
                css: WDARK,
            },
            Theme::WLight => Self {
                theme: StyleSheet::parse(WLIGHT),
                css: WLIGHT,
            },
            Theme::Kicad2020 => Self {
                theme: StyleSheet::parse(KICAD2020),
                css: KICAD2020,
            },
        }
    }
    pub fn css(&self) -> &'a str {
        self.css
    }

    pub fn get_stroke(&self, mut stroke: Stroke, style: &[Style]) -> Stroke {
        if stroke.linetype.is_empty() || stroke.linetype == "default" {
            //TODO get linetype
        }
        if matches!(stroke.linecolor, Color::None) { //TODO handle also default color in kicad schema files
            for style in style {
                match style {
                    Style::Fill(_) => {},
                    _ => {
                        for rule in &self.theme.rules {
                            let style = style.to_string();
                            let root = Node(&style);
                            if rule.selector.matches(&root) {
                                for decl in &rule.declarations {
                                    if decl.name == "stroke" {
                                        stroke.linecolor = self.parse_rgb(decl.value).unwrap().into();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if stroke.fillcolor == Color::None { //handle also default value in Kicad schema
            for style in style {
                if let Style::Fill(style) = style {
                    for rule in &self.theme.rules {
                        let style = style.to_string();
                        let root = Node(&style);
                        if rule.selector.matches(&root) {
                            for decl in &rule.declarations {
                                if decl.name == "fill" {
                                    stroke.fillcolor = self.parse_rgb(decl.value).unwrap().into();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        if stroke.linewidth == 0.0 {
            if let Some(width) = self.select(style, "stroke-width") {
                stroke.linewidth = width.parse::<f64>().unwrap_or(1.0);
            }
        }
        stroke
    }

    pub fn get_effects(&self, mut effects: Effects, style: &[Style]) -> Effects {
        let mut font_family: Vec<String> = Vec::new();
        let mut font_size = String::new();
        if let Some(font) = self.select(style, "font") {
            for token in font.split(' ') {
                if token.trim().ends_with("pt") {
                    font_size = token.strip_suffix("pt").unwrap().to_string();
                } else {
                    let token = if token.ends_with(',') {
                        token.strip_suffix(',').unwrap()
                    } else { token };
                    font_family.push(token.trim().to_string());
                }
            }
        }
    
        effects.font_face = font_family.first().unwrap().to_string();
        effects.font_size = vec![font_size.parse::<f64>().unwrap(), font_size.parse::<f64>().unwrap()];

        let mut font_color = Color::Rgb(0, 0, 0);
        if let Some(color) = self.select(style, "fill") {
            font_color = self.parse_rgb(color).unwrap().into();
        }

        effects.font_color = font_color;

        effects
    }

    fn select(&self, styles: &[Style], selector: &str) -> Option<&str> {
        for style in styles {
            for rule in &self.theme.rules {
                let style = style.to_string();
                let root = Node(&style);
                if rule.selector.matches(&root) {
                    for decl in &rule.declarations {
                        if decl.name == selector {
                            return Some(decl.value);
                        }
                    }
                }
            }
        }
        None
    }





    pub fn stroke(&self, styles: &Vec<Style>) -> (f64, f64, f64, f64) {
        for style in styles {
            for rule in &self.theme.rules {
                let style = style.to_string();
                let root = Node(&style);
                if rule.selector.matches(&root) {
                    for decl in &rule.declarations {
                        if decl.name == el::STROKE {
                            return self.parse_color(decl.value).unwrap();
                        }
                    }
                }
            }
        }
        println!(
            "no color defined, {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
        (1.0, 0.0, 0.0, 1.0)
        /* TODO panic!(
            "no color defined for: {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        ); */
    }
    /* pub fn font_size(&self, defined: Option<f64>, styles: &Vec<Style>) -> f64 {
        if let Some(defined) = defined {
            if defined != 0.0 {
                return defined;
            }
        }
        for rule in &self.theme.rules {
            for style in styles {
                let style = style.to_string();
                let root = Node(&style);
                if rule.selector.matches(&root) {
                    for decl in &rule.declarations {
                        if decl.name == "font" {
                            for token in decl.value.split(' ') {
                                if token.ends_with("pt") {
                                    return token
                                        .strip_suffix("pt")
                                        .unwrap()
                                        .trim()
                                        .parse::<f64>()
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
        panic!(
            "no font size defined for: {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
    } */
    /* pub fn fill(&self, styles: &Vec<Style>) -> Option<(f64, f64, f64, f64)> {
        for style in styles {
            for rule in &self.theme.rules {
                let style = style.to_string();
                let root = Node(&style);
                if rule.selector.matches(&root) {
                    for decl in &rule.declarations {
                        if decl.name == "fill" {
                            return Some(self.parse_color(decl.value).unwrap());
                        }
                    }
                }
            }
        }
        None
    } */
    fn parse_rgb(&self, color: &str) -> Result<Vec<u16>, Error> {
        let content = if color.starts_with("rgba") {
            color
                .strip_prefix("rgba(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
        } else {
            color
                .strip_prefix("rgb(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
        };
        Ok(content
            .split(',')
            .map(|c| c.trim().parse::<u16>().unwrap())
            .collect())
    }
    fn parse_color(&self, color: &str) -> Result<(f64, f64, f64, f64), Error> {
        let content = if color.starts_with("rgba") {
            color
                .strip_prefix("rgba(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
        } else {
            color
                .strip_prefix("rgb(")
                .unwrap()
                .strip_suffix(')')
                .unwrap()
        };
        let res: Vec<f64> = content
            .split(',')
            .map(|c| c.trim().parse::<f64>().unwrap() / 255.0)
            .collect();
        Ok((res[0], res[1], res[2], 1.0))
    }
    pub fn hex_color(&self, color: (f64, f64, f64, f64)) -> String {
        format!(
            "#{:x}{:x}{:x}",
            (color.0 * 255.0) as i16,
            (color.1 * 255.0) as i16,
            (color.2 * 255.0) as i16
        )
    }
}

struct Node<'a>(&'a str);
impl simplecss::Element for Node<'_> {
    fn parent_element(&self) -> Option<Self> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    fn has_local_name(&self, _local_name: &str) -> bool {
        false
    }

    fn attribute_matches(&self, local_name: &str, operator: AttributeOperator) -> bool {
        if local_name == "class" {
            operator.matches(self.0)
        } else {
            false
        }
    }

    fn pseudo_class_matches(&self, _class: PseudoClass) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {

    use crate::{Color, Stroke};

    use super::super::{FillType, Style};

    use super::{Theme, Themer};

    #[test]
    fn parse_color() {
        let themer = Themer::new(Theme::Kicad2020);
        assert_eq!(
            (0.0 / 255.0, 150.0 / 255.0, 0.0 / 255.0, 255.0 / 255.0),
            themer.parse_color("rgb(0, 150, 0)").unwrap()
        );
    }
    #[test]
    fn themes() {
        let themer = Themer::new(Theme::Kicad2020);
        assert_eq!(
            (0.0, 150.0 / 255.0, 0.0, 1.0),
            themer.stroke(&vec![Style::Wire])
        );
    }
    #[test]
    fn hex_color() {
        let themer = Themer::new(Theme::Kicad2020);
        assert_eq!(
            (77.0 / 255.0, 127.0 / 255.0, 196.0 / 255.0, 1.0),
            themer.stroke(&vec![Style::BCu])
        );
        assert_eq!(
            "#4d7fc4",
            themer.hex_color(themer.stroke(&vec![Style::BCu]))
        );
    }

    //TODO new tests
    #[test]
    fn test_parse_rgb() {
        let themer = Themer::new(Theme::Kicad2020);
        assert_eq!(vec![0, 150, 0], themer.parse_rgb("rgb(0, 150, 0)").unwrap());


    }
    #[test]
    fn stroke_wire() {
        let themer = Themer::new(Theme::Kicad2020);
        let stroke = themer.get_stroke(Stroke::new(), &[Style::Wire]);
        assert_eq!(0.25, stroke.linewidth);
        assert_eq!(String::from("default"), stroke.linetype);
        assert_eq!(Color::Rgb(0, 150, 0), stroke.linecolor);
        assert_eq!(Color::None, stroke.fillcolor);
    }
    #[test]
    fn stroke_polygon() {
        let themer = Themer::new(Theme::Kicad2020);
        let stroke = themer.get_stroke(Stroke::new(), &[Style::Outline,  Style::Fill(FillType::Background)]);
        assert_eq!(0.35, stroke.linewidth);
        assert_eq!(String::from("default"), stroke.linetype);
        assert_eq!(Color::Rgb(132, 0, 0), stroke.linecolor);
        assert_eq!(Color::Rgb(255, 255, 194), stroke.fillcolor);
    }
}
