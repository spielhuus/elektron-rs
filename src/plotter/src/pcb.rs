//! Plot the PCB
use crate::{
    border, error::Error, schema::Themer, Arc, Circle, Color, Effects, Line, Outline, PlotItem,
    PlotterImpl, Polyline, Stroke, Style, Text, Theme,
};
use log::*;
use log::{debug, error, warn};
use ndarray::{arr1, arr2, Array1, Array2, ArrayView};

use regex::Regex;
use sexp::{
    el,
    math::{Shape, Transform},
    PaperSize, Sexp, SexpParser, SexpTree, SexpValueQuery, SexpValuesQuery,
};

pub const LAYERS: &[&str; 9] = &[
    "F.Cu",
    "B.Cu",
    "F.Paste",
    "B.Paste",
    "F.SilkS",
    "B.SilkS",
    "F.Mask",
    "B.Mask",
    "Edge.Cuts",
];

const SKIP_ELEMENTS: &[&str; 11] = &[
    "general",
    "setup",
    "layers",
    "layer",
    "net",
    "paper",
    "title_block",
    "generator",
    "generator_version",
    "version",
    "dimension",
];

const SKIP_FP_ELEMENTS: &[&str; 11] = &[
    "path",
    "attr",
    "uuid",
    "descr",
    "tags",
    "at",
    "model",
    "layer",
    "locked",
    "sheetfile",
    "sheetname",
];

#[derive(Debug, Eq, PartialEq)]
enum PadType {
    ThruHole,
    Smd,
    Connect,
    NpThruHole,
}

///create a new PadType from String.
impl From<String> for PadType {
    fn from(pad_type: String) -> Self {
        match pad_type.as_str() {
            "thru_hole" => PadType::ThruHole,
            "smd" => PadType::Smd,
            "connect" => PadType::Connect,
            "np_thru_hole" => PadType::NpThruHole,
            _ => panic!("unknown pad type: {}", pad_type),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum PadShape {
    Circle,
    Rect,
    Oval,
    Trapezoid,
    RoundRect,
    Custom,
}

///create a new PadShape from String.
impl From<String> for PadShape {
    fn from(pad_type: String) -> Self {
        match pad_type.as_str() {
            "circle" => PadShape::Circle,
            "rect" => PadShape::Rect,
            "oval" => PadShape::Oval,
            "trapezoid" => PadShape::Trapezoid,
            "roundrect" => PadShape::RoundRect,
            "custom" => PadShape::Custom,
            _ => panic!("unknown pad shape: {}", pad_type),
        }
    }
}

#[derive(Debug)]
struct DrillHole {
    _oval: bool,
    diameter: f64,
    width: Option<f64>,
    _offset: Option<Array1<f64>>,
}

///create a new DrillHole from Sexp.
impl<'a> From<&'a Sexp> for DrillHole {
    fn from(element: &'a Sexp) -> Self {
        let mut oval = false;
        let diameter: f64;
        let width: Option<f64>;

        let token: String = element.get(0).unwrap();
        if token == "oval" {
            oval = true;
            diameter = element.get(1).unwrap();
            width = element.get(2);
        } else {
            diameter = element.get(0).unwrap();
            width = element.get(1);
        }
        let offset = <Sexp as SexpValueQuery<Array1<f64>>>::value(element, el::OFFSET);
        DrillHole {
            _oval: oval,
            diameter,
            width,
            _offset: offset,
        }
    }
}

// -----------------------------------------------------------------------------------------------------------
// ---                             collect the plot model from the sexp tree                               ---
// -----------------------------------------------------------------------------------------------------------

pub struct PcbPlot<'a> {
    theme: Themer<'a>,
    border: bool,
    scale: f64,
    name: Option<String>,
    path: String,
    tree: Option<SexpTree>,
    layers: Vec<String>,
}

impl Outline for PcbPlot<'_> {}

//default trait implementations for PcbPlot
impl Default for PcbPlot<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// collect the plot model from the sexp tree
impl<'a> PcbPlot<'a> {
    /// Select the color theme.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Themer::new(theme);
        self
    }
    /// Draw a border around the plot, otherwise crop the plot.
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }
    /// Scale the plot
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
    /// The name of the plot.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    /// create a new SchemaPlot with defalt values.
    pub fn new() -> Self {
        Self {
            theme: Themer::new(Theme::default()),
            border: true,
            scale: 1.0,
            name: None,
            path: String::new(),
            tree: None,
            layers: Vec::new(),
        }
    }

