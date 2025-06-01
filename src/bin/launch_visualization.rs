use std::process::Command;
use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use IMO::tasks::run_hae_als_analysis::run_hae_als_analysis_task;

// --- Add Plotters imports ---
use plotters::prelude::*;

const PYTHON_INTERPRETER: &str = "python"; // This will be removed
const PYTHON_SCRIPT_NAME: &str = "visualize_als_choices.py"; // This will be removed

// --- New function to generate plot with Plotters ---
fn generate_plot_with_plotters(csv_file_path: &str, output_image_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(csv_file_path).exists() {
        return Err(format!("CSV file not found at {}", csv_file_path).into());
    }

    let file = File::open(csv_file_path)?;
    let reader = BufReader::new(file);

    let mut als_data: Vec<(u32, u32, f64)> = Vec::new(); // Iteration, ChosenOperator, ProbEdgeExchange
    for (idx, line) in reader.lines().enumerate() {
        if idx == 0 { continue; } // Skip header
        let line_str = line?;
        let parts: Vec<&str> = line_str.split(',').collect();
        if parts.len() == 3 {
            let iteration: u32 = parts[0].parse()?;
            let chosen_operator: u32 = parts[1].parse()?; // 0 for EE, 1 for OrOpt
            let prob_ee: f64 = parts[2].parse()?;
            als_data.push((iteration, chosen_operator, prob_ee));
        }
    }

    if als_data.is_empty() {
        return Err("No data found in CSV file.".into());
    }

    let root_area = BitMapBackend::new(output_image_path, (1200, 700)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let max_iter = als_data.last().map_or(1, |d| d.0) as u32;

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Adaptive Local Search Operator Choice Dynamics (Rust/Plotters)", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .right_y_label_area_size(80) // For secondary axis
        .build_cartesian_2d(0u32..max_iter, 0.0f64..1.05f64)? // Primary Y-axis for P(EdgeExchange)
        .set_secondary_coord(0u32..max_iter, 0.0f64..1.5f64); // Secondary Y-axis for Chosen Operator

    chart.configure_mesh()
        .x_desc("Iteration")
        .y_desc("P(EdgeExchange)")
        .y_label_formatter(&|y| format!("{:.2}", y))
        .draw()?;
        
    chart.configure_secondary_axes()
        .y_desc("Chosen Operator (0=EE, 1=OrOpt)")
        .y_label_formatter(&|y| {
            if (y - 0.0).abs() < 0.1 { "EdgeExchange".to_string() }
            else if (y - 1.0).abs() < 0.1 { "OrOpt".to_string() }
            else { "".to_string() }
        })
        .draw()?;

    // Plot P(EdgeExchange) on primary Y-axis
    chart.draw_series(LineSeries::new(
        als_data.iter().map(|(iter, _, prob)| (*iter, *prob)),
        &BLUE,
    ))?
    .label("P(EdgeExchange)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.filled()));

    // Plot ChosenOperator on secondary Y-axis using scatter points
    // Mapping 0 to 0.0 (EE) and 1 to 1.0 (OrOpt) for the secondary axis's scale
    chart.draw_secondary_series(
        als_data.iter().map(|(iter, choice, _)| {
            let y_val = if *choice == 0 { 0.0 } else { 1.0 }; // Map to 0.0 or 1.0 for y-axis positioning
            Circle::new((*iter, y_val), 3, RED.filled())
        })
    )?
    .label("Actual Choice")
    .legend(|(x,y)| Circle::new((x+10,y), 3, RED.filled()));
    
    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperMiddle)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root_area.present()?;
    println!("Plot saved to {}", output_image_path);

    Ok(())
}

fn main() {
    let instance_name = "kroa200"; // Or make this configurable via args
    let als_data_output_dir = "output/als_analysis";
    let plot_output_dir = "output/als_analysis"; // Can be the same or different

    println!("--- Step 1: Generating ALS data using Rust task ---");
    if !Path::new(als_data_output_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(als_data_output_dir) {
            eprintln!("Failed to create directory '{}': {}. Please check permissions.", als_data_output_dir, e);
            return;
        }
    }
    run_hae_als_analysis_task(instance_name, als_data_output_dir);
    println!("--- ALS data generation complete. ---");

    let csv_file_name = format!("{}/{}_als_choices.csv", als_data_output_dir, instance_name);
    let plot_file_name = format!("{}/{}_als_plot_rust.png", plot_output_dir, instance_name); // Changed extension/name

    if !Path::new(&csv_file_name).exists() {
        eprintln!("Error: CSV file '{}' was not found after running the Rust task.", csv_file_name);
        eprintln!("Check the output of the data generation step.");
        return;
    }

    println!("--- Step 2: Generating plot with Rust/Plotters ---");
    if let Err(e) = generate_plot_with_plotters(&csv_file_name, &plot_file_name) {
        eprintln!("Failed to generate plot: {}", e);
    } else {
        println!("--- Plot generation finished. ---");
    }
}
