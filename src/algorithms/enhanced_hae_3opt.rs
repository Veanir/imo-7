use crate::algorithm::{ProgressCallback, TspAlgorithm};

use crate::algorithms::local_search::base::{
    HeuristicAlgorithm, InitialSolutionType, LocalSearch, NeighborhoodType, SearchVariant,
};
use crate::algorithms::perturbation::{repair, Perturbation, SmallPerturbation};
use crate::tsplib::{CycleId, Solution, TsplibInstance};
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Enhanced Hybrid Adaptive Evolution with 3-opt
pub struct EnhancedHae3Opt {
    pop_size: usize,
    min_diff: i32,
    edge_exchange_ls: LocalSearch,
    three_opt_ls: LocalSearch,
    heuristic_init_ls: LocalSearch,
    adaptive_local_search: bool,
    elite_size: usize,
    name_str: String,
}

impl EnhancedHae3Opt {
    pub fn new(
        pop_size: usize,
        min_diff: i32,
        adaptive_local_search: bool,
        elite_size: usize,
        verify_3opt_delta: bool,
    ) -> Self {
        let edge_exchange_ls = LocalSearch::new(
            SearchVariant::CandidateSteepest(10),
            NeighborhoodType::EdgeExchange,
            InitialSolutionType::Random,
            false,
        );

        let heuristic_init_ls = LocalSearch::new(
            SearchVariant::CandidateSteepest(10),
            NeighborhoodType::EdgeExchange, 
            InitialSolutionType::Heuristic(HeuristicAlgorithm::WeightedRegret),
            false, 
        );
        
        let three_opt_ls = LocalSearch::new(
            SearchVariant::Steepest, 
            NeighborhoodType::ThreeOpt,
            InitialSolutionType::Random,
            verify_3opt_delta,
        );
        
        let name_str = format!(
            "Enhanced-HAE-3opt (pop={}, min_diff={}, adaptive={}, elite={}, verify3Opt={})",
            pop_size, min_diff, adaptive_local_search, elite_size, verify_3opt_delta
        );
        
        Self {
            pop_size,
            min_diff,
            edge_exchange_ls,
            three_opt_ls,
            heuristic_init_ls,
            adaptive_local_search,
            elite_size,
            name_str,
        }
    }
    
