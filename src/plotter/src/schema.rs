use std::collections::HashMap;

use log::{error, log_enabled, Level};
use ndarray::{arr1, arr2, Array1, Array2, ArrayView};

pub use crate::{
    border, error::Error, themer::Themer, Arc, Circle, Effects, FillType, Line, PlotItem, Polyline,
    Rectangle, Stroke, Style, Text,
};
use crate::{Outline, PlotterImpl, Theme};

use simulation::{Netlist, Point};

use sexp::{
    el,
    math::{normalize_angle, CalcArc, MathUtils, PinOrientation, Shape, Transform},
    utils, PaperSize, PinGraphicalStyle, Sexp, SexpParser, SexpProperty, SexpTree, SexpValueQuery,
    SexpValuesQuery,
};

const PIN_NUMER_OFFSET: f64 = 0.6;

// -----------------------------------------------------------------------------------------------------------
// ---                             collect the plot model from the sexp tree                               ---
// -----------------------------------------------------------------------------------------------------------

pub struct SchemaPlot<'a> {
    schema_pages: HashMap<usize, String>,
    pages: Option<&'a [usize]>,
    theme: Themer<'a>,
    border: bool,
    netlist: bool,
    scale: f64,
    tree: Option<SexpTree>,
}

impl Outline for SchemaPlot<'_> {}

