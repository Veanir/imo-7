use crate::tsplib::{Solution, TsplibInstance};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

pub type ProgressCallback<'a> = &'a mut dyn FnMut(String);

pub trait TspAlgorithm {
    fn name(&self) -> &str;

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution;
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub cost: i32,
    pub solution: Solution,
    pub time_ms: u128,
    pub iterations: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ExperimentStats {
    pub algorithm_name: String,
    pub instance_name: String,
    pub min_cost: i32,
    pub max_cost: i32,
    pub avg_cost: f64,
    pub best_solution: Solution,
    pub avg_time_ms: f64,
    pub avg_iterations: Option<f64>,
    pub num_runs: usize,
}

pub fn run_experiment(
    algorithm: &(dyn TspAlgorithm + Send + Sync),
    instance: &TsplibInstance,
    num_runs: usize,
) -> ExperimentStats {
    run_experiment_base(algorithm.name(), instance, num_runs, |progress_callback| {
        (
            algorithm.solve_with_feedback(instance, progress_callback),
            None,
        )
    })
}

pub type TimedSolveFn<'a, T> =
    Box<dyn Fn(&T, &TsplibInstance, ProgressCallback) -> (Solution, usize) + Send + Sync + 'a>;

pub fn run_timed_experiment<T: Send + Sync + ?Sized>(
    algorithm: &T,
    timed_solve_fn: TimedSolveFn<T>,
    instance: &TsplibInstance,
    num_runs: usize,
    algorithm_name: &str,
) -> ExperimentStats {
    run_experiment_base(algorithm_name, instance, num_runs, |progress_callback| {
        let (solution, iterations) = timed_solve_fn(algorithm, instance, progress_callback);
        (solution, Some(iterations))
    })
}

fn run_experiment_base<F>(
    algorithm_name: &str,
    instance: &TsplibInstance,
    num_runs: usize,
    solve_fn: F,
) -> ExperimentStats
where
    F: Fn(ProgressCallback) -> (Solution, Option<usize>),
{
    if num_runs == 0 {
        return ExperimentStats {
            algorithm_name: algorithm_name.to_string(),
            instance_name: instance.name.clone(),
            min_cost: 0,
            max_cost: 0,
            avg_cost: 0.0,
            best_solution: Solution::new(vec![], vec![]),
            avg_time_ms: 0.0,
            avg_iterations: None,
            num_runs: 0,
        };
    }

    let mut results = Vec::with_capacity(num_runs);

    let pb = ProgressBar::new(num_runs as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}",
            )
            .unwrap()
            .progress_chars("# >-"),
    );
    pb.set_prefix(format!("Running {}", algorithm_name));
    pb.set_message("Starting...");

    for run_index in 0..num_runs {
        let start = Instant::now();

        let mut callback = |status: String| {
            pb.set_message(format!("[Run {}/{}] {}", run_index + 1, num_runs, status));
        };

        let (solution, iterations_opt) = solve_fn(&mut callback);
        let elapsed = start.elapsed();

        assert!(
            solution.is_valid(instance),
            "Invalid solution produced by {}",
            algorithm_name
        );

        let result = RunResult {
            cost: solution.calculate_cost(instance),
            solution,
            time_ms: elapsed.as_millis(),
            iterations: iterations_opt,
        };
        results.push(result);
        pb.inc(1);
        pb.set_message("Done run.");
    }
    pb.finish_with_message("Finished all runs.");

    let mut min_cost = i32::MAX;
    let mut max_cost = i32::MIN;
    let mut sum_cost: i64 = 0;
    let mut sum_time: u128 = 0;
    let mut sum_iterations: u64 = 0;
    let mut iteration_count = 0;
    let mut best_solution = None;

    for result in &results {
        if result.cost < min_cost {
            min_cost = result.cost;
            best_solution = Some(result.solution.clone());
        }
        max_cost = max_cost.max(result.cost);
        sum_cost += result.cost as i64;
        sum_time += result.time_ms;
        if let Some(iters) = result.iterations {
            sum_iterations += iters as u64;
            iteration_count += 1;
        }
    }

    let final_best_solution = best_solution.expect("Best solution should exist if num_runs > 0");
    let avg_iterations = if iteration_count > 0 {
        Some(sum_iterations as f64 / iteration_count as f64)
    } else {
        None
    };

    ExperimentStats {
        algorithm_name: algorithm_name.to_string(),
        instance_name: instance.name.clone(),
        min_cost,
        max_cost,
        avg_cost: sum_cost as f64 / num_runs as f64,
        best_solution: final_best_solution,
        avg_time_ms: sum_time as f64 / num_runs as f64,
        avg_iterations,
        num_runs,
    }
}

pub fn format_stats_row(stats: &ExperimentStats) -> String {
    if stats.num_runs == 0 {
        return format!(
            "| {:<28} | No runs executed | N/A | N/A |",
            stats.algorithm_name
        );
    }
    let iter_str = match stats.avg_iterations {
        Some(avg_iters) => format!("{:.1}", avg_iters),
        None => "N/A".to_string(),
    };
    let algo_name_padded = format!("{:<28}", stats.algorithm_name);
    format!(
        "| {} | {} ({:.2} - {}) | {:>14.2} | {:>16} |",
        algo_name_padded,
        stats.min_cost,
        stats.avg_cost,
        stats.max_cost,
        stats.avg_time_ms,
        iter_str
    )
}