    fn get_layers(&mut self) {
        if let Some(tree) = &self.tree {
            for layer in tree
                .root()
                .expect("root expected")
                .query("layers")
                .next()
                .expect("layers expected")
                .iter()
            {
                if let sexp::SexpAtom::Node(node) = layer {
                    let name: String = node.get(0).expect("layer name expected");
                    self.layers.push(name);
                }
            }
        }
        trace!("layers: {:?}", self.layers);
    }

    pub fn open(&mut self, path: &str) -> Result<(), Error> {
        debug!("open pcb: {}", path);
        if let Some(dir) = std::path::Path::new(&path).parent() {
            self.path = dir.to_str().unwrap().to_string();
        }
        let Ok(document) = SexpParser::load(path) else {
            return Err(Error::Plotter(format!("could not load schema: {}", path)));
        };
        let tree = SexpTree::from(document.iter())?;
        self.tree = Some(tree);
        self.get_layers();
        Ok(())
    }

    pub fn write(&self, plotter: &mut dyn PlotterImpl, layers: Vec<String>) -> Result<(), Error> {
        trace!("write layer: {:?}", layers);
        let tree = if let Some(tree) = &self.tree {
            tree.clone()
        } else {
            return Err(Error::Plotter("no root schema loaded".into()));
        };

        let paper_size: (f64, f64) =
            <Sexp as SexpValueQuery<PaperSize>>::value(tree.root().unwrap(), "paper").unwrap().into();

        //TODO handle portraint and landscape

        let mut plot_items = Vec::<PlotItem>::new();
        for layer in layers {
            //check if layer exists
            if !self.layers.contains(&layer) {
                return Err(Error::Plotter(format!("layer {} not found", layer)));
            }
            plot_items.append(&mut self.parse_items(&tree, &layer)?);
        }

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

        plotter.plot(plot_items.as_slice(), size, self.scale, self.name.clone())?;

        Ok(())
    }

    ///check if the item matches the layer, if layer is None, return true.
    fn is_layer(item: &Sexp, layer: &str) -> bool {
        if let Some(name) = <Sexp as SexpValueQuery<String>>::value(item, "layer") {
            return name == layer || PcbPlot::is_layer_in(&name, layer);
        } else {
            warn!("no layer in item: {:?}", item);
        }
        true
    }

    ///check if the layer matches an item in the layers list, layer can also contain a wildcard.
    fn is_layer_in(layer: &str, name: &str) -> bool {
        if layer == name {
            return true;
        } else if name.contains('*') {
            let name = name.replace('.', "\\.");
            let mut name = name.replace('*', ".*");
            name.push('$');
            let regex = Regex::new(&name).unwrap();
            if regex.is_match(layer) {
                return true;
            }
        }
        false
    }

