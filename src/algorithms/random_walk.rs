use crate::algorithm::ProgressCallback;
use crate::algorithm::TspAlgorithm;
use crate::moves::types::{CycleId, Move};
use crate::tsplib::{Solution, TsplibInstance};
use crate::utils::generate_random_solution;
use rand::{Rng, thread_rng};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RandomWalk {
    max_iterations: usize,
}

impl Default for RandomWalk {
    fn default() -> Self {
        Self {
            max_iterations: 10000,
        }
    }
}

impl RandomWalk {
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    fn generate_random_move(&self, solution: &Solution, rng: &mut impl Rng) -> Option<Move> {
        let n1 = solution.cycle1.len();
        let n2 = solution.cycle2.len();

        if n1 + n2 < 3 {
            return None;
        }

        let move_type_choice = rng.gen_range(0..=2);

        match move_type_choice {
            0 if n1 > 0 && n2 > 0 => {
                let pos1 = rng.gen_range(0..n1);
                let pos2 = rng.gen_range(0..n2);
                let v1 = solution.cycle1[pos1];
                let v2 = solution.cycle2[pos2];
                Some(Move::InterRouteExchange { v1, v2 })
            }
            1 => {
                let cycle_choice = if n1 >= 2 && (n2 < 2 || rng.gen_bool(0.5)) {
                    CycleId::Cycle1
                } else if n2 >= 2 {
                    CycleId::Cycle2
                } else {
                    return None;
                };
                let n = if cycle_choice == CycleId::Cycle1 {
                    n1
                } else {
                    n2
                };
                if n < 2 {
                    return None;
                }
                let pos1 = rng.gen_range(0..n);
                let mut pos2 = rng.gen_range(0..n);
                while pos1 == pos2 {
                    pos2 = rng.gen_range(0..n);
                }
                let cycle_vec = solution.get_cycle(cycle_choice);
                let v1 = cycle_vec[pos1];
                let v2 = cycle_vec[pos2];
                Some(Move::IntraRouteVertexExchange {
                    v1,
                    v2,
                    cycle: cycle_choice,
                })
            }
            2 => {
                let cycle_choice = if n1 >= 3 && (n2 < 3 || rng.gen_bool(0.5)) {
                    CycleId::Cycle1
                } else if n2 >= 3 {
                    CycleId::Cycle2
                } else {
                    return None;
                };
                let n = if cycle_choice == CycleId::Cycle1 {
                    n1
                } else {
                    n2
                };
                if n < 3 {
                    return None;
                }

                let pos1 = rng.gen_range(0..n);
                let mut pos2 = rng.gen_range(0..n);
                while pos1 == pos2 || (pos1 + 1) % n == pos2 || (pos2 + 1) % n == pos1 {
                    pos2 = rng.gen_range(0..n);
                }

                let cycle_vec = solution.get_cycle(cycle_choice);
                let a = cycle_vec[pos1];
                let b = cycle_vec[(pos1 + 1) % n];
                let c = cycle_vec[pos2];
                let d = cycle_vec[(pos2 + 1) % n];

                Some(Move::IntraRouteEdgeExchange {
                    a,
                    b,
                    c,
                    d,
                    cycle: cycle_choice,
                })
            }
            _ => None,
        }
    }
}

impl TspAlgorithm for RandomWalk {
    fn name(&self) -> &str {
        "Random Walk"
    }

    fn solve_with_feedback(
        &self,
        instance: &TsplibInstance,
        progress_callback: ProgressCallback,
    ) -> Solution {
        let mut current_solution = generate_random_solution(instance);
        let mut best_solution = current_solution.clone();
        let mut best_cost = best_solution.calculate_cost(instance);
        let mut rng = thread_rng();

        for i in 0..self.max_iterations {
            if i % 100 == 0 || i == self.max_iterations - 1 {
                progress_callback(format!(
                    "[Iter: {}/{}] Best Cost: {}",
                    i + 1,
                    self.max_iterations,
                    best_cost
                ));
            }

            if let Some(random_move) = self.generate_random_move(&current_solution, &mut rng) {
                random_move.apply(&mut current_solution);
                let current_cost = current_solution.calculate_cost(instance);
                if current_cost < best_cost {
                    best_cost = current_cost;
                    best_solution = current_solution.clone();
                }
            } else {
                break;
            }
        }
        progress_callback(format!("[Finished] Final Best Cost: {}", best_cost));
        best_solution
    }
}
