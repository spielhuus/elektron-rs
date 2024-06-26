//! Plot the PCB
use std::path::Path;

use crate::LineCap;
use crate::{
    border, error::Error, schema::Themer, Arc, Circle, Effects, Line, Outline, PlotItem,
    PlotterImpl, Polyline, Stroke, Style, Text, Theme,
};
use log::*;
use log::{debug, error, warn};
use ndarray::{arr1, arr2, s, Array, Array1, Array2, ArrayView};

use regex::Regex;
use sexp::round;
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
    Unknown,
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
            _ => PadShape::Unknown,
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
    name: String,
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
        self.name = name.to_string();
        self
    }
    pub fn layers(mut self, layers: Vec<String>) -> Self {
        self.layers = layers;
        self
    }
    /// create a new SchemaPlot with defalt values.
    pub fn new() -> Self {
        Self {
            theme: Themer::new(Theme::default()),
            border: true,
            scale: 1.0,
            name: String::from("none"),
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
                    if name != "B.Fab" && name != "F.Fab" {
                        self.layers.push(name);
                    }
                }
            }
        }
        trace!("layers: {:?}", self.layers);
    }

    fn get_border(&self) -> Result<Array2<f64>, Error> {
        let mut array = Vec::new();
        let mut rows: usize = 0;
        if let Some(tree) = &self.tree {
            for element in tree.root().unwrap().nodes() {
                if element.name == el::GR_LINE {
                    let line_layer: String = element.value(el::LAYER).unwrap();
                    if line_layer == "Edge.Cuts" {
                        let start: Array1<f64> = element.value(el::START).unwrap();
                        let end: Array1<f64> = element.value(el::END).unwrap();
                        array.extend_from_slice(&[start[0], start[1]]);
                        array.extend_from_slice(&[end[0], end[1]]);
                        rows += 2;
                    }
                }
            }
        }

        if rows > 0 {
            let boundery: Array2<f64>;
            let array = Array::from_shape_vec((rows, 2), array).unwrap();
            let axis1 = array.slice(s![.., 0]);
            let axis2 = array.slice(s![.., 1]);
            boundery = arr2(&[
                [
                    *axis1
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        //.min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
                [
                    *axis1
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
            ]);
            Ok(boundery)
        } else {
            Err(Error(
                "no border found, maybe the PCB does not have Edge.Cuts".to_string(),
            ))
        }
    }

    pub fn open(&mut self, path: &Path) -> Result<Vec<String>, Error> {
        debug!("open pcb: {}", path.to_str().unwrap());
        if let Some(dir) = std::path::Path::new(&path).parent() {
            self.path = dir.to_str().unwrap().to_string();
        }
        let Ok(document) = SexpParser::load(path.to_str().unwrap()) else {
            //TODO also use Path
            return Err(Error(format!(
                "could not load file: {}",
                path.to_str().unwrap()
            )));
        };
        let tree = SexpTree::from(document.iter())?;
        self.tree = Some(tree);
        if self.layers.is_empty() {
            self.get_layers();
        }
        Ok(self.layers.clone())
    }

    pub fn write(&self, plotter: &mut dyn PlotterImpl, layers: Vec<String>) -> Result<(), Error> {
        let tree = if let Some(tree) = &self.tree {
            tree.clone()
        } else {
            return Err(Error("no root schema loaded".into()));
        };

        let paper_size: (f64, f64) =
            <Sexp as SexpValueQuery<PaperSize>>::value(tree.root().unwrap(), "paper")
                .unwrap()
                .into();

        //TODO handle portraint and landscape

        let mut plot_items = Vec::<PlotItem>::new();
        for layer in &layers {
            //check if layer exists
            if !self.layers.contains(layer) {
                return Err(Error(format!("layer {} not found", layer)));
            }
            plot_items.append(&mut self.parse_items(&tree, layer)?);
        }

        let size = if self.border {
            arr2(&[[0.0, 0.0], [paper_size.0, paper_size.1]])
        } else {
            //when the border is not plotted, the plotter will just use the default bounds
            let rect = self.get_border()?;
            let x = rect[[0, 0]];
            let y = rect[[0, 1]];
            let offset = arr1(&[x, y]);
            for item in plot_items.iter_mut() {
                match item {
                    PlotItem::Arc(_, arc) => {
                        arc.start = round!(arc.start.clone() - &offset);
                        arc.end = round!(arc.end.clone() - &offset);
                        arc.mid = round!(arc.mid.clone() - &offset);
                        arc.center = round!(arc.center.clone() - &offset);
                    }
                    PlotItem::Circle(_, circle) => {
                        circle.pos = round!(circle.pos.clone() - &offset)
                    }
                    PlotItem::Line(_, line) => line.pts = round!(line.pts.clone() - &offset),
                    PlotItem::Rectangle(_, rect) => rect.pts = round!(rect.pts.clone() - &offset),
                    PlotItem::Polyline(_, poly) => poly.pts = round!(poly.pts.clone() - &offset),
                    PlotItem::Text(_, text) => text.pos = round!(text.pos.clone() - &offset),
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

macro_rules! class {
    ($name:expr, $layer:expr) => {
        Some(format!("{}_{}", $name, $layer.replace('.', "_")))
    };
}

trait PlotElement<T> {
    fn plot(&self, item: T, layer: &str, plot_items: &mut Vec<PlotItem>) -> Result<(), Error>;
}
trait PlotPad<T> {
    fn plot_pad(
        &self,
        item: T,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error>;
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

            let line_layer: String = item.item.value(el::LAYER).unwrap();
            let mut stroke = Stroke::new();
            stroke.linewidth = width;
            stroke.linecolor = self.theme.layer_color(&[Style::from(line_layer.clone())]);

            plot_items.push(PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[start[0], start[1]], [end[0], end[1]]]),
                    stroke,
                    None,
                    class!(self.name, line_layer),
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
            let color = self.theme.layer_color(&[Style::from(layer.clone())]);
            stroke.linecolor = color.clone();
            stroke.fillcolor = color;

            plot_items.push(PlotItem::Polyline(
                20,
                Polyline::new(pts, stroke, Some(LineCap::Square), class!(self.name, layer)),
            ));
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
            plot_items.push(PlotItem::Circle(
                1,
                Circle::new(center, radius, stroke, class!(self.name, layer)),
            ));
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
        for polygon in item.item.query(el::FILLED_POLYGON) {
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
                let color = self.theme.layer_color(&[Style::from(layer.clone())]);
                stroke.linecolor = color.clone();
                stroke.fillcolor = color;

                plot_items.push(PlotItem::Polyline(
                    20,
                    Polyline::new(pts, stroke, Some(LineCap::Round), class!(self.name, layer)),
                ));
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
                    class!(self.name, layer),
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
                    Circle::new(
                        at.clone(),
                        size - linewidth,
                        stroke,
                        class!(self.name, layer),
                    ),
                ));
            }
        }
        Ok(())
    }
}

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
                Text::new(at, 0.0, text, effects, class!(self.name, layer)),
            ));
        }
        Ok(())
    }
}

