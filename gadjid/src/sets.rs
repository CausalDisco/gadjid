// SPDX-License-Identifier: MPL-2.0
//! Sets of nodes (Node≡usize)

use core::hash::{BuildHasherDefault, Hasher};
// use rustc_hash::FxHasher;
use std::collections::HashSet;

type Node = usize;

pub type NodeSet = HashSet<Node, BuildHasherDefault<FibonacciUsizeHasher>>;
// pub type NodeSet = HashSet<Node, BuildHasherDefault<FxHasher>>;

#[derive(Default)]
pub struct FibonacciUsizeHasher {
    hash: usize,
}

// const C: usize = 11400714819323198485;

impl Hasher for FibonacciUsizeHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("FibonacciUsizeHasher accepts only exactly one call to write_usize.")
    }

    fn write_usize(&mut self, n: usize) {
        self.hash = n;
        // self.hash = C.wrapping_mul(n);
    }

    fn finish(&self) -> u64 {
        self.hash as u64
    }
}
