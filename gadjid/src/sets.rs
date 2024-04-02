// SPDX-License-Identifier: MPL-2.0
//! Sets of nodes (Node≡usize)

use core::hash::{BuildHasherDefault, Hasher};
// use rustc_hash::FxHasher;
use std::collections::HashSet;

type Node = usize;

pub type NodeSet = HashSet<Node, BuildHasherDefault<FibonacciU64Hasher>>;
// pub type NodeSet = HashSet<Node, BuildHasherDefault<FxHasher>>;

#[derive(Default)]
pub struct FibonacciU64Hasher {
    hash: u64,
}

// floor(2^64 / GoldenRatio)
const C: u64 = 11400714819323198485;

impl Hasher for FibonacciU64Hasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("FibonacciU64Hasher accepts only exactly one call to write_{{u64, usize}}.")
    }

    fn write_u64(&mut self, n: u64) {
        self.hash = C.wrapping_mul(n);
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}
