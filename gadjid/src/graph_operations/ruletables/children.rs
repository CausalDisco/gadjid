// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all children of a set of nodes. Unused for now, but kept in the codebase for convenience.

use crate::partially_directed_acyclic_graph::Edge;

use super::ruletable::RuleTable;

/// Implements a ruletable to get children of a set of nodes
#[allow(dead_code)]
pub struct Children {}

impl RuleTable for Children {
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

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{graph_operations::get_children, PDAG};

    #[test]
    fn children() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let result = get_children(&dag, [0].iter());
        let expected = HashSet::from([1]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_children(&dag, [1].iter());
        let expected = HashSet::from([2]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_children(&dag, [0, 2].iter());
        let expected = HashSet::from([1]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_children(&dag, [2].iter());
        let expected = HashSet::from([]);
        assert_eq!(expected, HashSet::from_iter(result));

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

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let result = get_children(&dag, [4].iter());
        let expected = HashSet::from([2, 3]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_children(&dag, [0, 4].iter());
        let expected = HashSet::from([1, 2, 3]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_children(&dag, [0, 1, 2].iter());
        let expected = HashSet::from([1, 2, 3]);
        assert_eq!(expected, HashSet::from_iter(result));
    }
}
