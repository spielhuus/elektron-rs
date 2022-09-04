mod border;
mod cairo_plotter;
mod schema;
mod theme;

pub use self::cairo_plotter::{CairoPlotter, ImageType, PlotItem, Plotter};
pub use self::schema::{PlotIterator, SchemaPlot};
pub use self::theme::{Theme, Themer};

macro_rules! text {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr) => {
        PlotItem::Text(
            99,
            Text::new(
                $pos,
                $angle,
                $content,
                $effects.color.clone(),
                $effects.font_size.0,
                $effects.font.as_str(),
                $effects.justify.clone(),
            ),
        )
    };
}
pub(crate) use text;