#[inline]
fn is_flipped(item: &Sexp) -> bool {
    <Sexp as SexpValueQuery<String>>::value(item, el::LAYER).unwrap() == "B.Cu"
}

struct FootprintElement<'a> {
    item: &'a Sexp,
}

impl<'a> PlotElement<FootprintElement<'a>> for PcbPlot<'a> {
    fn plot(
        &self,
        item: FootprintElement,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        for element in item.item.nodes() {
            let name: &String = &element.name;
            if name == el::FP_ARC {
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
                            class!(self.name, layer),
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
                                is_flipped(item.item),
                                None,
                                &arr2(&[
                                    [line_start[0], line_start[1]],
                                    [line_end[0], line_end[1]],
                                ]),
                            ),
                            stroke,
                            None,
                            class!(self.name, layer),
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
                            Shape::transform_pad(item.item, is_flipped(item.item), None, &pts),
                            stroke,
                            Some(LineCap::Round),
                            class!(self.name, layer),
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
                            Shape::transform_pad(item.item, is_flipped(item.item), None, &center),
                            radius,
                            stroke,
                            class!(self.name, layer),
                        ),
                    ));
                }
            } else if name == el::FP_TEXT {
                let text_layer: String = element.value(el::LAYER).unwrap();
                if PcbPlot::is_layer_in(layer, &text_layer) {
                    let at = sexp::utils::at(element).unwrap();
                    let angle = sexp::utils::angle(element).unwrap_or(0.0);
                    let mut effects = Effects::from(element);
                    effects.font_color = self.theme.layer_color(&[Style::from(text_layer)]);
                    let text: String = element.get(1).unwrap();

                    plot_items.push(PlotItem::Text(
                        10,
                        Text::new(
                            Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                            angle,
                            text,
                            effects,
                            class!(self.name, layer),
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
                //skip when the property is hidden
                if let Some(element) = element.query(el::HIDE).next() {
                    if <Sexp as SexpValueQuery<String>>::get(element, 0).unwrap() == "yes" {
                        continue;
                    }
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
                            Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                            angle,
                            text,
                            effects,
                            class!(self.name, layer),
                        ),
                    ));
                }
            } else if name == el::PAD {
                let pad_shape =
                    PadShape::from(<Sexp as SexpValueQuery<String>>::get(element, 2).unwrap());

                let layers_node: &Sexp = element.query("layers").next().expect("expect layers");
                let layers: Vec<String> = layers_node.values();
                for act_layer in layers {
                    if PcbPlot::is_layer_in(layer, &act_layer) {
                        match pad_shape {
                            PadShape::Circle => {
                                self.plot_pad(
                                    PadCircle { item: item.item },
                                    element,
                                    &act_layer,
                                    plot_items,
                                )?;
                            }
                            PadShape::Oval => {
                                self.plot_pad(
                                    PadOval { item: item.item },
                                    element,
                                    &act_layer,
                                    plot_items,
                                )?;
                            }
                            PadShape::Rect => {
                                self.plot_pad(
                                    PadRect { item: item.item },
                                    element,
                                    &act_layer,
                                    plot_items,
                                )?;
                            }
                            PadShape::RoundRect => {
                                self.plot_pad(
                                    PadRoundRect { item: item.item },
                                    element,
                                    &act_layer,
                                    plot_items,
                                )?;
                            }
                            PadShape::Custom => {
                                self.plot_pad(
                                    PadCustom { item: item.item },
                                    element,
                                    &act_layer,
                                    plot_items,
                                )?;
                            }
                            _ => {
                                warn!("unknown pad shape {:?}", pad_shape);
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

struct PadCustom<'a> {
    item: &'a Sexp,
}

impl<'a> PlotPad<PadCustom<'a>> for PcbPlot<'a> {
    fn plot_pad(
        &self,
        item: PadCustom,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let primitives = element.query(el::PRIMITIVES).next().unwrap();
        for c_item in primitives.nodes() {
            match c_item.name.as_str() {
                //el::SEGMENT => self.plot(SegmentElement { item }, layer, plot_items)?,
                //el::GR_LINE => {
                el::GR_POLY => {
                    let mut pts: Array2<f64> = Array2::zeros((0, 2));
                    //if PcbPlot::is_layer(item.item, layer) {
                    for pt in c_item.query(el::PTS) {
                        for xy in pt.query(el::XY) {
                            pts.push_row(ArrayView::from(&[
                                xy.get(0).unwrap(),
                                xy.get(1).unwrap(),
                            ]))
                            .unwrap();
                        }
                    }
                    let mut stroke = Stroke::new();
                    trace!("custom pad on layer: {}", layer);
                    let color = self.theme.layer_color(&[Style::from(layer.to_string())]);
                    stroke.linecolor = color.clone();
                    if let Some(linewidth) = <Sexp as SexpValueQuery<f64>>::value(c_item, "width") {
                        stroke.linewidth = linewidth;
                    }
                    if let Some(fill) = <Sexp as SexpValueQuery<String>>::value(c_item, "fill") {
                        if fill == "yes" {
                            stroke.fillcolor = color;
                        }
                    }
                    plot_items.push(PlotItem::Polyline(
                        20,
                        Polyline::new(
                            Shape::transform_pad(item.item, is_flipped(item.item), None, &pts),
                            stroke,
                            Some(LineCap::Round),
                            class!(self.name, layer),
                        ),
                    ));
                }
                _ => {
                    if log_enabled!(Level::Error) && !SKIP_ELEMENTS.contains(&c_item.name.as_str())
                    {
                        error!("unparsed custom pad node: {}", c_item.name);
                    }
                }
            }
        }
        Ok(())
    }
}

struct PadRoundRect<'a> {
    item: &'a Sexp,
}

impl<'a> PlotPad<PadRoundRect<'a>> for PcbPlot<'a> {
    fn plot_pad(
        &self,
        item: PadRoundRect,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let at = sexp::utils::at(element).unwrap();
        let size: Array1<f64> = element.value("size").unwrap();
        let pad_type = PadType::from(<Sexp as SexpValueQuery<String>>::get(element, 1).unwrap());
        let rx: f64 = element.value("roundrect_rratio").unwrap();

        let mut stroke = Stroke::new();
        stroke.linewidth = 0.1;
        if layer.starts_with("F.") {
            stroke.linecolor = self.theme.layer_color(&[Style::PadFront]);
        } else {
            stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
        }
        stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);

        let pts: Array2<f64> = arr2(&[[at[0], at[1]], [at[0] + size[0], at[1] + size[1]]]);
        let pts = pts - arr1(&[size[0] / 2.0, size[1] / 2.0]); //TODO Reealy?
        plot_items.push(PlotItem::Rectangle(
            1,
            crate::Rectangle::new(
                Shape::transform_pad(item.item, is_flipped(item.item), None, &pts),
                Some(size[0] * rx),
                stroke.clone(),
                class!(self.name, layer),
            ),
        ));

        if let PadType::ThruHole = pad_type {
            let sexp_drill = element.query(el::DRILL).next().unwrap();
            let drill = DrillHole::from(sexp_drill);
            plot_items.push(PlotItem::Circle(
                10,
                Circle::new(
                    Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                    drill.diameter / 2.0,
                    stroke,
                    class!(self.name, layer),
                ),
            ));
        } else if !matches!(pad_type, PadType::Smd) {
            warn!("unknown roundrect pad type {:?}", pad_type);
        }
        Ok(())
    }
}

struct PadRect<'a> {
    item: &'a Sexp,
}

