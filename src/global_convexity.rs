use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::LocalSearch;
use crate::tsplib::{Solution, TsplibInstance};
use crate::utils::generate_random_solution;
use plotters::prelude::*;
use std::collections::HashSet;
use std::path::Path;

/// Similarity measures between two TSP solutions
#[derive(Debug, Clone)]
pub struct SimilarityMeasures {
    /// Percentage of vertex pairs assigned to the same cycle in both solutions (0.0 to 1.0)
    pub vertex_pairs_same_cycle: f64,
    /// Percentage of common edges between solutions (0.0 to 1.0)
    pub common_edges: f64,
}

/// Data point for global convexity analysis
#[derive(Debug, Clone)]
pub struct ConvexityDataPoint {
    pub cost: i32,
    pub similarity_to_best: SimilarityMeasures,
    pub avg_similarity_to_others: SimilarityMeasures,
}

/// Results of global convexity analysis
#[derive(Debug, Clone)]
pub struct ConvexityAnalysisResult {
    pub instance_name: String,
    pub best_solution: Solution,
    pub best_cost: i32,
    pub data_points: Vec<ConvexityDataPoint>,
    pub correlation_vertex_pairs: f64,
    pub correlation_common_edges: f64,
    pub correlation_avg_vertex_pairs: f64,
    pub correlation_avg_common_edges: f64,
}

impl SimilarityMeasures {
    pub fn new() -> Self {
        Self {
            vertex_pairs_same_cycle: 0.0,
            common_edges: 0.0,
        }
    }
}

/// Calculate similarity between two solutions
pub fn calculate_similarity(sol1: &Solution, sol2: &Solution, instance: &TsplibInstance) -> SimilarityMeasures {
    let mut similarity = SimilarityMeasures::new();
    
    // Measure 1: Percentage of vertex pairs assigned to the same cycle in both solutions
    let total_possible_pairs = (instance.size() * (instance.size() - 1)) / 2;
    similarity.vertex_pairs_same_cycle = count_vertex_pairs_same_cycle(sol1, sol2, instance) as f64 / total_possible_pairs as f64;
    
    // Measure 2: Percentage of common edges (normalized by total edges in solution)
    let total_edges = instance.size(); // Each solution has exactly n edges (n vertices in cycles)
    similarity.common_edges = count_common_edges(sol1, sol2) as f64 / total_edges as f64;
    
    similarity
}

/// Count vertex pairs that are in the same cycle in both solutions
fn count_vertex_pairs_same_cycle(sol1: &Solution, sol2: &Solution, instance: &TsplibInstance) -> usize {
    let n = instance.size();
    let mut count = 0;
    
    // Create mappings: vertex -> cycle_id for both solutions
    let mut sol1_cycle_map = vec![0; n]; // 0 for cycle1, 1 for cycle2
    let mut sol2_cycle_map = vec![0; n];
    
    // Fill sol1 mapping
    for &vertex in &sol1.cycle1 {
        sol1_cycle_map[vertex] = 0;
    }
    for &vertex in &sol1.cycle2 {
        sol1_cycle_map[vertex] = 1;
    }
    
    // Fill sol2 mapping
    for &vertex in &sol2.cycle1 {
        sol2_cycle_map[vertex] = 0;
    }
    for &vertex in &sol2.cycle2 {
        sol2_cycle_map[vertex] = 1;
    }
    
    // Count pairs that are in the same cycle in both solutions
    for i in 0..n {
        for j in i + 1..n {
            let same_cycle_sol1 = sol1_cycle_map[i] == sol1_cycle_map[j];
            let same_cycle_sol2 = sol2_cycle_map[i] == sol2_cycle_map[j];
            
            if same_cycle_sol1 && same_cycle_sol2 {
                count += 1;
            }
        }
    }
    
    count
}

/// Count common edges between two solutions
fn count_common_edges(sol1: &Solution, sol2: &Solution) -> usize {
    let mut sol1_edges = HashSet::new();
    let mut count = 0;
    
    // Collect all edges from sol1
    for cycle in [&sol1.cycle1, &sol1.cycle2] {
        for i in 0..cycle.len() {
            let a = cycle[i];
            let b = cycle[(i + 1) % cycle.len()];
            // Store edge in canonical form (smaller vertex first)
            let edge = if a < b { (a, b) } else { (b, a) };
            sol1_edges.insert(edge);
        }
    }
    
    // Check edges from sol2 against sol1
    for cycle in [&sol2.cycle1, &sol2.cycle2] {
        for i in 0..cycle.len() {
            let a = cycle[i];
            let b = cycle[(i + 1) % cycle.len()];
            // Store edge in canonical form (smaller vertex first)
            let edge = if a < b { (a, b) } else { (b, a) };
            if sol1_edges.contains(&edge) {
                count += 1;
            }
        }
    }
    
    count
}

