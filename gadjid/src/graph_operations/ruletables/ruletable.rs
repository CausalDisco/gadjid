// SPDX-License-Identifier: MPL-2.0
//! Defines the ruletable trait for use with generalized search algorithm

use crate::partially_directed_acyclic_graph::Edge;

/// A trait that implements ruletable lookup behaviour for the generalized graph search algorithm
pub trait RuleTable {
    /// Given context of
    ///
    /// - `current_edge`, how we arrived at the current node
    /// - `current_node`,
    /// - `next_edge`, how we would get to the next node
    /// - `next_node`,
    ///
    /// this function call should return a bool-tuple specifying whether to respectively `(continue, yield)`, meaning whether
    /// to continue the search and whether to yield the node to the result set.
    ///
    /// To eliminate all ambguity, `-> Y <- X` and `<- Y <- X` would be specified as
    /// `Incoming, Y, Outgoing, X` and `Outgoing, Y, Outgoing, X` respectively. This is because
    /// the edge direction is associated with the node it is revealed in conjunction with (the node
    /// on the right of the edge, think `(-> Y) (<- X)`).
    fn lookup(
        &self,
        current_edge: &Edge,
        current_node: &usize,
        next_edge: &Edge,
        next_node: &usize,
    ) -> (bool, bool);
}
