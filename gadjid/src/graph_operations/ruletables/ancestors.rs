// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all ancestors of a set of nodes. Unused for now, but kept in the codebase for convenience.

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

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
pub struct AncestorsRuletable {}

impl RuleTable for AncestorsRuletable {
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

/// Gets all ancestors of a set of nodes. Will also return the starting nodes.
#[allow(dead_code)]
pub fn ancestors<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> FxHashSet<usize> {
    let ruletable = crate::graph_operations::ruletables::ancestors::AncestorsRuletable {};
    // gensearch yield_starting_vertices 'true' because $a \in Ancestors(a)$
    super::super::gensearch::gensearch(dag, ruletable, starting_vertices, true)
}

#[cfg(test)]
mod tests {

    use super::ancestors;
    use crate::PDAG;
    use std::collections::HashSet;

    #[test]
    fn ancestors_search() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let expected = HashSet::from([0, 1, 2]);
        let result = ancestors(&dag, [1, 2].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1, 2]);
        let result = ancestors(&dag, [2].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1]);
        let result = ancestors(&dag, [1].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0]);
        let result = ancestors(&dag, [0].iter());
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

        let expected = HashSet::from([0, 1, 2, 4]);
        let result = ancestors(&dag, [2].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1]);
        let result = ancestors(&dag, [0, 1].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([4]);
        let result = ancestors(&dag, [4].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1, 2, 3, 4]);
        let result = ancestors(&dag, [3].iter());
        assert_eq!(expected, result.iter().copied().collect());
    }
}
