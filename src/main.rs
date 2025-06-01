mod algorithm;
mod algorithms;
mod global_convexity;
mod moves;
mod tsplib;
mod utils;
mod visualization;

use algorithms::local_search::base::{
    InitialSolutionType, LocalSearch, NeighborhoodType, SearchVariant,
};
use algorithms::hae::Hae;
use global_convexity::{analyze_global_convexity, plot_convexity_analysis};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::Arc;
use tsplib::TsplibInstance;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading instances...");

    create_dir_all("output")?;
    let results_file_path = "output/lab6_correlation_results.txt";
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(results_file_path)?;

    // Define instances
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

    // Define base local search for generating local optima
    let base_ls = LocalSearch::new(
        SearchVariant::CandidateSteepest(10),
        NeighborhoodType::EdgeExchange,
        InitialSolutionType::Random,
        false, // verify_delta_computationally - false for this usage
    );

    // Define the best algorithm for generating high-quality solutions
    // The Hae::new constructor now handles its internal LS verify flags.
    // The base_ls passed here is for its main LS step; if it involved OrOpt/ThreeOpt, it should be pre-configured.
    // Since base_ls here is EdgeExchange, its verify_delta_computationally=false is appropriate.
    let best_algorithm = Hae::new(base_ls.clone(), 20, 40, false); 

    println!("\n=== ZADANIE 6: TESTY GLOBALNEJ WYPUKŁOŚCI ===");

    for (name, instance) in &instances {
        println!("\nProcessing instance: {}", name);

        // --- Global Convexity Analysis (Lab 6) ---
        println!("  Running Global Convexity Analysis...");
        
        let convexity_result = analyze_global_convexity(
            instance,
            name,
            &best_algorithm,
            &base_ls,
            &mut |s| println!("    [Convexity] {}", s),
        );
        
        // Plot results for similarity to S_best
        let vertex_pairs_s_best_path = format!("output/{}_convexity_vertex_pairs_s_best.png", name);
        plot_convexity_analysis(
            &convexity_result,
            Path::new(&vertex_pairs_s_best_path),
            "vertex_pairs",
            "similarity_to_best",
        )?;
        
        let common_edges_s_best_path = format!("output/{}_convexity_common_edges_s_best.png", name);
        plot_convexity_analysis(
            &convexity_result,
            Path::new(&common_edges_s_best_path),
            "common_edges",
            "similarity_to_best",
        )?;

        // Plot results for average similarity to others
        let vertex_pairs_avg_path = format!("output/{}_convexity_vertex_pairs_avg_others.png", name);
        plot_convexity_analysis(
            &convexity_result,
            Path::new(&vertex_pairs_avg_path),
            "vertex_pairs",
            "avg_similarity_to_others",
        )?;

        let common_edges_avg_path = format!("output/{}_convexity_common_edges_avg_others.png", name);
        plot_convexity_analysis(
            &convexity_result,
            Path::new(&common_edges_avg_path),
            "common_edges",
            "avg_similarity_to_others",
        )?;
        
        println!("    Global Convexity Analysis completed for {}", name);
        println!("      Correlation (S_best - Vertex Pairs): {:.4}", convexity_result.correlation_vertex_pairs);
        println!("      Correlation (S_best - Common Edges): {:.4}", convexity_result.correlation_common_edges);
        println!("      Correlation (Avg. Others - Vertex Pairs): {:.4}", convexity_result.correlation_avg_vertex_pairs);
        println!("      Correlation (Avg. Others - Common Edges): {:.4}", convexity_result.correlation_avg_common_edges);
        println!("      Plots S_best: {}, {}", vertex_pairs_s_best_path, common_edges_s_best_path);
        println!("      Plots Avg. Others: {}, {}", vertex_pairs_avg_path, common_edges_avg_path);

        // Append results to text file
        let mut file_appender = OpenOptions::new().append(true).create(true).open(results_file_path)?;
        writeln!(file_appender, "Instance: {}", name)?;
        writeln!(file_appender, "  Correlation (S_best - Vertex Pairs): {:.4}", convexity_result.correlation_vertex_pairs)?;
        writeln!(file_appender, "  Correlation (S_best - Common Edges): {:.4}", convexity_result.correlation_common_edges)?;
        writeln!(file_appender, "  Correlation (Avg. Others - Vertex Pairs): {:.4}", convexity_result.correlation_avg_vertex_pairs)?;
        writeln!(file_appender, "  Correlation (Avg. Others - Common Edges): {:.4}", convexity_result.correlation_avg_common_edges)?;
        writeln!(file_appender, "  Plots (S_best): {}, {}", vertex_pairs_s_best_path, common_edges_s_best_path)?;
        writeln!(file_appender, "  Plots (Avg. Others): {}, {}", vertex_pairs_avg_path, common_edges_avg_path)?;
        writeln!(file_appender, "---")?;
    }

    println!("\n=== PODSUMOWANIE WYNIKÓW ===");
    println!("Analiza globalnej wypukłości została zakończona.");
    println!("Wykresy zostały zapisane w katalogu 'output'.");
    println!("Wyniki korelacji zostały zapisane do: {}", results_file_path);
    println!("\nInterpretacja wyników:");
    println!("- Współczynnik korelacji bliski -1: silna korelacja negatywna (lepsze rozwiązania są bardziej podobne do najlepszego)");
    println!("- Współczynnik korelacji bliski 0: brak korelacji");
    println!("- Współczynnik korelacji bliski 1: silna korelacja pozytywna");
    
    Ok(())
}
