use crate::sexp::{self, normalize_angle, PinGraphicalStyle, PinOrientation, PinType};
use crate::spice::{Netlist, Point};
use ndarray::{arr1, arr2, Array1, Array2};

use super::plotter::{
    text, Arc, Circle, FillType, ItemPlot, Line, PlotItem, Polyline, Rectangle, Style, Text,
};
use crate::sexp::{
    CalcArc, Effects, Graph, MathUtils, Pin, Schema, SchemaElement, Shape, TitleBlock, Transform,
};

const BORDER_RASTER: i32 = 60;
const PIN_NUMER_OFFSET: f64 = 0.6;

fn arc_tangente(dy: f64, dx: f64) -> f64 {
    //TODO:
    /* gcc is surprisingly smart in optimizing these conditions in
    a tree! */

    /* if( dx == 0 && dy == 0 )
        return 0;

    if( dy == 0 )
    {
        if( dx >= 0 )
            return 0;
        else
            return -1800;
    }

    if( dx == 0 )
    {
        if( dy >= 0 )
            return 900;
        else
            return -900;
    }

    if( dx == dy )
    {
        if( dx >= 0 )
            return 450;
        else
            return -1800 + 450;
    }

    if( dx == -dy )
    {
        if( dx >= 0 )
            return -450;
        else
            return 1800 - 450;
    } */

    // Of course dy and dx are treated as double
    // return RAD2DECIDEG( std::atan2( (double) dy, (double) dx ) );
    dy.atan2(dx).to_degrees()
}

fn calc_angle(start: Array1<f64>, mid: Array1<f64>, end: Array1<f64>) -> (u32, u32) {
    let centerStartVector = start - mid.clone();
    let centerEndVector = end - mid.clone();

    let start_angle = arc_tangente(centerStartVector[1], centerStartVector[0]);
    let end_angle = arc_tangente(centerEndVector[1], centerEndVector[0]);

    /* if( ( aEndAngle - aStartAngle ) > 1800 )
        aEndAngle -= 3600;
    else if( ( aEndAngle - aStartAngle ) <= -1800 )
        aEndAngle += 3600;

    while( ( aEndAngle - aStartAngle ) >= 1800 )
    {
        aEndAngle--;
        aStartAngle++;
    }

    while( ( aStartAngle - aEndAngle ) >= 1800 )
    {
        aEndAngle++;
        aStartAngle--;
    } */

    /* NORMALIZE_ANGLE_POS( aStartAngle );

    if( !IsMoving() )
        NORMALIZE_ANGLE_POS( aEndAngle ); */

    (start_angle as u32, end_angle as u32)
}

pub struct SchemaPlot<'a, I> {
    iter: I,
    schema: &'a Schema,
    border: bool,
    title_block: &'a Option<TitleBlock>,
    paper_size: (f64, f64),
    netlist: &'a Option<Netlist<'a>>,
}

impl<'a, I> Iterator for SchemaPlot<'a, I>
where
    I: Iterator<Item = &'a SchemaElement>,
{
    type Item = Vec<PlotItem>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.border {
            self.border = false; //dont draw the border twice
            if let Some(title_block) = self.title_block {
                return self.item(title_block);
            }
        }
        match self.iter.next() {
            Some(SchemaElement::Sheet(sheet)) => self.item(sheet),
            Some(SchemaElement::Wire(wire)) => self.item(wire),
            Some(SchemaElement::Polyline(line)) => self.item(line),
            Some(SchemaElement::Bus(bus)) => self.item(bus),
            Some(SchemaElement::BusEntry(bus)) => self.item(bus),
            Some(SchemaElement::Text(text)) => self.item(text),
            Some(SchemaElement::NoConnect(no_connect)) => self.item(no_connect),
            Some(SchemaElement::Junction(junction)) => self.item(junction),
            Some(SchemaElement::Label(label)) => self.item(label),
            Some(SchemaElement::GlobalLabel(label)) => self.item(label),
            Some(SchemaElement::HierarchicalLabel(label)) => self.item(label),
            Some(SchemaElement::Symbol(symbol)) => self.item(symbol),
            None => None,
        }
    }
}