impl<'a> PlotPad<PadRect<'a>> for PcbPlot<'a> {
    fn plot_pad(
        &self,
        item: PadRect,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let at = sexp::utils::at(element).unwrap();
        let size: Array1<f64> = element.value("size").unwrap();
        let pad_type = PadType::from(<Sexp as SexpValueQuery<String>>::get(element, 1).unwrap());

        let mut stroke = Stroke::new();
        stroke.linewidth = 0.1;
        if layer.starts_with("F.") {
            stroke.linecolor = self.theme.layer_color(&[Style::PadFront]);
        } else {
            stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
        }
        stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);

        let pts: Array2<f64> = arr2(&[[at[0], at[1]], [at[0] - size[0], at[1] - size[1]]]);
        let pts = pts + arr1(&[size[0] / 2.0, size[1] / 2.0]);
        plot_items.push(PlotItem::Rectangle(
            1,
            crate::Rectangle::new(
                Shape::transform_pad(item.item, is_flipped(item.item), None, &pts),
                None,
                stroke.clone(),
                class!(self.name, layer),
            ),
        ));
        if let PadType::ThruHole = pad_type {
            let sexp_drill = element.query(el::DRILL).next().unwrap();
            let drill = DrillHole::from(sexp_drill);
            plot_items.push(PlotItem::Circle(
                10,
                Circle::new(
                    Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                    drill.width.unwrap_or(0.0),
                    stroke,
                    class!(self.name, layer),
                ),
            ));
        } else if !matches!(pad_type, PadType::Smd) {
            warn!("unknown pad type for rect {:?}", pad_type);
        }
        Ok(())
    }
}

