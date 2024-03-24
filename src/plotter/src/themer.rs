use simplecss::{AttributeOperator, PseudoClass, StyleSheet};

use crate::{error::Error, Style, Theme};
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
        /* panic!(
            "no color defined for: {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        ); */
    }
    pub fn stroke_width(&self, defined: Option<f64>, styles: &Vec<Style>) -> f64 {
        if let Some(defined) = defined {
            if defined > 0.0 {
                return defined;
            }
        }
        for rule in &self.theme.rules {
            for style in styles {
                let style = style.to_string();
                let root = Node(&style);
                if rule.selector.matches(&root) {
                    for decl in &rule.declarations {
                        if decl.name == "stroke-width" {
                            return decl.value.parse::<f64>().unwrap();
                        }
                    }
                }
            }
        }
        println!(
            "no stroke width defined for: {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
        1.0
    }
    pub fn font(&self, defined: Option<String>, styles: &Vec<Style>) -> String {
        if let Some(defined) = defined {
            if defined.is_empty() {
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
                                if !token.ends_with("pt") && !token.ends_with("px") {
                                    if token.ends_with(',') {
                                        return token.strip_suffix(',').unwrap().to_string();
                                    } else {
                                        return token.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        panic!(
            "no font defined for: {:?}",
            styles
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
        );
    }
    pub fn font_size(&self, defined: Option<f64>, styles: &Vec<Style>) -> f64 {
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
    }
    pub fn fill(&self, styles: &Vec<Style>) -> Option<(f64, f64, f64, f64)> {
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
        assert_eq!(
            (1.0, 1.0, 194.0 / 255.0, 1.0),
            themer
                .fill(&vec![Style::Fill(FillType::Background)])
                .unwrap()
        );
        assert_eq!(0.25, themer.stroke_width(None, &vec![Style::Wire]));
        assert_eq!(
            String::from("osifont"),
            themer.font(None, &vec![Style::Property])
        );
        assert_eq!(1.25, themer.font_size(None, &vec![Style::Property]));
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
}