/// get the pin position
/// returns an array containing the number of pins:
///   3
/// 0   2
///   1
fn pin_position(symbol: &sexp::Symbol, pin: &Pin) -> Vec<usize> {
    let mut position: Vec<usize> = vec![0; 4];
    let symbol_shift: usize = (symbol.angle / 90.0).round() as usize;

    let lib_pos: usize = (pin.angle / 90.0).round() as usize;
    position[lib_pos] += 1;

    position.rotate_right(symbol_shift);
    if let Some(mirror) = &symbol.mirror {
        if mirror == "x" {
            position = vec![position[0], position[3], position[2], position[1]];
        } else if mirror == "y" {
            position = vec![position[2], position[1], position[0], position[3]];
        }
    }
    position
}

impl<'a, I> SchemaPlot<'a, I> {
    pub fn new(
        iter: I,
        schema: &'a Schema,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
        netlist: &'a Option<Netlist<'a>>,
    ) -> Self {
        Self {
            iter,
            border,
            schema,
            title_block,
            paper_size,
            netlist,
        }
    }
}

pub trait PlotIterator<T>: Iterator<Item = T> + Sized {
    fn plot<'a>(
        self,
        schema: &'a Schema,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
        netlist: &'a Option<Netlist<'a>>,
    ) -> SchemaPlot<'a, Self> {
        SchemaPlot::new(self, schema, title_block, paper_size, border, netlist)
    }
}

impl<T, I: Iterator<Item = T>> PlotIterator<T> for I {}

impl<T> ItemPlot<sexp::Wire> for T {
    fn item(&self, item: &sexp::Wire) -> Option<Vec<PlotItem>> {
        let stroke = if item.stroke.width > 0.0 {
            Some(item.stroke.width)
        } else {
            None
        };
        let color = if item.stroke.color != (0.0, 0.0, 0.0, 0.0) {
            Some(item.stroke.color)
        } else {
            None
        };
        let linetype = if item.stroke.linetype.is_empty() {
            Some(item.stroke.linetype.to_string())
        } else {
            None
        };
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    item.pts.clone(),
                    stroke,
                    color,
                    linetype,
                    None,
                    vec![Style::Wire],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::Polyline> for T {
    fn item(&self, item: &sexp::Polyline) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    item.pts.clone(),
                    None,
                    None,
                    None,
                    None,
                    vec![Style::Polyline],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::NoConnect> for T {
    fn item(&self, item: &sexp::NoConnect) -> Option<Vec<PlotItem>> {
        let pos: Array1<f64> = item.at.clone();
        let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
        let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;

        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(lines1, None, None, None, None, vec![Style::NoConnect]),
            )),
            PlotItem::Line(
                10,
                Line::new(lines2, None, None, None, None, vec![Style::NoConnect]),
            ),
        ])
    }
}

impl<T> ItemPlot<sexp::Junction> for T {
    fn item(&self, item: &sexp::Junction) -> Option<Vec<PlotItem>> {
        Some(vec![PlotItem::Circle(
            99,
            Circle::new(
                item.at.clone(),
                0.4,
                None,
                None,
                None,
                vec![Style::Junction, Style::Fill(FillType::Outline)],
            ),
        )])
    }
}

