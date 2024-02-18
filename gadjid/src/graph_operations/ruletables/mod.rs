// SPDX-License-Identifier: MPL-2.0
//! Holds ruletables that implement specific behaviour for the generalized graph search algorithm
// the actual type definition for the ruletable trait
pub mod ruletable;

// implementations of the ruletable trait
pub mod ancestors;
pub mod children;
pub mod descendants;
pub mod parents;
pub mod proper_ancestors;
