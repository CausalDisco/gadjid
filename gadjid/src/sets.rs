// SPDX-License-Identifier: MPL-2.0
//! Sets of nodes (Node≡usize)

use core::hash::BuildHasherDefault;
use std::{collections::HashSet, hash::Hasher};

type Node = usize;

pub type FibSet<T> = HashSet<T, BuildHasherDefault<FibonacciU64Hasher>>;
pub type NodeSet = FibSet<Node>;

#[derive(Default)]
pub struct FibonacciU64Hasher {
    hash: u64,
    // default is false
    has_been_applied: bool,
}

// floor(2^64 / GoldenRatio)
const C: u64 = 11400714819323198485;

impl Hasher for FibonacciU64Hasher {
    fn write(&mut self, _: &[u8]) {
        panic!("FibonacciU64Hasher accepts only exactly one call to write_{{u64, usize}}.")
    }

    fn write_u64(&mut self, n: u64) {
        if self.has_been_applied {
            panic!("FibonacciU64Hasher accepts only exactly one call to write_{{u64, usize}}.")
        }
        self.hash = C.wrapping_mul(n);
        self.has_been_applied = true;
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn speedtest_hash() {
        let reps = 100;
        let mut total_time_nanos = 0;

        for _ in 0..reps {
            let start = std::time::Instant::now();
            let mut set = NodeSet::default();
            for i in 0..1000 {
                set.insert(i);
            }
            let elapsed = start.elapsed().as_nanos();
            total_time_nanos += elapsed;
        }
        println!("Elapsed: {} ns", total_time_nanos);
    }

    #[test]
    fn speedtest_algos() {
        let reps = 100;
        let mut total_time_nanos = 0;
        let mut rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(0);

        let truth = crate::PDAG::random_pdag(0.3, 100, &mut rng);

        for _ in 0..reps {
            let guess = crate::PDAG::random_pdag(0.3, 100, &mut rng);

            let start = std::time::Instant::now();
            let _ = crate::graph_operations::oset_aid(&truth, &guess);
            let elapsed = start.elapsed().as_nanos();
            total_time_nanos += elapsed;
        }
        println!("Elapsed: {} ns", total_time_nanos);
    }
}
