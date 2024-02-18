// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all children of a set of nodes

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

use super::ruletable::RuleTable;

/// Implements a ruletable to get children of a set of nodes
pub struct ChildrenRuletable {}

impl RuleTable for ChildrenRuletable {
    fn lookup(
        &self,
        current_edge: &Edge,
        _current_node: &usize,
        next_edge: &Edge,
        _next_node: &usize,
    ) -> (bool, bool) {
        match (current_edge, next_edge) {
            (Edge::Init, Edge::Incoming) => (false, true),
            _ => (false, false),
        }
    }
}

/// Gets the union of children of each node. This is more efficient than calling `children_of` for each node and then joining the results.
pub fn children<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> FxHashSet<usize> {
    let ruletable = crate::graph_operations::ruletables::children::ChildrenRuletable {};
    super::super::gensearch::gensearch(dag, ruletable, starting_vertices, false)
}

#[cfg(test)]
mod tests {

    use super::children;
    use crate::PDAG;
    use std::collections::HashSet;

    #[test]
    fn get_children() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = children(&dag, [0].iter());
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = children(&dag, [1].iter());
        let expected = HashSet::from([2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = children(&dag, [0, 2].iter());
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = children(&dag, [2].iter());
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 2 ----> 3
        //           ^       ^
        //            \_   _/
        //               4
        let v_dag = vec![
            vec![0, 1, 0, 0, 0], //
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 1, 1, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = children(&dag, [4].iter());
        let expected = HashSet::from([2, 3]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = children(&dag, [0, 4].iter());
        let expected = HashSet::from([1, 2, 3]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = children(&dag, [0, 1, 2].iter());
        let expected = HashSet::from([1, 2, 3]);
        assert_eq!(expected, result.iter().copied().collect());
    }
}