//default trait implementations for SchemaPlot
impl Default for SchemaPlot<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// collect the plot model from the sexp tree
impl<'a> SchemaPlot<'a> {
    pub fn pages(mut self, pages: &'a [usize]) -> Self {
        self.pages = Some(pages);
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Themer::new(theme);
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn netlist(mut self, netlist: bool) -> Self {
        self.netlist = netlist;
        self
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn new() -> Self {
        Self {
            schema_pages: HashMap::new(),
            pages: None,
            theme: Themer::new(Theme::Kicad2020),
            border: true,
            netlist: false,
            scale: 1.0,
            tree: None,
        }
    }

    pub fn open_buffer(&mut self, tree: SexpTree) {
        //collect all the sheets
        let sheet_instance = tree.root().unwrap().query(el::SHEET_INSTANCES).next();
        if let Some(sheet_instance) = sheet_instance {
            for page in sheet_instance.query("path") {
                let path: String = page.get(0).unwrap();
                let number: usize = page.value("page").unwrap();
                self.schema_pages.insert(number, path);
            }
        } else {
            self.schema_pages.insert(1, String::from("/"));
        }
        for page in tree.root().unwrap().query("sheet") {
            let sheetfile: Sexp = page.property("Sheetfile").unwrap();
            let path: String = sheetfile.get(1).unwrap();
            let instances = page.query("instances").next().unwrap();
            let project = instances.query("project").next().unwrap();
            let sheetpath = project.query("path").next().unwrap();
            let number: usize = sheetpath.value("page").unwrap();
            self.schema_pages.insert(number, path);
        }
        self.tree = Some(tree);
    }

    pub fn open(&mut self, path: &str) -> Result<(), Error> {
        let Ok(document) = SexpParser::load(path) else {
            return Err(Error::Plotter(format!("could not load schema: {}", path)));
        };
        let tree = SexpTree::from(document.iter())?;
        self.open_buffer(tree);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&usize, &String)> {
        self.schema_pages.iter()
    }

    pub fn write(&self, page: &usize, plotter: &mut dyn PlotterImpl) -> Result<(), Error> {
        let tree = if page == &1 {
            if let Some(tree) = &self.tree {
                tree.clone()
            } else {
                return Err(Error::Plotter("no root schema loaded".into()));
            }
        } else {
            let document = SexpParser::load("").unwrap();
            SexpTree::from(document.iter()).unwrap()
        };

        //load the netlist
        let netlist = if self.netlist {
            Some(Netlist::from(&tree).unwrap())
        } else {
            None
        };

        let paper_size: (f64, f64) =
            <Sexp as SexpValueQuery<PaperSize>>::value(tree.root().unwrap(), "paper")
                .unwrap()
                .into();

        //TODO handle portraint and landscape
        
        let mut plot_items = self.parse_items(&tree, netlist, Some(paper_size));
        let size = if self.border {
            arr2(&[[0.0, 0.0], [paper_size.0, paper_size.1]])
        } else {
            //when the border is not plotted, the plotter will just use the default bounds
            let rect = self.bounds(&plot_items) + arr2(&[[-2.54, -2.54], [2.54, 2.54]]);
            let x = rect[[0, 0]];
            let y = rect[[0, 1]];
            let offset = arr1(&[x, y]);
            for item in plot_items.iter_mut() {
                match item {
                    PlotItem::Arc(_, arc) => {
                        arc.start = arc.start.clone() - &offset;
                        arc.end = arc.end.clone() - &offset;
                        arc.center = arc.center.clone() - &offset;
                    }
                    PlotItem::Circle(_, circle) => circle.pos = circle.pos.clone() - &offset,
                    PlotItem::Line(_, line) => line.pts = line.pts.clone() - &offset,
                    PlotItem::Rectangle(_, rect) => rect.pts = rect.pts.clone() - &offset,
                    PlotItem::Polyline(_, poly) => poly.pts = poly.pts.clone() - &offset,
                    PlotItem::Text(_, text) => text.pos = text.pos.clone() - &offset,
                }
            }
            arr2(&[
                [0.0, 0.0],
                [rect[[1, 0]] - rect[[0, 0]], rect[[1, 1]] - rect[[0, 1]]],
            ])
        };

        plotter.plot(plot_items.as_slice(), size)?;

        Ok(())
    }

    fn parse_items(
        &self,
        document: &SexpTree,
        netlist: Option<Netlist>,
        paper_size: Option<(f64, f64)>,
    ) -> Vec<PlotItem> {

        let mut plot_items: Vec<PlotItem> = Vec::new();
        for item in document.root().unwrap().nodes() {
            match item.name.as_str() {
                el::LABEL => self.plot(LabelElement{ item, global: false }, &mut plot_items),
                el::GLOBAL_LABEL => self.plot(LabelElement{ item, global: true }, &mut plot_items),
                el::JUNCTION => self.plot(JunctionElement{ item }, &mut plot_items),
                el::NO_CONNECT => self.plot(NoConnectElement{ item }, &mut plot_items),
                el::SYMBOL => self.plot(SymbolElement { item, document, netlist: &netlist }, &mut plot_items),
                el::WIRE => self.plot(WireElement{ item }, &mut plot_items),
                el::TEXT => self.plot(TextElement{ item }, &mut plot_items),
                el::TITLE_BLOCK => {
                    if self.border {
                        if let Some(paper_size) = paper_size {
                            plot_items.append(&mut border(item, paper_size, &self.theme).unwrap());
                        }
                    }
                },
                _ => { 
                    if log_enabled!(Level::Error) {
                        let items = [
                            "generator_version",
                            "version",
                            "generator",
                            "uuid",
                            "paper",
                            "lib_symbols",
                            "sheet_instances",
                        ];
                        if !items.contains(&item.name.as_str()) {
                            error!("unparsed node: {}", item.name);
                        }
                    }
                },
            }
        }
        plot_items
    }
}

trait PlotElement<T> {
    fn plot(&self, item: T, plot_items: &mut Vec<PlotItem>);
}

struct LabelElement<'a> {
    item: &'a Sexp,
    global: bool,
}

impl LabelElement<'_> {
    fn make_label(size: Array1<f64>) -> Array2<f64> {
        const ARROW_PADDING: f64 = 1.0;
        const ARROW_VPADDING: f64 = 0.3;
        arr2(&[
            [0.0, 0.0],
            [ARROW_PADDING, size[1] / 2.0 + ARROW_VPADDING],
            [size[0] + ARROW_PADDING, size[1] / 2.0 + ARROW_VPADDING],
            [size[0] + ARROW_PADDING, size[1] / 2.0 + ARROW_VPADDING],
            [size[0] + ARROW_PADDING, -size[1] / 2.0 - ARROW_VPADDING],
            [size[0] + ARROW_PADDING, -size[1] / 2.0 - ARROW_VPADDING],
            [ARROW_PADDING, -size[1] / 2.0 - ARROW_VPADDING],
            [0.0, 0.0],
        ])
    }
}

