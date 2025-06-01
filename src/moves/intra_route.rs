use crate::moves::types::{CycleId, EvaluatedMove, Move};
use crate::tsplib::{Solution, TsplibInstance};

pub fn evaluate_intra_route_vertex_exchange(
    solution: &Solution,
    instance: &TsplibInstance,
    cycle: CycleId,
    pos1: usize,
    pos2: usize,
) -> Option<EvaluatedMove> {
    let cycle_vec = solution.get_cycle(cycle);
    let n = cycle_vec.len();

    // Need at least 2 nodes to swap.
    if n < 2 || pos1 == pos2 || pos1 >= n || pos2 >= n {
        return None; // Invalid move
    }

    // Ensure pos1 < pos2 for easier neighbor calculation, doesn't affect result
    let (pos1, pos2) = (pos1.min(pos2), pos1.max(pos2));

    let v1 = cycle_vec[pos1];
    let v2 = cycle_vec[pos2];

    // Calculate delta based on adjacency
    let delta = if n == 2 {
        // Only two nodes, swapping them doesn't change the cycle or cost.
        0
    } else if pos2 == pos1 + 1 || (pos1 == 0 && pos2 == n - 1) {
        // Adjacent nodes (including wrap-around)
        // Find neighbours correctly considering wrap-around for both cases
        let prev1 = cycle_vec[if pos1 == 0 { n - 1 } else { pos1 - 1 }];
        let next2 = cycle_vec[(pos2 + 1) % n]; // next of v2

        // If adjacent: ..., prev1, v1, v2, next2, ... swapped to ..., prev1, v2, v1, next2, ...
        // Edges removed: (prev1, v1), (v1, v2), (v2, next2)
        // Edges added:   (prev1, v2), (v2, v1), (v1, next2)
        // Delta = Added - Removed
        (instance.distance(prev1, v2) + instance.distance(v2, v1) + instance.distance(v1, next2))
            - (instance.distance(prev1, v1)
                + instance.distance(v1, v2)
                + instance.distance(v2, next2))
    } else {
        // Non-adjacent nodes
        let prev1 = cycle_vec[if pos1 == 0 { n - 1 } else { pos1 - 1 }];
        let next1 = cycle_vec[(pos1 + 1) % n]; // Should exist since n > 2 and not adjacent
        let prev2 = cycle_vec[if pos2 == 0 { n - 1 } else { pos2 - 1 }]; // Should exist
        let next2 = cycle_vec[(pos2 + 1) % n];

        // Edges removed: (prev1, v1), (v1, next1), (prev2, v2), (v2, next2)
        // Edges added:   (prev1, v2), (v2, next1), (prev2, v1), (v1, next2)
        // Delta = Added - Removed
        (instance.distance(prev1, v2)
            + instance.distance(v2, next1)
            + instance.distance(prev2, v1)
            + instance.distance(v1, next2))
            - (instance.distance(prev1, v1)
                + instance.distance(v1, next1)
                + instance.distance(prev2, v2)
                + instance.distance(v2, next2))
    };

    Some(EvaluatedMove {
        move_type: Move::IntraRouteVertexExchange { v1, v2, cycle }, // Use correct field names
        delta,
    })
}

/// Calculates the cost delta for exchanging edges `(a, b)` and `(c, d)`
/// within the specified `cycle`, where `a=cycle[pos1]`, `b=cycle[pos1+1]`,
/// `c=cycle[pos2]`, `d=cycle[pos2+1]`.
/// This is a 2-opt move.
///
/// Assumes `pos1` and `pos2` represent the *start* indices of the edges to be removed.
/// Returns `None` if the move is invalid (e.g., cycle size < 3, adjacent edges).
pub fn evaluate_intra_route_edge_exchange(
    solution: &Solution,
    instance: &TsplibInstance,
    cycle: CycleId,
    pos1: usize, // Index of node `a`
    pos2: usize, // Index of node `c`
) -> Option<EvaluatedMove> {
    let cycle_vec = solution.get_cycle(cycle);
    let n = cycle_vec.len();

    // Need at least 3 nodes for non-degenerate 2-opt.
    // Ensure pos1 and pos2 are valid indices.
    // Ensure edges are not adjacent or overlapping.
    if n < 3
        || pos1 >= n
        || pos2 >= n
        || pos1 == pos2
        || (pos1 + 1) % n == pos2
        || (pos2 + 1) % n == pos1
    {
        return None;
    }

    // Nodes defining the edges to be removed: (a, b) and (c, d)
    let a = cycle_vec[pos1];
    let b = cycle_vec[(pos1 + 1) % n];
    let c = cycle_vec[pos2];
    let d = cycle_vec[(pos2 + 1) % n];

    // Cost removed: dist(a, b) + dist(c, d)
    let cost_removed = instance.distance(a, b) + instance.distance(c, d);

    // Cost added: dist(a, c) + dist(b, d)
    let cost_added = instance.distance(a, c) + instance.distance(b, d);

    let delta = cost_added - cost_removed;

    Some(EvaluatedMove {
        move_type: Move::IntraRouteEdgeExchange { a, b, c, d, cycle }, // Use correct field names
        delta,
    })
}

