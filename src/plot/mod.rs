
mod cairo_plotter;
mod schema;
mod theme;
mod border;

pub use self::schema::{SchemaPlot, PlotIterator};
pub use self::cairo_plotter::{ImageType, Plotter, PlotItem, CairoPlotter};
pub use self::theme::{Theme, Themer};

macro_rules! text {
    ($pos:expr, $angle:expr, $content:expr, $effects:expr) => {
        PlotItem::TextItem(
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