impl<'a> PlotElement<LabelElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: LabelElement, plot_items: &mut Vec<PlotItem>) {

        let angle: f64 = utils::angle(item.item).unwrap();
        let pos: Array1<f64> = utils::at(item.item).unwrap();
        let text_pos: Array1<f64> = if angle == 0.0 {
            arr1(&[pos[0] + 1.0, pos[1]])
        } else if angle == 90.0 {
            arr1(&[pos[0], pos[1] - 1.0])
        } else if angle == 180.0 {
            arr1(&[pos[0] - 1.0, pos[1]])
        } else {
            arr1(&[pos[0], pos[1] + 1.0])
        };
        let text_angle = if angle >= 180.0 { angle - 180.0 } else { angle };
        let text: String = item.item.get(0).unwrap();
        let gtext = Text::new(
            text_pos.clone() + arr1(&[0.0, 0.2]),
            text_angle,
            text,
            self.theme.get_effects(item.item.into(), &[Style::Label]),
            false,
        );
        let size = self.text_size(&gtext);
        plot_items.push(PlotItem::Text(12, gtext));

        if item.global {
            let mut outline = LabelElement::make_label(size);
            if angle != 0.0 {
                let theta = angle.to_radians();
                let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
                outline = outline.dot(&rot);
            }
            outline = outline + pos.clone();
            plot_items.push(PlotItem::Polyline(
                10,
                Polyline::new(
                    outline,
                    self.theme.get_stroke(
                        Stroke::new(),
                        &[Style::GlobalLabel, Style::Fill(FillType::Background)],
                    ),
                ),
            ));
        }
    /* ooo
    fn plot(&self, item: LabelElement, plot_items: &mut Vec<PlotItem>) {
        let angle: f64 = utils::angle(item.item).unwrap();
        let pos: Array1<f64> = utils::at(item.item).unwrap();
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
        let text: String = item.item.get(0).unwrap();
        plot_items.push(PlotItem::Text(
            10,
            Text::new(
                pos,
                angle,
                text,
                self.theme.get_effects(item.item.into(), &[Style::Label]),
                false,
            ),
        )); */
    } 
}

struct TextElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<TextElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: TextElement, plot_items: &mut Vec<PlotItem>) {

        let angle: f64 = utils::angle(item.item).unwrap();
        let pos: Array1<f64> = utils::at(item.item).unwrap();
        let text_pos: Array1<f64> = if angle == 0.0 {
            arr1(&[pos[0] + 1.0, pos[1]])
        } else if angle == 90.0 {
            arr1(&[pos[0], pos[1] - 1.0])
        } else if angle == 180.0 {
            arr1(&[pos[0] - 1.0, pos[1]])
        } else {
            arr1(&[pos[0], pos[1] + 1.0])
        };
        let text: String = item.item.get(0).unwrap();
        let gtext = Text::new(
            text_pos.clone(),
            0.0,
            text,
            self.theme.get_effects(item.item.into(), &[Style::Text]),
            false,
        );
        plot_items.push(PlotItem::Text(12, gtext));
    } 
}

struct WireElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<WireElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: WireElement, plot_items: &mut Vec<PlotItem>) {
        let pts = item.item.query(el::PTS).next().unwrap();
        let xy = pts.query(el::XY).collect::<Vec<&Sexp>>();
        let xy1: Array1<f64> = xy.first().unwrap().values();
        let xy2: Array1<f64> = xy.get(1).unwrap().values();
        plot_items.push(PlotItem::Line(
            10,
            Line::new(
                arr2(&[[xy1[0], xy1[1]], [xy2[0], xy2[1]]]),
                self.theme.get_stroke(item.item.into(), &[Style::Wire]),
                None,
            ),
        ));
    }
}

struct JunctionElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<JunctionElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: JunctionElement, plot_items: &mut Vec<PlotItem>) {
        let mut stroke = self
            .theme
            .get_stroke(Stroke::from(item.item), &[Style::Junction]);
        stroke.fillcolor = stroke.linecolor.clone();
        plot_items.push(PlotItem::Circle(
            100,
            Circle::new(utils::at(item.item).unwrap(), 0.4, stroke),
        ));
    }
}

struct NoConnectElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<NoConnectElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: NoConnectElement, plot_items: &mut Vec<PlotItem>) {
        let pos: Array1<f64> = utils::at(item.item).unwrap();
        let lines1 = arr2(&[[-0.8, 0.8], [0.8, -0.8]]) + &pos;
        let lines2 = arr2(&[[0.8, 0.8], [-0.8, -0.8]]) + &pos;
        plot_items.push(PlotItem::Line(
            10,
            Line::new(
                lines1,
                self.theme.get_stroke(Stroke::new(), &[Style::NoConnect]),
                None,
            ),
        ));
        plot_items.push(PlotItem::Line(
            10,
            Line::new(
                lines2,
                self.theme.get_stroke(Stroke::new(), &[Style::NoConnect]),
                None,
            ),
        ));
    }
}

struct SymbolElement<'a> {
    item: &'a Sexp,
    document: &'a SexpTree,
    netlist: &'a Option<Netlist<'a>>,
}

