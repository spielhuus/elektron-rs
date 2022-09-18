use super::cairo_plotter::{Circle, Line, LineCap, PlotItem, Text};
use super::theme::{Theme, Themer, ThemerMerge};
use crate::plot::text;
use crate::sexp::model::{PcbElements, Stroke};
use crate::sexp::pcb::Pcb;
use crate::sexp::{Shape, Transform};
use ndarray::arr2;

macro_rules! theme {
    ($self:expr, $element:expr) => {
        Themer::get(
            &Stroke {
                width: $element.width,
                linetype: "default".to_string(),
                color: (0.0, 0.0, 0.0, 0.0),
                filltype: String::new(),
            },
            &$self.theme.stroke(&$element.layer).unwrap(),
        )
    };
}

pub struct PcbPlot<'a, I> {
    iter: I,
    theme: Theme,
    border: bool,
    pcb: &'a Pcb,
}

impl<'a, I> Iterator for PcbPlot<'a, I>
where
    I: Iterator<Item = &'a PcbElements>,
{
    type Item = Vec<PlotItem>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(PcbElements::Line(line)) => {
                    let stroke = theme!(self, line);
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                arr2(&[[line.start[0], line.start[1]], [line.end[0], line.end[1]]]),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Butt,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(PcbElements::Segment(segment)) => {
                    let stroke = theme!(self, segment);
                    return Some(vec![
                        (PlotItem::Line(
                            10,
                            Line::new(
                                arr2(&[
                                    [segment.start[0], segment.start[1]],
                                    [segment.end[0], segment.end[1]],
                                ]),
                                stroke.width,
                                stroke.linetype.clone(),
                                LineCap::Round,
                                stroke.color,
                            ),
                        )),
                    ]);
                }
                Some(PcbElements::Footprint(footprint)) => {
                    let mut graphics = Vec::new();
                    for graphic in &footprint.graphics {
                        match graphic {
                            crate::sexp::model::Graphics::FpText(text) => {
                                if !text.hidden {
                                    let effects = Themer::get(
                                        &text.effects,
                                        &self.theme.effects("footprint").unwrap(),
                                    );
                                    graphics.push(text!(
                                        Shape::transform(footprint, &text.at),
                                        text.angle,
                                        text.value.clone(),
                                        effects
                                    ));
                                }
                            }
                            crate::sexp::model::Graphics::FpLine(line) => {
                                let stroke = theme!(self, line);
                                graphics.push(PlotItem::Line(
                                    10,
                                    Line::new(
                                        Shape::transform(
                                            footprint,
                                            &arr2(&[
                                                [line.start[0], line.start[1]],
                                                [line.end[0], line.end[1]],
                                            ]),
                                        ),
                                        stroke.width,
                                        stroke.linetype.clone(),
                                        LineCap::Butt,
                                        stroke.color,
                                    ),
                                ));
                            }
                            crate::sexp::model::Graphics::FpCircle(circle) => {
                                let stroke = theme!(self, circle);
                                graphics.push(PlotItem::Circle(
                                    1,
                                    Circle::new(
                                        Shape::transform(footprint, &circle.center),
                                        ((circle.end[0] - circle.center[0]).powf(2.0)
                                            + (circle.end[1] - circle.center[1]).powf(2.0))
                                        .sqrt(),
                                        stroke.width,
                                        stroke.linetype,
                                        stroke.color,
                                        self.theme.color(&stroke.filltype),
                                    ),
                                ));
                            }
                            crate::sexp::model::Graphics::FpArc(_) => {}
                        }
                    }
                    return Some(graphics);
                }
                None => {
                    return None;
                }
                _ => {}
            }
        }
    }
}

impl<'a, I> PcbPlot<'a, I> {
    pub fn new(iter: I, pcb: &'a Pcb, theme: Theme, border: bool) -> Self {
        Self {
            iter,
            pcb,
            border,
            theme,
        }
    }
}

pub trait PcbPlotIterator<T>: Iterator<Item = T> + Sized {
    fn plot(self, pcb: &'_ Pcb, theme: Theme, border: bool) -> PcbPlot<Self> {
        PcbPlot::new(self, pcb, theme, border)
    }
}
impl<T, I: Iterator<Item = T>> PcbPlotIterator<T> for I {}

#[cfg(test)]
mod tests {
    /* use crate::sexp::Schema;
    use std::path::Path;

    #[test]
    fn bom() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        doc.plot("/tmp/summe.svg", 1.0, true, "kicad_2000").unwrap();
        assert!(Path::new("/tmp/summe.svg").exists());
        assert!(Path::new("/tmp/summe.svg").metadata().unwrap().len() > 0);
    } */
}