    fn parse_items(&self, document: &SexpTree, layer: &str) -> Result<Vec<PlotItem>, Error> {
        //plot the border
        let mut plot_items: Vec<PlotItem> = Vec::new();
        let title_block =
            if let Some(title_block) = document.root().unwrap().query(el::TITLE_BLOCK).next() {
                Some(title_block)
            } else if let Some(tree) = &self.tree {
                tree.root().unwrap().query(el::TITLE_BLOCK).next()
            } else {
                None
            };
        if self.border {
            if let Some(title_block) = title_block {
                if let Some(paper_size) = <Sexp as SexpValueQuery<PaperSize>>::value(
                    document.root().unwrap(),
                    el::TITLE_BLOCK_PAPER,
                ) {
                    plot_items.append(&mut border(title_block, paper_size, &self.theme).unwrap());
                }
            }
        }

        for item in document.root().unwrap().nodes() {
            match item.name.as_str() {
                el::SEGMENT => self.plot(SegmentElement { item }, layer, &mut plot_items)?,
                el::GR_LINE => self.plot(GrLineElement { item }, layer, &mut plot_items)?,
                el::GR_POLY => self.plot(GrPolyElement { item }, layer, &mut plot_items)?,
                el::GR_CIRCLE => self.plot(GrCircleElement { item }, layer, &mut plot_items)?,
                el::VIA => self.plot(ViaElement { item }, layer, &mut plot_items)?,
                el::GR_TEXT => self.plot(GrTextElement { item }, layer, &mut plot_items)?,
                el::FOOTPRINT => self.plot(FootprintElement { item }, layer, &mut plot_items)?,
                el::ZONE => self.plot(ZoneElement { item }, layer, &mut plot_items)?,
                _ => {
                    if log_enabled!(Level::Error) && !SKIP_ELEMENTS.contains(&item.name.as_str()) {
                        error!("unparsed node: {}", item.name);
                    }
                }
            }
        }
        Ok(plot_items)
    }
}

macro_rules! stroke {
    ($sexp:expr) => {
        if let Some(stroke) = $sexp.query(el::STROKE).next() {
            stroke.value(el::WIDTH).unwrap()
        } else {
            1.0
        }
    };
}

trait PlotElement<T> {
    fn plot(&self, item: T, layer: &str, plot_items: &mut Vec<PlotItem>) -> Result<(), Error>;
}

struct GrLineElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<GrLineElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: GrLineElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        if PcbPlot::is_layer(item.item, layer) {
            let start: Array1<f64> = item.item.value(el::START).unwrap();
            let end: Array1<f64> = item.item.value(el::END).unwrap();
            let width: f64 = stroke!(item.item);

            let mut stroke = Stroke::new();
            stroke.linewidth = width;
            stroke.linecolor = Color::Rgb(0, 255, 0);

            plot_items.push(PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[start[0], start[1]], [end[0], end[1]]]),
                    stroke,
                    None,
                ),
            ));
        }
        Ok(())
    }
}

struct GrPolyElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<GrPolyElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: GrPolyElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let mut pts: Array2<f64> = Array2::zeros((0, 2));
        if PcbPlot::is_layer(item.item, layer) {
            for pt in item.item.query(el::PTS) {
                for xy in pt.query(el::XY) {
                    pts.push_row(ArrayView::from(&[xy.get(0).unwrap(), xy.get(1).unwrap()]))
                        .unwrap();
                }
            }
            let layer: String = item.item.value(el::LAYER).unwrap();
            let mut stroke = Stroke::new();
            stroke.linewidth = 0.0;
            let color = self.theme.layer_color(&[Style::from(layer)]);
            stroke.linecolor = color.clone();
            stroke.fillcolor = color;

            plot_items.push(PlotItem::Polyline(20, Polyline::new(pts, stroke)));
        }
        Ok(())
    }
}

struct GrCircleElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<GrCircleElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: GrCircleElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let circle_layer: String = item.item.value(el::LAYER).unwrap();
        if PcbPlot::is_layer_in(layer, &circle_layer) {
            let center: Array1<f64> = item.item.value(el::CENTER).unwrap();
            let end: Array1<f64> = item.item.value(el::END).unwrap();
            let width = stroke!(item.item);

            let mut stroke = Stroke::new();
            stroke.linewidth = width;
            stroke.linecolor = self.theme.layer_color(&[Style::from(circle_layer)]);
            //TODO fill

            let radius = Circle::radius(&center, &end);
            plot_items.push(PlotItem::Circle(1, Circle::new(center, radius, stroke)));
        }
        Ok(())
    }
}

