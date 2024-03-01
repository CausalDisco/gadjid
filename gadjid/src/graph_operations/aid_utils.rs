// SPDX-License-Identifier: MPL-2.0
//! Holds utility functions for the AID algorithm.

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum WalkStatus {
    /// Possible Descendant / Partially Directed, Amenable, and Open Walk
    PD_OPEN_AM,
    /// Possible Descendant / Partially Directed, Amenable, and Blocked Walk
    PD_BLOCK_AM,
    /// Possible Descendant / Partially Directed, Not Amenable, and Open Walk
    PD_OPEN_NAM,
    /// Possible Descendant / Partially Directed, Not Amenable, and Blocked Walk
    PD_BLOCK_NAM,
    /// Non-Causal walk
    NON_CAUSAL,
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
            WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => {
                not_amenable.insert(node);
                // we want the property that not_amenable is a subset of not_vas
                // so, if we insert a node into not_amenable, we also insert it into not_vas
                not_vas.insert(node);
            }
            WalkStatus::NON_CAUSAL | WalkStatus::PD_BLOCK_AM => {
                not_vas.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in get_next_steps(arrived_by, node, node_is_adjustment) {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::PD_OPEN_AM)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NON_CAUSAL)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_OPEN_NAM)),
                    _ => None,
                },
                WalkStatus::PD_OPEN_AM | WalkStatus::PD_BLOCK_AM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_AM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_AM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL))
                    }
                    _ => None,
                },
                WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_NAM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_NAM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL))
                    }
                    _ => None,
                },
                WalkStatus::NON_CAUSAL if !blocked => Some((move_on_by, w, WalkStatus::NON_CAUSAL)),
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

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Follows Algorithm 4 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
/// This includes all NAM, so NAM is a subset NVA.
pub fn get_pd_nam_nva(
    truth_dag: &PDAG,
    t: &[usize],
    z: FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>, FxHashSet<usize>) {
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();
    let mut poss_de = FxHashSet::from_iter(t.iter().copied());

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
            WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => {
                not_amenable.insert(node);
                // we want the property that not_amenable is a subset of not_vas
                // so, if we insert a node into not_amenable, we also insert it into not_vas
                not_vas.insert(node);
                poss_de.insert(node);
            }
            WalkStatus::NON_CAUSAL => {
                not_vas.insert(node);
            }
            WalkStatus::PD_BLOCK_AM => {
                not_vas.insert(node);
                poss_de.insert(node);
            }
            WalkStatus::PD_OPEN_AM => {
                poss_de.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in get_next_steps(arrived_by, node, node_is_adjustment) {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::PD_OPEN_AM)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NON_CAUSAL)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_OPEN_NAM)),
                    _ => None,
                },
                WalkStatus::PD_OPEN_AM | WalkStatus::PD_BLOCK_AM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_AM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_AM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL))
                    }
                    _ => None,
                },
                WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_NAM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_NAM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL))
                    }
                    _ => None,
                },
                WalkStatus::NON_CAUSAL if !blocked => Some((move_on_by, w, WalkStatus::NON_CAUSAL)),
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

    (poss_de, not_amenable, not_vas)
}

/// Checks amenability of some DAG/CPDAG for a given set T of treatments and all response variables
///
/// Follows Algorithm 5 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
pub fn get_pd_nam(truth_dag: &PDAG, t: &[usize]) -> (FxHashSet<usize>, FxHashSet<usize>) {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Alg5WalkStatus {
        /// Possible Descendant, Amenable
        POSS_DESC_AM,
        /// Possible Descendant, Not Amenable
        POSS_DESC_NAM,
        /// Initial status
        Init,
    }

    let mut not_amenable = FxHashSet::<usize>::default();
    let mut poss_de = FxHashSet::from_iter(t.iter().copied());

    let mut visited = FxHashSet::<(Edge, usize, Alg5WalkStatus)>::default();
    let mut to_visit_stack = Vec::<(Edge, usize, Alg5WalkStatus)>::new();
    t.iter()
        .for_each(|v| to_visit_stack.push((Edge::Init, *v, Alg5WalkStatus::Init)));

    let get_next_steps = |v: usize| {
        let mut next = Vec::<(Edge, usize)>::new();
        truth_dag
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u));
            });
        truth_dag
            .children_of(v)
            .iter()
            .filter(|c| !t.contains(*c))
            .for_each(|c| {
                next.push((Edge::Incoming, *c));
            });
        next
    };

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            Alg5WalkStatus::POSS_DESC_NAM => {
                not_amenable.insert(node);
                poss_de.insert(node);
            }
            // any AM walk
            Alg5WalkStatus::POSS_DESC_AM => {
                poss_de.insert(node);
            }
            _ => (),
        }

        for (move_on_by, w) in get_next_steps(node) {
            let next = match walkstatus {
                Alg5WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, Alg5WalkStatus::POSS_DESC_AM)),
                    Edge::Undirected => Some((move_on_by, w, Alg5WalkStatus::POSS_DESC_NAM)),
                    _ => None,
                },
                Alg5WalkStatus::POSS_DESC_AM | Alg5WalkStatus::POSS_DESC_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => Some((move_on_by, w, walkstatus)),
                    _ => None,
                },
                _ => None,
            };

            if let Some(next) = next {
                if !visited.contains(&next) {
                    to_visit_stack.push(next);
                }
            }
        }
    }

    (poss_de, not_amenable)
}

