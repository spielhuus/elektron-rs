use super::plotter::{text, Circle, Line, PlotItem, Text, Theme};
use super::{
    plotter::{FillType, ItemPlot, Style},
    theme::Themer,
};
use crate::plot::plotter::LineCap;
use crate::sexp;
use crate::sexp::{Effects, Pcb, PcbElements, Shape, TitleBlock, Transform};
use ndarray::arr2;

/* macro_rules! theme {
    ($self:expr, $element:expr) => {
        Themer::get(
            &Stroke {
                width: $element.width,
                linetype: "default".to_string(),
                color: (0.0, 0.0, 0.0, 0.0),
            },
            &$self.theme.stroke(&$element.layer).unwrap(),
        )
    };
} */

pub struct GerberPlot<'a, I> {
    iter: I,
    border: bool,
    title_block: &'a Option<TitleBlock>,
    paper_size: (f64, f64),
    pcb: &'a Pcb,
}

impl<'a, I> Iterator for GerberPlot<'a, I>
where
    I: Iterator<Item = &'a PcbElements>,
{
    type Item = Vec<PlotItem>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            /* match self.iter.next() {
                Some(PcbElements::Line(line)) => {
                    return self.item(line);
                }
                Some(PcbElements::Segment(segment)) => {
                    return self.item(segment);
                }
                Some(PcbElements::Footprint(footprint)) => {
                    return self.item(footprint);
                }
                Some(PcbElements::Text(text)) => {
                    return self.item(text);
                }
                Some(PcbElements::Zone(zone)) => {
                    //TODO: return self.item(footprint);
                }
                Some(PcbElements::Via(via)) => {
                    return self.item(via);
                }
                Some(PcbElements::GrPoly(poly)) => {
                    return self.item(poly);
                }
                Some(PcbElements::GrCircle(circle)) => {
                     return self.item(circle);
                }
                None => {
                    return None;
                }
                Some(e) => {
                    println!("unknown element {:?}", e);
                }
            } */
        }
    }
}

impl<'a, I> GerberPlot<'a, I> {
    pub fn new(
        iter: I,
        pcb: &'a Pcb,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
    ) -> Self {
        Self {
            iter,
            pcb,
            title_block,
            paper_size,
            border,
        }
    }
}

pub trait GerberPlotIterator<T>: Iterator<Item = T> + Sized {
    fn plot<'a>(
        self,
        pcb: &'a Pcb,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
    ) -> GerberPlot<'a, Self> {
        GerberPlot::new(self, pcb, title_block, paper_size, border)
    }
}

impl<T, I: Iterator<Item = T>> GerberPlotIterator<T> for I {}

/* impl<T> ItemPlot<sexp::Footprint> for T {
    fn item(&self, item: &sexp::Footprint) -> Option<Vec<PlotItem>> {
        let mut graphics = Vec::new();
        for graphic in &item.graphics {
            match graphic {
                sexp::Graphics::FpText(text) => {
                    if !text.hidden {
                        let angle = if let Some(angle) = text.angle {
                            angle
                        } else {
                            0.0
                        };
                        graphics.push(text!(
                            Shape::transform(item, &text.at),
                            angle,
                            text.value.clone(),
                            Effects::new(),    //TODO:
                            vec![Style::Text, Style::Layer(text.layer.replace('.', "_"))]
                        ));
                    }
                }
                sexp::Graphics::FpLine(line) => {
                    graphics.push(PlotItem::Line(
                        10,
                        Line::new(
                            Shape::transform(
                                item,
                                &arr2(&[
                                    [line.start[0], line.start[1]],
                                    [line.end[0], line.end[1]],
                                ]),
                            ),
                            Some(line.width),
                            None,
                            None,
                            None,
                            vec![Style::Layer(line.layer.replace('.', "_"))],
                        ),
                    ));
                }
                sexp::Graphics::FpCircle(circle) => {
                    // let stroke = theme!(self, circle);
                    graphics.push(PlotItem::Circle(
                        1,
                        Circle::new(
                            Shape::transform(item, &circle.center),
                            ((circle.end[0] - circle.center[0]).powf(2.0)
                                + (circle.end[1] - circle.center[1]).powf(2.0))
                            .sqrt(),
                            Some(circle.width),
                            None,
                            None,
                            vec![Style::Layer(circle.layer.replace('.', "_"))],
                        ),
                    ));
                }
                sexp::Graphics::FpArc(_) => {}
            }
        }
        return Some(graphics);
    }
}

impl<T> ItemPlot<sexp::GrText> for T {
    fn item(&self, item: &sexp::GrText) -> Option<Vec<PlotItem>> {
        /*TODO:  if !item.hidden {
            let angle = if let Some(angle) = text.angle {
                angle
            } else {
                0.0
            }; */
            Some(vec![text!(
                item.at.clone(),
                item.angle,
                item.text.clone(),
                Effects::new(),    //TODO:
                vec![Style::Text, Style::Layer(item.layer.replace('.', "_"))]
            )])
        // } else { vec![] }
    }
}

impl<T> ItemPlot<sexp::Segment> for T {
    fn item(&self, item: &sexp::Segment) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[item.start[0], item.start[1]], [item.end[0], item.end[1]]]),
                    Some(item.width),
                    None,
                    None,
                    Some(LineCap::Round),
                    vec![Style::Segment, Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrLine> for T {
    fn item(&self, item: &sexp::GrLine) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[item.start[0], item.start[1]], [item.end[0], item.end[1]]]),
                    Some(item.width),
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrPoly> for T {
    fn item(&self, item: &sexp::GrPoly) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    item.pts.clone(),
                    Some(item.width),
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrCircle> for T {
    fn item(&self, item: &sexp::GrCircle) -> Option<Vec<PlotItem>> {
        let radius = ((item.center[0] -item.end[0]).powf(2.0) + (item.center[1] - item.end[1]).powf(2.0)).sqrt().abs();
        Some(vec![
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.center.clone(),
                    radius,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::Via> for T {
    fn item(&self, item: &sexp::Via) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.at.clone(),
                    item.size,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layers[0].replace('.', "_"))],
                ),
            )),
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.at.clone(),
                    item.drill,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layers[0].replace('.', "_"))],
                ),
            )),
        ])
    }
} */

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