/// Calculates the cost delta for a specific candidate 2-opt move:
/// removing edges (a, a_next) and (b, b_next) and adding (a, b) and (a_next, b_next).
/// This is used in the Candidate Moves strategy. It considers performing a
/// 2-opt move by removing edges (a, a_next) and (b, b_next), and adding
/// edges (a, b) and (a_next, b_next).
/// `pos_a` is the index of node `a`, `pos_b` is the index of node `b`.
pub fn evaluate_candidate_intra_route_edge_exchange(
    solution: &Solution,
    instance: &TsplibInstance,
    cycle_id: CycleId,
    pos_a: usize,
    pos_b: usize,
) -> Option<EvaluatedMove> {
    let cycle_vec = solution.get_cycle(cycle_id);
    let n = cycle_vec.len();

    // Basic validation
    if n < 3 || pos_a >= n || pos_b >= n || pos_a == pos_b {
        return None;
    }

    let a = cycle_vec[pos_a];
    let b = cycle_vec[pos_b];

    let pos_a_next = (pos_a + 1) % n;
    let pos_b_next = (pos_b + 1) % n;

    // Ensure the edges we intend to remove are not adjacent or overlapping
    // i.e., a_next != b and b_next != a
    if pos_a_next == pos_b || pos_b_next == pos_a {
        return None; // Invalid 2-opt move topology for these positions
    }

    let a_next = cycle_vec[pos_a_next];
    let b_next = cycle_vec[pos_b_next];

    // Cost removed: dist(a, a_next) + dist(b, b_next)
    let cost_removed = instance.distance(a, a_next) + instance.distance(b, b_next);

    // Cost added: dist(a, b) + dist(a_next, b_next)
    let cost_added = instance.distance(a, b) + instance.distance(a_next, b_next);

    let delta = cost_added - cost_removed;

    // Store the move in the standard IntraRouteEdgeExchange format.
    // Removed edges were (a, a_next) and (b, b_next).
    // Apply function expects { a: w, b: x, c: y, d: z } where removed edges are (w, x) and (y, z).
    Some(EvaluatedMove {
        move_type: Move::IntraRouteEdgeExchange {
            a,         // w = a
            b: a_next, // x = a_next
            c: b,      // y = b
            d: b_next, // z = b_next
            cycle: cycle_id,
        },
        delta,
    })
}

