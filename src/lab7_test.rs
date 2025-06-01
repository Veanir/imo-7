use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod algorithm;
mod algorithms;
mod global_convexity;
mod moves;
mod tsplib;
mod utils;
mod visualization;

use algorithm::TspAlgorithm;
use algorithms::enhanced_hae_or_opt::EnhancedHaeOrOpt;
use algorithms::hae::Hae;
use algorithms::local_search::base::{
    HeuristicAlgorithm, InitialSolutionType, LocalSearch, NeighborhoodType, SearchVariant,
};
use algorithms::msls::Msls;
use tsplib::{TsplibInstance, Solution};
use crate::visualization::plot_solution;

const NUM_RUNS: usize = 20; // Number of runs for each algorithm per instance

#[derive(Debug, Clone)]
struct AlgoRunStats {
    costs: Vec<i32>,
    total_time: Duration, // Sum of times, average will be calculated
    iterations: Vec<usize>,
}

impl AlgoRunStats {
    fn new() -> Self {
        AlgoRunStats {
            costs: Vec::new(),
            total_time: Duration::ZERO,
            iterations: Vec::new(),
        }
    }

    fn add_run(&mut self, cost: i32, time: Duration, iterations: usize) {
        self.costs.push(cost);
        self.total_time += time;
        self.iterations.push(iterations);
    }

    fn avg_cost(&self) -> f64 {
        if self.costs.is_empty() { return 0.0; }
        self.costs.iter().sum::<i32>() as f64 / self.costs.len() as f64
    }

    fn min_cost(&self) -> Option<i32> {
        self.costs.iter().min().cloned()
    }

