use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::LocalSearch;
use crate::tsplib::{Solution, TsplibInstance};
// Removed: use crate::utils::generate_random_solution; // Not used directly here
use std::time::Instant;

pub struct Msls {
    base_local_search: LocalSearch,
    pub num_starts: usize,
    name_str: String,
}

impl Msls {
    pub fn new(base_local_search: LocalSearch, num_starts: usize) -> Self {
        let name = format!(
            "MSLS (starts={}, base_ls_var={:?}, base_ls_neigh={:?}, base_ls_verify={})",
            num_starts,
            base_local_search.variant,
            base_local_search.neighborhood,
            base_local_search.verify_delta_computationally
        );
        Self {
            base_local_search,
            num_starts,
            name_str: name,
        }
    }
}

impl TspAlgorithm for Msls {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        mut progress_callback: ProgressCallback,
    ) -> Solution {
        let mut best_solution: Option<Solution> = None;
        let mut best_cost = i32::MAX;

        progress_callback(format!(
            "[{}] Starting MSLS with {} random starts.",
            self.name_str, self.num_starts
        ));

        for i in 0..self.num_starts {
            let ls_for_this_start = LocalSearch::new(
                self.base_local_search.variant,
                self.base_local_search.neighborhood,
                self.base_local_search.initial_solution_type,
                self.base_local_search.verify_delta_computationally, // Pass the flag
            );

            progress_callback(format!(
                "[{}] Start {}/{}: Running Local Search ({:?}, {:?}, Init: {:?}, Verify={})...",
                self.name_str,
                i + 1,
                self.num_starts,
                ls_for_this_start.variant,
                ls_for_this_start.neighborhood,
                ls_for_this_start.initial_solution_type,
                ls_for_this_start.verify_delta_computationally
            ));

            let current_solution = ls_for_this_start.solve_with_feedback(instance, &mut |s_ls| {
                progress_callback(format!(
                    "  [{}] Start {}, LS [{}]: {}",
                    self.name_str,
                    i + 1,
                    ls_for_this_start.name_str,
                    s_ls
                ));
            });
            let current_cost = current_solution.calculate_cost(instance);

            progress_callback(format!(
                "[{}] Start {}/{}: Completed. Cost: {}",
                self.name_str, 
                i + 1, 
                self.num_starts, 
                current_cost
            ));

            if current_cost < best_cost {
                best_cost = current_cost;
                best_solution = Some(current_solution);
                progress_callback(format!(
                    "[{}] New best MSLS cost: {}",
                    self.name_str, best_cost
                ));
            }
        }
        progress_callback(format!(
            "[{}] MSLS Finished. Final best cost: {}",
            self.name_str, best_cost
        ));
        best_solution.expect("MSLS should find at least one solution")
    }
}
