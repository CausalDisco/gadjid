// SPDX-License-Identifier: MPL-2.0
#![warn(missing_docs)]
//! gadjid -  Graph Adjustment Incompatibility Distance library
mod ascending_list_utils;
mod graph_loading;
mod partially_directed_acyclic_graph;
pub mod graph_operations;

pub use graph_loading::constructor::EdgelistIterator as EdgelistIterator;
pub use partially_directed_acyclic_graph::PDAG;
pub use partially_directed_acyclic_graph::LoadError as LoadError;