    fn avg_time(&self) -> Duration {
        if self.costs.is_empty() { return Duration::ZERO; }
        self.total_time / self.costs.len() as u32
    }
    fn std_dev_cost(&self) -> f64 {
        if self.costs.len() < 2 { return 0.0; }
        let mean = self.avg_cost();
        let variance = self.costs.iter().map(|value| {
            let diff = mean - (*value as f64);
            diff * diff
        }).sum::<f64>() / self.costs.len() as f64;
        variance.sqrt()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZADANIE 7: WŁASNA METODA (UŚREDNIONE WYNIKI Z {} URUCHOMIEŃ) ===", NUM_RUNS);
    
    create_dir_all("output/lab7")?;
    
    let instance_files = ["kroa200", "krob200"];
    let mut instances = HashMap::new();
    
    for name in instance_files {
        match TsplibInstance::from_file(Path::new(&format!("tsplib/{}.tsp", name))) {
            Ok(mut instance) => {
                println!("  Precomputing nearest neighbors (k=10) for {}...", name);
                instance.precompute_nearest_neighbors(10);
                instances.insert(name.to_string(), Arc::new(instance));
            }
            Err(e) => println!("Error loading {}: {}", name, e),
        }
    }
    
    let base_ls_edge = LocalSearch::new(
        SearchVariant::CandidateSteepest(10),
        NeighborhoodType::EdgeExchange,
        InitialSolutionType::Random,
        false, 
    );
    
    let hae_baseline_algo = Hae::new(base_ls_edge.clone(), 20, 40, true);
    let msls_baseline_algo = Msls::new(base_ls_edge.clone(), 200);
    
    let enhanced_hae_adaptive_or_opt_algo = EnhancedHaeOrOpt::new(
        20, 40, true, 5, false, 
    );
    
    // Store AlgoRunStats for each algorithm and instance
    let mut results: HashMap<String, HashMap<String, AlgoRunStats>> = HashMap::new();

    for (instance_name, instance_arc) in &instances {
        println!("\n=== Processing instance: {} ===", instance_name);
        results.insert(instance_name.clone(), HashMap::new());

        // --- MSLS Baseline (run once to get time limit, but store its stats too) ---
        println!("\n1. Measuring MSLS performance (1 run for time limit determination)...");
        let msls_start_time = Instant::now();
        let msls_solution = msls_baseline_algo.solve_with_feedback(instance_arc, &mut |_| {});
        let msls_run_time = msls_start_time.elapsed();
        let msls_cost = msls_solution.calculate_cost(instance_arc);
        let msls_time_limit = msls_run_time; // Use this single run time as the limit for others
        
        let mut msls_stats = AlgoRunStats::new();
        // For MSLS, we effectively do 1 run in terms of how its num_starts works internally.
        // The time reported by MSLS is its total execution time for all its internal starts.
        // We store this single observation.
        msls_stats.add_run(msls_cost, msls_run_time, msls_baseline_algo.num_starts); // num_starts as iterations proxy
        results.get_mut(instance_name).unwrap().insert("MSLS".to_string(), msls_stats);
        println!("  MSLS completed in {:?} with cost {}. Time limit for other algos: {:?}", msls_run_time, msls_cost, msls_time_limit);

        // --- HAE Baseline (multiple runs) ---
        println!("\n2. Running HAE_baseline ({} runs) with time limit {:?}...", NUM_RUNS, msls_time_limit);
        let mut hae_stats = AlgoRunStats::new();
        let mut best_hae_solution_for_plot: Option<Solution> = None;
        let mut best_hae_cost_for_plot = i32::MAX;

        for run in 0..NUM_RUNS {
            let run_start_time = Instant::now();
            let (hae_solution, hae_iterations) = hae_baseline_algo.solve_timed(instance_arc, msls_time_limit, &mut |_| {});
            let hae_run_time = run_start_time.elapsed();
            let hae_cost = hae_solution.calculate_cost(instance_arc);
            hae_stats.add_run(hae_cost, hae_run_time, hae_iterations);
            println!("  HAE Run {}/{}: cost {}, time {:?}", run + 1, NUM_RUNS, hae_cost, hae_run_time);
            if hae_cost < best_hae_cost_for_plot {
                best_hae_cost_for_plot = hae_cost;
                best_hae_solution_for_plot = Some(hae_solution);
            }
        }
        results.get_mut(instance_name).unwrap().insert("HAE_baseline".to_string(), hae_stats);
        if let Some(sol) = best_hae_solution_for_plot {
            let plot_title = format!("{} - HAE_baseline (Best of {} runs) - Cost: {}", instance_name, NUM_RUNS, best_hae_cost_for_plot);
            let plot_path_str = format!("output/lab7/{}_HAE_baseline_best.png", instance_name);
            plot_solution(&**instance_arc, &sol, &plot_title, Path::new(&plot_path_str))?;
            println!("  HAE best solution plot saved to: {}", plot_path_str);
        }
        
        // --- EnhancedHaeOrOpt (multiple runs) ---
        println!("\n3. Running {} ({} runs) with time limit {:?}...", enhanced_hae_adaptive_or_opt_algo.name(), NUM_RUNS, msls_time_limit);
        let mut enhanced_stats = AlgoRunStats::new();
        let mut best_enhanced_solution_for_plot: Option<Solution> = None;
        let mut best_enhanced_cost_for_plot = i32::MAX;

        for run in 0..NUM_RUNS {
            let run_start_time = Instant::now();
            let (enhanced_solution, enhanced_iterations, _) = enhanced_hae_adaptive_or_opt_algo.solve_timed(instance_arc, msls_time_limit, &mut |_| {});
            let enhanced_run_time = run_start_time.elapsed();
            let enhanced_cost = enhanced_solution.calculate_cost(instance_arc);
            enhanced_stats.add_run(enhanced_cost, enhanced_run_time, enhanced_iterations);
            println!("  {} Run {}/{}: cost {}, time {:?}", enhanced_hae_adaptive_or_opt_algo.name(), run + 1, NUM_RUNS, enhanced_cost, enhanced_run_time);
            if enhanced_cost < best_enhanced_cost_for_plot {
                best_enhanced_cost_for_plot = enhanced_cost;
                best_enhanced_solution_for_plot = Some(enhanced_solution);
            }
        }
        results.get_mut(instance_name).unwrap().insert(enhanced_hae_adaptive_or_opt_algo.name().to_string(), enhanced_stats);
        if let Some(sol) = best_enhanced_solution_for_plot {
            let plot_title = format!("{} - {} (Best of {} runs) - Cost: {}", instance_name, enhanced_hae_adaptive_or_opt_algo.name(), NUM_RUNS, best_enhanced_cost_for_plot);
            let plot_path_str = format!("output/lab7/{}_EnhancedHAEOrOpt_best.png", instance_name);
            plot_solution(&**instance_arc, &sol, &plot_title, Path::new(&plot_path_str))?;
            println!("  {} best solution plot saved to: {}", enhanced_hae_adaptive_or_opt_algo.name(), plot_path_str);
        }
    }
    
    println!("\n\n=== PODSUMOWANIE WYNIKÓW ({} URUCHOMIEŃ) ===", NUM_RUNS);
    println!("{:<15} {:<70} {:<15} {:<15} {:<15} {:<15} {:<20}", "Instance", "Algorithm", "Avg Cost", "Min Cost", "Std Dev", "Avg Time", "Avg Impr. vs HAE");
    println!("{}", "-".repeat(165));
    
    for instance_name_str in &instance_files {
        let instance_key = instance_name_str.to_string();
        if let Some(instance_results) = results.get(&instance_key) {
            let hae_avg_cost = instance_results.get("HAE_baseline").map_or(0.0, |s| s.avg_cost());

            let algo_keys = vec!["MSLS".to_string(), "HAE_baseline".to_string(), enhanced_hae_adaptive_or_opt_algo.name().to_string()];

            for algo_name_key in algo_keys {
                if let Some(stats) = instance_results.get(&algo_name_key) {
                    let avg_cost = stats.avg_cost();
                    let min_cost = stats.min_cost().unwrap_or(0);
                    let std_dev = stats.std_dev_cost();
                    let avg_time = stats.avg_time();
                    let improvement = if hae_avg_cost > 0.0 && algo_name_key != "HAE_baseline" {
                        ((hae_avg_cost - avg_cost) / hae_avg_cost) * 100.0
                    } else if algo_name_key == "HAE_baseline" {
                        0.0
                    } else { 
                        0.0 // Avoid division by zero if hae_avg_cost is 0, or for MSLS vs itself initially
                    };
                    println!(
                        "{: <15} {: <70} {: <15.2} {: <15} {: <15.2} {: <15.3?} {: <+20.2}%",
                        instance_key,
                        algo_name_key,
                        avg_cost,
                        min_cost,
                        std_dev,
                        avg_time,
                        improvement
                    );
                }
            }
            println!(); 
        }
    }
    
    println!("\nInterpretacja wyników:");
    println!("- Prezentowane są średnie, minimalne koszty i odch. std. z {} uruchomień.", NUM_RUNS);
    println!("- Poprawa jest liczona względem średniego kosztu HAE_baseline.");
    
    Ok(())
} 