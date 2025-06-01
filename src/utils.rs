use crate::tsplib::{Solution, TsplibInstance};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn generate_random_solution(instance: &TsplibInstance) -> Solution {
    let mut vertices: Vec<usize> = (0..instance.size()).collect();
    vertices.shuffle(&mut thread_rng());

    let half = vertices.len() / 2;
    let cycle1 = vertices[0..half].to_vec();
    let cycle2 = vertices[half..].to_vec();

    Solution::new(cycle1, cycle2)
}