    pub fn solve_timed(
        &self,
        instance: &TsplibInstance,
        time_limit: Duration,
        mut progress_callback: ProgressCallback,
    ) -> (Solution, usize) {
        let mut rng = thread_rng();
        let start_time = Instant::now();
        
        progress_callback("[Enhanced-HAE-3opt] Generating diverse initial population...".to_string());
        let mut population: Vec<(Solution, i32)> = Vec::with_capacity(self.pop_size);
        
        let heuristic_count = self.pop_size / 4;
        for i in 0..heuristic_count {
            progress_callback(format!("[Init {}] Generating heuristic solution", i + 1));
            let sol = self.heuristic_init_ls.solve_with_feedback(instance, &mut |s| {
                progress_callback(format!("[Init Heuristic {}] {}", i + 1, s))
            });
            let cost = sol.calculate_cost(instance);
            population.push((sol, cost));
        }
        
        // Rest with random + local search
        for i in heuristic_count..self.pop_size {
            progress_callback(format!("[Init {}] Generating random LS solution", i + 1));
            let sol = self.edge_exchange_ls.solve_with_feedback(instance, &mut |s| {
                progress_callback(format!("[Init Random {}] {}", i + 1, s))
            });
            let cost = sol.calculate_cost(instance);
            population.push((sol, cost));
        }
        
        // Sort population by cost
        population.sort_by_key(|(_, cost)| *cost);
        
        let mut best_solution = population[0].0.clone();
        let mut best_cost = population[0].1;
        
        let mut iterations = 0;
        let mut neighborhood_performance = [0i32; 2]; // Track performance of each neighborhood
        let mut neighborhood_uses = [0usize; 2];
        
        while start_time.elapsed() < time_limit {
            iterations += 1;
            
            // Select parents with tournament selection
            let parent1_idx = self.tournament_selection(&population, 3, &mut rng);
            let parent2_idx = self.tournament_selection(&population, 3, &mut rng);
            
            let parent1 = &population[parent1_idx].0;
            let parent2 = &population[parent2_idx].0;
            
            progress_callback(format!(
                "[Iter {}] Parents: idx {} (cost {}) and idx {} (cost {})",
                iterations, parent1_idx, population[parent1_idx].1, 
                parent2_idx, population[parent2_idx].1
            ));
            
            // Advanced recombination
            let mut child = self.advanced_recombine(parent1, parent2, instance, &mut rng);
            
            // Apply adaptive local search
            if self.adaptive_local_search {
                let neighborhood_choice = if neighborhood_uses[0] == 0 || neighborhood_uses[1] == 0 {
                    // Ensure both are tried at least once
                    if neighborhood_uses[0] == 0 { 0 } else { 1 }
                } else {
                    // Choose based on average performance with probability
                    let avg_perf_0 = neighborhood_performance[0] as f64 / neighborhood_uses[0] as f64;
                    let avg_perf_1 = neighborhood_performance[1] as f64 / neighborhood_uses[1] as f64;
                    
                    // Choose based on performance difference, mapping to a probability using tanh
                    let scale_factor = 200.0; // Heuristic: typical difference in avg_perf values. Adjust if necessary.
                    let diff_scaled = (avg_perf_0 - avg_perf_1) / scale_factor;
                    // prob_0 is now in [0.0, 1.0] as tanh output is [-1.0, 1.0]
                    let prob_0 = 0.5 * (1.0 + diff_scaled.tanh()); 
                    
                    if rng.gen_bool(prob_0) { 0 } else { 1 }
                };
                
                let cost_before_ls = child.calculate_cost(instance);
                
                child = if neighborhood_choice == 0 {
                    progress_callback(format!("[Iter {}] Applying EdgeExchange LS", iterations));
                    self.edge_exchange_ls.solve_from_solution(
                        instance,
                        child,
                        &mut |s| progress_callback(format!("[Iter {} Edge-LS] {}", iterations, s)),
                    )
                } else {
                    progress_callback(format!("[Iter {}] Applying 3-opt LS", iterations));
                    self.three_opt_ls.solve_from_solution(
                        instance,
                        child,
                        &mut |s| progress_callback(format!("[Iter {} 3opt-LS] {}", iterations, s)),
                    )
                };
                
                let cost_after_ls = child.calculate_cost(instance);
                let improvement = cost_before_ls - cost_after_ls;
                
                neighborhood_performance[neighborhood_choice] += improvement;
                neighborhood_uses[neighborhood_choice] += 1;
                
                progress_callback(format!(
                    "[Iter {}] LS improvement: {} (neighborhood {})",
                    iterations, improvement, if neighborhood_choice == 0 { "EdgeExchange" } else { "3-opt" }
                ));
            } else {
                // Standard local search with edge exchange
                child = self.edge_exchange_ls.solve_from_solution(
                    instance,
                    child,
                    &mut |s| progress_callback(format!("[Iter {} LS] {}", iterations, s)),
                );
            }
            
            let child_cost = child.calculate_cost(instance);
            
            // Elite preservation and replacement strategy
            if child_cost < best_cost {
                best_solution = child.clone();
                best_cost = child_cost;
                progress_callback(format!("[Iter {}] New global best: {}", iterations, best_cost));
            }
            
            // Replace worst if child is better and diverse enough
            let too_similar = population.iter()
                .take(self.elite_size)
                .any(|(_, cost)| (child_cost - *cost).abs() < self.min_diff);
            
            if !too_similar && child_cost < population.last().unwrap().1 {
                population.pop(); // Remove worst
                population.push((child, child_cost));
                population.sort_by_key(|(_, cost)| *cost);
                progress_callback(format!("[Iter {}] Child accepted with cost {}", iterations, child_cost));
            } else if child_cost < population[population.len() / 2].1 {
                // If child is in top 50%, replace a random solution from bottom 50%
                let replace_idx = rng.gen_range(population.len() / 2..population.len());
                population[replace_idx] = (child, child_cost);
                population.sort_by_key(|(_, cost)| *cost);
                progress_callback(format!("[Iter {}] Child replaced solution at idx {}", iterations, replace_idx));
            }
            
            // Diversity injection every 50 iterations
            if iterations % 50 == 0 {
                progress_callback(format!("[Iter {}] Injecting diversity...", iterations));
                let worst_idx = population.len() - 1;
                let perturbation = SmallPerturbation::new(10);
                perturbation.perturb(&mut population[worst_idx].0, instance, &mut rng);
                population[worst_idx].1 = population[worst_idx].0.calculate_cost(instance);
            }
        }
        
        progress_callback(format!(
            "[Enhanced-HAE-3opt] Finished. Iterations: {}, Best cost: {}",
            iterations, best_cost
        ));
        
        if self.adaptive_local_search {
            for i in 0..2 {
                if neighborhood_uses[i] > 0 {
                    let avg_perf = neighborhood_performance[i] as f64 / neighborhood_uses[i] as f64;
                    progress_callback(format!(
                        "[Enhanced-HAE-3opt] Neighborhood {}: uses={}, avg_improvement={:.2}",
                        if i == 0 { "EdgeExchange" } else { "3-opt" },
                        neighborhood_uses[i], avg_perf
                    ));
                }
            }
        }
        
        (best_solution, iterations)
    }
    