impl<T> ItemPlot<sexp::Label> for T {
    fn item(&self, item: &sexp::Label) -> Option<Vec<PlotItem>> {
        let mut angle: f64 = item.angle;
        let pos: Array1<f64> = if angle < 0.0 {
            arr1(&[item.at[0] + 1.0, item.at[1]])
        } else if angle < 90.0 {
            arr1(&[item.at[0], item.at[1] - 1.0])
        } else if angle < 180.0 {
            arr1(&[item.at[0] - 1.0, item.at[1]])
        } else {
            arr1(&[item.at[0], item.at[1] + 1.0])
        };
        if angle >= 180.0 {
            angle -= 180.0;
        }
        Some(vec![PlotItem::Text(
            10,
            Text::new(
                pos,
                angle,
                item.text.clone(),
                item.effects.color,
                item.effects.font_size.0,
                item.effects.font.as_str(),
                item.effects.justify.clone(),
                false,
                vec![Style::Label],
            ),
        )])
    }
}

impl<T> ItemPlot<sexp::GlobalLabel> for T {
    fn item(&self, item: &sexp::GlobalLabel) -> Option<Vec<PlotItem>> {
        let pos: Array1<f64> = item.at.clone();
        let mut angle: f64 = item.angle;
        if angle > 180.0 {
            angle -= 180.0;
        }
        Some(vec![PlotItem::Text(
            10,
            Text::new(
                pos,
                angle,
                item.text.clone(),
                item.effects.color,
                item.effects.font_size.0,
                item.effects.font.as_str(),
                item.effects.justify.clone(),
                true,
                vec![Style::Label],
            ),
        )])
    }
}

