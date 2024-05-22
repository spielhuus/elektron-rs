use std::io::Write;

use sexp::{Pt, Pts, Rect};

use crate::{theme::{Effects, Stroke}, transform::Transform};

#[derive(Debug, Clone)]
pub enum Mirror {
    X,
    Y,
    XY,
}

//impl display trait for Mirror
impl std::fmt::Display for Mirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mirror='{}'", match self {
            Mirror::X => "x",
            Mirror::Y => "y",
            Mirror::XY => "xy",
        })
    }
}

impl Mirror {
    pub fn from(s: String) -> Option<Self> {
        match s.as_str() {
            "x" => Some(Mirror::X),
            "y" => Some(Mirror::Y),
            "xy" => Some(Mirror::XY),
            _ => None,
        }
    }
    //fn transform(&self) -> Transform {
    //    match self {
    //        Mirror::X => Transform2F::row_major(1.0, 0.0, 0.0, -1.0, 0.0, 0.0),
    //        Mirror::Y => Transform2F::row_major(-1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
    //        Mirror::XY => Transform2F::row_major(0.0, 1.0, 1.0, 0.0, 0.0, 0.0),
    //    }
    //}
}

pub trait Plotter {

    fn write<W: Write>(self, writer: &mut W) -> std::io::Result<()>;

    fn set_view_box(&mut self, rect: Rect); 

    fn move_to(&mut self, pt: Pt); 
    fn line_to(&mut self, pt: Pt); 
    fn close(&mut self); 


    fn polyline(&mut self, pts: Pts); 
    fn rect(&mut self, r: Rect); 
    fn circle(&mut self, center: Pt, radius: f32); 
    fn text(&mut self, text: &str, pt: Pt, effects: Effects);

    fn mirror(&mut self, mirror: Option<Mirror>);
    fn transform(&mut self, transform: Transform); 
    fn stroke(&mut self, stroke: Stroke); 

    fn save(&mut self);
    fn restore(&mut self);
    
    fn flush(&mut self); 
}

#[derive(Debug, Clone)]
pub struct GraphicsState {
    pub transform: Transform,
    pub mirror: Option<Mirror>,
    pub stroke_style: Stroke,

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

#[derive(Clone)]
pub struct TextState {
    pub text_matrix: Transform, // tracks current glyph
    pub line_matrix: Transform, // tracks current line
    pub char_space: f32, // Character spacing
    pub word_space: f32, // Word spacing
    pub horiz_scale: f32, // Horizontal scaling
    pub leading: f32, // Leading
    //pub font_entry: Option<Arc<FontEntry>>, // Text font
    pub font_size: f32, // Text font size
    //pub mode: TextMode, // Text rendering mode
    pub rise: f32, // Text rise
    pub knockout: f32, //Text knockout
}
impl TextState {
    pub fn new() -> TextState {
        TextState {
            text_matrix: Transform::default(),
            line_matrix: Transform::default(),
            char_space: 0.,
            word_space: 0.,
            horiz_scale: 1.,
            leading: 0.,
            //font_entry: None,
            font_size: 0.,
            //mode: TextMode::Fill,
            rise: 0.,
            knockout: 0.
        }
    }
}


#[cfg(test)]
mod test    {
    use nalgebra::Matrix2;
    
    #[test]
    fn test_nalgebra() {
        //let mat = Matrix2::new(1.0, 1.0,
        //               1.0, -1.0,
        //               -1.0, -1.0,
        //               -1.0, 1.0,
        //               1.0, 1.0);
    }
}
