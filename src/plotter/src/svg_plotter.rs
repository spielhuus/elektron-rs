use std::io::Write;

use pathfinder_color::ColorU;
use pathfinder_content::{
    outline::{Contour, Outline},
    stroke::{OutlineStrokeToFill, StrokeStyle},
};
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};
use pathfinder_renderer::{
    paint::Paint,
    scene::{DrawPath, DrawPathId, Scene},
};

use log::trace;

use sexp::{schema::Color, Pt, Pts, Rect};

use crate::{plotter::{GraphicsState, Mirror, Plotter, TextState}, theme::{Effects, Stroke}, transform::Transform};

pub fn mirror(axis: &str) -> Transform2F {
    if axis == "x" {
        Transform2F::row_major(1.0, 0.0, 0.0, -1.0, 0.0, 0.0)
    } else if axis == "y" {
        Transform2F::row_major(-1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    } else if axis == "xy" {
        Transform2F::row_major(0.0, 1.0, 1.0, 0.0, 0.0, 0.0)
    } else {
        Transform2F::row_major(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }
}

pub struct SvgPlotter {
    items: Vec<Vec<u8>>,
    graphics_state: GraphicsState,
    text_state: TextState,

    scene: Scene,
    current_outline: Outline,
    current_contour: Contour,
    stack: Vec<(GraphicsState, TextState)>,
}

impl SvgPlotter {
    pub fn new(transform: Transform) -> Self {
        Self {
            items: Vec::new(),
            text_state: TextState::new(),
            graphics_state: GraphicsState {
                transform,
                mirror: None,
                stroke_style: Stroke::default(),
            },
            scene: Scene::new(),
            current_outline: Outline::new(),
            current_contour: Contour::new(),
            stack: Vec::new(),
        }
    }
}

impl Plotter for SvgPlotter {
    fn set_view_box(&mut self, view_box: Rect) {
        let rect = RectF::from_points(Vector2F::new(view_box[[0, 0]], view_box[[0, 1]]), Vector2F::new(view_box[[1, 0]], view_box[[1, 1]]));
        self.scene.set_view_box(rect);

        let white = self.scene.push_paint(&Paint::from_color(ColorU::white()));
        self.scene
            .push_draw_path(DrawPath::new(Outline::from_rect(rect), white));
    }

    fn mirror(&mut self, mirror: Option<Mirror>) {
        self.graphics_state.mirror = mirror;
    }

    fn move_to(&mut self, pt: Pt) {
        self.current_contour.push_endpoint(Vector2F::new(pt[0], pt[1]));
    }

    fn line_to(&mut self, pt: Pt) {
        self.current_contour.push_endpoint(Vector2F::new(pt[0], pt[1]));
    }

    fn close(&mut self) {
        self.current_contour.close();
    }

    fn polyline(&mut self, pts: Pts) {

    }

    fn rect(&mut self, rect: Rect) {
        let rect = RectF::from_points(Vector2F::new(rect[[0, 0]], rect[[0, 1]]), Vector2F::new(rect[[1, 0]], rect[[1, 1]]));
        println!("rect {:?}", rect);
        self.flush();
        self.current_outline.push_contour(Contour::from_rect(rect));
    }

    fn circle(&mut self, center: Pt, diameter: f32) {
        self.flush();
        let transform =
            Transform2F::from_translation(Vector2F::new(center[0], center[1])).scale(Vector2F::new(diameter, diameter));
        self.current_contour.push_ellipse(&transform);
    }

    fn text(&mut self, text: &str, pt: Pt, effects: Effects) {
        //let res = self.graphics_state.transform * pt;
        //trace!("text pt: {:?} {:?}", pt, res);

        let mut buffer = Vec::<u8>::new();
        write!(buffer, "<text x=\"{}\" y=\"{}\" fill=\"red\" font-size=\"{}\" >{}</text>", pt[0], pt[1], effects.size, text).unwrap();
        self.items.push(buffer);
    }

    fn flush(&mut self) {
        if !self.current_contour.is_empty() {
            self.current_outline
                .push_contour(self.current_contour.clone());
            self.current_contour.clear();
        }
    }

    fn transform(&mut self, matrix: Transform) {
        self.graphics_state.transform = matrix; //TODO what to do here
    }

    fn stroke(&mut self, stroke: Stroke) {
        self.flush();

        //self.backend.draw(&self.current_outline, mode, fill_rule, self.graphics_state.transform, self.graphics_state.clip_path_id);
        //
        //

        //let dashed = OutlineDash::new(&self.current_outline, &[], 0.0).into_outline();
        //let mut stroke = OutlineStrokeToFill::new(&dashed, StrokeStyle::default());
        //stroke.offset();
        //let stroke = stroke.into_outline();

        //let mut stroke_style = StrokeStyle::{ line_width: stroke.linewidth, ..Default};
        let mut stroke_style = StrokeStyle {
            line_width: stroke.linewidth,
            ..StrokeStyle::default()
        };
        stroke_style.line_width = stroke.linewidth;
        let mut plot_stroke = OutlineStrokeToFill::new(&self.current_outline, stroke_style);
        plot_stroke.offset();
        let plot_stroke = plot_stroke.into_outline();

        let paint = Paint::from_color(stroke.linecolor.convert());
        let paint_id = self.scene.push_paint(&paint);
        trace!("        transform: {:?}", self.graphics_state.transform);

        if let Some(mirror) = &self.graphics_state.mirror {
            trace!("      found mirror: {}", mirror);
        }

        let draw_path = DrawPath::new(
            plot_stroke, //TODO.transformed(&self.graphics_state.transform),
            paint_id,
        ); //(contour.transformed(&transform), paint);
           //draw_path.set_clip_path(clip);
           //draw_path.set_fill_rule(fill_rule);
           //draw_path.set_blend_mode(blend_mode(stroke.mode));

        let paint = self.scene.get_paint(draw_path.paint);

        let mut buffer = Vec::<u8>::new();
        write!(buffer, "<path").unwrap();
        if !draw_path.name.is_empty() {
            write!(buffer, " id=\"{}\"", draw_path.name).unwrap();
        }
        writeln!(
            buffer,
            " fill=\"{:?}\" d=\"{:?}\" />",
            paint.base_color(),
            draw_path.outline
        ).unwrap();

        self.items.push(buffer);

        self.scene.push_draw_path(draw_path);
        self.current_outline.clear();
    }


    fn write<W: Write>(self, writer: &mut W) -> std::io::Result<()> {
        let view_box = self.scene.view_box();
        writeln!(
            writer,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\">",
            view_box.origin().x(),
            view_box.origin().y(),
            view_box.size().x(),
            view_box.size().y()
        )?;
        for item in self.items {
            writer.write_all(&item).unwrap();
        }
        writeln!(writer, "</svg>")?;
        Ok(())
    }
    fn save(&mut self) {
        self.stack.push((self.graphics_state.clone(), self.text_state.clone()));
    }
    fn restore(&mut self) {
        let (gs, ts) = self.stack.pop().unwrap(); //TODO check first
        self.graphics_state = gs;
        self.text_state = ts;
    }
}

trait Convert<F, T> {
    fn convert(&self) -> T;
}

impl Convert<Color, ColorU> for Color {
    fn convert(&self) -> ColorU {
        ColorU::new(self.r, self.g, self.b, (self.a * 255.0) as u8)
    }
}
