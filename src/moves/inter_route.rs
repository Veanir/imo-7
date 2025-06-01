use crate::moves::types::{CycleId, EvaluatedMove, Move};
use crate::tsplib::{Solution, TsplibInstance};

pub fn evaluate_inter_route_exchange(
    solution: &Solution,
    instance: &TsplibInstance,
    pos1: usize, // Position of node u in cycle 1
    pos2: usize, // Position of node v in cycle 2
) -> Option<EvaluatedMove> {
    let cycle1 = &solution.cycle1;
    let cycle2 = &solution.cycle2;
    let n1 = cycle1.len();
    let n2 = cycle2.len();

    // Ensure cycles are non-empty and positions are valid
    if n1 == 0 || n2 == 0 || pos1 >= n1 || pos2 >= n2 {
        return None;
    }

    // Vertices to be swapped
    let u = cycle1[pos1]; // Node from cycle 1
    let v = cycle2[pos2]; // Node from cycle 2

    let delta = if n1 == 1 && n2 == 1 {
        // Swapping two single-node cycles doesn't change cost
        0
    } else if n1 == 1 {
        // Cycle 1 has only node u
        // Remove v from cycle 2 and insert u
        let prev_v = cycle2[if pos2 == 0 { n2 - 1 } else { pos2 - 1 }];
        let next_v = cycle2[(pos2 + 1) % n2];
        // Delta = (dist(prev_v, u) + dist(u, next_v)) - (dist(prev_v, v) + dist(v, next_v))
        // If n2 == 2, prev_v == next_v, delta = 2*dist(prev_v, u) - 2*dist(prev_v, v)
        if n2 == 2 {
            2 * instance.distance(prev_v, u) - 2 * instance.distance(prev_v, v)
        } else {
            (instance.distance(prev_v, u) + instance.distance(u, next_v))
                - (instance.distance(prev_v, v) + instance.distance(v, next_v))
        }
    } else if n2 == 1 {
        // Cycle 2 has only node v
        // Remove u from cycle 1 and insert v
        let prev_u = cycle1[if pos1 == 0 { n1 - 1 } else { pos1 - 1 }];
        let next_u = cycle1[(pos1 + 1) % n1];
        // Delta = (dist(prev_u, v) + dist(v, next_u)) - (dist(prev_u, u) + dist(u, next_u))
        // If n1 == 2, prev_u == next_u, delta = 2*dist(prev_u, v) - 2*dist(prev_u, u)
        if n1 == 2 {
            2 * instance.distance(prev_u, v) - 2 * instance.distance(prev_u, u)
        } else {
            (instance.distance(prev_u, v) + instance.distance(v, next_u))
                - (instance.distance(prev_u, u) + instance.distance(u, next_u))
        }
    } else {
        // Both cycles have >= 2 nodes
        let prev_u = cycle1[if pos1 == 0 { n1 - 1 } else { pos1 - 1 }];
        let next_u = cycle1[(pos1 + 1) % n1];
        let prev_v = cycle2[if pos2 == 0 { n2 - 1 } else { pos2 - 1 }];
        let next_v = cycle2[(pos2 + 1) % n2];

        // Calculate cost change in Cycle 1 (replace u with v)
        let delta_c1 = if n1 == 2 {
            // remove 2*dist(prev_u, u), add 2*dist(prev_u, v)
            2 * instance.distance(prev_u, v) - 2 * instance.distance(prev_u, u)
        } else {
            (instance.distance(prev_u, v) + instance.distance(v, next_u))
                - (instance.distance(prev_u, u) + instance.distance(u, next_u))
        };

        // Calculate cost change in Cycle 2 (replace v with u)
        let delta_c2 = if n2 == 2 {
            // remove 2*dist(prev_v, v), add 2*dist(prev_v, u)
            2 * instance.distance(prev_v, u) - 2 * instance.distance(prev_v, v)
        } else {
            (instance.distance(prev_v, u) + instance.distance(u, next_v))
                - (instance.distance(prev_v, v) + instance.distance(v, next_v))
        };

        delta_c1 + delta_c2
    };

    Some(EvaluatedMove {
        move_type: Move::InterRouteExchange { v1: u, v2: v }, // Store node IDs
        delta,
    })
}
