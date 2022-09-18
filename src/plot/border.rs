use super::cairo_plotter::{Line, LineCap, PlotItem, Rectangle, Text};
use super::{text, Theme};
use crate::error::Error;
use crate::sexp::model::{Effects, Stroke, TitleBlock};
use ndarray::{arr1, arr2, Array2};

const BORDER_RASTER: i32 = 60;

pub fn draw_border(
    title_block: Option<&TitleBlock>,
    paper_size: (f64, f64),
    theme: &Theme,
) -> Result<Vec<PlotItem>, Error> {
    let mut plotter: Vec<PlotItem> = Vec::new();
    let stroke: Stroke = theme.stroke("border_stroke")?;
    let effects: Effects = theme.effects("border_effects")?;
    //outline
    let pts: Array2<f64> = arr2(&[[5.0, 5.0], [paper_size.0 - 5.0, paper_size.1 - 5.0]]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(
            pts,
            stroke.color,
            stroke.width,
            stroke.linetype.clone(),
            None,
        ),
    ));

    //horizontal raster
    for j in &[(0.0_f64, 5.0_f64), (paper_size.1 - 5.0, paper_size.1)] {
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.0],
                [(i as f64 + 1.0) * BORDER_RASTER as f64, j.1],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, stroke.color, 0.1, stroke.linetype.clone(), None),
            ));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(text!(
                arr1(&[
                    (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0,
                    j.0 + 2.5
                ]),
                0.0,
                (i + 1).to_string(),
                effects
            ));
        }
    }

    //vertical raster
    let letters = vec![
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ];
    for j in &[(0.0_f64, 5.0_f64), (paper_size.0 - 5.0, paper_size.0)] {
        for i in 0..(paper_size.1 as i32 / BORDER_RASTER) {
            let pts: Array2<f64> = arr2(&[
                [j.0, (i as f64 + 1.0) * BORDER_RASTER as f64],
                [j.1, (i as f64 + 1.0) * BORDER_RASTER as f64],
            ]);
            plotter.push(PlotItem::Rectangle(
                99,
                Rectangle::new(pts, stroke.color, 0.1, stroke.linetype.clone(), None),
            ));
        }
        for i in 0..(paper_size.0 as i32 / BORDER_RASTER + 1) {
            plotter.push(text!(
                arr1(&[
                    j.0 + 2.5,
                    (i as f64) * BORDER_RASTER as f64 + BORDER_RASTER as f64 / 2.0
                ]),
                0.0,
                letters[i as usize].to_string(),
                effects
            ));
        }
    }

    // the head
    let pts: Array2<f64> = arr2(&[
        [paper_size.0 - 120.0, paper_size.1 - 40.0],
        [paper_size.0 - 5.0, paper_size.1 - 5.0],
    ]);
    plotter.push(PlotItem::Rectangle(
        99,
        Rectangle::new(
            pts,
            stroke.color,
            stroke.width,
            stroke.linetype.clone(),
            None,
        ),
    ));
    plotter.push(PlotItem::Line(
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
    ));
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_size.0 - 120.0, paper_size.1 - 10.0],
                [paper_size.0 - 5.0, paper_size.1 - 10.0],
            ]),
            stroke.width,
            stroke.linetype.clone(),
            LineCap::Butt,
            stroke.color,
        ),
    ));
    plotter.push(PlotItem::Line(
        99,
        Line::new(
            arr2(&[
                [paper_size.0 - 120.0, paper_size.1 - 16.0],
                [paper_size.0 - 5.0, paper_size.1 - 16.0],
            ]),
            stroke.width,
            stroke.linetype.clone(),
            LineCap::Butt,
            stroke.color,
        ),
    ));

    if let Some(title_block) = title_block {
        let left = paper_size.0 - 118.0;
        let effects: Effects = theme.effects("subtitle_effects").unwrap();
        for (key, comment) in &title_block.comment {
            if *key == 1 {
                plotter.push(text!(
                    arr1(&[left, paper_size.1 - 30.0]),
                    0.0,
                    comment.to_string(),
                    effects
                ));
            } else if *key == 2 {
                plotter.push(text!(
                    arr1(&[left, paper_size.1 - 35.0]),
                    0.0,
                    comment.to_string(),
                    effects
                ));
            } else if *key == 3 {
                plotter.push(text!(
                    arr1(&[left, paper_size.1 - 40.0]),
                    0.0,
                    comment.to_string(),
                    effects
                ));
            } else if *key == 4 {
                plotter.push(text!(
                    arr1(&[left, paper_size.1 - 45.0]),
                    0.0,
                    comment.to_string(),
                    effects
                ));
            }
        }
        /* let effects: Effects = style.schema_title_effects();
        let title: String = get!(node, "title", 0);
        plotter.push(text!(arr1(&[left, paper_size.1 - 15.0]), 0.0, title, effects)); */
        if !title_block.company.is_empty() {
            let effects: Effects = theme.effects("title_effects").unwrap();
            plotter.push(text!(
                arr1(&[left, paper_size.1 - 25.0]),
                0.0,
                title_block.company.clone(),
                effects
            ));
        }
        if !title_block.title.is_empty() {
            let effects: Effects = theme.effects("title_effects").unwrap();
            plotter.push(text!(
                arr1(&[left, paper_size.1 - 13.0]),
                0.0,
                format!("Title: {}", title_block.title),
                effects
            ));
        }
        let effects: Effects = theme.effects("title_effects").unwrap();
        plotter.push(text!(
            arr1(&[left, paper_size.1 - 8.0]),
            0.0,
            format!("Paper: {}", String::from("xxx")),
            effects
        ));

        if !title_block.date.is_empty() {
            let effects: Effects = theme.effects("title_effects").unwrap();
            plotter.push(text!(
                arr1(&[paper_size.0 - 90.0, paper_size.1 - 8.0]),
                0.0,
                format!("Data: {}", title_block.date),
                effects
            ));
        }
        if !title_block.rev.is_empty() {
            let effects: Effects = theme.effects("title_effects").unwrap();
            plotter.push(text!(
                arr1(&[paper_size.0 - 20.0, paper_size.1 - 8.0]),
                0.0,
                format!("Rev: {}", title_block.rev),
                effects
            ));
        }
    }
    Ok(plotter)
}
