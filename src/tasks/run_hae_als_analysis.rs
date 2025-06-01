use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use crate::algorithms::enhanced_hae_or_opt::EnhancedHaeOrOpt;
use crate::tsplib::TsplibInstance;
use crate::algorithm::TspAlgorithm; // Ensure TspAlgorithm trait is in scope

pub fn run_hae_als_analysis_task(instance_name: &str, output_dir: &str) {
    println!(
        "Starting EnhancedHAEOrOpt ALS Analysis task for instance: {}",
        instance_name
    );

    let instance_path = format!("tsplib/{}.tsp", instance_name);
    let instance = match TsplibInstance::from_file(&instance_path) {
        Ok(inst) => inst,
        Err(e) => {
            eprintln!("Failed to load instance {}: {}", instance_name, e);
            return;
        }
    };

    // Precompute nearest neighbors as EnhancedHaeOrOpt uses them in recombination
    let mut instance = instance; // Make instance mutable to call precompute
    instance.precompute_nearest_neighbors(5); // Use k=5 as that's what recombine uses
    println!("Nearest neighbors precomputed for instance: {}", instance_name);

    // Configure EnhancedHaeOrOpt
    // These parameters can be adjusted as needed
    let pop_size = 20;
    let min_diff = 40;
    let adaptive_local_search = true; // Must be true to get ALS data
    let elite_size = 5;
    let verify_or_opt_delta = false; // Set to true for debugging Or-Opt, false for performance

    let algorithm = EnhancedHaeOrOpt::new(
        pop_size,
        min_diff,
        adaptive_local_search,
        elite_size,
        verify_or_opt_delta,
    );

    // Set a time limit for the algorithm run
    // For example, 30 seconds. Adjust as needed for your analysis.
    let time_limit_seconds = 30;
    let time_limit = Duration::from_secs(time_limit_seconds);

    println!(
        "Running {} for {} seconds...",
        algorithm.name(),
        time_limit_seconds
    );

    let mut progress_callback = |message: String| {
        // You can choose to print progress or not
        // println!("[Progress] {}", message);
        if message.contains("New global best") || message.contains("Finished") {
            println!("{}", message);
        }
    };

    // Run the algorithm
    let (best_solution, iterations, als_history) =
        algorithm.solve_timed(&instance, time_limit, &mut progress_callback);

    println!(
        "Finished run. Best solution cost: {}, Iterations: {}",
        best_solution.calculate_cost(&instance),
        iterations
    );

    if !adaptive_local_search {
        println!("Adaptive local search was disabled. No ALS data to save.");
        return;
    }

    if als_history.is_empty() {
        println!("ALS history is empty. No data to save.");
        return;
    }

    // Ensure output directory exists
    if !Path::new(output_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(output_dir) {
            eprintln!("Failed to create output directory '{}': {}", output_dir, e);
            return;
        }
    }
    
    let output_file_name = format!("{}/{}_als_choices.csv", output_dir, instance_name);
    match File::create(&output_file_name) {
        Ok(mut file) => {
            // Write CSV header
            if let Err(e) = writeln!(file, "Iteration,ChosenOperator,ProbEdgeExchange") {
                eprintln!("Failed to write CSV header: {}", e);
                return;
            }

            // Write ALS data
            for (i, (choice, prob_ee)) in als_history.iter().enumerate() {
                if let Err(e) = writeln!(file, "{},{},{}", i + 1, choice, prob_ee) {
                    eprintln!("Failed to write ALS data row: {}", e);
                    // Decide if you want to stop or continue on error
                }
            }
            println!("ALS choice data saved to {}", output_file_name);
        }
        Err(e) => {
            eprintln!("Failed to create output file '{}': {}", output_file_name, e);
        }
    }
} 