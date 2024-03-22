use log::{debug, log_enabled, Level};
use ndarray::{arr1, arr2, Array1, Array2, ArrayView};

pub use crate::{
    Arc, Circle, Effects, FillType, Line, Rectangle, Stroke, Style, PlotItem, Polyline, Text,
    border,
    error::Error,
    themer::Themer,
};

use simulation::{Netlist, Point};

use sexp::{
    el,
    math::{normalize_angle, CalcArc, MathUtils, PinOrientation, Shape, Transform},
    utils, PinGraphicalStyle, Sexp, SexpTree, SexpValueQuery, SexpValuesQuery,
};

const PIN_NUMER_OFFSET: f64 = 0.6;

// -----------------------------------------------------------------------------------------------------------
// ---                             collect the plot model from the sexp tree                               ---
// -----------------------------------------------------------------------------------------------------------

/// get the pin position
/// returns an array containing the number of pins:
///   3
/// 0   2
///   1
fn pin_position(symbol: &Sexp, pin: &Sexp) -> Vec<usize> {
    let mut position: Vec<usize> = vec![0; 4];
    let symbol_shift: usize = (utils::angle(symbol).unwrap() / 90.0).round() as usize;

    let lib_pos: usize = (utils::angle(pin).unwrap() / 90.0).round() as usize;
    position[lib_pos] += 1;

    position.rotate_right(symbol_shift);
    if let Some(mirror) = <Sexp as SexpValueQuery<String>>::value(symbol, el::MIRROR) {
        if mirror == "x" {
            position = vec![position[0], position[3], position[2], position[1]];
        } else if mirror == "y" {
            position = vec![position[2], position[1], position[0], position[3]];
        }
    }
    position
}

