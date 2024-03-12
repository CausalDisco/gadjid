// SPDX-License-Identifier: MPL-2.0
//! Implements functions that take graphs, such as SHD, generalized search, ...

mod ancestor_aid;
mod oset_aid;
mod parent_aid;
mod possible_descendants;
mod reachability;
mod shd;
mod sid;

pub(crate) mod gensearch;
pub(crate) mod ruletables;

pub use ancestor_aid::ancestor_aid;
pub use oset_aid::oset_aid;
pub use parent_aid::parent_aid;
pub use shd::shd;
pub use sid::sid;

pub(crate) use gensearch::gensearch;
pub(crate) use reachability::get_nam;
pub(crate) use reachability::{get_d_pd_nam, get_invalid_un_blocked, get_pd_nam, get_pd_nam_nva};
pub(crate) use ruletables::parents::parents;
pub(crate) use ruletables::proper_ancestors::proper_ancestors;

#[cfg(test)]
pub(crate) use oset_aid::optimal_adjustment_set;
#[cfg(test)]
pub(crate) use possible_descendants::possible_descendants;
#[cfg(test)]
pub(crate) use reachability::get_nam_nva;
#[cfg(test)]
pub(crate) use ruletables::descendants::descendants;