impl<'a> PlotElement<SymbolElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: SymbolElement, plot_items: &mut Vec<PlotItem>) {
        let on_schema: bool = if let Some(on_schema) = item.item.query("on_schema").next() {
            let v: String = on_schema.get(0).unwrap();
            v == *"yes" || v == *"true"
        } else {
            true
        };
        if on_schema {
            // let mut items: Vec<PlotItem> = Vec::new();
            for property in item.item.query(el::PROPERTY) {
                let mut effects: Effects = property.into();
                let i_angle = utils::angle(item.item).unwrap();
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
                    plot_items.push(PlotItem::Text(
                        10,
                        Text::new(
                            utils::at(property).unwrap(),
                            prop_angle,
                            text.to_string(),
                            self.theme.get_effects(effects.clone(), &[Style::Property]),
                            false,
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
            let lib_id: String = item.item.value(el::LIB_ID).unwrap();
            let item_unit: usize = item.item.value(el::SYMBOL_UNIT).unwrap();
            if let Some(lib) = utils::get_library(item.document.root().unwrap(), &lib_id) {
                for _unit in lib.query(el::SYMBOL) {
                    let unit: usize = utils::unit_number(_unit.get(0).unwrap());
                    if unit == 0 || unit == item_unit {
                        for graph in _unit.query(el::GRAPH_POLYLINE) {
                            let mut classes = vec![Style::Outline, Style::Fill(graph.into())];
                            let on_board: bool = item.item.value("on_board").unwrap();
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
                                    Shape::transform(item.item, &pts),
                                    self.theme.get_stroke(Stroke::new(), classes.as_slice()),
                                ),
                            ));
                        }
                        for graph in _unit.query(el::GRAPH_RECTANGLE) {
                            let start: Vec<f64> =
                                graph.query(el::GRAPH_START).next().unwrap().values();
                            let end: Vec<f64> = graph.query(el::GRAPH_END).next().unwrap().values();
                            let pts: Array2<f64> = arr2(&[[start[0], start[1]], [end[0], end[1]]]);
                            let filltype: String =
                                graph.query("fill").next().unwrap().value("type").unwrap();
                            let mut classes =
                                vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                            let on_board: bool = item.item.value("on_board").unwrap();
                            if !on_board {
                                classes.push(Style::NotOnBoard);
                            }
                            plot_items.push(PlotItem::Rectangle(
                                1,
                                Rectangle::new(
                                    Shape::transform(item.item, &pts),
                                    self.theme.get_stroke(graph.into(), classes.as_slice()),
                                ),
                            ));
                        }
                        for graph in _unit.query(el::GRAPH_CIRCLE) {
                            let filltype: String =
                                graph.query("fill").next().unwrap().value("type").unwrap();
                            let mut classes =
                                vec![Style::Outline, Style::Fill(FillType::from(&filltype))];
                            let on_board: bool = item.item.value("on_board").unwrap();
                            if !on_board {
                                classes.push(Style::NotOnBoard);
                            }
                            let center: Array1<f64> = graph.value("center").unwrap();
                            let radius: f64 = graph.value("radius").unwrap();
                            plot_items.push(PlotItem::Circle(
                                1,
                                Circle::new(
                                    Shape::transform(item.item, &center),
                                    radius,
                                    self.theme
                                        .get_stroke(Stroke::from(graph), &[Style::Outline]),
                                ),
                            ));
                        }

                        for graph in _unit.query(el::GRAPH_ARC) {
                            let mut arc_start: Array1<f64> = graph.value(el::GRAPH_START).unwrap();
                            //TODO let arc_mid: Array1<f64> = graph.value("mid").unwrap();
                            let mut arc_end: Array1<f64> = graph.value(el::GRAPH_END).unwrap();
                            let mirror: Option<String> = graph.value(el::MIRROR);
                            let mut start_angle =
                                normalize_angle(graph.start_angle() + utils::angle(item.item).unwrap());
                            let mut end_angle =
                                normalize_angle(graph.end_angle() + utils::angle(item.item).unwrap());
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

                            let classes = vec![Style::Outline, Style::Fill(item.item.into())];
                            /* TODO if item.on_board == false {
                                classes.push(Style::NotOnBoard);
                            } */
                            plot_items.push(PlotItem::Arc(
                                100,
                                Arc::new(
                                    Shape::transform(item.item, &graph.center()),
                                    Shape::transform(item.item, &arc_start),
                                    Shape::transform(item.item, &arc_end),
                                    graph.radius(),
                                    start_angle,
                                    end_angle,
                                    self.theme.get_stroke(graph.into(), classes.as_slice()),
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

                            let power = lib.query("power").count() == 1;
                            let pin_numbers: Option<String> = lib.value("pin_numbers");
                            let pin_numbers = if let Some(pin_numbers) = pin_numbers {
                                pin_numbers != "hide"
                            } else {
                                true
                            };
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
                            self.plot(PinElement{item: pin, symbol:item.item, netlist: item.netlist, power, pin_numbers, pin_names, pin_names_offset}, plot_items);
                        }
                    }
                }
            } else {
                let pts = arr2(&[[0.0, 0.0], [10.0, 10.0]]);
                plot_items.push(PlotItem::Rectangle(
                    10,
                    Rectangle::new(
                        Shape::transform(item.item, &pts),
                        self.theme.get_stroke(Stroke::new(), &[Style::NotFound]),
                    ),
                ));
            }
        }
    }
}

struct PinElement<'a> {
    item: &'a Sexp,
    symbol: &'a Sexp,
    netlist: &'a Option<Netlist<'a>>,
    power: bool,
    pin_numbers: bool,
    pin_names: bool,
    pin_names_offset: f64,
}

impl PinElement<'_> {
    /// get the pin position
    /// returns an array containing the number of pins:
    ///   3
    /// 0   2
    ///   1
    pub fn pin_position(symbol: &Sexp, pin: &Sexp) -> Vec<usize> {
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
}

impl<'a> PlotElement<PinElement<'a>> for SchemaPlot<'a> {
    fn plot(&self, item: PinElement, plot_items: &mut Vec<PlotItem>) {
        //calculate the pin line
        //TODO: there are also symbols like inverting and so on (see:
        //sch_painter.cpp->848)
        let orientation = PinOrientation::from(item.symbol, item.item);
        let pin_length: f64 = item.item.value("length").unwrap();
        let pin_at: Array1<f64> = utils::at(item.item).unwrap(); //TODO remove
                                                           //all at below
        let pin_angle: f64 = utils::angle(item.item).unwrap();
        let pin_end = MathUtils::projection(
            &pin_at,
            utils::angle(item.item).unwrap(),
            pin_length,
        );
        let pin_line: Array2<f64> =
            arr2(&[[pin_at[0], pin_at[1]], [pin_end[0], pin_end[1]]]);
        let pin_graphical_style: String = item.item.get(1).unwrap();
        let pin_graphic_style: PinGraphicalStyle =
            PinGraphicalStyle::from(pin_graphical_style);
        let stroke = Stroke::from(item.item);
        match pin_graphic_style {
            PinGraphicalStyle::Line => {
                plot_items.push(PlotItem::Line(
                    10,
                    Line::new(
                        Shape::transform(item.symbol, &pin_line),
                        self.theme.get_stroke(stroke, &[Style::Pin]),
                        None,
                    ),
                ));
            }
            PinGraphicalStyle::Inverted => {
                plot_items.push(PlotItem::Line(
                    10,
                    Line::new(
                        Shape::transform(item.symbol, &pin_line),
                        self.theme.get_stroke(stroke.clone(), &[Style::Pin]),
                        None,
                    ),
                ));
                plot_items.push(PlotItem::Circle(
                    11,
                    Circle::new(
                        Shape::transform(
                            item.symbol,
                            &arr1(&[pin_end[0], pin_end[1]]),
                        ),
                        0.5,
                        self.theme.get_stroke(
                            Stroke::from(item.item),
                            &[Style::PinDecoration],
                        ),
                    ),
                ));
            }
            PinGraphicalStyle::Clock => {
                plot_items.push(PlotItem::Line(
                    10,
                    Line::new(
                        Shape::transform(item.symbol, &pin_line),
                        self.theme.get_stroke(stroke, &[Style::Pin]),
                        None,
                    ),
                ));
                plot_items.push(PlotItem::Polyline(
                    10,
                    Polyline::new(
                        Shape::transform(
                            item.symbol,
                            &arr2(&[
                                [pin_end[0], pin_end[1] - 1.0],
                                [pin_end[0] + 1.0, pin_end[1]],
                                [pin_end[0], pin_end[1] + 1.0],
                            ]),
                        ),
                        self.theme
                            .get_stroke(Stroke::new(), &[Style::PinDecoration]),
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

        if !item.power && item.pin_numbers {
            let pos = Shape::transform(item.symbol, &utils::at(item.item).unwrap())
                + match PinOrientation::from(item.symbol, item.item) {
                    PinOrientation::Left | PinOrientation::Right => arr1(&[
                        Shape::pin_angle(item.symbol, item.item).to_radians().cos()
                            * pin_length
                            / 2.0,
                        -PIN_NUMER_OFFSET,
                    ]),
                    PinOrientation::Up | PinOrientation::Down => arr1(&[
                        PIN_NUMER_OFFSET,
                        -Shape::pin_angle(item.symbol, item.item).to_radians().sin()
                            * pin_length
                            / 2.0,
                    ]),
                };

            let pin_number: String =
                item.item.query(el::PIN_NUMBER).next().unwrap().get(0).unwrap();
            plot_items.push(PlotItem::Text(
                10,
                Text::new(
                    pos,
                    utils::angle(item.item).unwrap(),
                    pin_number,
                    self.theme
                        .get_effects(Effects::new(), &[Style::TextPinNumber]),
                    false,
                ),
            ));
        }

        let pin_name: String =
            item.item.query(el::PIN_NAME).next().unwrap().get(0).unwrap();
        if !item.power && pin_name != "~" && item.pin_names {
            if item.pin_names_offset != 0.0 {
                let name_pos = MathUtils::projection(
                    &utils::at(item.item).unwrap(),
                    utils::angle(item.item).unwrap(),
                    pin_length + item.pin_names_offset + 0.5,
                );
                let mut effects: Effects = item.item.into();
                effects.justify = vec![match orientation {
                    PinOrientation::Left => String::from("left"),
                    PinOrientation::Right => String::from("right"),
                    PinOrientation::Up => String::from("left"),
                    PinOrientation::Down => String::from("right"),
                }];
                plot_items.push(PlotItem::Text(
                    200,
                    Text::new(
                        Shape::transform(item.symbol, &name_pos),
                        utils::angle(item.item).unwrap(),
                        pin_name.clone(),
                        self.theme.get_effects(effects, &[Style::TextPinName]),
                        false,
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
                let mut effects: Effects = item.item.into();
                effects.justify = vec![String::from("center")];
                plot_items.push(PlotItem::Text(
                    200,
                    Text::new(
                        Shape::transform(item.symbol, &name_pos),
                        pin_angle,
                        pin_name.clone(),
                        self.theme.get_effects(effects, &[Style::TextPinName]),
                        false,
                    ),
                ));
            }
        }

        // draw the netlist name
        if item.power {
            if let Some(netlist) = item.netlist {
                let orientation = PinElement::pin_position(item.symbol, item.item);
                let pin_length: f64 = item.item.value("length").unwrap();
                let pos = if orientation == vec![1, 0, 0, 0] {
                    Shape::transform(item.symbol, &utils::at(item.item).unwrap())
                        + arr1(&[
                            utils::angle(item.item).unwrap().to_radians().cos()
                                * pin_length
                                / 2.0,
                            1.0,
                        ])
                } else if orientation == vec![0, 1, 0, 0] {
                    Shape::transform(item.symbol, &utils::at(item.item).unwrap())
                        + arr1(&[
                            -1.0,
                            utils::angle(item.item).unwrap().to_radians().cos()
                                * pin_length
                                / 2.0,
                        ])
                } else if orientation == vec![0, 0, 1, 0] {
                    Shape::transform(item.symbol, &utils::at(item.item).unwrap())
                        + arr1(&[
                            utils::angle(item.item).unwrap().to_radians().cos()
                                * pin_length
                                / 2.0,
                            1.0,
                        ])
                } else if orientation == vec![0, 0, 0, 1] {
                    Shape::transform(item.symbol, &utils::at(item.item).unwrap())
                        + arr1(&[
                            -1.0,
                            -utils::angle(item.item).unwrap().to_radians().cos()
                                * pin_length
                                / 2.0,
                        ])
                } else {
                    panic!("unknown pin position: {:?}", orientation)
                    //TODO Error
                };

                let effects = Effects::new(); //TODO
                let pin_pos = Shape::transform(item.symbol, &utils::at(item.item).unwrap());

                plot_items.push(PlotItem::Text(
                    99,
                    Text::new(
                        pos,
                        0.0,
                        netlist
                            .node_name(&Point::new(pin_pos[0], pin_pos[1]))
                            .unwrap_or_else(|| String::from("NaN")),
                        self.theme.get_effects(effects, &[Style::TextNetname]),
                        false,
                    ),
                ));
            }
        }
    }
}