/// Evaluate 3-opt move - removes 3 edges and reconnects in the best way
pub fn evaluate_intra_route_3opt(
    solution: &Solution,
    instance: &TsplibInstance,
    cycle_id: CycleId,
    pos1: usize,
    pos2: usize,
    pos3: usize,
) -> Option<EvaluatedMove> {
    let cycle = solution.get_cycle(cycle_id);
    let n = cycle.len();
    
    if n < 6 {
        return None; // Need at least 6 nodes for 3-opt
    }
    
    // Ensure positions are ordered and valid
    if pos1 >= pos2 || pos2 >= pos3 || pos3 >= n {
        return None;
    }
    
    // Ensure segments have at least 1 node
    if pos2 - pos1 < 1 || pos3 - pos2 < 1 {
        return None;
    }
    
    // Get the nodes at break points
    let a = cycle[pos1];
    let b = cycle[(pos1 + 1) % n];
    let c = cycle[pos2];
    let d = cycle[(pos2 + 1) % n];
    let e = cycle[pos3];
    let f = cycle[(pos3 + 1) % n];
    
    // Current cost of the 3 edges
    let current_cost = instance.distance(a, b) + instance.distance(c, d) + instance.distance(e, f);
    
    // Try all possible reconnections (excluding the current one)
    let mut best_delta = 0;
    let mut best_case = 0;
    
    // Case 1: a-c, b-e, d-f (reverse segment 1)
    let case1_cost = instance.distance(a, c) + instance.distance(b, e) + instance.distance(d, f);
    let case1_delta = case1_cost - current_cost;
    if case1_delta < best_delta {
        best_delta = case1_delta;
        best_case = 1;
    }
    
    // Case 2: a-e, f-c, d-b (reverse segment 2)
    let case2_cost = instance.distance(a, e) + instance.distance(f, c) + instance.distance(d, b);
    let case2_delta = case2_cost - current_cost;
    if case2_delta < best_delta {
        best_delta = case2_delta;
        best_case = 2;
    }
    
    // Case 3: a-d, e-b, f-c (reverse segment 3)
    let case3_cost = instance.distance(a, d) + instance.distance(e, b) + instance.distance(f, c);
    let case3_delta = case3_cost - current_cost;
    if case3_delta < best_delta {
        best_delta = case3_delta;
        best_case = 3;
    }
    
    // Case 4: a-d, e-c, f-b (reverse segments 1 and 2)
    let case4_cost = instance.distance(a, d) + instance.distance(e, c) + instance.distance(f, b);
    let case4_delta = case4_cost - current_cost;
    if case4_delta < best_delta {
        best_delta = case4_delta;
        best_case = 4;
    }
    
    if best_delta < 0 {
        Some(EvaluatedMove {
            move_type: Move::IntraRoute3Opt {
                pos1,
                pos2,
                pos3,
                cycle: cycle_id,
                case: best_case,
            },
            delta: best_delta,
        })
    } else {
        None
    }
}

/// Evaluate Or-opt move - relocate a chain of k nodes
pub fn evaluate_intra_route_or_opt(
    solution: &Solution,
    instance: &TsplibInstance,
    cycle_id: CycleId,
    from_pos: usize,
    chain_length: usize,
    to_pos: usize,
) -> Option<EvaluatedMove> {
    let cycle_vec = solution.get_cycle(cycle_id);
    let n = cycle_vec.len();

    if n < 4 || chain_length == 0 || chain_length > 3 {
        return None;
    }

    if from_pos + chain_length > n { 
        return None;
    }
    
    // Adjusted condition to prevent inserting chain into itself or creating trivial moves.
    // This logic was refined through several iterations.
    if from_pos + chain_length == n { // Chain is at the end, N is cycle_vec[0]
        if to_pos >= from_pos || to_pos == 0 { 
            return None;
        }
    } else { // N is at from_pos + chain_length
        if to_pos >= from_pos && to_pos <= from_pos + chain_length {
            return None;
        }
    }
    // Additional check for inserting back into its own exact place
    let p_idx = if from_pos == 0 { n - 1 } else { from_pos - 1 };
    let n_idx = (from_pos + chain_length) % n;
    let ip_idx = if to_pos == 0 { n - 1 } else { to_pos - 1 };
    let in_idx = to_pos % n; 
    if p_idx == ip_idx && n_idx == in_idx {
        return None; 
    }

    let p_node = cycle_vec[p_idx];
    let s1_node = cycle_vec[from_pos];
    let sk_node = cycle_vec[from_pos + chain_length - 1];
    let n_node = cycle_vec[n_idx];
    let ip_node = cycle_vec[ip_idx];
    let in_node = cycle_vec[in_idx];

    let cost_removed = instance.distance(p_node, s1_node) +
                       instance.distance(sk_node, n_node) +
                       instance.distance(ip_node, in_node);

    let cost_added = instance.distance(p_node, n_node) +
                     instance.distance(ip_node, s1_node) +
                     instance.distance(sk_node, in_node);

    let delta = cost_added - cost_removed;

    if delta < 0 {
        Some(EvaluatedMove {
            move_type: Move::IntraRouteOrOpt {
                from_pos,
                chain_length,
                to_pos,
                cycle: cycle_id,
            },
            delta,
        })
    } else {
        None
    }
}