    fn tournament_selection<R: Rng>(
        &self,
        population: &[(Solution, i32)],
        tournament_size: usize,
        rng: &mut R,
    ) -> usize {
        let mut best_idx = rng.gen_range(0..population.len());
        let mut best_cost = population[best_idx].1;
        
        for _ in 1..tournament_size {
            let idx = rng.gen_range(0..population.len());
            if population[idx].1 < best_cost {
                best_idx = idx;
                best_cost = population[idx].1;
            }
        }
        
        best_idx
    }
    
    fn advanced_recombine<R: Rng>(
        &self,
        p1: &Solution,
        p2: &Solution,
        instance: &TsplibInstance,
        rng: &mut R,
    ) -> Solution {
        // Start from better parent
        let mut child = if p1.calculate_cost(instance) < p2.calculate_cost(instance) {
            p1.clone()
        } else {
            p2.clone()
        };
        
        let mut destroyed: HashSet<usize> = HashSet::new();
        
        // Destroy edges not common to both parents
        for &cycle_id in &[CycleId::Cycle1, CycleId::Cycle2] {
            let cycle = child.get_cycle(cycle_id);
            let n = cycle.len();
            for i in 0..n {
                let a = cycle[i];
                let b = cycle[(i + 1) % n];
                
                // Check if edge exists in other parent
                if p2.has_edge(a, b).is_none() && p1.has_edge(a, b).is_none() {
                    destroyed.insert(a);
                    destroyed.insert(b);
                } else if rng.gen_bool(0.1) {
                    // 10% chance to destroy even common edges for diversity
                    destroyed.insert(a);
                    destroyed.insert(b);
                }
            }
        }
        
        // Additional targeted destruction based on node costs
        let mut node_costs: Vec<(usize, i32)> = Vec::new();
        for node in 0..instance.dimension {
            if !destroyed.contains(&node) {
                let mut total_cost = 0;
                for other in 0..instance.dimension {
                    if node != other {
                        total_cost += instance.distance(node, other);
                    }
                }
                node_costs.push((node, total_cost));
            }
        }
        node_costs.sort_by_key(|(_, cost)| -*cost);
        
        // Destroy 10% of highest cost nodes
        let destroy_count = (node_costs.len() as f64 * 0.1).ceil() as usize;
        for i in 0..destroy_count.min(node_costs.len()) {
            destroyed.insert(node_costs[i].0);
        }
        
        // Remove destroyed nodes
        child.cycle1.retain(|v| !destroyed.contains(v));
        child.cycle2.retain(|v| !destroyed.contains(v));
        
        // Repair using advanced heuristic
        repair(&mut child, instance, destroyed);
        
        child
    }
}

impl TspAlgorithm for EnhancedHae3Opt {
    fn name(&self) -> &str {
        &self.name_str
    }
    
    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        let time_limit = Duration::from_secs(60);
        let (solution, _) = self.solve_timed(instance, time_limit, progress_callback);
        solution
    }
} 