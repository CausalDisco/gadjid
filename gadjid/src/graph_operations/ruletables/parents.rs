// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all parents of a set of nodes

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

use super::ruletable::RuleTable;

/// Implements a ruletable to get children of a set of nodes
pub struct Parents {}

impl RuleTable for Parents {
    fn lookup(
        &self,
        current_edge: &Edge,
        _current_node: &usize,
        next_edge: &Edge,
        _next_node: &usize,
    ) -> (bool, bool) {
        match (current_edge, next_edge) {
            (Edge::Init, Edge::Outgoing) => (false, true),
            _ => (false, false),
        }
    }
}

/// Gets the union of parents of each node. This is more efficient than calling `parents_of` for each node and then joining the results.
pub fn parents<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> FxHashSet<usize> {
    let ruletable = Parents {};
    // gensearch yield_starting_vertices 'false' because $a \notin Parents(a)$
    crate::graph_operations::gensearch(dag, ruletable, starting_vertices, false)
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::PDAG;

    use super::parents;

    #[test]
    fn get_parents() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = parents(&dag, [0].iter());
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = parents(&dag, [1].iter());
        let expected = HashSet::from([0]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = parents(&dag, [0, 2].iter());
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = parents(&dag, [2].iter());
        let expected = HashSet::from([1]);
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

        let result = parents(&dag, [4].iter());
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = parents(&dag, [2].iter());
        let expected = HashSet::from([1, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = parents(&dag, [1, 3].iter());
        let expected = HashSet::from([0, 2, 4]);
        assert_eq!(expected, result.iter().copied().collect());
    }
}
