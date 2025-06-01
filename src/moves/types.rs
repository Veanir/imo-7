use crate::tsplib::Solution;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CycleId {
    Cycle1,
    Cycle2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Move {
    InterRouteExchange {
        v1: usize,
        v2: usize,
    },
    IntraRouteVertexExchange {
        v1: usize,
        v2: usize,
        cycle: CycleId,
    },
    IntraRouteEdgeExchange {
        a: usize,
        b: usize,
        c: usize,
        d: usize,
        cycle: CycleId,
    },
    IntraRoute3Opt {
        pos1: usize,
        pos2: usize,
        pos3: usize,
        cycle: CycleId,
        case: u8,
    },
    IntraRouteOrOpt {
        from_pos: usize,
        chain_length: usize,
        to_pos: usize,
        cycle: CycleId,
    },
}

#[derive(Debug, Clone)]
pub struct EvaluatedMove {
    pub move_type: Move,
    pub delta: i32,
}

impl Move {
    pub fn apply(&self, solution: &mut Solution) {
        match self {
            Move::InterRouteExchange { v1, v2 } => {
                let pos1_opt = solution.find_node(*v1);
                let pos2_opt = solution.find_node(*v2);

                if let (Some((CycleId::Cycle1, pos1)), Some((CycleId::Cycle2, pos2))) =
                    (pos1_opt, pos2_opt)
                {
                    solution.cycle1[pos1] = *v2;
                    solution.cycle2[pos2] = *v1;
                } else if let (Some((CycleId::Cycle2, pos1)), Some((CycleId::Cycle1, pos2))) =
                    (pos1_opt, pos2_opt)
                {
                    solution.cycle2[pos1] = *v2;
                    solution.cycle1[pos2] = *v1;
                } else {
                    eprintln!(
                        "Warning: InterRouteExchange apply failed. Nodes {} or {} not found in expected cycles.",
                        v1, v2
                    );
                }
            }
            Move::IntraRouteVertexExchange { v1, v2, cycle } => {
                if let (Some((c1, pos1)), Some((c2, pos2))) =
                    (solution.find_node(*v1), solution.find_node(*v2))
                {
                    if c1 == *cycle && c2 == *cycle {
                        let cycle_vec = solution.get_cycle_mut(*cycle);
                        cycle_vec.swap(pos1, pos2);
                    } else {
                        eprintln!(
                            "Warning: IntraRouteVertexExchange apply failed. Nodes {} or {} not in cycle {:?}.",
                            v1, v2, cycle
                        );
                    }
                } else {
                    eprintln!(
                        "Warning: IntraRouteVertexExchange apply failed. Nodes {} or {} not found.",
                        v1, v2
                    );
                }
            }
            Move::IntraRouteEdgeExchange {
                a,
                b,
                c,
                d: _,
                cycle,
            } => {
                if let (Some((cb, pos_b)), Some((cc, pos_c))) =
                    (solution.find_node(*b), solution.find_node(*c))
                {
                    if cb == *cycle && cc == *cycle {
                        let cycle_vec = solution.get_cycle_mut(*cycle);
                        let n = cycle_vec.len();
                        if n < 2 {
                            return;
                        }

                        let mut start = pos_b;
                        let mut end = pos_c;

                        if start > end {
                            let mut temp_slice = Vec::with_capacity(n);
                            temp_slice.extend_from_slice(&cycle_vec[start..]);
                            temp_slice.extend_from_slice(&cycle_vec[..=end]);
                            temp_slice.reverse();
                            let mut temp_iter = temp_slice.into_iter();
                            for i in start..n {
                                cycle_vec[i] = temp_iter.next().unwrap();
                            }
                            for i in 0..=end {
                                cycle_vec[i] = temp_iter.next().unwrap();
                            }
                        } else {
                            cycle_vec[start..=end].reverse();
                        }
                    } else {
                        eprintln!(
                            "Warning: IntraRouteEdgeExchange apply failed. Nodes {} or {} not in cycle {:?}.",
                            b, c, cycle
                        );
                    }
                } else {
                    eprintln!(
                        "Warning: IntraRouteEdgeExchange apply failed. Nodes {} or {} not found.",
                        b, c
                    );
                }
            }
            Move::IntraRoute3Opt { pos1, pos2, pos3, cycle, case } => {
                let cycle_vec = solution.get_cycle_mut(*cycle);
                let n = cycle_vec.len();
                
                // Create segments
                let mut new_cycle = Vec::with_capacity(n);
                
                match case {
                    1 => {
                        // a-c, b-e, d-f (reverse segment 1)
                        new_cycle.extend_from_slice(&cycle_vec[0..*pos1 + 1]);
                        // Reverse segment 1
                        for i in (pos1 + 1..=*pos2).rev() {
                            new_cycle.push(cycle_vec[i]);
                        }
                        new_cycle.extend_from_slice(&cycle_vec[pos2 + 1..*pos3 + 1]);
                        new_cycle.extend_from_slice(&cycle_vec[pos3 + 1..]);
                    }
                    2 => {
                        // a-e, f-c, d-b (reverse segment 2)
                        new_cycle.extend_from_slice(&cycle_vec[0..*pos1 + 1]);
                        new_cycle.extend_from_slice(&cycle_vec[pos2 + 1..*pos3 + 1]);
                        // Reverse segment 2
                        for i in (*pos1 + 1..=*pos2).rev() {
                            new_cycle.push(cycle_vec[i]);
                        }
                        new_cycle.extend_from_slice(&cycle_vec[pos3 + 1..]);
                    }
                    3 => {
                        // a-d, e-b, f-c (reorder segments)
                        new_cycle.extend_from_slice(&cycle_vec[0..*pos1 + 1]);
                        new_cycle.extend_from_slice(&cycle_vec[pos2 + 1..*pos3 + 1]);
                        new_cycle.extend_from_slice(&cycle_vec[pos1 + 1..*pos2 + 1]);
                        new_cycle.extend_from_slice(&cycle_vec[pos3 + 1..]);
                    }
                    4 => {
                        // a-d, e-c, f-b (reverse segments 1 and 2)
                        new_cycle.extend_from_slice(&cycle_vec[0..*pos1 + 1]);
                        // Reverse segment 2
                        for i in (*pos2 + 1..=*pos3).rev() {
                            new_cycle.push(cycle_vec[i]);
                        }
                        // Reverse segment 1
                        for i in (*pos1 + 1..=*pos2).rev() {
                            new_cycle.push(cycle_vec[i]);
                        }
                        new_cycle.extend_from_slice(&cycle_vec[pos3 + 1..]);
                    }
                    _ => panic!("Invalid 3-opt case"),
                }
                
                *cycle_vec = new_cycle;
            }
            Move::IntraRouteOrOpt { from_pos, chain_length, to_pos, cycle } => {
                let cycle_vec = solution.get_cycle_mut(*cycle);
                
                // Extract the chain
                let mut chain = Vec::new();
                for i in 0..*chain_length {
                    chain.push(cycle_vec[from_pos + i]);
                }
                
                // Remove the chain
                for _ in 0..*chain_length {
                    cycle_vec.remove(*from_pos);
                }
                
                // Adjust insertion position if needed
                let adjusted_to_pos = if *to_pos > *from_pos {
                    to_pos - chain_length
                } else {
                    *to_pos
                };
                
                // Insert the chain at new position
                for (i, &node) in chain.iter().enumerate() {
                    cycle_vec.insert(adjusted_to_pos + i, node);
                }
            }
        }
    }
}
