use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::{
    HeuristicAlgorithm, InitialSolutionType, LocalSearch, NeighborhoodType, SearchVariant,
};
use crate::algorithms::perturbation::{Perturbation, SmallPerturbation};
use crate::tsplib::{Solution, TsplibInstance};
use rand::{Rng, thread_rng};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;

pub struct Hae {
    base_local_search: LocalSearch,
    pop_size: usize,
    min_diff: i32,
    init_ls_for_heuristic: LocalSearch,
    init_ls_for_random: LocalSearch,
    use_heuristic_init: bool,
    name_str: String,
}

impl Hae {
    pub fn new(
        base_local_search: LocalSearch,
        pop_size: usize,
        min_diff: i32,
        use_heuristic_init: bool,
    ) -> Self {
        let init_ls_heuristic = LocalSearch::new(
            base_local_search.variant,
            base_local_search.neighborhood,
            InitialSolutionType::Heuristic(HeuristicAlgorithm::WeightedRegret),
            false,
        );
        let init_ls_random = LocalSearch::new(
            base_local_search.variant,
            base_local_search.neighborhood,
            InitialSolutionType::Random,
            false,
        );

        let name = format!(
            "HAE (pop={}, min_diff={}, heur_init={}, base_ls_var={:?}, base_ls_neigh={:?}, base_ls_verify={})",
            pop_size, 
            min_diff, 
            use_heuristic_init, 
            base_local_search.variant, 
            base_local_search.neighborhood,
            base_local_search.verify_delta_computationally
        );

        Self {
            base_local_search,
            pop_size,
            min_diff,
            init_ls_for_heuristic: init_ls_heuristic,
            init_ls_for_random: init_ls_random,
            use_heuristic_init,
            name_str: name,
        }
    }

    pub fn name(&self) -> &str {
        &self.name_str
    }

    pub fn solve_timed(
        &self,
        instance: &TsplibInstance,
        time_limit: Duration,
        mut progress_callback: ProgressCallback,
    ) -> (Solution, usize) {
        let mut rng = thread_rng();
        let start_time = Instant::now();

        let mut population: Vec<(Solution, i32)> = Vec::with_capacity(self.pop_size);

        progress_callback(format!("[{}] Generating initial population...", self.name_str));

        let num_heuristic = if self.use_heuristic_init {
            self.pop_size / 4
        } else {
            0
        };

        for i in 0..num_heuristic {
            progress_callback(format!("[{}] Init heuristic {}/{}...", self.name_str, i + 1, num_heuristic));
            let sol = self.init_ls_for_heuristic.solve_with_feedback(instance, &mut |s_ls| {
                progress_callback(format!("  [{}] {}", self.init_ls_for_heuristic.name_str, s_ls));
            });
            let cost = sol.calculate_cost(instance);
            population.push((sol, cost));
        }

        for i in num_heuristic..self.pop_size {
            progress_callback(format!("[{}] Init random {}/{}...", self.name_str, i + 1 - num_heuristic, self.pop_size - num_heuristic));
            let sol = self.init_ls_for_random.solve_with_feedback(instance, &mut |s_ls| {
                progress_callback(format!("  [{}] {}", self.init_ls_for_random.name_str, s_ls));
            });
            let cost = sol.calculate_cost(instance);
            population.push((sol, cost));
        }

        population.sort_by_key(|(_, cost)| *cost);

        let mut best_solution = population[0].0.clone();
        let mut best_cost = population[0].1;
        progress_callback(format!(
            "[{}] Initial best cost: {}",
            self.name_str, best_cost
        ));

        let mut iterations = 0;
        while start_time.elapsed() < time_limit {
            iterations += 1;

            let mut p1_idx = rng.gen_range(0..self.pop_size);
            let mut p2_idx = rng.gen_range(0..self.pop_size);
            while p1_idx == p2_idx {
                p2_idx = rng.gen_range(0..self.pop_size);
            }

            let parent1 = &population[p1_idx].0;
            let parent2 = &population[p2_idx].0;

            let mut child = self.recombine(parent1, parent2, instance, &mut rng);
            
            child = self.base_local_search.solve_from_solution(instance, child, &mut |s_ls| {
                 progress_callback(format!("  [{}] Iter {}, LS from recombine: {}", self.name_str, iterations, s_ls));
            });
            let child_cost = child.calculate_cost(instance);

            if child_cost < best_cost {
                best_solution = child.clone();
                best_cost = child_cost;
                progress_callback(format!(
                    "[{}] Iter {}: New best: {}",
                    self.name_str, iterations, best_cost
                ));
            }

            let mut replaced = false;
            for i in (0..self.pop_size).rev() {
                if child_cost < population[i].1 {
                    if (child_cost - population[i].1).abs() > self.min_diff || i == self.pop_size -1 {
                        population[i] = (child.clone(), child_cost);
                        replaced = true;
                        break;
                    }
                }
            }

            if replaced {
                population.sort_by_key(|(_, cost)| *cost);
            }
            
            if iterations % 100 == 0 {
                progress_callback(format!("[{}] Iter {}: Perturbing worst solution...", self.name_str, iterations));
                let worst_idx = self.pop_size -1;
                let mut sol_to_perturb = population[worst_idx].0.clone();
                let perturbation_op = SmallPerturbation::new(5);
                perturbation_op.perturb(&mut sol_to_perturb, instance, &mut rng);
                
                population[worst_idx].0 = self.base_local_search.solve_from_solution(instance, sol_to_perturb, &mut |_| {});
                population[worst_idx].1 = population[worst_idx].0.calculate_cost(instance);
                population.sort_by_key(|(_, cost)| *cost);
            }
        }
        progress_callback(format!(
            "[{}] Finished. Iterations: {}, Best cost: {}",
            self.name_str, iterations, best_cost
        ));
        (best_solution, iterations)
    }

    fn recombine<R: Rng>(
        &self,
        p1: &Solution,
        p2: &Solution,
        instance: &TsplibInstance,
        rng: &mut R,
    ) -> Solution {
        let mut child = p1.clone();
        let mut edges_from_p2 = Vec::new();

        for cycle_id_p2 in [crate::tsplib::CycleId::Cycle1, crate::tsplib::CycleId::Cycle2] {
            let cycle_p2 = p2.get_cycle(cycle_id_p2);
            if cycle_p2.len() < 2 { continue; }
            for i in 0..cycle_p2.len() {
                let u = cycle_p2[i];
                let v = cycle_p2[(i + 1) % cycle_p2.len()];
                if child.has_edge(u, v).is_none() {
                    edges_from_p2.push((u, v));
                }
            }
        }

        edges_from_p2.shuffle(rng);

        let mut current_nodes = child.cycle1.iter().chain(child.cycle2.iter()).cloned().collect::<std::collections::HashSet<usize>>();

        for (u, v) in edges_from_p2.iter().take(instance.dimension / 10) {
            if !current_nodes.contains(u) || !current_nodes.contains(v) {
            }
        }
        if rng.gen_bool(0.5) { p1.clone() } else { p2.clone() }
    }
}

impl TspAlgorithm for Hae {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        let time_limit = Duration::from_secs_f64(instance.dimension as f64 / 100.0 * 1.0);
        let (solution, _) = self.solve_timed(instance, time_limit, progress_callback);
        solution
    }
} 