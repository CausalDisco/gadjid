// SPDX-License-Identifier: MPL-2.0
//! Implements functions that take graphs, such as SHD, generalized search, ...
mod aid_utils;
mod ancestor_aid;
pub(crate) mod gensearch;
mod oset_aid;
mod parent_aid;
mod possible_descendants;
pub(crate) mod ruletables;
mod shd;
mod sid;

pub use ancestor_aid::ancestor_aid;
pub use oset_aid::oset_aid;
pub use parent_aid::parent_aid;
pub use shd::shd;
pub use sid::sid;

pub(crate) use aid_utils::{get_nam, get_nam_nva};
pub(crate) use possible_descendants::possible_descendants;
#[cfg(test)]
pub(crate) use ruletables::ancestors::ancestors;
#[cfg(test)]
pub(crate) use ruletables::children::children;
pub(crate) use ruletables::descendants::descendants;
pub(crate) use ruletables::parents::parents;
pub(crate) use ruletables::proper_ancestors::proper_ancestors;