struct PadCircle<'a> {
    item: &'a Sexp,
}

impl<'a> PlotPad<PadCircle<'a>> for PcbPlot<'a> {
    fn plot_pad(
        &self,
        item: PadCircle,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let pad_size: Array1<f64> = element.value(el::SIZE).unwrap();
        let pad_type = PadType::from(<Sexp as SexpValueQuery<String>>::get(element, 1).unwrap());
        let at = sexp::utils::at(element).unwrap();

        if let PadType::ThruHole | PadType::NpThruHole = pad_type {
            let sexp_drill = element.query(el::DRILL).next().unwrap();
            let drill = DrillHole::from(sexp_drill);

            let linewidth = pad_size[0] - drill.diameter;
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
                    Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                    (pad_size[0] / 2.0) - linewidth / 2.0,
                    stroke,
                    class!(self.name, layer),
                ),
            ));
        } else if let PadType::Connect = pad_type {
            let mut stroke = Stroke::new();
            stroke.linewidth = 0.0;
            stroke.fillcolor = self.theme.layer_color(&[Style::from(layer.to_string())]); //TODO was act_layer

            plot_items.push(PlotItem::Circle(
                10,
                Circle::new(
                    Shape::transform_pad(item.item, is_flipped(item.item), None, &at),
                    pad_size[0] / 2.0,
                    stroke,
                    class!(self.name, layer),
                ),
            ));
        } else {
            warn!("unknown circle pad type {:?}", pad_type);
        }
        Ok(())
    }
}

