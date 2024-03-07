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

pub(crate) use ancestors::Ancestors;
pub(crate) use children::Children;
pub(crate) use descendants::Descendants;
pub(crate) use parents::Parents;
pub(crate) use proper_ancestors::ProperAncestors;
pub(crate) use ruletable::RuleTable;