struct ZoneElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<ZoneElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: ZoneElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        for polygon in item.item.query("filled_polygon") {
            let mut pts: Array2<f64> = Array2::zeros((0, 2));
            if PcbPlot::is_layer(polygon, layer) {
                for pt in polygon.query(el::PTS) {
                    for xy in pt.query(el::XY) {
                        pts.push_row(ArrayView::from(&[xy.get(0).unwrap(), xy.get(1).unwrap()]))
                            .unwrap();
                    }
                }
                let layer: String = polygon.value(el::LAYER).unwrap();
                let mut stroke = Stroke::new();
                stroke.linewidth = 0.0;
                let color = self.theme.layer_color(&[Style::from(layer)]);
                stroke.linecolor = color.clone();
                stroke.fillcolor = color;

                plot_items.push(PlotItem::Polyline(20, Polyline::new(pts, stroke)));
            }
        }
        Ok(())
    }
}

struct SegmentElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<SegmentElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: SegmentElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        if PcbPlot::is_layer(item.item, layer) {
            let start: Array1<f64> = item.item.value(el::START).unwrap();
            let end: Array1<f64> = item.item.value(el::END).unwrap();
            let width: f64 = item.item.value(el::WIDTH).unwrap();
            let mut stroke = Stroke::new();
            stroke.linewidth = width;
            stroke.linecolor = self.theme.layer_color(&[Style::from(layer.to_string())]);

            plot_items.push(PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[start[0], start[1]], [end[0], end[1]]]),
                    stroke,
                    None,
                ),
            ));
        }
        Ok(())
    }
}

struct ViaElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<ViaElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: ViaElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let at = sexp::utils::at(item.item).unwrap();
        let size: f64 = item.item.value(el::SIZE).unwrap();
        let drill: f64 = item.item.value("drill").unwrap();
        let layers_node: &Sexp = item.item.query("layers").next().expect("expect layers");
        let layers: Vec<String> = layers_node.values();
        let linewidth = size - drill;

        for act_layer in layers {
            if PcbPlot::is_layer_in(&act_layer, layer) {
                let mut stroke = Stroke::new();
                stroke.linewidth = linewidth;
                stroke.linecolor = self.theme.layer_color(&[Style::ViaThrough]);
                stroke.fillcolor = self.theme.layer_color(&[Style::ViaHole]);

                plot_items.push(PlotItem::Circle(
                    10,
                    Circle::new(at.clone(), size - linewidth, stroke),
                ));
            }
        }
        Ok(())
    }
}

//23092   │     (gr_text "summe"
//23093   │         (locked yes)
//23094   │         (at 68.9 158.06 0)
//23095   │         (layer "B.SilkS")
//23096   │         (uuid "9c8fdf06-c8c5-49d5-b787-6d9bada2d902")
//23097   │         (effects
//23098   │             (font
//23099   │                 (size 0.8 1)
//23100   │                 (thickness 0.15)
//23101   │             )
//23102   │             (justify left mirror)
//23103   │         )
//23104   │     )

struct GrTextElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<GrTextElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: GrTextElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let at = sexp::utils::at(item.item).unwrap();
        let text = item.item.get(0).unwrap();
        let effects = Effects::from(item.item);
        let text_layer: String = item.item.value(el::LAYER).unwrap();

        if !PcbPlot::is_layer(item.item, layer) {
            //TODO stroke is not used
            let mut stroke = Stroke::new();
            stroke.linewidth = 0.1;
            stroke.linecolor = self.theme.layer_color(&[Style::from(text_layer)]);

            plot_items.push(PlotItem::Text(
                10,
                Text::new(
                    at, 0.0, text,
                    //self.theme.get_stroke(item.item.into(), &[Style::Wire]),
                    effects, false,
                ),
            ));
        }
        Ok(())
    }
}

