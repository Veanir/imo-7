use crate::algorithm::{ProgressCallback, TspAlgorithm};
use crate::tsplib::{Solution, TsplibInstance};
use rand::Rng;
use rand::thread_rng;

pub struct WeightedRegretCycle {
    pub k_regret: usize,
    pub regret_weight: f64,
    pub greedy_weight: f64,
}

impl WeightedRegretCycle {
    pub fn new(regret_weight: f64, greedy_weight: f64) -> Self {
        Self {
            k_regret: 2,
            regret_weight,
            greedy_weight,
        }
    }

    pub fn default() -> Self {
        Self::new(1.0, -1.0)
    }

    fn find_nearest(&self, from: usize, available: &[usize], instance: &TsplibInstance) -> usize {
        available
            .iter()
            .min_by_key(|&&vertex| instance.distance(from, vertex))
            .copied()
            .unwrap_or(available[0])
    }

    fn calculate_insertion_cost(
        &self,
        vertex: usize,
        pos: usize,
        cycle: &[usize],
        instance: &TsplibInstance,
    ) -> i32 {
        if cycle.is_empty() {
            return 0;
        }
        if cycle.len() == 1 {
            return instance.distance(cycle[0], vertex) * 2;
        }

        let prev = cycle[if pos == 0 { cycle.len() - 1 } else { pos - 1 }];
        let next = cycle[pos % cycle.len()];

        instance.distance(prev, vertex) + instance.distance(vertex, next)
            - instance.distance(prev, next)
    }

    fn calculate_weighted_score(
        &self,
        vertex: usize,
        cycle: &[usize],
        instance: &TsplibInstance,
    ) -> (f64, usize) {
        if cycle.is_empty() {
            return (0.0, 0);
        }

        let mut costs: Vec<(usize, i32)> = (0..=cycle.len())
            .map(|pos| {
                (
                    pos,
                    self.calculate_insertion_cost(vertex, pos, cycle, instance),
                )
            })
            .collect();

        costs.sort_by_key(|&(_, cost)| cost);

        let best_cost = costs[0].1;
        let k_best_cost = costs
            .get(self.k_regret - 1)
            .map_or(best_cost, |&(_, cost)| cost);
        let regret = k_best_cost - best_cost;

        let weighted_score =
            self.regret_weight * regret as f64 + self.greedy_weight * best_cost as f64;

        (weighted_score, costs[0].0)
    }

    fn select_best_vertex(
        &self,
        cycle: &[usize],
        available: &[usize],
        instance: &TsplibInstance,
    ) -> Option<(usize, usize)> {
        if available.is_empty() {
            return None;
        }

        available
            .iter()
            .map(|&vertex| {
                let (score, pos) = self.calculate_weighted_score(vertex, cycle, instance);
                (vertex, pos, score)
            })
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(v, p, _)| (v, p))
    }
}

impl TspAlgorithm for WeightedRegretCycle {
    fn name(&self) -> &str {
        "Weighted 2-Regret Cycle"
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        let n = instance.size();
        progress_callback(format!("[Init] Size: {}", n));

        if n == 0 {
            return Solution::new(vec![], vec![]);
        }
        if n == 1 {
            return Solution::new(vec![0], vec![]);
        }

        let mut rng = thread_rng();
        let start1 = rng.gen_range(0..n);

        let start2 = (0..n)
            .filter(|&j| j != start1)
            .max_by_key(|&j| instance.distance(start1, j))
            .expect("Should find a furthest node if n >= 2");

        let mut cycle1 = vec![start1];
        let mut cycle2 = vec![start2];
        let mut available: Vec<usize> = (0..n).filter(|&x| x != start1 && x != start2).collect();
        let initial_available_count = available.len();

        progress_callback(format!("[Init] Start nodes: {}, {}", start1, start2));

        if !available.is_empty() {
            let nearest1 = self.find_nearest(start1, &available, instance);
            cycle1.push(nearest1);
            available.retain(|&x| x != nearest1);
            progress_callback(format!("[Init Cycle 1] Added {}", nearest1));

            if !available.is_empty() {
                let nearest2 = self.find_nearest(start2, &available, instance);
                cycle2.push(nearest2);
                available.retain(|&x| x != nearest2);
                progress_callback(format!("[Init Cycle 2] Added {}", nearest2));
            }
        }

        let mut current_cycle_id = 1;
        let total_iterations = available.len();
        let mut iterations_done = 0;

        while !available.is_empty() {
            iterations_done += 1;
            let progress_percent = (iterations_done * 100 / total_iterations.max(1));

            if current_cycle_id == 1 {
                progress_callback(format!(
                    "[{}% C1] Avail: {}",
                    progress_percent,
                    available.len()
                ));
                if let Some((best_vertex, best_pos)) =
                    self.select_best_vertex(&cycle1, &available, instance)
                {
                    cycle1.insert(best_pos, best_vertex);
                    available.retain(|&x| x != best_vertex);
                }
                current_cycle_id = 2;
            } else {
                progress_callback(format!(
                    "[{}% C2] Avail: {}",
                    progress_percent,
                    available.len()
                ));
                if let Some((best_vertex, best_pos)) =
                    self.select_best_vertex(&cycle2, &available, instance)
                {
                    cycle2.insert(best_pos, best_vertex);
                    available.retain(|&x| x != best_vertex);
                }
                current_cycle_id = 1;
            }
        }
        progress_callback("[Finished]".to_string());
        Solution::new(cycle1, cycle2)
    }
}
