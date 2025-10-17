// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all ancestors of a set of nodes

use crate::partially_directed_acyclic_graph::Edge;

use super::ruletable::RuleTable;

/// ```text
/// | current_edge | current_node | next_edge | next_node | continue | yield W |
/// |--------------|--------------|-----------|-----------|----------|---------|
/// | spawn        | V            | ->        | W         | false    | false   |
/// | spawn        | V            | <-        | W         | true     | true    |
/// | ->           | V            | ->        | W         | -        | -       |
/// | ->           | V            | <-        | W         | -        | -       |
/// | <-           | V            | <-        | W         | true     | true    |
/// | <-           | V            | ->        | W         | false    | false   |
/// ````
/// Implements a ruletable to get ancestors
pub struct Ancestors {}

impl RuleTable for Ancestors {
    fn lookup(
        &self,
        current_edge: &Edge,
        _current_node: &usize,
        next_edge: &Edge,
        _next_node: &usize,
    ) -> (bool, bool) {
        match (current_edge, next_edge) {
            (_, Edge::Outgoing) => (true, true),
            _ => (false, false),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{graph_operations::get_ancestors, PDAG};

    #[test]
    fn ancestors() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let expected = HashSet::from([0, 1, 2]);
        let result = get_ancestors(&dag, [1, 2].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([0, 1, 2]);
        let result = get_ancestors(&dag, [2].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([0, 1]);
        let result = get_ancestors(&dag, [1].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([0]);
        let result = get_ancestors(&dag, [0].iter());
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

        let expected = HashSet::from([0, 1, 2, 4]);
        let result = get_ancestors(&dag, [2].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([0, 1]);
        let result = get_ancestors(&dag, [0, 1].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([4]);
        let result = get_ancestors(&dag, [4].iter());
        assert_eq!(expected, HashSet::from_iter(result));

        let expected = HashSet::from([0, 1, 2, 3, 4]);
        let result = get_ancestors(&dag, [3].iter());
        assert_eq!(expected, HashSet::from_iter(result));
    }
}
