use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::algorithms::local_search::base::{
    HeuristicAlgorithm, InitialSolutionType, LocalSearch, NeighborhoodType, SearchVariant,
};
use crate::algorithms::perturbation::{Perturbation, SmallPerturbation};
use crate::tsplib::{Solution, TsplibInstance};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Adaptive Variable Neighborhood Search with Learning
pub struct Avns {
    neighborhoods: Vec<NeighborhoodType>,
    search_variant: SearchVariant,
    perturbation_strength: usize,
    learning_rate: f64,
    name_str: String,
}

impl Avns {
    pub fn new(
        neighborhoods: Vec<NeighborhoodType>,
        search_variant: SearchVariant,
        perturbation_strength: usize,
        learning_rate: f64,
    ) -> Self {
        let name_str = format!(
            "AVNS-L (neighborhoods: {:?}, variant: {:?}, perturb: {}, lr: {})",
            neighborhoods, search_variant, perturbation_strength, learning_rate
        );
        Self {
            neighborhoods,
            search_variant,
            perturbation_strength,
            learning_rate,
            name_str,
        }
    }

    pub fn solve_timed(
        &self,
        instance: &TsplibInstance,
        time_limit: Duration,
        mut progress_callback: ProgressCallback,
    ) -> (Solution, usize) {
        let start_time = Instant::now();
        let mut rng = thread_rng();
        
        // Initialize with heuristic solution
        progress_callback("[AVNS-L] Generating initial solution with Weighted Regret...".to_string());
        let initial_ls = LocalSearch::new(
            self.search_variant,
            self.neighborhoods[0],
            InitialSolutionType::Heuristic(HeuristicAlgorithm::WeightedRegret),
            false,
        );
        let mut current_solution = initial_ls.solve_with_feedback(instance, &mut |s| {
            progress_callback(format!("[AVNS-L Init] {}", s))
        });
        let mut current_cost = current_solution.calculate_cost(instance);
        
        let mut best_solution = current_solution.clone();
        let mut best_cost = current_cost;
        
        // Track performance of each neighborhood
        let mut neighborhood_stats: HashMap<usize, (usize, i32)> = HashMap::new(); // (uses, total_improvement)
        for i in 0..self.neighborhoods.len() {
            neighborhood_stats.insert(i, (0, 0));
        }
        
        let mut iterations = 0;
        let mut no_improvement_count = 0;
        let max_no_improvement = 10;
        
        while start_time.elapsed() < time_limit {
            iterations += 1;
            let mut improved = false;
            
            // Select neighborhood based on performance (adaptive selection)
            let neighborhood_idx = if iterations < self.neighborhoods.len() * 2 {
                // Exploration phase: try all neighborhoods equally
                iterations % self.neighborhoods.len()
            } else {
                // Exploitation phase: select based on performance
                self.select_neighborhood_adaptive(&neighborhood_stats, &mut rng)
            };
            
            let neighborhood = self.neighborhoods[neighborhood_idx];
            progress_callback(format!(
                "[AVNS-L Iter {}] Trying neighborhood {:?}, current cost: {}",
                iterations, neighborhood, current_cost
            ));
            
            // Apply local search with selected neighborhood
            let ls = LocalSearch::new(
                self.search_variant,
                neighborhood,
                InitialSolutionType::Random,
                (neighborhood == NeighborhoodType::OrOpt || neighborhood == NeighborhoodType::ThreeOpt),
            );
            
            // Apply local search starting from current solution
            let ls_result = ls.solve_from_solution(
                instance,
                current_solution.clone(),
                &mut |s| progress_callback(format!("[AVNS-L LS] {}", s)),
            );
            let new_cost = ls_result.calculate_cost(instance);
            
            // Update statistics
            let improvement = current_cost - new_cost;
            let stats = neighborhood_stats.get_mut(&neighborhood_idx).unwrap();
            stats.0 += 1;
            stats.1 += improvement;
            
            if new_cost < current_cost {
                current_solution = ls_result;
                current_cost = new_cost;
                improved = true;
                no_improvement_count = 0;
                
                progress_callback(format!(
                    "[AVNS-L Iter {}] Improved! New cost: {} (improvement: {})",
                    iterations, new_cost, improvement
                ));
                
                if new_cost < best_cost {
                    best_solution = current_solution.clone();
                    best_cost = new_cost;
                    progress_callback(format!(
                        "[AVNS-L Iter {}] New global best: {}",
                        iterations, best_cost
                    ));
                }
            } else {
                no_improvement_count += 1;
            }
            
            // Apply perturbation if stuck
            if no_improvement_count >= max_no_improvement {
                progress_callback(format!(
                    "[AVNS-L Iter {}] No improvement for {} iterations, applying perturbation...",
                    iterations, no_improvement_count
                ));
                
                // Apply smart perturbation
                let perturbation_size = self.perturbation_strength + (no_improvement_count / 5);
                let perturbation = SmallPerturbation::new(perturbation_size);
                perturbation.perturb(&mut current_solution, instance, &mut rng);
                current_cost = current_solution.calculate_cost(instance);
                
                no_improvement_count = 0;
                progress_callback(format!(
                    "[AVNS-L Iter {}] After perturbation, cost: {}",
                    iterations, current_cost
                ));
            }
            
            // Check time limit
            if start_time.elapsed() >= time_limit {
                break;
            }
        }
        
        progress_callback(format!(
            "[AVNS-L] Finished. Iterations: {}, Best cost: {}",
            iterations, best_cost
        ));
        
        // Log neighborhood performance
        for (idx, (uses, improvement)) in &neighborhood_stats {
            let avg_improvement = if *uses > 0 {
                (*improvement as f64) / (*uses as f64)
            } else {
                0.0
            };
            progress_callback(format!(
                "[AVNS-L] Neighborhood {:?}: uses={}, avg_improvement={:.2}",
                self.neighborhoods[*idx], uses, avg_improvement
            ));
        }
        
        (best_solution, iterations)
    }
    
    fn select_neighborhood_adaptive<R: Rng>(
        &self,
        stats: &HashMap<usize, (usize, i32)>,
        rng: &mut R,
    ) -> usize {
        // Calculate scores for each neighborhood
        let mut scores: Vec<(usize, f64)> = Vec::new();
        
        for idx in 0..self.neighborhoods.len() {
            let (uses, improvement) = stats.get(&idx).unwrap();
            let score = if *uses == 0 {
                1.0 // Give unexplored neighborhoods a chance
            } else {
                // Score based on average improvement and exploration bonus
                let avg_improvement = (*improvement as f64) / (*uses as f64);
                let exploration_bonus = 1.0 / ((*uses as f64).sqrt() + 1.0);
                avg_improvement.max(0.0) + exploration_bonus * self.learning_rate
            };
            scores.push((idx, score));
        }
        
        // Use roulette wheel selection
        let total_score: f64 = scores.iter().map(|(_, s)| s).sum();
        if total_score <= 0.0 {
            // If all scores are non-positive, select randomly
            return rng.gen_range(0..self.neighborhoods.len());
        }
        
        let mut cumulative = 0.0;
        let random_value = rng.gen_range(0.0..1.0) * total_score;
        
        for (idx, score) in scores {
            cumulative += score;
            if cumulative >= random_value {
                return idx;
            }
        }
        
        // Fallback (should not reach here)
        self.neighborhoods.len() - 1
    }

}

impl TspAlgorithm for Avns {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        // Default time limit based on instance size
        let time_limit = Duration::from_secs(60);
        let (solution, _) = self.solve_timed(instance, time_limit, progress_callback);
        solution
    }
} 