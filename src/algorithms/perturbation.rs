use crate::algorithms::constructive::weighted_regret_cycle::WeightedRegretCycle;
use crate::moves::types::{CycleId, Move};
use crate::tsplib::{Solution, TsplibInstance};
use rand::Rng;
use rand::seq::{IndexedMutRandom, SliceRandom};
use std::collections::HashSet;

pub trait Perturbation {
    fn name(&self) -> String;
    fn perturb<R: Rng + ?Sized>(
        &self,
        solution: &mut Solution,
        instance: &TsplibInstance,
        rng: &mut R,
    );
}

// --- Small Perturbation (for ILS) ---

#[derive(Debug, Clone)]
pub struct SmallPerturbation {
    num_moves: usize,
}

impl SmallPerturbation {
    pub fn new(num_moves: usize) -> Self {
        Self { num_moves }
    }
}

impl Perturbation for SmallPerturbation {
    fn name(&self) -> String {
        format!("SmallPerturbation(n_moves={})", self.num_moves)
    }

    fn perturb<R: Rng + ?Sized>(
        &self,
        solution: &mut Solution,
        _instance: &TsplibInstance,
        rng: &mut R,
    ) {
        for _ in 0..self.num_moves {
            if let Some(random_move) = generate_random_move(solution, rng) {
                // Apply the move directly without checking delta
                random_move.apply(solution);
            } else {
                // Could happen if cycles are too small for any moves
                break;
            }
        }
    }
}

fn generate_random_move<R: Rng + ?Sized>(solution: &Solution, rng: &mut R) -> Option<Move> {
    let n1 = solution.cycle1.len();
    let n2 = solution.cycle2.len();

    // Available move types depend on cycle sizes
    let mut possible_move_types = Vec::new();
    if n1 >= 2 && n2 >= 2 {
        possible_move_types.push(0);
    } // Inter-route exchange
    if n1 >= 2 {
        possible_move_types.push(1);
    } // Intra-vertex C1
    if n2 >= 2 {
        possible_move_types.push(2);
    } // Intra-vertex C2
    if n1 >= 4 {
        possible_move_types.push(3);
    } // Intra-edge C1
    if n2 >= 4 {
        possible_move_types.push(4);
    } // Intra-edge C2

    if possible_move_types.is_empty() {
        return None; // No possible moves
    }

    // Choose a random move type and generate it
    let choice = *possible_move_types.choose_mut(rng).unwrap();
    match choice {
        0 => generate_random_inter_route_exchange(solution, rng),
        1 => generate_random_intra_vertex_exchange(solution, rng, CycleId::Cycle1),
        2 => generate_random_intra_vertex_exchange(solution, rng, CycleId::Cycle2),
        3 => generate_random_intra_edge_exchange(solution, rng, CycleId::Cycle1),
        4 => generate_random_intra_edge_exchange(solution, rng, CycleId::Cycle2),
        _ => unreachable!(),
    }
}

fn generate_random_inter_route_exchange<R: Rng + ?Sized>(
    solution: &Solution,
    rng: &mut R,
) -> Option<Move> {
    let n1 = solution.cycle1.len();
    let n2 = solution.cycle2.len();
    if n1 == 0 || n2 == 0 {
        return None;
    }
    let pos1 = rng.gen_range(0..n1);
    let pos2 = rng.gen_range(0..n2);
    Some(Move::InterRouteExchange {
        v1: solution.cycle1[pos1],
        v2: solution.cycle2[pos2],
    })
}

fn generate_random_intra_vertex_exchange<R: Rng + ?Sized>(
    solution: &Solution,
    rng: &mut R,
    cycle_id: CycleId,
) -> Option<Move> {
    let cycle = solution.get_cycle(cycle_id);
    let n = cycle.len();
    if n < 2 {
        return None;
    }
    let pos1 = rng.gen_range(0..n);
    let mut pos2 = rng.gen_range(0..n);
    while pos1 == pos2 {
        pos2 = rng.gen_range(0..n);
    }
    Some(Move::IntraRouteVertexExchange {
        v1: cycle[pos1],
        v2: cycle[pos2],
        cycle: cycle_id,
    })
}

fn generate_random_intra_edge_exchange<R: Rng + ?Sized>(
    solution: &Solution,
    rng: &mut R,
    cycle_id: CycleId,
) -> Option<Move> {
    let cycle = solution.get_cycle(cycle_id);
    let n = cycle.len();
    if n < 4 {
        // Need at least 4 nodes to ensure non-adjacent edges can be picked
        return None;
    }

    // Pick first edge (a, b)
    let pos1 = rng.gen_range(0..n);
    let a = cycle[pos1];
    let b = cycle[(pos1 + 1) % n];

    // Pick second edge (c, d), ensuring it's not adjacent to the first
    let mut pos2 = rng.gen_range(0..n);
    // Avoid picking the same edge or adjacent edges
    while pos2 == pos1 || pos2 == (pos1 + 1) % n || pos2 == (pos1 + n - 1) % n {
        pos2 = rng.gen_range(0..n);
    }
    let c = cycle[pos2];
    let d = cycle[(pos2 + 1) % n];

    Some(Move::IntraRouteEdgeExchange {
        a,
        b,
        c,
        d,
        cycle: cycle_id,
    })
}

// --- Large Perturbation (for LNS) ---

#[derive(Debug, Clone)]
pub struct LargePerturbation {
    destroy_fraction: f64, // e.g., 0.2 for 20%
                           // We'll use WeightedRegretCycle for repair implicitly for now
}

