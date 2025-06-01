use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::LocalSearch;
use crate::algorithms::perturbation::Perturbation;
use crate::tsplib::{Solution, TsplibInstance};
use crate::utils::generate_random_solution;
use rand::{Rng, thread_rng};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

// Make Ils generic over the perturbation type P
pub struct Ils<P: Perturbation + Send + Sync> {
    base_local_search: LocalSearch,
    perturbation: P,
    name_str: String,
    _marker: PhantomData<P>, // Use PhantomData if P is not used directly in struct fields
}

// Update impl block to include the generic parameter P
impl<P: Perturbation + Send + Sync> Ils<P> {
    pub fn new(base_local_search: LocalSearch, perturbation: P) -> Self {
        let name_str = format!(
            "ILS (Base: {}, Perturb: {})",
            base_local_search.name(),
            perturbation.name()
        );
        Self {
            base_local_search,
            perturbation,
            name_str,
            _marker: PhantomData,
        }
    }

    // Add public name accessor
    pub fn name(&self) -> &str {
        &self.name_str
    }

    // solve_timed remains largely the same, but can now call perturbation.perturb directly
    pub fn solve_timed(
        &self,
        instance: &TsplibInstance,
        time_limit: Duration,
        progress_callback: ProgressCallback,
    ) -> (Solution, usize) {
        // Return iterations count as well
        let start_time = Instant::now();
        let mut rng = thread_rng();

        // 1. Generate Initial Solution
        progress_callback("Generating initial random solution...".to_string());
        let initial_solution = generate_random_solution(instance);

        // 2. Apply Local Search to Initial Solution
        progress_callback("Running initial Local Search...".to_string());
        let mut best_solution = self
            .base_local_search
            .solve_with_feedback(instance, &mut |s| {
                progress_callback(format!("Initial LS: {}", s))
            });
        let mut best_cost = best_solution.calculate_cost(instance);
        progress_callback(format!("Initial LS finished. Cost: {}", best_cost));

        let mut iterations = 0;
        while start_time.elapsed() < time_limit {
            iterations += 1;
            let loop_start_time = Instant::now();

            // 3. Perturbation
            let mut current_solution = best_solution.clone();
            // Now we can call perturb directly
            self.perturbation
                .perturb(&mut current_solution, instance, &mut rng);
            progress_callback(format!("[Iter {}] Perturbed solution.", iterations));

            // 4. Local Search on Perturbed Solution
            let mut ls_callback = |s: String| {
                progress_callback(format!(
                    "[Iter {}] LS on perturbed: {} (Time left: {:?})",
                    iterations,
                    s,
                    time_limit.saturating_sub(start_time.elapsed())
                ));
            };
            current_solution = self
                .base_local_search
                .solve_with_feedback(instance, &mut ls_callback);
            let current_cost = current_solution.calculate_cost(instance);

            // 5. Acceptance Criterion (Accept if better)
            if current_cost < best_cost {
                best_solution = current_solution;
                best_cost = current_cost;
                progress_callback(format!(
                    "[Iter {}] New best solution found: {}. Loop time: {:?}",
                    iterations,
                    best_cost,
                    loop_start_time.elapsed()
                ));
            } else {
                progress_callback(format!(
                    "[Iter {}] Solution not improved ({} >= {}). Loop time: {:?}",
                    iterations,
                    current_cost,
                    best_cost,
                    loop_start_time.elapsed()
                ));
            }

            // Check time limit again before next iteration
            if start_time.elapsed() >= time_limit {
                progress_callback(format!("[Iter {}] Time limit reached.", iterations));
                break;
            }
        }

        progress_callback(format!(
            "ILS finished. Total iterations: {}, Best cost: {}, Total time: {:?}",
            iterations,
            best_cost,
            start_time.elapsed()
        ));
        (best_solution, iterations)
    }
}

// We might need a way to run ILS with a time limit based on MSLS average time.
// TspAlgorithm trait expects solve_with_feedback which doesn't naturally take a time limit.
// For now, we'll add a separate solve_timed method.
// Alternatively, we could adapt the TspAlgorithm trait or ExperimentRunner later.

/*
impl TspAlgorithm for Ils {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        // This would need a default time limit or another way to determine it.
        unimplemented!("ILS solve_with_feedback needs time limit handling.");
    }
}
*/