/// Generate 1000 random local optima using the given local search algorithm
pub fn generate_random_local_optima(
    instance: &TsplibInstance,
    local_search: &LocalSearch,
    num_optima: usize,
    progress_callback: ProgressCallback,
) -> Vec<Solution> {
    let mut local_optima = Vec::with_capacity(num_optima);
    
    for i in 0..num_optima {
        if i % 100 == 0 {
            progress_callback(format!("Generating local optimum {}/{}", i + 1, num_optima));
        }
        
        // Generate random starting solution
        let _random_solution = generate_random_solution(instance);
        
        // Apply local search to get local optimum
        // Note: LocalSearch with InitialSolutionType::Random will generate its own random solution
        let mut dummy_callback = |_: String| {};
        let local_optimum = local_search.solve_with_feedback(instance, &mut dummy_callback);
        
        local_optima.push(local_optimum);
    }
    
    progress_callback(format!("Generated {} local optima", num_optima));
    local_optima
}

/// Calculate average similarity between a solution and a list of other solutions
fn calculate_average_similarity(
    target_solution: &Solution,
    other_solutions: &[Solution],
    instance: &TsplibInstance,
) -> SimilarityMeasures {
    if other_solutions.is_empty() {
        return SimilarityMeasures::new();
    }
    
    let mut total_vertex_pairs = 0.0;
    let mut total_common_edges = 0.0;
    
    for other_solution in other_solutions {
        let similarity = calculate_similarity(target_solution, other_solution, instance);
        total_vertex_pairs += similarity.vertex_pairs_same_cycle;
        total_common_edges += similarity.common_edges;
    }
    
    SimilarityMeasures {
        vertex_pairs_same_cycle: total_vertex_pairs / other_solutions.len() as f64,
        common_edges: total_common_edges / other_solutions.len() as f64,
    }
}

/// Calculate Pearson correlation coefficient
fn calculate_correlation(x_values: &[f64], y_values: &[f64]) -> f64 {
    if x_values.len() != y_values.len() || x_values.len() < 2 {
        return 0.0;
    }
    
    let n = x_values.len() as f64;
    let sum_x: f64 = x_values.iter().sum();
    let sum_y: f64 = y_values.iter().sum();
    let sum_xy: f64 = x_values.iter().zip(y_values.iter()).map(|(x, y)| x * y).sum();
    let sum_x2: f64 = x_values.iter().map(|x| x * x).sum();
    let sum_y2: f64 = y_values.iter().map(|y| y * y).sum();
    
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
    
    if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    }
}

/// Perform global convexity analysis
pub fn analyze_global_convexity<T: TspAlgorithm>(
    instance: &TsplibInstance,
    instance_name: &str,
    best_algorithm: &T,
    local_search: &LocalSearch,
    progress_callback: ProgressCallback,
) -> ConvexityAnalysisResult {
    progress_callback("Starting global convexity analysis...".to_string());
    
    // 1. Generate very good solution using best algorithm
    progress_callback("Generating best solution...".to_string());
    let mut best_callback = |s: String| {
        progress_callback(format!("Best algorithm: {}", s));
    };
    let best_solution = best_algorithm.solve_with_feedback(instance, &mut best_callback);
    let best_cost = best_solution.calculate_cost(instance);
    progress_callback(format!("Best solution cost: {}", best_cost));
    
    // 2. Generate 1000 random local optima
    progress_callback("Generating 1000 random local optima...".to_string());
    let local_optima = generate_random_local_optima(
        instance,
        local_search,
        1000,
        &mut |s| progress_callback(format!("Local optima: {}", s)),
    );
    
    // 3. Calculate similarities and create data points
    progress_callback("Calculating similarities...".to_string());
    let mut data_points = Vec::with_capacity(local_optima.len());
    
    for (i, local_optimum) in local_optima.iter().enumerate() {
        if i % 100 == 0 {
            progress_callback(format!("Processing optimum {}/{}", i + 1, local_optima.len()));
        }
        
        let cost = local_optimum.calculate_cost(instance);
        
        // Similarity to best solution
        let similarity_to_best = calculate_similarity(local_optimum, &best_solution, instance);
        
        // Average similarity to all other local optima
        let other_optima: Vec<&Solution> = local_optima.iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, sol)| sol)
            .collect();
        let avg_similarity_to_others = calculate_average_similarity(
            local_optimum,
            &other_optima.into_iter().cloned().collect::<Vec<_>>(),
            instance,
        );
        
        data_points.push(ConvexityDataPoint {
            cost,
            similarity_to_best,
            avg_similarity_to_others,
        });
    }
    
    // 4. Calculate correlations
    progress_callback("Calculating correlations...".to_string());
    let costs: Vec<f64> = data_points.iter().map(|dp| dp.cost as f64).collect();
    
    // Correlations for similarity to S_best
    let s_best_vertex_pairs_similarities: Vec<f64> = data_points.iter()
        .map(|dp| dp.similarity_to_best.vertex_pairs_same_cycle)
        .collect();
    
    let s_best_common_edges_similarities: Vec<f64> = data_points.iter()
        .map(|dp| dp.similarity_to_best.common_edges)
        .collect();
    
    let correlation_vertex_pairs = calculate_correlation(&costs, &s_best_vertex_pairs_similarities);
    let correlation_common_edges = calculate_correlation(&costs, &s_best_common_edges_similarities);

    // Correlations for average similarity to others
    let avg_vertex_pairs_similarities: Vec<f64> = data_points.iter()
        .map(|dp| dp.avg_similarity_to_others.vertex_pairs_same_cycle)
        .collect();

    let avg_common_edges_similarities: Vec<f64> = data_points.iter()
        .map(|dp| dp.avg_similarity_to_others.common_edges)
        .collect();

    let correlation_avg_vertex_pairs = calculate_correlation(&costs, &avg_vertex_pairs_similarities);
    let correlation_avg_common_edges = calculate_correlation(&costs, &avg_common_edges_similarities);
    
    progress_callback(format!(
        "Correlations - S_best VP: {:.4}, S_best CE: {:.4}, Avg VP: {:.4}, Avg CE: {:.4}",
        correlation_vertex_pairs, correlation_common_edges, correlation_avg_vertex_pairs, correlation_avg_common_edges
    ));
    
    ConvexityAnalysisResult {
        instance_name: instance_name.to_string(),
        best_solution,
        best_cost,
        data_points,
        correlation_vertex_pairs,
        correlation_common_edges,
        correlation_avg_vertex_pairs,
        correlation_avg_common_edges,
    }
}