struct PadOval<'a> {
    item: &'a Sexp,
}

impl<'a> PlotPad<PadOval<'a>> for PcbPlot<'a> {
    fn plot_pad(
        &self,
        item: PadOval,
        element: &Sexp,
        layer: &str,
        plot_items: &mut Vec<PlotItem>,
    ) -> Result<(), Error> {
        let at = sexp::utils::at(element).unwrap();
        let mut size: Array1<f64> = element.value("size").unwrap();
        if size[0] == size[1] {
            //shape is a circle
            return self.plot_pad(PadCircle { item: item.item }, element, layer, plot_items);
        }

        if size[0] > size[1] {
            (size[0], size[1]) = (size[1], size[0]);
        }

        let angle = sexp::utils::angle(element);
        let mut stroke = Stroke::new();
        stroke.linewidth = 0.1;
        if layer.starts_with("F.") {
            stroke.fillcolor = self.theme.layer_color(&[Style::PadThroughHole]);
        } else {
            stroke.linecolor = self.theme.layer_color(&[Style::PadBack]);
        }

        let deltaxy = size[1] - size[0]; /* distance between centers of the oval */
        let radius = size[0] / 2.0;
        let half_height = deltaxy / 2.0;

        let points = arr2(&[
            [[-half_height, radius], [-half_height, -radius]], // the line
            [[-size[1] / 2.0, 0.0], [size[1] / 2.0, 0.0]],
            [[half_height, radius], [half_height, -radius]], // the line
        ]);

        let mut fill_stroke = stroke.clone();
        fill_stroke.linecolor = fill_stroke.fillcolor.clone();
        plot_items.push(PlotItem::Rectangle(
            1,
            crate::Rectangle::new(
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr2(&[points[[0, 0]], points[[2, 1]]]) + &at),
                ),
                None,
                fill_stroke,
                class!(self.name, layer),
            ),
        ));
        plot_items.push(PlotItem::Line(
            90,
            Line::new(
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr2(&[points[[0, 0]], points[[2, 0]]]) + &at),
                ),
                stroke.clone(),
                None,
                class!(self.name, layer),
            ),
        ));
        plot_items.push(PlotItem::Line(
            90,
            Line::new(
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr2(&[points[[0, 1]], points[[2, 1]]]) + &at),
                ),
                stroke.clone(),
                None,
                class!(self.name, layer),
            ),
        ));
        plot_items.push(PlotItem::Arc(
            100,
            Arc::new(
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[0, 0]]) + &at),
                ),
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[1, 0]]) + &at),
                ),
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[0, 1]]) + &at),
                ),
                0.0,
                None,
                stroke.clone(),
                class!(self.name, layer),
            ),
        ));

        plot_items.push(PlotItem::Arc(
            100,
            Arc::new(
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[2, 1]]) + &at),
                ),
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[1, 1]]) + &at),
                ),
                Shape::transform_pad(
                    item.item,
                    is_flipped(item.item),
                    angle,
                    &(arr1(&points[[2, 0]]) + &at),
                ),
                0.0,
                None,
                stroke,
                class!(self.name, layer),
            ),
        ));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

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
    fn test_get_border() {
        let mut pcb = super::PcbPlot::default();
        pcb.open(Path::new("tests/cp3.kicad_pcb")).unwrap();
        let border = pcb.get_border().unwrap();
        assert_eq!(border, ndarray::arr2(&[[50.8, 50.8], [91.1, 158.98]]));
    }
}
