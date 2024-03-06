// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all descendants of a set of nodes

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

use super::ruletable::RuleTable;

/// ```text
/// | current_edge | current_node | next_edge | next_node | continue | yield W |
/// |--------------|--------------|-----------|-----------|----------|---------|
/// | spawn        | V            | ->        | W         | true     | true    |
/// | spawn        | V            | <-        | W         | false    | false   |
/// | ->           | V            | ->        | W         | true     | true    |
/// | ->           | V            | <-        | W         | false    | false   |
/// | <-           | V            | <-        | W         | -        | -       |
/// | <-           | V            | ->        | W         | -        | -       |
/// ````
/// Implements a ruletable to get descendants
pub struct Descendants {}

impl RuleTable for Descendants {
    fn lookup(
        &self,
        current_edge: &Edge,
        _current_node: &usize,
        next_edge: &Edge,
        _next_node: &usize,
    ) -> (bool, bool) {
        match (current_edge, next_edge) {
            (_, Edge::Incoming) => (true, true),
            _ => (false, false),
        }
    }
}

/// Gets all descendants of a set of nodes. Will also return the starting nodes.
pub fn descendants<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> FxHashSet<usize> {
    let start: Vec<usize> = starting_vertices.copied().collect();
    let ruletable = crate::graph_operations::ruletables::Descendants {};
    // gensearch yield_starting_vertices 'true' because $a \in Descendants(a)$
    crate::graph_operations::gensearch(dag, ruletable, start.iter(), true)
}

#[cfg(test)]
mod tests {

    use super::descendants;
    use crate::PDAG;
    use std::collections::HashSet;

    #[test]
    fn descendants_search() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let expected = HashSet::from([1, 2]);
        let result = descendants(&dag, [2, 1].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([2]);
        let result = descendants(&dag, [2].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([1, 2]);
        let result = descendants(&dag, [1].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1, 2]);
        let result = descendants(&dag, [0].iter());
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

        let expected = HashSet::from([2, 3]);
        let result = descendants(&dag, [2].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([0, 1, 2, 3]);
        let result = descendants(&dag, [0, 1].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([2, 3, 4]);
        let result = descendants(&dag, [4].iter());
        assert_eq!(expected, result.iter().copied().collect());

        let expected = HashSet::from([3]);
        let result = descendants(&dag, [3].iter());
        assert_eq!(expected, result.iter().copied().collect());
    }
}