/// Plot global convexity analysis results
pub fn plot_convexity_analysis(
    result: &ConvexityAnalysisResult,
    output_path: &Path,
    similarity_type: &str, // "vertex_pairs" or "common_edges"
    plot_target: &str, // "similarity_to_best" or "avg_similarity_to_others"
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;
    
    let (costs, similarities): (Vec<i32>, Vec<f64>) = result.data_points.iter()
        .map(|dp| {
            let sim_measure_set = match plot_target {
                "similarity_to_best" => &dp.similarity_to_best,
                "avg_similarity_to_others" => &dp.avg_similarity_to_others,
                _ => &dp.similarity_to_best, // Default
            };
            let similarity = match similarity_type {
                "vertex_pairs" => sim_measure_set.vertex_pairs_same_cycle,
                "common_edges" => sim_measure_set.common_edges,
                _ => sim_measure_set.vertex_pairs_same_cycle, // Default
            };
            (dp.cost, similarity)
        })
        .unzip();
    
    let min_cost = *costs.iter().min().unwrap_or(&0);
    let max_cost = *costs.iter().max().unwrap_or(&1);
    let min_similarity = similarities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_similarity = similarities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    
    // Ensure we have valid ranges
    let min_similarity = if min_similarity.is_finite() { min_similarity } else { 0.0 };
    let max_similarity = if max_similarity.is_finite() { max_similarity } else { 1.0 };
    
    let correlation = match (plot_target, similarity_type) {
        ("similarity_to_best", "vertex_pairs") => result.correlation_vertex_pairs,
        ("similarity_to_best", "common_edges") => result.correlation_common_edges,
        ("avg_similarity_to_others", "vertex_pairs") => result.correlation_avg_vertex_pairs,
        ("avg_similarity_to_others", "common_edges") => result.correlation_avg_common_edges,
        _ => result.correlation_vertex_pairs, // Default
    };
    
    let y_axis_label = match plot_target {
        "similarity_to_best" => "Similarity to Best Solution",
        "avg_similarity_to_others" => "Avg. Similarity to Other Optima",
        _ => "Similarity",
    };

    let title_suffix = match plot_target {
        "similarity_to_best" => "vs S_best",
        "avg_similarity_to_others" => "vs Avg. Others",
        _ => "",
    };

    let title = format!(
        "Global Convexity Analysis - {} - {} {} (r={:.4})",
        result.instance_name,
        similarity_type.replace("_", " "),
        title_suffix,
        correlation
    );
    
    let mut chart = ChartBuilder::on(&root)
        .caption(&title, ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(60) // Increased for potentially longer y_axis_label
        .build_cartesian_2d(
            min_cost..max_cost,
            min_similarity..max_similarity,
        )?;
    
    chart
        .configure_mesh()
        .x_desc("Cost")
        .y_desc(y_axis_label) // Use dynamic Y-axis label
        .draw()?;
    
    chart.draw_series(
        costs.iter().zip(similarities.iter()).map(|(&cost, &similarity)| {
            Circle::new((cost, similarity), 2, BLUE.filled())
        })
    )?;
    
    root.present()?;
    Ok(())
} 