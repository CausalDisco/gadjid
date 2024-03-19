// SPDX-License-Identifier: MPL-2.0
//! Holds utility functions for the AID algorithm.

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum WalkStatus {
    /// Partially Directed, Amenable, Open
    OpenAmenable,
    /// Partially Directed, Amenable, Blocked
    BlockedAmenable,
    /// Partially Directed, Not Amenable, Open
    OpenNonAmenable,
    /// Partially Directed, Not Amenable, Blocked
    BlockedNonAmenable,
    /// Non-Causal walk
    NonCausal,
    /// Initial status
    Init,
}

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Follows Algorithm 3 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
/// This includes all NAM, so NAM is a subset NVA.
pub fn get_nam_nva(
    truth_dag: &PDAG,
    t: &[usize],
    z: FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>) {
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::<(Edge, usize, WalkStatus)>::new();
    t.iter()
        .for_each(|v| to_visit_stack.push((Edge::Init, *v, WalkStatus::Init)));

    let get_next_steps = |arrived_by: Edge, v: usize, node_is_adjustment: bool| {
        let mut next = Vec::<(Edge, usize, bool)>::new();
        match arrived_by {
            Edge::Incoming => {
                truth_dag
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, !node_is_adjustment));
                    });
            }
            Edge::Init | Edge::Outgoing => {
                truth_dag
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, node_is_adjustment));
                    });
            }
            _ => (),
        }
        truth_dag
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u, node_is_adjustment));
            });
        truth_dag
            .children_of(v)
            .iter()
            .filter(|c| !t.contains(*c))
            .for_each(|c| {
                next.push((Edge::Incoming, *c, node_is_adjustment));
            });
        next
    };

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            WalkStatus::OpenNonAmenable | WalkStatus::BlockedNonAmenable => {
                not_amenable.insert(node);
                // we want the property that not_amenable is a subset of not_vas
                // so, if we insert a node into not_amenable, we also insert it into not_vas
                not_vas.insert(node);
            }
            WalkStatus::NonCausal | WalkStatus::BlockedAmenable => {
                not_vas.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in get_next_steps(arrived_by, node, node_is_adjustment) {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::OpenAmenable)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NonCausal)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::OpenNonAmenable)),
                    _ => None,
                },
                WalkStatus::OpenAmenable | WalkStatus::BlockedAmenable => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::BlockedAmenable)),
                    },
                    Edge::Outgoing
                        if !blocked && matches!(walkstatus, WalkStatus::OpenAmenable) =>
                    {
                        Some((move_on_by, w, WalkStatus::NonCausal))
                    }
                    _ => None,
                },
                WalkStatus::OpenNonAmenable | WalkStatus::BlockedNonAmenable => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::BlockedNonAmenable)),
                    },
                    Edge::Outgoing
                        if !blocked && matches!(walkstatus, WalkStatus::OpenNonAmenable) =>
                    {
                        Some((move_on_by, w, WalkStatus::NonCausal))
                    }
                    _ => None,
                },
                WalkStatus::NonCausal if !blocked => Some((move_on_by, w, WalkStatus::NonCausal)),
                _ => None,
            };

            if let Some(next) = next {
                if !visited.contains(&next) {
                    to_visit_stack.push(next);
                }
            }
        }
    }

    assert!(not_amenable.is_subset(&not_vas));

    (not_amenable, not_vas)
}

/// Check amenability of a CPDAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Returns set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
///
/// Follows Algorithm 2 in https://doi.org/10.48550/arXiv.2402.08616
pub fn get_nam(cpdag: &PDAG, t: &[usize]) -> FxHashSet<usize> {
    let mut to_visit_stack: Vec<(Edge, usize)> = Vec::new();
    t.iter().for_each(|v| to_visit_stack.push((Edge::Init, *v)));

    let mut visited = FxHashSet::<usize>::default();
    let mut not_amenable = FxHashSet::<usize>::default();

    while let Some((arrived_by, node)) = to_visit_stack.pop() {
        visited.insert(node);
        match arrived_by {
            Edge::Init => {
                cpdag
                    .adjacent_undirected_of(node)
                    .iter()
                    .filter(|p| !visited.contains(p) && !t.contains(p))
                    .for_each(|p| {
                        to_visit_stack.push((Edge::Undirected, *p));
                    });
            }
            // Edge::Incoming | Edge::Outgoing | Edge::Undirected
            _ => {
                not_amenable.insert(node);
                cpdag
                    .adjacent_undirected_of(node)
                    .iter()
                    .filter(|p| !visited.contains(p) && !t.contains(p))
                    .for_each(|p| {
                        to_visit_stack.push((Edge::Undirected, *p));
                    });
                cpdag
                    .children_of(node)
                    .iter()
                    .filter(|p| !visited.contains(p) && !t.contains(p))
                    .for_each(|p| {
                        to_visit_stack.push((Edge::Incoming, *p));
                    });
            }
        }
    }
    not_amenable
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashSet;

    use crate::graph_operations::{ancestor_aid, oset_aid, parent_aid};
    use crate::PDAG;

    use super::get_nam;

    #[test]
    pub fn nam_test() {
        // 0 -> 1 -- 2
        // |
        // 3

        let cpdag = vec![
            vec![0, 1, 0, 2], //
            vec![0, 0, 2, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];
        let cpdag = PDAG::from_row_to_col_vecvec(cpdag);

        assert!(get_nam(&cpdag, &[0]) == FxHashSet::from_iter([3]));
    }

    #[test]
    pub fn nam_correctly_counted_as_mistake() {
        // this test checks mistakes between the cpdag X - Y and dag X -> Y for all distances.

        let dag = vec![
            vec![0, 1], //
            vec![0, 0],
        ];
        let cpdag = vec![
            vec![0, 2], //
            vec![0, 0],
        ];
        let dag = PDAG::from_row_to_col_vecvec(dag);
        let cpdag = PDAG::from_row_to_col_vecvec(cpdag);

        assert_eq!((1.0, 2), parent_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), parent_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), ancestor_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), ancestor_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), oset_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), oset_aid(&cpdag, &dag));
    }
}