/// Checks amenability of some DAG/CPDAG for a given set T of treatments and all response variables
///
/// Follows Algorithm 6 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set D of descendants of T in G
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
pub fn get_d_pd_nam(
    truth_dag: &PDAG,
    t: &[usize],
) -> (FxHashSet<usize>, FxHashSet<usize>, FxHashSet<usize>) {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Alg6WalkStatus {
        /// Descendant, always Amenable
        DESC,
        /// Possible Descendant, Amenable
        POSS_DESC_AM,
        /// Possible Descendant, Not Amenable
        POSS_DESC_NAM,
        /// Initial status
        Init,
    }

    let mut not_amenable = FxHashSet::<usize>::default();
    let mut poss_desc = FxHashSet::from_iter(t.iter().copied());
    let mut desc = FxHashSet::from_iter(t.iter().copied());

    let mut visited = FxHashSet::<(Edge, usize, Alg6WalkStatus)>::default();
    let mut to_visit_stack = Vec::<(Edge, usize, Alg6WalkStatus)>::new();
    t.iter()
        .for_each(|v| to_visit_stack.push((Edge::Init, *v, Alg6WalkStatus::Init)));

    let get_next_steps = |v: usize| {
        let mut next = Vec::<(Edge, usize)>::new();
        truth_dag
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u));
            });
        truth_dag
            .children_of(v)
            .iter()
            .filter(|c| !t.contains(*c))
            .for_each(|c| {
                next.push((Edge::Incoming, *c));
            });
        next
    };

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            Alg6WalkStatus::POSS_DESC_NAM => {
                not_amenable.insert(node);
                poss_desc.insert(node);
            }
            Alg6WalkStatus::POSS_DESC_AM => {
                poss_desc.insert(node);
            }
            Alg6WalkStatus::DESC => {
                poss_desc.insert(node);
                desc.insert(node);
            }
            _ => (),
        }

        for (move_on_by, w) in get_next_steps(node) {
            let next = match walkstatus {
                Alg6WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, Alg6WalkStatus::DESC)),
                    Edge::Undirected => Some((move_on_by, w, Alg6WalkStatus::POSS_DESC_NAM)),
                    _ => None,
                },
                Alg6WalkStatus::POSS_DESC_AM | Alg6WalkStatus::POSS_DESC_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => Some((move_on_by, w, walkstatus)),
                    _ => None,
                },
                Alg6WalkStatus::DESC => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, Alg6WalkStatus::DESC)),
                    Edge::Undirected => Some((move_on_by, w, Alg6WalkStatus::POSS_DESC_AM)),
                    _ => None,
                },
                _ => None,
            };

            if let Some(next) = next {
                if !visited.contains(&next) {
                    to_visit_stack.push(next);
                }
            }
        }
    }

    (desc, poss_desc, not_amenable)
}

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Follows Algorithm 7 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
pub fn get_invalid_unblocked(
    truth_dag: &PDAG,
    t: &[usize],
    z: FxHashSet<usize>,
) -> FxHashSet<usize> {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Alg7WalkStatus {
        /// Possible Descendant / Partially Directed, and Open Walk
        PD_OPEN,
        /// Possible Descendant / Partially Directed, and Blocked Walk
        PD_BLOCK,
        /// Non-Causal walk that has not been blocked
        NON_CAUSAL_OPEN,
        /// Initial status
        Init,
    }

    let mut ivb = z.clone();
    let mut visited = FxHashSet::<(Edge, usize, Alg7WalkStatus)>::default();
    let mut to_visit_stack = Vec::<(Edge, usize, Alg7WalkStatus)>::new();
    t.iter()
        .for_each(|v| to_visit_stack.push((Edge::Init, *v, Alg7WalkStatus::Init)));

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
            // when the node is reached on a causal path but blocked, or an unblocked non-causal path
            Alg7WalkStatus::PD_BLOCK | Alg7WalkStatus::NON_CAUSAL_OPEN => {
                ivb.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in get_next_steps(arrived_by, node, node_is_adjustment) {
            let next = match walkstatus {
                Alg7WalkStatus::Init => match move_on_by {
                    Edge::Incoming | Edge::Undirected => {
                        Some((move_on_by, w, Alg7WalkStatus::PD_OPEN))
                    }
                    Edge::Outgoing => Some((move_on_by, w, Alg7WalkStatus::NON_CAUSAL_OPEN)),
                    _ => None,
                },
                Alg7WalkStatus::PD_OPEN | Alg7WalkStatus::PD_BLOCK => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, Alg7WalkStatus::PD_BLOCK)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, Alg7WalkStatus::PD_OPEN) => {
                        Some((move_on_by, w, Alg7WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                Alg7WalkStatus::NON_CAUSAL_OPEN if !blocked => {
                    Some((move_on_by, w, Alg7WalkStatus::NON_CAUSAL_OPEN))
                }
                _ => None,
            };

            if let Some(next) = next {
                if !visited.contains(&next) {
                    to_visit_stack.push(next);
                }
            }
        }
    }

    ivb
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
mod tests {
    use rustc_hash::FxHashSet;

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
        let cpdag = PDAG::from_vecvec(cpdag);

        assert!(get_nam(&cpdag, &[0]) == FxHashSet::from_iter([3]));
    }

    use crate::graph_operations::{ancestor_aid, oset_aid, parent_aid};

    #[test]
    pub fn nam_correctly_counted_as_mistake() {
        // this tests checks mistakes between the cpdag X - Y and dag X -> Y for all distances.

        let dag = vec![
            vec![0, 1], //
            vec![0, 0],
        ];
        let cpdag = vec![
            vec![0, 2], //
            vec![0, 0],
        ];
        let dag = PDAG::from_vecvec(dag);
        let cpdag = PDAG::from_vecvec(cpdag);

        assert_eq!((1.0, 2), parent_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), parent_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), ancestor_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), ancestor_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), oset_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), oset_aid(&cpdag, &dag));
    }
}