impl LargePerturbation {
    pub fn new(destroy_fraction: f64) -> Self {
        assert!(
            destroy_fraction > 0.0 && destroy_fraction < 1.0,
            "Destroy fraction must be between 0 and 1"
        );
        Self { destroy_fraction }
    }
}

impl Perturbation for LargePerturbation {
    fn name(&self) -> String {
        format!("LargePerturbation(destroy={:.2})", self.destroy_fraction)
    }

    fn perturb<R: Rng + ?Sized>(
        &self,
        solution: &mut Solution,
        instance: &TsplibInstance,
        rng: &mut R,
    ) {
        let nodes_to_remove_count =
            ((instance.dimension as f64 * self.destroy_fraction) / 2.0).round() as usize * 2;
        if nodes_to_remove_count == 0 {
            return;
        }

        let destroyed_nodes = destroy(solution, nodes_to_remove_count, rng);
        repair(solution, instance, destroyed_nodes);
    }
}

fn destroy<R: Rng + ?Sized>(
    solution: &mut Solution,
    nodes_to_remove_count: usize,
    rng: &mut R,
) -> HashSet<usize> {
    let mut all_nodes: Vec<usize> = solution
        .cycle1
        .iter()
        .chain(solution.cycle2.iter())
        .cloned()
        .collect();
    all_nodes.shuffle(rng);

    let nodes_to_remove: HashSet<usize> =
        all_nodes.into_iter().take(nodes_to_remove_count).collect();

    solution
        .cycle1
        .retain(|node| !nodes_to_remove.contains(node));
    solution
        .cycle2
        .retain(|node| !nodes_to_remove.contains(node));

    nodes_to_remove
}

pub(crate) fn repair(solution: &mut Solution, instance: &TsplibInstance, destroyed_nodes: HashSet<usize>) {
    // Compute target sizes for two cycles to enforce balance
    let total_size = instance.size();
    let target1 = (total_size + 1) / 2;
    let target2 = total_size - target1;
    let mut remaining_nodes: Vec<usize> = destroyed_nodes.into_iter().collect();

    // Implementation based on `solve_regret_init` from python_reference.py
    while !remaining_nodes.is_empty() {
        let mut best_node_idx = 0;
        let mut best_insertion: Option<(usize, CycleId)> = None; // (insert_pos, cycle_id)
        let mut max_weighted_regret = -f64::INFINITY;

        for (node_idx, &node_to_insert) in remaining_nodes.iter().enumerate() {
            let mut insertion_costs: Vec<(i32, usize, CycleId)> = Vec::new(); // (cost_delta, insert_pos, cycle_id)

            // Evaluate insertion only into cycles that haven't reached target size
            for cycle_id in [CycleId::Cycle1, CycleId::Cycle2] {
                let cycle = solution.get_cycle(cycle_id);
                let n = cycle.len();
                // Determine capacity for this cycle
                let cap = if cycle_id == CycleId::Cycle1 { target1 } else { target2 };
                if n >= cap {
                    // Skip insertion into a full cycle
                    continue;
                }
                if n == 0 {
                    // Inserting into an empty cycle: delta is 0 for the first node
                    insertion_costs.push((0, 0, cycle_id));
                    continue;
                }
                for i in 0..=n {
                    let prev_node = cycle[if i == 0 { n - 1 } else { i - 1 }];
                    let next_node = cycle[i % n];
                    let delta = instance.distance(prev_node, node_to_insert)
                        + instance.distance(node_to_insert, next_node)
                        - instance.distance(prev_node, next_node);
                    insertion_costs.push((delta, i, cycle_id));
                }
            }

            if insertion_costs.is_empty() {
                // Should not happen if instance has nodes
                continue;
            }

            // Sort by cost delta to find best and second best
            insertion_costs.sort_unstable_by_key(|k| k.0);

            let best_cost = insertion_costs[0].0;
            let current_best_insertion = (insertion_costs[0].1, insertion_costs[0].2);

            // Calculate regret (Python: np.diff(np.partition(scores, 1)[:, :2]))
            let regret = if insertion_costs.len() > 1 {
                (insertion_costs[1].0 - best_cost) as f64
            } else {
                0.0 // No regret if only one possible insertion spot
            };

            // Weighted Regret (Python: weight = regret - 0.37 * np.min(scores, axis=1))
            let weight_factor = 0.37; // Same as in the Python reference
            let weighted_regret = regret - weight_factor * (best_cost as f64);

            if weighted_regret > max_weighted_regret {
                max_weighted_regret = weighted_regret;
                best_node_idx = node_idx;
                best_insertion = Some(current_best_insertion);
            }
        }

        // Perform the best insertion found based on weighted regret
        if let Some((insert_pos, cycle_id)) = best_insertion {
            let node_to_insert = remaining_nodes.remove(best_node_idx);
            let cycle = solution.get_cycle_mut(cycle_id);
            // Ensure insertion position is valid for the current cycle length
            let actual_insert_pos = insert_pos % (cycle.len() + 1);
            cycle.insert(actual_insert_pos, node_to_insert);
        } else {
            // This might happen if remaining_nodes was empty initially or no valid insertions found
            if !remaining_nodes.is_empty() {
                eprintln!(
                    "[WARN] Repair phase could not find best insertion for remaining nodes. Aborting."
                );
            }
            break;
        }
    }

    if !remaining_nodes.is_empty() {
        eprintln!(
            "[WARN] Repair phase finished with {} un-inserted nodes.",
            remaining_nodes.len()
        );
    }
}
