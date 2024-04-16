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
    has_been_written_to: bool,
}

// floor(2^64 / GoldenRatio)
const C: u64 = 11400714819323198485;

impl Hasher for FibonacciU64Hasher {
    fn write(&mut self, _: &[u8]) {
        panic!("FibonacciU64Hasher accepts only exactly one call to write_{{u64, usize}}.")
    }

    fn write_u64(&mut self, n: u64) {
        if self.has_been_written_to {
            panic!("FibonacciU64Hasher accepts only exactly one call to write_{{u64, usize}}.")
        }
        self.hash = C.wrapping_mul(n);
        self.has_been_written_to = true;
    }

    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }

    fn finish(&self) -> u64 {
        self.hash
    }
}
