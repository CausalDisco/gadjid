// SPDX-License-Identifier: MPL-2.0
//! Ruletable for getting all *proper* ancestors of a set of nodes

use crate::{partially_directed_acyclic_graph::Edge, sets::NodeSet};

use super::ruletable::RuleTable;

/// Implements a ruletable for getting proper ancestors of response variables, given the treatment set.
/// Does not include the treatments themselves.
///
/// ```text
///+--------------+--------------+-----------+-----------+------------+------------+
///| current_edge | current_node | next_edge | next_node |  continue  |  yield W   |
///+--------------+--------------+-----------+-----------+------------+------------+
///| any edge     | X            | ->        | Y         | -          | -          |
///| any edge     | X            | <-        | Y         | Y \notin T | Y \notin T |
///+--------------+--------------+-----------+-----------+------------+------------+
/// ```
/// where T is the treatment set
pub struct ProperAncestors {
    /// The treatment variables T that are the first not to be included as ancestors
    pub treatments: NodeSet,
}

impl RuleTable for ProperAncestors {
    fn lookup(
        &self,
        current_edge: &Edge,
        _current_node: &usize,
        next_edge: &Edge,
        next_node: &usize,
    ) -> (bool, bool) {
        match (current_edge, next_edge) {
            // Line 2, any V <- W
            (_, Edge::Outgoing) => {
                let is_not_t = !self.treatments.contains(next_node);
                (is_not_t, is_not_t)
            }

            // Line 1, any V -> W (and all else is unreachable)
            _ => (false, false),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{graph_operations::get_proper_ancestors, PDAG};

    #[test]
    fn proper_ancestors() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let result = get_proper_ancestors(&dag, [].iter(), [2].iter());
        let expected = HashSet::from([0, 1, 2]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_proper_ancestors(&dag, [1].iter(), [2].iter());
        let expected = HashSet::from([2]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_proper_ancestors(&dag, [0].iter(), [2].iter());
        let expected = HashSet::from([1, 2]);
        assert_eq!(expected, HashSet::from_iter(result));

        // 0 -> 1 -> 3 and 0 -> 2 -> 3
        let v_dag = vec![
            vec![0, 1, 1, 0], //
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let result = get_proper_ancestors(&dag, [].iter(), [3].iter());
        let expected = HashSet::from([0, 1, 2, 3]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_proper_ancestors(&dag, [1].iter(), [3].iter());
        let expected = HashSet::from([0, 2, 3]);
        assert_eq!(expected, HashSet::from_iter(result));

        // 0 -> 1 -> 2 -> 4 and 0 -> 3 -> 4
        let v_dag = vec![
            vec![0, 1, 0, 1, 0], //
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 1],
            vec![0, 0, 0, 0, 1],
            vec![0, 0, 0, 0, 0],
        ];
        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        let result = get_proper_ancestors(&dag, [].iter(), [4].iter());
        let expected = HashSet::from([0, 1, 2, 3, 4]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_proper_ancestors(&dag, [2].iter(), [4].iter());
        let expected = HashSet::from([0, 3, 4]);
        assert_eq!(expected, HashSet::from_iter(result));

        let result = get_proper_ancestors(&dag, [1].iter(), [4].iter());
        let expected = HashSet::from([0, 2, 3, 4]);
        assert_eq!(expected, HashSet::from_iter(result));
    }
}
