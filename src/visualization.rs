use crate::tsplib::{Solution, TsplibInstance};
use plotters::prelude::*;
use std::path::Path;

const POINT_SIZE: u32 = 3;
const LINE_WIDTH: u32 = 2;

pub fn plot_solution(
    instance: &TsplibInstance,
    solution: &Solution,
    title: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (min_x, max_x, min_y, max_y) = instance
        .coordinates
        .iter()
        .fold((f64::MAX, f64::MIN, f64::MAX, f64::MIN), |acc, &(x, y)| {
            (acc.0.min(x), acc.1.max(x), acc.2.min(y), acc.3.max(y))
        });

    let padding = ((max_x - min_x) + (max_y - min_y)).max(1.0) * 0.05;

    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(
            (min_x - padding)..(max_x + padding),
            (min_y - padding)..(max_y + padding),
        )?;

    chart.configure_mesh().draw()?;

    {
        let cycle = &solution.cycle1;
        let points: Vec<(f64, f64)> = cycle.iter().map(|&idx| instance.coordinates[idx]).collect();

        let mut line_data = Vec::with_capacity(points.len() * 2);
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            line_data.push((x1, y1));
            line_data.push((x2, y2));
        }

        chart
            .draw_series(LineSeries::new(line_data, BLUE.stroke_width(LINE_WIDTH)))?
            .label("Cycle 1")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.clone()));

        chart.draw_series(
            points
                .iter()
                .map(|&(x, y)| Circle::new((x, y), POINT_SIZE, BLUE.filled())),
        )?;
    }

    {
        let cycle = &solution.cycle2;
        let points: Vec<(f64, f64)> = cycle.iter().map(|&idx| instance.coordinates[idx]).collect();

        let mut line_data = Vec::with_capacity(points.len() * 2);
        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            line_data.push((x1, y1));
            line_data.push((x2, y2));
        }

        chart
            .draw_series(LineSeries::new(line_data, RED.stroke_width(LINE_WIDTH)))?
            .label("Cycle 2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.clone()));

        chart.draw_series(
            points
                .iter()
                .map(|&(x, y)| Circle::new((x, y), POINT_SIZE, RED.filled())),
        )?;
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()?;

    Ok(())
}
