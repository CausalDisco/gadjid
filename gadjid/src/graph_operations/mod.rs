// SPDX-License-Identifier: MPL-2.0
//! Implements functions that take graphs, such as SHD, generalized search, ...

mod ancestor_aid;
mod gensearch;
mod gensearch_wrappers;
mod oset_aid;
mod parent_aid;
mod reachability;
mod shd;
mod sid;

pub(crate) mod ruletables;

pub use ancestor_aid::ancestor_aid;
pub use oset_aid::oset_aid;
pub use parent_aid::parent_aid;
pub use shd::shd;
pub use sid::sid;

pub(crate) use gensearch::gensearch;
pub(crate) use gensearch_wrappers::get_descendants;
pub(crate) use gensearch_wrappers::get_parents;
pub(crate) use gensearch_wrappers::get_proper_ancestors;
pub(crate) use reachability::{
    get_d_pd_nam, get_invalidly_un_blocked, get_nam, get_pd_nam, get_pd_nam_nva,
};

#[cfg(test)]
mod possible_descendants;

#[cfg(test)]
pub(crate) use gensearch_wrappers::get_ancestors;
#[cfg(test)]
pub(crate) use gensearch_wrappers::get_children;
#[cfg(test)]
pub(crate) use oset_aid::optimal_adjustment_set;
#[cfg(test)]
pub(crate) use possible_descendants::get_possible_descendants;
#[cfg(test)]
pub(crate) use reachability::get_nam_nva;