struct FootprintElement<'a> {
    item: &'a Sexp,
}

impl FootprintElement<'_> {
    fn is_flipped(&self) -> bool {
        <Sexp as SexpValueQuery<String>>::value(self.item, el::LAYER).unwrap() == "B.Cu"
    }
}

impl<'a> PlotElement<FootprintElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: FootprintElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        //create a tmp element to fix the angle and mirror

        for element in item.item.nodes() {
            let name: &String = &element.name;
            if name == "fp_arc" {
                //(fp_arc
                //	(start -0.29 -1.235516)
                //	(mid 1.366487 -1.987659)
                //	(end 2.942335 -1.078608)
                //	(stroke
                //		(width 0.12)
                //		(type solid)
                //	)
                //	(layer "F.SilkS")
                //	(uuid "52502052-4743-4caa-863e-91187c15e848")
                //)
                let arc_start: Array1<f64> = element.value(el::GRAPH_START).unwrap();
                let arc_mid: Array1<f64> = element.value("mid").unwrap();
                let arc_end: Array1<f64> = element.value(el::GRAPH_END).unwrap();
                let width: f64 = stroke!(element);
                let act_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &act_layer) {
                    let mut stroke = Stroke::new();
                    stroke.linewidth = width;
                    stroke.linecolor = self.theme.layer_color(&[Style::from(act_layer)]);
                    plot_items.push(PlotItem::Arc(
                        100,
                        Arc::new(
                            Shape::transform(item.item, &arc_start),
                            Shape::transform(item.item, &arc_mid),
                            Shape::transform(item.item, &arc_end),
                            0.0,
                            None,
                            stroke,
                        ),
                    ));
                }
            } else if name == el::FP_LINE {
                let line_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &line_layer) {
                    let line_start: Array1<f64> = element.value(el::START).unwrap();
                    let line_end: Array1<f64> = element.value(el::END).unwrap();
                    let line_width = stroke!(element);

                    let mut stroke = Stroke::new();
                    stroke.linewidth = line_width;
                    stroke.linecolor = self.theme.layer_color(&[Style::from(line_layer)]);

                    plot_items.push(PlotItem::Line(
                        10,
                        Line::new(
                            Shape::transform_pad(
                                item.item,
                                item.is_flipped(),
                                0.0,
                                &arr2(&[
                                    [line_start[0], line_start[1]],
                                    [line_end[0], line_end[1]],
                                ]),
                            ),
                            stroke,
                            None,
                        ),
                    ));
                }
            } else if name == el::FP_POLY {
                let poly_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &poly_layer) {
                    let mut pts: Array2<f64> = Array2::zeros((0, 2));
                    for pt in element.query(el::PTS) {
                        for xy in pt.query(el::XY) {
                            pts.push_row(ArrayView::from(&[
                                xy.get(0).unwrap(),
                                xy.get(1).unwrap(),
                            ]))
                            .unwrap();
                        }
                    }
                    let line_width: f64 = stroke!(element);

                    let mut stroke = Stroke::new();
                    stroke.linewidth = line_width;
                    stroke.linecolor = self.theme.layer_color(&[Style::from(poly_layer)]);

                    plot_items.push(PlotItem::Polyline(
                        20,
                        Polyline::new(
                            Shape::transform_pad(item.item, item.is_flipped(), 0.0, &pts),
                            stroke,
                        ),
                    ));
                }
            } else if name == el::FP_CIRCLE {
                let center: Array1<f64> = element.value(el::CENTER).unwrap();
                let end: Array1<f64> = element.value(el::END).unwrap();
                let line_width = stroke!(element);
                let circle_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &circle_layer) {
                    let mut stroke = Stroke::new();
                    stroke.linewidth = line_width;
                    stroke.linecolor = self.theme.layer_color(&[Style::from(circle_layer)]);

                    let radius = Circle::radius(&center, &end);
                    plot_items.push(PlotItem::Circle(
                        1,
                        Circle::new(
                            Shape::transform_pad(item.item, item.is_flipped(), 0.0, &center),
                            radius,
                            stroke,
                        ),
                    ));
                }
            } else if name == el::FP_TEXT {
                //(fp_text user "KEEPOUT"
                //	(at 0 0 -90)
                //	(layer "Cmts.User")
                //	(uuid "76715af0-64ec-4e97-b724-005a93bf8b23")
                //	(effects
                //		(font
                //			(size 0.4 0.4)
                //			(thickness 0.051)
                //		)
                //	)
                //)

                let at = sexp::utils::at(item.item).unwrap();
                let angle = sexp::utils::angle(item.item).unwrap_or(0.0);
                let act_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &act_layer) {
                    let effects = Effects::from(item.item);
                    let text: String = element.get(1).unwrap();

                    plot_items.push(PlotItem::Text(
                        10,
                        Text::new(
                            Shape::transform(item.item, &at),
                            angle,
                            text,
                            effects,
                            false,
                        ),
                    ));
                }
            } else if name == el::PROPERTY {
                //(property "Reference" "J4"
                //	(at -4.13 -5.63 90)
                //	(layer "F.SilkS")
                //	(uuid "34c2fb4d-c21d-4d83-98dd-b5ff38696392")
                //	(effects
                //		(font
                //			(size 1 1)
                //			(thickness 0.15)
                //		)
                //	)
                //)

                //there are properties without a position
                if element.query(el::AT).next().is_none() {
                    continue;
                }
                let text: String = element.get(1).unwrap();

                let at = sexp::utils::at(element).unwrap();
                let angle = sexp::utils::angle(element).unwrap_or(0.0);
                let act_layer: String = element.value(el::LAYER).unwrap();

                if PcbPlot::is_layer_in(layer, &act_layer) {
                    let mut effects = Effects::from(item.item);
                    effects.font_color = self.theme.layer_color(&[Style::from(act_layer)]);

                    plot_items.push(PlotItem::Text(
                        10,
                        Text::new(
                            Shape::transform(item.item, &at),
                            angle,
                            text,
                            effects,
                            false,
                        ),
                    ));
                }
            } else if name == el::PAD {
                //(pad "TN" thru_hole circle
                //	(at 0 -3.38 180)
                //	(size 2.13 2.13)
                //	(drill 1.42)
                //	(layers "*.Cu" "*.Mask")
                //	(remove_unused_layers no)
                //	(net 28 "unconnected-(J5-PadTN)")
                //	(pintype "passive+no_connect")
                //	(uuid "e935c61c-e7a7-4f98-90a3-eede5998165e")
                //)

                let at = sexp::utils::at(element).unwrap();
                let angle = sexp::utils::angle(element).unwrap_or(0.0);

                let pad_type =
                    PadType::from(<Sexp as SexpValueQuery<String>>::get(element, 1).unwrap());
                let pad_shape =
                    PadShape::from(<Sexp as SexpValueQuery<String>>::get(element, 2).unwrap());

                let layers_node: &Sexp = element.query("layers").next().expect("expect layers");
                let layers: Vec<String> = layers_node.values();
                let pad_size: Array1<f64> = element.value(el::SIZE).unwrap();

                for act_layer in layers {
                    if PcbPlot::is_layer_in(layer, &act_layer) {
                        match pad_shape {
                        //match PadType::from(<Sexp as SexpValueQuery<String>>::get(element, 1).unwrap()) {
                            PadShape::Circle => {
                                let front = act_layer.starts_with("F."); //this should be set per footprint
                                if let PadType::ThruHole = pad_type {
                                    let sexp_drill = element.query(el::DRILL).next().unwrap();
                                    let drill = DrillHole::from(sexp_drill);

                                    let linewidth = pad_size[0] - drill.diameter;
                                    let mut stroke = Stroke::new();
                                    stroke.linewidth = linewidth;
                                    if layer.starts_with("F.") {
                                        stroke.fillcolor =
                                            self.theme.layer_color(&[Style::PadThroughHole]);
                                    } else {
                                        stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
                                    }

                                    plot_items.push(PlotItem::Circle(
                                        10,
                                        Circle::new(
                                            Shape::transform_pad(item.item, front, angle, &at),
                                            (pad_size[0] / 2.0) - linewidth / 2.0,
                                            stroke,
                                        ),
                                    ));
                                } else {
                                    warn!("unknown circle pad type {:?}", pad_type);
                                }
                            }
                            PadShape::Oval => {
                                //} else if pad_type == "thru_hole" && pad_sub_type == "oval" {
                                //(pad "" thru_hole oval
                                //	(at 0 -4.84 180)
                                //	(size 2.72 3.24)
                                //	(drill oval 1.1 1.8)
                                //	(layers "*.Cu" "*.Mask")
                                //	(remove_unused_layers no)
                                //	(uuid "a6691f96-af2b-47d4-b2be-bba45e234131")
                                //)
                                let at = sexp::utils::at(element).unwrap();
                                let sexp_drill = element.query(el::DRILL).next().unwrap();
                                let drill = DrillHole::from(sexp_drill);
                                let size: f64 = drill.diameter;
                                let drill: f64 = drill.width.unwrap_or(0.0);

                                let linewidth = size - drill;
                                let mut stroke = Stroke::new();
                                stroke.linewidth = linewidth;
                                if layer.starts_with("F.") {
                                    stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);
                                } else {
                                    stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
                                }

                                plot_items.push(PlotItem::Circle(
                                    10,
                                    Circle::new(
                                        Shape::transform_pad(item.item, item.is_flipped(), angle, &at),
                                        size - 2.0 * linewidth,
                                        stroke,
                                    ),
                                ));
                            }
                            PadShape::Rect => {
                                //} else if pad_type == "thru_hole" && pad_sub_type == "rect" {
                                //(pad "1" thru_hole rect
                                //	(at 0 0 180)
                                //	(size 1.8 1.8)
                                //	(drill 0.9)
                                //	(layers "*.Cu" "*.Mask")
                                //	(remove_unused_layers no)
                                //	(net 2 "GND")
                                //	(pinfunction "K")
                                //	(pintype "passive")
                                //	(uuid "1978304b-f521-4fe8-bb5e-3f65076a5c96")
                                //)
                                let at = sexp::utils::at(element).unwrap();
                                let size: Array1<f64> = element.value("size").unwrap();

                                let mut stroke = Stroke::new();
                                stroke.linewidth = 0.1;
                                if layer.starts_with("F.") {
                                    stroke.linecolor = self.theme.layer_color(&[Style::PadFront]);
                                } else {
                                    stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
                                }
                                stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);

                                let pts: Array2<f64> =
                                    arr2(&[[at[0], at[1]], [at[0] - size[0], at[1] - size[1]]]);
                                let pts = pts + arr1(&[size[0] / 2.0, size[1] / 2.0]);
                                plot_items.push(PlotItem::Rectangle(
                                    1,
                                    crate::Rectangle::new(
                                        Shape::transform_pad(item.item, item.is_flipped(), angle, &pts),
                                        stroke.clone(),
                                    ),
                                ));
                                let sexp_drill = element.query(el::DRILL).next().unwrap();
                                let drill = DrillHole::from(sexp_drill);
                                plot_items.push(PlotItem::Circle(
                                    10,
                                    Circle::new(
                                        Shape::transform(item.item, &at),
                                        drill.width.unwrap_or(0.0),
                                        stroke,
                                    ),
                                ));
                            }
                            PadShape::RoundRect => {
                                //} else if pad_sub_type == "roundrect" {
                                let at = sexp::utils::at(element).unwrap();
                                let size: Array1<f64> = element.value("size").unwrap();

                                let mut stroke = Stroke::new();
                                stroke.linewidth = 0.1;
                                if layer.starts_with("F.") {
                                    stroke.linecolor = self.theme.layer_color(&[Style::PadFront]);
                                } else {
                                    stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
                                }
                                stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);

                                let pts: Array2<f64> =
                                    arr2(&[[at[0], at[1]], [at[0] - size[0], at[1] - size[1]]]);
                                let pts = pts + arr1(&[size[0] / 2.0, size[1] / 2.0]);
                                plot_items.push(PlotItem::Rectangle(
                                    1,
                                    crate::Rectangle::new(
                                        Shape::transform_pad(item.item, item.is_flipped(), angle, &pts),
                                        stroke.clone(),
                                    ),
                                ));

                                if let PadType::ThruHole = pad_type {
                                    let sexp_drill = element.query(el::DRILL).next().unwrap();
                                    let drill = DrillHole::from(sexp_drill);
                                    plot_items.push(PlotItem::Circle(
                                        10,
                                        Circle::new(
                                            Shape::transform_pad(
                                                item.item,
                                                item.is_flipped(),
                                                angle,
                                                &at,
                                            ),
                                            drill.diameter / 2.0,
                                            stroke,
                                        ),
                                    ));
                                } else if !matches!(PadType::Smd, pad_type) {
                                    warn!("unknown roundrect pad type {:?}", pad_type);
                                }
                            }
                            _ => {
                                debug!("unknown pad shape {:?}", pad_shape);
                            }
                        }
                    }
                }
            } else if !SKIP_FP_ELEMENTS.contains(&name.as_str()) {
                log::trace!("Unknown footprint element: {:?}", name);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use ndarray::arr1;
    use sexp::{SexpParser, SexpTree};

    // parse drillhome
    #[test]
    fn test_parse_drill() {
        let drill_text = "(drill 2.2)";
        let sexp = SexpParser::from(drill_text.to_string());
        let tree = SexpTree::from(sexp.iter()).unwrap();
        let drill: super::DrillHole = super::DrillHole::from(tree.root().unwrap());
        assert!(!drill._oval);
        assert_eq!(drill.diameter, 2.2);
        assert_eq!(drill.width, None);
        assert_eq!(drill._offset, None);
    }
    #[test]
    fn test_parse_drill_oval() {
        let drill_text = "(drill oval 1.1 1.8)";
        let sexp = SexpParser::from(drill_text.to_string());
        let tree = SexpTree::from(sexp.iter()).unwrap();
        let drill: super::DrillHole = super::DrillHole::from(tree.root().unwrap());
        assert!(drill._oval);
        assert_eq!(drill.diameter, 1.1);
        assert_eq!(drill.width, Some(1.8));
        assert_eq!(drill._offset, None);
    }
    #[test]
    fn test_parse_drill_() {
        let drill_text = "(drill oval 1.1 1.8 (offset 1.0 2.0))";
        let sexp = SexpParser::from(drill_text.to_string());
        let tree = SexpTree::from(sexp.iter()).unwrap();
        let drill: super::DrillHole = super::DrillHole::from(tree.root().unwrap());
        assert!(drill._oval);
        assert_eq!(drill.diameter, 1.1);
        assert_eq!(drill.width, Some(1.8));
        assert_eq!(drill._offset, Some(arr1(&[1.0, 2.0])));
    }

    #[test]
    fn test_is_layer_in() {
        assert!(super::PcbPlot::is_layer_in("F.Cu", "F.Cu"));
        assert!(!super::PcbPlot::is_layer_in("F.Cu", "B.Cu"));
        assert!(super::PcbPlot::is_layer_in("F.Cu", "*.Cu"));
        assert!(!super::PcbPlot::is_layer_in("F.Cu", "*.SilkS"));
        assert!(!super::PcbPlot::is_layer_in("Edge.Cuts", "*.Cu"));
    }
    #[test]
    fn iterate_with_color() {}
}
