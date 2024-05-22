use std::io::Write;

use pathfinder_color::ColorU;
use pathfinder_content::{outline::{Contour, Outline}, stroke::{OutlineStrokeToFill, StrokeStyle}};
use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};
use pathfinder_renderer::{paint::Paint, scene::{DrawPath, Scene}};
use pathfinder_export::{Export, FileFormat};
use sexp::schema::Color;

use crate::{plotter::Plotter, theme::Stroke};

trait Convert<F, T> {
    fn convert(&self) -> T;
}

impl Convert<Color, ColorU> for Color {
    fn convert(&self) -> ColorU {
        ColorU::new(self.r, self.g, self.b, (self.a * 255.0) as u8)
    }
}


pub struct PathfinderPlotter {
    graphics_state: GraphicsState,

    scene: Scene,
    current_outline: Outline,
    current_contour: Contour,
}

impl PathfinderPlotter {
    pub fn new(transform: Transform2F) -> Self {
        Self {
            graphics_state: GraphicsState {
                transform,
                stroke_style: StrokeStyle::default(),
            },
            scene: Scene::new(),
            current_outline: Outline::new(),
            current_contour: Contour::new(),
        }
    }
}

impl Plotter for PathfinderPlotter {
    fn set_view_box(&mut self, view_box: RectF) {
        self.scene.set_view_box(view_box);

        let white = self.scene.push_paint(&Paint::from_color(ColorU::white()));
        self.scene.push_draw_path(DrawPath::new(Outline::from_rect(view_box), white));

    }

    fn move_to(&mut self, pt: Vector2F) {
        self.current_contour.push_endpoint(pt);
    }

    fn line_to(&mut self, pt: Vector2F) {
        self.current_contour.push_endpoint(pt);
    }

    fn close(&mut self) {
        self.current_contour.close();
    }

    fn rect(&mut self, rect: RectF) {
        self.flush();
        self.current_outline
            .push_contour(Contour::from_rect(rect));
    }

    fn circle(&mut self, center: Vector2F, diameter: f32) {
        self.flush();
        let transform = Transform2F::from_translation(center)
            .scale(Vector2F::new(diameter, diameter));
        self.current_contour.push_ellipse(&transform);
    }

    fn text(&mut self, text: &str) {
    }

    fn flush(&mut self) {
        if !self.current_contour.is_empty() {
            self.current_outline.push_contour(self.current_contour.clone());
            self.current_contour.clear();
        }
    }

    fn transform(&mut self, matrix: Transform2F) {
        self.graphics_state.transform = matrix;
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
        let mut stroke_style = StrokeStyle{ line_width: stroke.linewidth, ..StrokeStyle::default()};
        stroke_style.line_width  = stroke.linewidth;
        let mut plot_stroke = OutlineStrokeToFill::new(&self.current_outline, stroke_style);
        plot_stroke.offset();
        let plot_stroke = plot_stroke.into_outline();


        let paint = Paint::from_color(stroke.linecolor.convert());
        let paint_id = self.scene.push_paint(&paint);
        let draw_path = DrawPath::new(plot_stroke.transformed(&self.graphics_state.transform), paint_id); //(contour.transformed(&transform), paint);
        //draw_path.set_clip_path(clip);
        //draw_path.set_fill_rule(fill_rule);
        //draw_path.set_blend_mode(blend_mode(stroke.mode));

        self.scene.push_draw_path(draw_path);
        self.current_outline.clear();
    }

    fn write<W: Write>(self, writer: &mut W) -> std::io::Result<()> {
        //let format = match file.extension().and_then(|s| s.to_str()) {
        //    Some("pdf") => FileFormat::PDF,
        //    Some("ps") => FileFormat::PS,
        //    Some("svg") => FileFormat::SVG,
        //    _ => panic!("output filename must have .ps or .pdf extension")
        //};
       self.scene.export(writer, FileFormat::SVG).unwrap();
    }
}

pub struct GraphicsState {
    pub transform: Transform2F,
    pub stroke_style: StrokeStyle,

    //pub fill_color: Fill,
    //pub fill_color_alpha: f32,
    //pub fill_paint: Option<PaintId>,
    //pub stroke_color: Fill,
    //pub stroke_color_alpha: f32,
    //pub stroke_paint: Option<PaintId>,
    //pub clip_path_id: Option<B::ClipPathId>,
    //pub clip_path: Option<ClipPath>,
    //pub clip_path_rect: Option<RectF>,
    //pub fill_color_space: &'a ColorSpace,
    //pub stroke_color_space: &'a ColorSpace,
    //pub dash_pattern: Option<(&'a [f32], f32)>,
    //
    //pub stroke_alpha: f32,
    //pub fill_alpha: f32,
    //
    //pub overprint_fill: bool,
    //pub overprint_stroke: bool,
    //pub overprint_mode: i32,
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::plotter::Plotter;
    use pathfinder_geometry::{rect::RectF, transform2d::Transform2F, vector::Vector2F};

    #[test]
    fn test_plot() {
        let mut plotter = super::PathfinderPlotter::new(Transform2F::default());
        plotter.set_view_box(RectF::new(
            Vector2F::new(0.0, 0.0),
            Vector2F::new(297.0, 210.0),
        ));
        plotter.move_to(Vector2F::new(50.0, 100.0));
        plotter.move_to(Vector2F::new(75.0, 50.0));
        plotter.move_to(Vector2F::new(100.0, 100.0));
        plotter.close();
        plotter.stroke(crate::theme::Stroke::new());

        plotter.transform(Transform2F::from_translation(Vector2F::new(150.0, 70.0)).rotate(45.0));

        plotter.move_to(Vector2F::new(0.0, 0.0));
        plotter.move_to(Vector2F::new(0.0, 20.0));
        plotter.move_to(Vector2F::new(20.0, 20.0));
        plotter.move_to(Vector2F::new(20.0, 0.0));
        plotter.close();
        plotter.stroke(crate::theme::Stroke::new());

        let mut file = File::create("test.svg").unwrap();
        plotter.write(&mut file);
    }
}