impl<T> ItemPlot<sexp::Bus> for T {
    fn item(&self, item: &sexp::Bus) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(item.pts.clone(), None, None, None, None, vec![Style::Bus]),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::BusEntry> for T {
    fn item(&self, item: &sexp::BusEntry) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    arr2(&[
                        [item.at[0], item.at[1]],
                        [item.at[1] + item.size[0], item.at[1] + item.size[1]],
                    ]),
                    None,
                    None,
                    None,
                    None,
                    vec![Style::BusEntry],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::Text> for T {
    fn item(&self, item: &sexp::Text) -> Option<Vec<PlotItem>> {
        let pos: Array1<f64> = item.at.clone();
        let mut angle: f64 = item.angle;
        if angle >= 180.0 {
            angle -= 180.0;
        }
        Some(vec![PlotItem::Text(
            10,
            Text::new(
                pos,
                angle,
                item.text.clone(),
                item.effects.color,
                item.effects.font_size.0,
                item.effects.font.as_str(),
                item.effects.justify.clone(),
                false,
                vec![Style::TextTitle],
            ),
        )])
    }
}

impl<T> ItemPlot<sexp::HierarchicalLabel> for T {
    fn item(&self, item: &sexp::HierarchicalLabel) -> Option<Vec<PlotItem>> {
        let pos: Array1<f64> = item.at.clone();
        let mut angle: f64 = item.angle;
        if angle >= 180.0 {
            angle -= 180.0;
        }
        Some(vec![PlotItem::Text(
            10,
            Text::new(
                pos,
                angle,
                item.text.clone(),
                item.effects.color,
                item.effects.font_size.0,
                item.effects.font.as_str(),
                item.effects.justify.clone(),
                false,
                vec![Style::Label],
            ),
        )])
    }
}

impl<T> ItemPlot<sexp::Sheet> for T {
    fn item(&self, item: &sexp::Sheet) -> Option<Vec<PlotItem>> {
        let prop = item
            .property
            .iter()
            .find(|p| p.key == "Sheet name")
            .unwrap();
        let effects = if let Some(effects) = &prop.effects {
            effects.clone()
        } else {
            Effects::new()
        };
        Some(vec![
            PlotItem::Text(
                10,
                Text::new(
                    item.at.clone(),
                    0.0,
                    prop.value.clone(),
                    effects.color,
                    effects.font_size.0,
                    effects.font.as_str(),
                    effects.justify,
                    false,
                    vec![Style::TextSheet],
                ),
            ),
            PlotItem::Rectangle(
                1,
                Rectangle::new(
                    arr2(&[
                        [item.at[0], item.at[1]],
                        [item.at[0] + item.size[0], item.at[1] + item.size[1]],
                    ]),
                    None,
                    None,
                    None,
                    vec![Style::Outline],
                ),
            ),
        ])
    }
}

impl<'a, I> ItemPlot<sexp::TitleBlock> for &mut SchemaPlot<'a, I> {
    fn item(&self, title_block: &sexp::TitleBlock) -> Option<Vec<PlotItem>> {
        let mut plotter: Vec<PlotItem> = Vec::new();
        //outline
        let pts: Array2<f64> = arr2(&[
            [5.0, 5.0],
            [self.paper_size.0 - 5.0, self.paper_size.1 - 5.0],
        ]);
        plotter.push(PlotItem::Rectangle(
            99,
            Rectangle::new(pts, None, None, None, vec![Style::Border]),
        ));

        //horizontal raster
        for j in &[
            (0.0_f64, 5.0_f64),
            (self.paper_size.1 - 5.0, self.paper_size.1),
        ] {
            for i in 0..(self.paper_size.0 as i32 / BORDER_RASTER) {
                let pts: Array2<f64> = arr2(&[
                    [(i as f64 + 1.0) * BORDER_RASTER as f64, j.0],
                    [(i as f64 + 1.0) * BORDER_RASTER as f64, j.1],
                ]);
                plotter.push(PlotItem::Rectangle(
                    99,
                    Rectangle::new(pts, None, None, None, vec![Style::Border]),
                ));
            }
            for i in 0..(self.paper_size.0 as i32 / BORDER_RASTER + 1) {
                plotter.push(text!(
                    arr1(&[
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0,
                        j.0 + 2.5
                    ]),
                    0.0,
                    (i + 1).to_string(),
                    Effects::new(),
                    vec![Style::TextSheet]
                ));
            }
        }

        //vertical raster
        let letters = vec![
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        for j in &[
            (0.0_f64, 5.0_f64),
            (self.paper_size.0 - 5.0, self.paper_size.0),
        ] {
            for i in 0..(self.paper_size.1 as i32 / BORDER_RASTER) {
                let pts: Array2<f64> = arr2(&[
                    [j.0, (i as f64 + 1.0) * BORDER_RASTER as f64],
                    [j.1, (i as f64 + 1.0) * BORDER_RASTER as f64],
                ]);
                plotter.push(PlotItem::Rectangle(
                    99,
                    Rectangle::new(pts, None, None, None, vec![Style::Border]),
                ));
            }
            for i in 0..(self.paper_size.0 as i32 / BORDER_RASTER + 1) {
                plotter.push(text!(
                    arr1(&[
                        j.0 + 2.5,
                        (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0
                    ]),
                    0.0,
                    letters[i as usize].to_string(),
                    Effects::new(),
                    vec![Style::TextHeader]
                ));
            }
        }

        // the head
        let pts: Array2<f64> = arr2(&[
            [self.paper_size.0 - 120.0, self.paper_size.1 - 40.0],
            [self.paper_size.0 - 5.0, self.paper_size.1 - 5.0],
        ]);
        plotter.push(PlotItem::Rectangle(
            99,
            Rectangle::new(pts, None, None, None, vec![Style::Border]),
        ));
        /* plotter.push(PlotItem::Line(
            99,
            Line::new(
                arr2(&[
                    [paper_size.0 - 120.0, paper_size.1 - 30.0],
                    [paper_size.0 - 5.0, paper_size.1 - 30.0],
                ]),
                stroke.width,
                stroke.linetype.clone(),
                LineCap::Butt,
                stroke.color,
            ),
        )); */
        plotter.push(PlotItem::Line(
            99,
            Line::new(
                arr2(&[
                    [self.paper_size.0 - 120.0, self.paper_size.1 - 10.0],
                    [self.paper_size.0 - 5.0, self.paper_size.1 - 10.0],
                ]),
                None,
                None,
                None,
                None,
                vec![Style::Border],
            ),
        ));
        plotter.push(PlotItem::Line(
            99,
            Line::new(
                arr2(&[
                    [self.paper_size.0 - 120.0, self.paper_size.1 - 16.0],
                    [self.paper_size.0 - 5.0, self.paper_size.1 - 16.0],
                ]),
                None,
                None,
                None,
                None,
                vec![Style::Border],
            ),
        ));

        // if let Some(title_block) = item {
        let left = self.paper_size.0 - 117.0;
        let mut effects: Effects = Effects::new();
        effects.justify.push(String::from("left"));
        for (key, comment) in &title_block.comment {
            if *key == 1 {
                plotter.push(text!(
                    arr1(&[left, self.paper_size.1 - 25.0]),
                    0.0,
                    comment.to_string(),
                    effects,
                    vec![Style::TextHeader]
                ));
            } else if *key == 2 {
                plotter.push(text!(
                    arr1(&[left, self.paper_size.1 - 29.0]),
                    0.0,
                    comment.to_string(),
                    effects,
                    vec![Style::TextHeader]
                ));
            } else if *key == 3 {
                plotter.push(text!(
                    arr1(&[left, self.paper_size.1 - 33.0]),
                    0.0,
                    comment.to_string(),
                    effects,
                    vec![Style::TextHeader]
                ));
            } else if *key == 4 {
                plotter.push(text!(
                    arr1(&[left, self.paper_size.1 - 37.0]),
                    0.0,
                    comment.to_string(),
                    effects,
                    vec![Style::TextHeader]
                ));
            }
        }
        if !title_block.company.is_empty() {
            plotter.push(text!(
                arr1(&[left, self.paper_size.1 - 21.0]),
                0.0,
                title_block.company.clone(),
                effects,
                vec![Style::TextHeader]
            ));
        }
        if !title_block.title.is_empty() {
            plotter.push(text!(
                arr1(&[left, self.paper_size.1 - 13.0]),
                0.0,
                format!("Title: {}", title_block.title),
                effects,
                vec![Style::TextHeader]
            ));
        }
        plotter.push(text!(
            arr1(&[left, self.paper_size.1 - 8.0]),
            0.0,
            format!("Paper: {}", String::from("xxx")),
            effects,
            vec![Style::TextHeader]
        ));

        if !title_block.date.is_empty() {
            plotter.push(text!(
                arr1(&[self.paper_size.0 - 90.0, self.paper_size.1 - 8.0]),
                0.0,
                format!("Data: {}", title_block.date),
                effects,
                vec![Style::TextHeader]
            ));
        }
        if !title_block.rev.is_empty() {
            plotter.push(text!(
                arr1(&[self.paper_size.0 - 20.0, self.paper_size.1 - 8.0]),
                0.0,
                format!("Rev: {}", title_block.rev),
                effects,
                vec![Style::TextHeader]
            ));
        }
        // }
        Some(plotter)
    }
}

impl<'a, I> ItemPlot<sexp::Symbol> for &mut SchemaPlot<'a, I> {
    fn item(&self, item: &sexp::Symbol) -> Option<Vec<PlotItem>> {
        if item.on_schema {
            let mut items: Vec<PlotItem> = Vec::new();
            for property in &item.property {
                let mut effects = if let Some(effects) = &property.effects {
                    effects.clone()
                } else {
                    Effects::new()
                };
                let mut justify: Vec<String> = Vec::new();
                for j in effects.justify {
                    if property.angle + item.angle >= 180.0
                        && property.angle + item.angle < 360.0
                        && j == "left"
                    {
                        justify.push(String::from("right"));
                    } else if (property.angle + item.angle).abs() >= 180.0
                        && property.angle + item.angle < 360.0
                        && j == "right"
                    {
                        justify.push(String::from("left"));
                    } else {
                        justify.push(j);
                    }
                }
                effects.justify = justify;
                let prop_angle = if (item.angle - property.angle).abs() >= 360.0 {
                    (item.angle - property.angle).abs() - 360.0
                } else {
                    (item.angle - property.angle).abs()
                };
                if !effects.hide {
                    items.push(text!(
                        property.at.clone(),
                        prop_angle.abs(),
                        property.value.clone(),
                        effects,
                        vec![Style::Property]
                    ));
                }
            }
            if let Some(lib) = self.schema.get_library(&item.lib_id) {
                for _unit in &self.schema.get_library(&item.lib_id).unwrap().symbols {
                    if _unit.unit == 0 || _unit.unit == item.unit {
                        for graph in &_unit.graph {
                            match graph {
                                Graph::Polyline(polyline) => {
                                    let mut classes = vec![
                                        Style::Outline,
                                        Style::Fill(FillType::from(&polyline.fill_type)),
                                    ];
                                    if item.on_board == false {
                                        classes.push(Style::NotOnBoard);
                                    }
                                    items.push(PlotItem::Polyline(
                                        1,
                                        Polyline::new(
                                            Shape::transform(item, &polyline.pts),
                                            None,
                                            None,
                                            None,
                                            classes,
                                        ),
                                    ));
                                }
                                Graph::Rectangle(rectangle) => {
                                    let start = &rectangle.start;
                                    let end = &rectangle.end;
                                    let pts: Array2<f64> =
                                        arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                    let mut classes = vec![
                                        Style::Outline,
                                        Style::Fill(FillType::from(&rectangle.fill_type)),
                                    ];
                                    if item.on_board == false {
                                        classes.push(Style::NotOnBoard);
                                    }
                                    items.push(PlotItem::Rectangle(
                                        1,
                                        Rectangle::new(
                                            Shape::transform(item, &pts),
                                            None,
                                            None,
                                            None,
                                            classes,
                                        ),
                                    ));
                                }
                                Graph::Circle(circle) => {
                                    let mut classes = vec![
                                        Style::Outline,
                                        Style::Fill(FillType::from(&circle.fill_type)),
                                    ];
                                    if item.on_board == false {
                                        classes.push(Style::NotOnBoard);
                                    }
                                    items.push(PlotItem::Circle(
                                        1,
                                        Circle::new(
                                            Shape::transform(item, &circle.center),
                                            circle.radius,
                                            if circle.stroke.width == 0.0 {
                                                None
                                            } else {
                                                Some(circle.stroke.width)
                                            },
                                            None,
                                            None,
                                            classes,
                                        ),
                                    ));
                                }
                                Graph::Arc(arc) => {
                                    let mut start_angle =
                                        normalize_angle(arc.start_angle() + item.angle);
                                    let mut end_angle =
                                        normalize_angle(arc.end_angle() + item.angle);
                                    let mut arc_start = &arc.start;
                                    let mut arc_end = &arc.end;
                                    if let Some(mirror) = &item.mirror {
                                        //TODO: is
                                        //this
                                        //needed?
                                        if mirror == "x" {
                                            start_angle = 180.0 - end_angle;
                                            end_angle = 180.0 - start_angle;
                                        } else {
                                            start_angle = -start_angle;
                                            end_angle = -end_angle;
                                        }
                                        std::mem::swap(&mut arc_start, &mut arc_end);
                                    }

                                    let mut classes = vec![
                                        Style::Outline,
                                        Style::Fill(FillType::from(&arc.fill_type)),
                                    ];
                                    if item.on_board == false {
                                        classes.push(Style::NotOnBoard);
                                    }
                                    items.push(PlotItem::Arc(
                                        1,
                                        Arc::new(
                                            Shape::transform(item, &arc.center()),
                                            Shape::transform(item, arc_start),
                                            Shape::transform(item, arc_end),
                                            arc.radius(),
                                            start_angle,
                                            end_angle,
                                            if arc.stroke.width == 0.0 {
                                                None
                                            } else {
                                                Some(arc.stroke.width)
                                            },
                                            None,
                                            None,
                                            classes,
                                        ),
                                    ));
                                }
                                Graph::Text(text) => {
                                    items.push(text!(
                                        Shape::transform(item, &text.at),
                                        text.angle,
                                        text.text.clone(),
                                        text.effects,
                                        vec![Style::Text]
                                    ));
                                }
                            }
                        }

                        for pin in &_unit.pin {
                            //calculate the pin line
                            //TODO: there are also symbols like inverting and so on (see:
                            //sch_painter.cpp->848)
                            let orientation = PinOrientation::from(item, pin);
                            let pin_end = MathUtils::projection(&pin.at, pin.angle, pin.length);
                            let pin_line: Array2<f64> =
                                arr2(&[[pin.at[0], pin.at[1]], [pin_end[0], pin_end[1]]]);

                            match pin.pin_graphic_style {
                                PinGraphicalStyle::Line => {
                                    items.push(PlotItem::Line(
                                        10,
                                        Line::new(
                                            Shape::transform(item, &pin_line),
                                            None,
                                            None,
                                            None,
                                            None,
                                            vec![Style::Pin],
                                        ),
                                    ));
                                }
                                PinGraphicalStyle::Inverted => {
                                    items.push(PlotItem::Line(
                                        10,
                                        Line::new(
                                            Shape::transform(item, &pin_line),
                                            None,
                                            None,
                                            None,
                                            None,
                                            vec![Style::Pin],
                                        ),
                                    ));
                                    items.push(PlotItem::Circle(
                                        11,
                                        Circle::new(
                                            Shape::transform(
                                                item,
                                                &arr1(&[pin_end[0], pin_end[1]]),
                                            ),
                                            0.5,
                                            None,
                                            None,
                                            None,
                                            vec![Style::PinDecoration],
                                        ),
                                    ));
                                }
                                PinGraphicalStyle::Clock => {
                                    items.push(PlotItem::Line(
                                        10,
                                        Line::new(
                                            Shape::transform(item, &pin_line),
                                            None,
                                            None,
                                            None,
                                            None,
                                            vec![Style::Pin],
                                        ),
                                    ));
                                    items.push(PlotItem::Polyline(
                                        10,
                                        Polyline::new(
                                            Shape::transform(
                                                item,
                                                &arr2(&[
                                                    [pin_end[0], pin_end[1] - 1.0],
                                                    [pin_end[0] + 1.0, pin_end[1]],
                                                    [pin_end[0], pin_end[1] + 1.0],
                                                ]),
                                            ),
                                            None,
                                            None,
                                            None,
                                            vec![Style::PinDecoration],
                                        ),
                                    ));
                                }
                                PinGraphicalStyle::InvertedClock => todo!(),
                                PinGraphicalStyle::InputLow => todo!(),
                                PinGraphicalStyle::ClockLow => todo!(),
                                PinGraphicalStyle::OutputLow => todo!(),
                                PinGraphicalStyle::EdgeClockHigh => todo!(),
                                PinGraphicalStyle::NonLogic => todo!(),
                            }

                            //show the pin number
                            if !lib.power && lib.pin_numbers_show {
                                let pos = Shape::transform(item, &pin.at)
                                    + match PinOrientation::from(item, pin) {
                                        PinOrientation::Left | PinOrientation::Right => arr1(&[
                                            Shape::pin_angle(item, pin).to_radians().cos()
                                                * pin.length
                                                / 2.0,
                                            -PIN_NUMER_OFFSET,
                                        ]),
                                        PinOrientation::Up | PinOrientation::Down => arr1(&[
                                            PIN_NUMER_OFFSET,
                                            -Shape::pin_angle(item, pin).to_radians().sin()
                                                * pin.length
                                                / 2.0,
                                        ]),
                                    };

                                items.push(text!(
                                    pos,
                                    0.0,
                                    pin.number.0.clone(),
                                    Effects::new(),
                                    vec![Style::TextPinNumber]
                                ));
                            }
                            if !lib.power && pin.name.0 != "~" && lib.pin_names_show {
                                if lib.pin_names_offset != 0.0 {
                                    let name_pos = MathUtils::projection(
                                        &pin.at,
                                        pin.angle,
                                        pin.length + lib.pin_names_offset + 0.5,
                                    );
                                    let effects = Effects::new(); //TODO
                                    items.push(PlotItem::Text(
                                        99,
                                        Text::new(
                                            Shape::transform(item, &name_pos),
                                            pin.angle,
                                            pin.name.0.clone(),
                                            effects.color,
                                            effects.font_size.0,
                                            &effects.font,
                                            vec![match orientation {
                                                PinOrientation::Left => String::from("left"),
                                                PinOrientation::Right => String::from("right"),
                                                PinOrientation::Up => String::from("left"),
                                                PinOrientation::Down => String::from("right"),
                                            }],
                                            false,
                                            vec![Style::TextPinName],
                                        ),
                                    ));
                                } else {
                                    let name_pos = arr1(&[
                                        pin.at[0]
                                            + pin.angle.to_radians().cos()
                                                * (pin.length/* + lib.pin_names_offset * 8.0 */),
                                        pin.at[1]
                                            + pin.angle.to_radians().sin()
                                                * (pin.length/* + lib.pin_names_offset * 8.0 */),
                                    ]);
                                    let effects = Effects::new(); //TODO
                                    items.push(PlotItem::Text(
                                        99,
                                        Text::new(
                                            Shape::transform(item, &name_pos),
                                            pin.angle,
                                            pin.name.0.clone(),
                                            effects.color,
                                            effects.font_size.0,
                                            &effects.font,
                                            vec![String::from("center")],
                                            false,
                                            vec![Style::TextPinName],
                                        ),
                                    ));
                                }
                            }
                            // draw the netlist name
                            if !lib.power {
                                if let Some(netlist) = self.netlist {
                                    let orientation = pin_position(item, pin);
                                    let pos = if orientation == vec![1, 0, 0, 0] {
                                        Shape::transform(item, &pin.at)
                                            + arr1(&[
                                                pin.angle.to_radians().cos() * pin.length / 2.0,
                                                1.0,
                                            ])
                                    } else if orientation == vec![0, 1, 0, 0] {
                                        Shape::transform(item, &pin.at)
                                            + arr1(&[
                                                -1.0,
                                                pin.angle.to_radians().cos() * pin.length / 2.0,
                                            ])
                                    } else if orientation == vec![0, 0, 1, 0] {
                                        Shape::transform(item, &pin.at)
                                            + arr1(&[
                                                pin.angle.to_radians().cos() * pin.length / 2.0,
                                                1.0,
                                            ])
                                    } else if orientation == vec![0, 0, 0, 1] {
                                        Shape::transform(item, &pin.at)
                                            + arr1(&[
                                                -1.0,
                                                -pin.angle.to_radians().cos() * pin.length / 2.0,
                                            ])
                                    } else {
                                        panic!("unknown pin position: {:?}", orientation)
                                    };

                                    let effects = Effects::new(); //TODO
                                    let pin_pos = Shape::transform(item, &pin.at);
                                    items.push(text!(
                                        pos,
                                        0.0,
                                        netlist
                                            .node_name(&Point::new(pin_pos[0], pin_pos[1]))
                                            .unwrap_or_else(|| String::from("NaN")),
                                        effects,
                                        vec![Style::TextNetname]
                                    ));
                                }
                            }
                        }
                    }
                }
            } else {
                let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]);
                items.push(PlotItem::Rectangle(
                    10,
                    Rectangle::new(
                        Shape::transform(item, &pts),
                        None,
                        None,
                        None,
                        vec![Style::NotFound],
                    ),
                ));
            }
            return Some(items);
        }
        None
    }
}
