use plotters::prelude::*;

pub fn plot(x: Vec<f64>, y: Vec<Vec<f64>>) -> String {
    let mut buffer = String::new();
    {
        // get the min and max values of the dataset.
        let max_x = x.iter().copied().fold(f64::NAN, f64::max);
        let min_x = x.iter().copied().fold(f64::NAN, f64::min);
        let mut max_y = 0.0;
        let mut min_y = 0.0;
        for inner in &y {
            let inner_max_y = inner.iter().copied().fold(f64::NAN, f64::max);
            if max_y < inner_max_y {
                max_y = inner_max_y;
            }
            let inner_min_y = inner.iter().copied().fold(f64::NAN, f64::min);
            if min_y > inner_min_y {
                min_y = inner_min_y;
            }
        }

        let root = SVGBackend::with_string(&mut buffer, (640, 480)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = ChartBuilder::on(&root)
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(min_x..max_x, min_y..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        for y in y {
            chart
                .draw_series(LineSeries::new(
                    x.iter().zip(y.iter()).map(|(x, y)| (*x, *y)),
                    RED,
                ))
                .unwrap()
                .label("y = x^2");
        }

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        root.present().unwrap();
    }
    buffer
}