pub fn plot(
    document: &SexpTree,
    netlist: &Option<Netlist>,
    paper_size: Option<(f64, f64)>,
) -> Vec<PlotItem> {
    let mut plot_items = Vec::new();
    for item in document.root().unwrap().nodes() {
        if item.name == el::GLOBAL_LABEL {
            //TODO
        } else if item.name == el::JUNCTION {
            plot_items.push(PlotItem::Circle(
                100,
                Circle::new(
                    utils::at(item).unwrap(),
                    0.4,
                    Stroke::new(),
                    vec![Style::Junction, Style::Fill(FillType::Outline)],
                ),
            ));
        } else if item.name == el::LABEL {
            let angle: f64 = utils::angle(item).unwrap();
            let pos: Array1<f64> = utils::at(item).unwrap();
            let mut angle: f64 = angle;
            let pos: Array1<f64> = if angle == 0.0 {
                arr1(&[pos[0] + 1.0, pos[1]])
            } else if angle == 90.0 {
                arr1(&[pos[0], pos[1] - 1.0])
            } else if angle == 180.0 {
                arr1(&[pos[0] - 1.0, pos[1]])
            } else {
                arr1(&[pos[0], pos[1] + 1.0])
            };
            if angle >= 180.0 {
                angle -= 180.0;
            }
            let text: String = item.get(0).unwrap();
            plot_items.push(PlotItem::Text(
                10,
                Text::new(pos, angle, text, item.into(), false, vec![Style::Label]),
            ));
        } else if item.name == el::NO_CONNECT {
            let pos: Array1<f64> = utils::at(item).unwrap();
            let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
            let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;
            plot_items.push(PlotItem::Line(
                10,
                Line::new(lines1, Stroke::new(), None, vec![Style::NoConnect]),
            ));
            plot_items.push(PlotItem::Line(
                10,
                Line::new(lines2, Stroke::new(), None, vec![Style::NoConnect]),
            ));
        } else if item.name == el::SYMBOL {
            let on_schema: bool = if let Some(on_schema) = item.query("on_schema").next() {
                let v: String = on_schema.get(0).unwrap();
                v == *"yes" || v == *"true"
            } else {
                true
            };
            if on_schema {
                // let mut items: Vec<PlotItem> = Vec::new();
                for property in item.query(el::PROPERTY) {
                    let mut effects: Effects = property.into();
                    let i_angle = utils::angle(item).unwrap();
                    let p_angle = utils::angle(property).unwrap();
                    let mut justify: Vec<String> = Vec::new();
                    for j in effects.justify {
                        if p_angle + i_angle >= 180.0 && p_angle + i_angle < 360.0 && j == "left" {
                            justify.push(String::from("right"));
                        } else if (p_angle + i_angle).abs() >= 180.0
                            && p_angle + i_angle < 360.0
                            && j == "right"
                        {
                            justify.push(String::from("left"));
                        } else {
                            justify.push(j);
                        }
                    }
                    effects.justify = justify.clone();
                    let prop_angle = if (i_angle - p_angle).abs() >= 360.0 {
                        (i_angle - p_angle).abs() - 360.0
                    } else {
                        (i_angle - p_angle).abs()
                    };

                    let text: String = property.get(1).unwrap(); 
                    if !effects.hide && !text.is_empty() {
                            
                        /* struct TextOutline;
                        impl Outline for TextOutline {}
                        let outline = TextOutline;
                        let at = outline.text_pos(utils::at(property).unwrap(), text.to_string(), prop_angle, effects.clone()); */

                        plot_items.push(PlotItem::Text(
                            10,
                            Text::new(
                                utils::at(property).unwrap(),
                                prop_angle,
                                text.to_string(),
                                effects.clone(),
                                false,
                                vec![Style::Property],
                            ),
                        ));

                        /* plot_items.push(PlotItem::Circle(
                            10,
                            Circle::new(
                                arr1(&[at[[0, 0]], at[[0, 1]]]),
                                0.4,
                                Stroke::new(),
                                vec![Style::Pin],
                            ),
                        ));
                        plot_items.push(PlotItem::Rectangle(
                            10,
                            Rectangle::new(
                                at,
                                Stroke::new(),
                                vec![if effects.justify.contains(&String::from("right")) { Style::Wire } else { Style::Pin }],
                            ),
                        )); */
                    }
                }
                let lib_id: String = item.value(el::LIB_ID).unwrap();
                let item_unit: usize = item.value(el::SYMBOL_UNIT).unwrap();
                if let Some(lib) = utils::get_library(document.root().unwrap(), &lib_id) {
                    for _unit in lib.query(el::SYMBOL) {
                        let unit: usize = utils::unit_number(_unit.get(0).unwrap());
                        if unit == 0 || unit == item_unit {
                            for graph in _unit.query(el::GRAPH_POLYLINE) {
                                let mut classes = vec![
                                    Style::Outline,
                                    Style::Fill(graph.into()),
                                ];
                                let on_board: bool = item.value("on_board").unwrap();
                                if !on_board {
                                    //Grey out item if it is not on pcb
                                    classes.push(Style::NotOnBoard);
                                }
                                let mut pts: Array2<f64> = Array2::zeros((0, 2));
                                for pt in graph.query(el::PTS) {
                                    for xy in pt.query(el::XY) {
                                        pts.push_row(ArrayView::from(&[
                                            xy.get(0).unwrap(),
                                            xy.get(1).unwrap(),
                                        ]))
                                        .unwrap();
                                    }
                                }
                                plot_items.push(PlotItem::Polyline(
                                    20,
                                    Polyline::new(
                                        Shape::transform(item, &pts),
                                        Stroke::new(),
                                        classes,
                                    ),
                                ));
                            }
                            for graph in _unit.query(el::GRAPH_RECTANGLE) {
                                let start: Vec<f64> = graph.query(el::GRAPH_START).next().unwrap().values();
                                let end: Vec<f64> = graph.query(el::GRAPH_END).next().unwrap().values();
                                let pts: Array2<f64> =
                                    arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                                let filltype: String =
                                    graph.query("fill").next().unwrap().value("type").unwrap();
                                let mut classes =
                                    vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                                let on_board: bool = item.value("on_board").unwrap();
                                if !on_board {
                                    classes.push(Style::NotOnBoard);
                                }
                                plot_items.push(PlotItem::Rectangle(
                                    1,
                                    Rectangle::new(Shape::transform(item, &pts), graph.into(), classes),
                                ));
                            }
                            for graph in _unit.query(el::GRAPH_CIRCLE) {
                                let filltype: String =
                                    graph.query("fill").next().unwrap().value("type").unwrap();
                                let mut classes =
                                    vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                                let on_board: bool = item.value("on_board").unwrap();
                                if !on_board {
                                    classes.push(Style::NotOnBoard);
                                }
                                let center: Array1<f64> = graph.value("center").unwrap();
                                let radius: f64 = graph.value("radius").unwrap();
                                plot_items.push(PlotItem::Circle(
                                    1,
                                    Circle::new(
                                        Shape::transform(item, &center),
                                        radius,
                                        graph.into(),
                                        /* TODO if stroke.linewidth == 0.0 {
                                            None
                                        } else {
                                            Some(stroke.linewidth)
                                        },
                                        None,
                                        None, */
                                        classes,
                                    ),
                                ));
                            }

                            for graph in _unit.query(el::GRAPH_ARC) {
                                let mut arc_start: Array1<f64> = graph.value(el::GRAPH_START).unwrap();
                                //TODO let arc_mid: Array1<f64> = graph.value("mid").unwrap();
                                let mut arc_end: Array1<f64> = graph.value(el::GRAPH_END).unwrap();
                                let mirror: Option<String> = graph.value(el::MIRROR);
                                let mut start_angle = normalize_angle(
                                    graph.start_angle() + utils::angle(item).unwrap(),
                                );
                                let mut end_angle = normalize_angle(
                                    graph.end_angle() + utils::angle(item).unwrap(),
                                );
                                if let Some(mirror) = mirror {
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

                                let classes = vec![
                                    Style::Outline,
                                    Style::Fill(item.into()),
                                ];
                                /* TODO if item.on_board == false {
                                    classes.push(Style::NotOnBoard);
                                } */
                                plot_items.push(PlotItem::Arc(
                                    100,
                                    Arc::new(
                                        Shape::transform(item, &graph.center()),
                                        Shape::transform(item, &arc_start),
                                        Shape::transform(item, &arc_end),
                                        graph.radius(),
                                        start_angle,
                                        end_angle,
                                        graph.into(),
                                        classes,
                                    ),
                                ));
                            }
                            /*        Graph::Text(text) => {
                                        items.push(text!(
                                            Shape::transform(item, &text.at),
                                            text.angle,
                                            text.text.clone(),
                                            text.effects,
                                            vec![Style::Text]
                                        ));
                                    }
                                }
                            } */

                            for pin in _unit.query(el::PIN) {
                                //calculate the pin line
                                //TODO: there are also symbols like inverting and so on (see:
                                //sch_painter.cpp->848)
                                let orientation = PinOrientation::from(item, pin);
                                let pin_length: f64 = pin.value("length").unwrap();
                                let pin_at: Array1<f64> = utils::at(pin).unwrap(); //TODO remove
                                                                                   //all at below
                                let pin_angle: f64 = utils::angle(pin).unwrap();
                                let pin_end = MathUtils::projection(
                                    &pin_at,
                                    utils::angle(pin).unwrap(),
                                    pin_length,
                                );
                                let pin_line: Array2<f64> =
                                    arr2(&[[pin_at[0], pin_at[1]], [pin_end[0], pin_end[1]]]);
                                let pin_graphical_style: String = pin.get(1).unwrap();
                                let pin_graphic_style: PinGraphicalStyle =
                                    PinGraphicalStyle::from(pin_graphical_style);
                                let stroke = Stroke::new(); //TODO stroke(pin);
                                match pin_graphic_style {
                                    PinGraphicalStyle::Line => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke,
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                    }
                                    PinGraphicalStyle::Inverted => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke.clone(),
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                        plot_items.push(PlotItem::Circle(
                                            11,
                                            Circle::new(
                                                Shape::transform(
                                                    item,
                                                    &arr1(&[pin_end[0], pin_end[1]]),
                                                ),
                                                0.5,
                                                stroke,
                                                vec![Style::PinDecoration],
                                            ),
                                        ));
                                    }
                                    PinGraphicalStyle::Clock => {
                                        plot_items.push(PlotItem::Line(
                                            10,
                                            Line::new(
                                                Shape::transform(item, &pin_line),
                                                stroke,
                                                None,
                                                vec![Style::Pin],
                                            ),
                                        ));
                                        plot_items.push(PlotItem::Polyline(
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
                                                Stroke::new(),
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

                                let power = lib.query("power").count() == 1;
                                let pin_numbers: Option<String> = lib.value("pin_numbers");
                                let pin_numbers = if let Some(pin_numbers) = pin_numbers {
                                    pin_numbers != "hide"
                                } else {
                                    true
                                };
                                if !power && pin_numbers {
                                    let pos = Shape::transform(item, &utils::at(pin).unwrap())
                                        + match PinOrientation::from(item, pin) {
                                            PinOrientation::Left | PinOrientation::Right => {
                                                arr1(&[
                                                    Shape::pin_angle(item, pin).to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    -PIN_NUMER_OFFSET,
                                                ])
                                            }
                                            PinOrientation::Up | PinOrientation::Down => arr1(&[
                                                PIN_NUMER_OFFSET,
                                                -Shape::pin_angle(item, pin).to_radians().sin()
                                                    * pin_length
                                                    / 2.0,
                                            ]),
                                        };

                                    let pin_number: String =
                                        pin.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
                                    plot_items.push(PlotItem::Text(
                                        10,
                                        Text::new(
                                            pos,
                                            utils::angle(pin).unwrap(),
                                            pin_number,
                                            Effects::new(),
                                            false,
                                            vec![Style::TextPinNumber],
                                        ),
                                    ));
                                }

                                let pin_names: Option<String> = lib.value(el::PIN_NAMES);
                                let pin_names = if let Some(pin_names) = pin_names {
                                    pin_names != "hide"
                                } else {
                                    true
                                };
                                let pin_names_offset: f64 =
                                    if let Some(pin_name) = lib.query(el::PIN_NAMES).next() {
                                        if let Some(pin_offset) = pin_name.value("offset") {
                                            pin_offset
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        0.0
                                    };
                                let pin_name: String =
                                    pin.query(el::PIN_NAME).next().unwrap().get(0).unwrap();
                                if !power && pin_name != "~" && pin_names {
                                    if pin_names_offset != 0.0 {
                                        let name_pos = MathUtils::projection(
                                            &utils::at(pin).unwrap(),
                                            utils::angle(pin).unwrap(),
                                            pin_length + pin_names_offset + 0.5,
                                        );
                                        let mut effects: Effects = pin.into();
                                        effects.justify = vec![match orientation {
                                            PinOrientation::Left => String::from("left"),
                                            PinOrientation::Right => String::from("right"),
                                            PinOrientation::Up => String::from("left"),
                                            PinOrientation::Down => String::from("right"),
                                        }];
                                        plot_items.push(PlotItem::Text(
                                            200,
                                            Text::new(
                                                Shape::transform(item, &name_pos),
                                                utils::angle(pin).unwrap(),
                                                pin_name.clone(),
                                                effects,
                                                false,
                                                vec![Style::TextPinName],
                                            ),
                                        ));
                                    } else {
                                        let name_pos = arr1(&[
                                            pin_at[0]
                                                + pin_angle.to_radians().cos()
                                                    * (pin_length/* + lib.pin_names_offset * 8.0 */),
                                            pin_at[1]
                                                + pin_angle.to_radians().sin()
                                                    * (pin_length/* + lib.pin_names_offset * 8.0 */),
                                        ]);
                                        let mut effects: Effects = pin.into();
                                        effects.justify = vec![String::from("center")];
                                        plot_items.push(PlotItem::Text(
                                            200,
                                            Text::new(
                                                Shape::transform(item, &name_pos),
                                                pin_angle,
                                                pin_name.clone(),
                                                effects,
                                                false,
                                                vec![Style::TextPinName],
                                            ),
                                        ));
                                    }
                                }

                                // draw the netlist name
                                let power = lib.query("power").next();
                                if power.is_none() {
                                    if let Some(netlist) = netlist {
                                        let orientation = pin_position(item, pin);
                                        let pin_length: f64 = pin.value("length").unwrap();
                                        let pos = if orientation == vec![1, 0, 0, 0] {
                                            Shape::transform(item, &utils::at(pin).unwrap())
                                                + arr1(&[
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    1.0,
                                                ])
                                        } else if orientation == vec![0, 1, 0, 0] {
                                            Shape::transform(item, &utils::at(pin).unwrap())
                                                + arr1(&[
                                                    -1.0,
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                ])
                                        } else if orientation == vec![0, 0, 1, 0] {
                                            Shape::transform(item, &utils::at(pin).unwrap())
                                                + arr1(&[
                                                    utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                    1.0,
                                                ])
                                        } else if orientation == vec![0, 0, 0, 1] {
                                            Shape::transform(item, &utils::at(pin).unwrap())
                                                + arr1(&[
                                                    -1.0,
                                                    -utils::angle(pin).unwrap().to_radians().cos()
                                                        * pin_length
                                                        / 2.0,
                                                ])
                                        } else {
                                            panic!("unknown pin position: {:?}", orientation)
                                            //TODO Error
                                        };

                                        let effects = Effects::new(); //TODO
                                        let pin_pos =
                                            Shape::transform(item, &utils::at(pin).unwrap());

                                        plot_items.push(PlotItem::Text(
                                            99,
                                            Text::new(
                                                pos,
                                                0.0,
                                                netlist
                                                    .node_name(&Point::new(pin_pos[0], pin_pos[1]))
                                                    .unwrap_or_else(|| String::from("NaN")),
                                                effects,
                                                false,
                                                vec![Style::TextNetname],
                                            ),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]);
                    plot_items.push(PlotItem::Rectangle(
                        10,
                        Rectangle::new(
                            Shape::transform(item, &pts),
                            Stroke::new(),
                            vec![Style::NotFound],
                        ),
                    ));
                }
            }

        } else if item.name == el::WIRE {
            let pts = item.query(el::PTS).next().unwrap();
            let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
            let xy1: Array1<f64> = xy.first().unwrap().values();
            let xy2: Array1<f64> = xy.get(1).unwrap().values();
            plot_items.push(PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[xy1[0], xy1[1]], [xy2[0], xy2[1]]]),
                    item.into(),
                    None,
                    vec![Style::Wire],
                ),
            ));
        } else if item.name == el::TITLE_BLOCK {
            if let Some(paper_size) = paper_size {
                plot_items.append(&mut border(item, paper_size).unwrap());
            }
        } else if log_enabled!(Level::Debug) {
            let items = ["version", "generator", "uuid", "paper", "lib_symbols", "sheet_instances"];
            if !items.contains(&item.name.as_str()) {
                debug!("unparsed node: {}", item.name);
            }
        }
    }
    plot_items
}
