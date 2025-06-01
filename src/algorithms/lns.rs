use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::LocalSearch;
use crate::algorithms::perturbation::Perturbation;
use crate::tsplib::{Solution, TsplibInstance};
use crate::utils::generate_random_solution;
use rand::{Rng, thread_rng};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

// Make Lns generic over the perturbation type P
pub struct Lns<P: Perturbation + Send + Sync> {
    base_local_search: LocalSearch,
    perturbation: P, // Should be a Destroy/Repair type
    apply_ls_after_repair: bool,
    apply_ls_to_initial: bool,
    name_str: String,
    _marker: PhantomData<P>,
}

// Update impl block to include the generic parameter P
impl<P: Perturbation + Send + Sync> Lns<P> {
    pub fn new(
        base_local_search: LocalSearch,
        perturbation: P,
        apply_ls_after_repair: bool,
        apply_ls_to_initial: bool, // LNSa variant check
    ) -> Self {
        let variant = if apply_ls_after_repair {
            "LNS"
        } else {
            "LNSa (no LS after repair)"
        };
        let initial_ls_info = if apply_ls_to_initial {
            " (LS on Initial)"
        } else {
            ""
        };
        let name_str = format!(
            "{} (Base: {}, Perturb: {}){}",
            variant,
            base_local_search.name(),
            perturbation.name(),
            initial_ls_info
        );
        Self {
            base_local_search,
            perturbation,
            apply_ls_after_repair,
            apply_ls_to_initial,
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
        let mut best_solution = generate_random_solution(instance);

        // 2. Apply Local Search to Initial Solution (Optional)
        if self.apply_ls_to_initial {
            progress_callback("Running initial Local Search...".to_string());
            best_solution = self
                .base_local_search
                .solve_with_feedback(instance, &mut |s| {
                    progress_callback(format!("Initial LS: {}", s))
                });
            progress_callback(format!(
                "Initial LS finished. Cost: {}",
                best_solution.calculate_cost(instance)
            ));
        }
        let mut best_cost = best_solution.calculate_cost(instance);

        let mut iterations = 0;
        while start_time.elapsed() < time_limit {
            iterations += 1;
            let loop_start_time = Instant::now();

            // 3. Perturbation (Destroy + Repair)
            let mut current_solution = best_solution.clone();
            // Now we can call perturb directly
            self.perturbation
                .perturb(&mut current_solution, instance, &mut rng);
            progress_callback(format!(
                "[Iter {}] Perturbed (Destroy/Repair) solution.",
                iterations
            ));

            // 4. Local Search on Repaired Solution (Optional)
            if self.apply_ls_after_repair {
                let mut ls_callback = |s: String| {
                    progress_callback(format!(
                        "[Iter {}] LS on repaired: {} (Time left: {:?})",
                        iterations,
                        s,
                        time_limit.saturating_sub(start_time.elapsed())
                    ));
                };
                current_solution = self
                    .base_local_search
                    .solve_with_feedback(instance, &mut ls_callback);
            }
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
            "LNS finished. Total iterations: {}, Best cost: {}, Total time: {:?}",
            iterations,
            best_cost,
            start_time.elapsed()
        ));
        (best_solution, iterations)
    }
}

// Similar time limit considerations as ILS apply here.
/*
impl TspAlgorithm for Lns {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        unimplemented!("LNS solve_with_feedback needs time limit handling.");
    }
}
*/
