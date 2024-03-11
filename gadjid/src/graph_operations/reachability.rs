// SPDX-License-Identifier: MPL-2.0
//! Walk-status-aware reachability algorithms for calculating the AID efficiently.

use rustc_hash::FxHashSet;

use crate::{partially_directed_acyclic_graph::Edge, PDAG};

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum WalkStatus {
    /// Possible Descendant / Partially Directed, Amenable (starts T→), and Open Walk
    PD_OPEN_AM,
    /// Possible Descendant / Partially Directed, Amenable (starts T→), and Blocked Walk
    PD_BLOCK_AM,
    /// Possible Descendant / Partially Directed, Not Amenable (starts T—), and Open Walk
    PD_OPEN_NAM,
    /// Possible Descendant / Partially Directed, Not Amenable (starts T–), and Blocked Walk
    PD_BLOCK_NAM,
    /// Non-Causal walk
    NON_CAUSAL,
    /// Initial status
    Init,
}

#[allow(unused)]
/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Follows Algorithm 3 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
///   This includes all NAM, so NAM is a subset NVA.
pub fn get_nam_nva(
    graph: &PDAG,
    t: &[usize],
    z: FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>) {
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    let get_next_steps = |arrived_by: Edge, v: usize, node_is_adjustment: bool| {
        let mut next = Vec::<(Edge, usize, bool)>::new();
        match arrived_by {
            Edge::Incoming => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, !node_is_adjustment));
                    });
            }
            Edge::Init | Edge::Outgoing => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, node_is_adjustment));
                    });
            }
            _ => (),
        }
        graph
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u, node_is_adjustment));
            });
        graph
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
///   This includes all NAM, so NAM is a subset NVA.
pub fn get_pd_nam_nva(
    graph: &PDAG,
    t: &[usize],
    z: FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>, FxHashSet<usize>) {
    let mut poss_de = FxHashSet::from_iter(t.iter().copied());
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    let get_next_steps = |arrived_by: Edge, v: usize, node_is_adjustment: bool| {
        let mut next = Vec::<(Edge, usize, bool)>::new();
        match arrived_by {
            Edge::Incoming => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, !node_is_adjustment));
                    });
            }
            Edge::Init | Edge::Outgoing => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, node_is_adjustment));
                    });
            }
            _ => (),
        }
        graph
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u, node_is_adjustment));
            });
        graph
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

/// Checks amenability of a (CP)DAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Follows Algorithm 5 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
pub fn get_pd_nam(graph: &PDAG, t: &[usize]) -> (FxHashSet<usize>, FxHashSet<usize>) {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Alg5WalkStatus {
        /// Possible Descendant / Partially Directed, Amenable (starts T→)
        POSS_DESC_AM,
        /// Possible Descendant / Partially Directed, Not Amenable (starts T—)
        POSS_DESC_NAM,
        /// Initial status
        Init,
    }

    let mut poss_de = FxHashSet::from_iter(t.iter().copied());
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<(Edge, usize, Alg5WalkStatus)>::default();
    let mut to_visit_stack =
        Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, Alg5WalkStatus::Init)));

    let get_next_steps = |v: usize| {
        let mut next = Vec::<(Edge, usize)>::new();
        graph
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u));
            });
        graph
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
            // any other PD walk
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

/// Checks amenability of a (CP)DAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Follows Algorithm 6 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set D of descendants of T in G
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
pub fn get_d_pd_nam(
    graph: &PDAG,
    t: &[usize],
) -> (FxHashSet<usize>, FxHashSet<usize>, FxHashSet<usize>) {
    #[allow(non_camel_case_types)]
    #[allow(clippy::upper_case_acronyms)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum Alg6WalkStatus {
        /// Descendant / Directed (always amenable)
        DESC,
        /// Possible Descendant / Partially Directed, Amenable (starts T→)
        POSS_DESC_AM,
        /// Possible Descendant / Partially Directed, Not Amenable (starts T—)
        POSS_DESC_NAM,
        /// Initial status
        Init,
    }

    let mut desc = FxHashSet::from_iter(t.iter().copied());
    let mut poss_desc = desc.clone();
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<(Edge, usize, Alg6WalkStatus)>::default();
    let mut to_visit_stack =
        Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, Alg6WalkStatus::Init)));

    let get_next_steps = |v: usize| {
        let mut next = Vec::<(Edge, usize)>::new();
        graph
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u));
            });
        graph
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
///   Here, amenability (condition 1.) is not verified, that is, NVA is not a superset of NAM;
///   instead, NVA contains Y for which condition 2. or 3.
///   of the modified adjustment criterion for walk-based verification
///   in https://doi.org/10.48550/arXiv.2402.08616 are violated
pub fn get_invalid_unblocked(
    graph: &PDAG,
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
    let mut to_visit_stack =
        Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, Alg7WalkStatus::Init)));

    let get_next_steps = |arrived_by: Edge, v: usize, node_is_adjustment: bool| {
        let mut next = Vec::<(Edge, usize, bool)>::new();
        match arrived_by {
            Edge::Incoming => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, !node_is_adjustment));
                    });
            }
            Edge::Init | Edge::Outgoing => {
                graph
                    .parents_of(v)
                    .iter()
                    .filter(|p| !t.contains(*p))
                    .for_each(|p| {
                        next.push((Edge::Outgoing, *p, node_is_adjustment));
                    });
            }
            _ => (),
        }
        graph
            .adjacent_undirected_of(v)
            .iter()
            .filter(|u| !t.contains(*u))
            .for_each(|u| {
                next.push((Edge::Undirected, *u, node_is_adjustment));
            });
        graph
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

/// Checks amenability of a CPDAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Returns set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
///
/// Follows Algorithm 2 in https://doi.org/10.48550/arXiv.2402.08616
pub fn get_nam(cpdag: &PDAG, t: &[usize]) -> FxHashSet<usize> {
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<usize>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v)));

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

    use crate::graph_operations::{
        ancestor_aid, descendants, gensearch, get_nam_nva, oset_aid, parent_aid,
        possible_descendants, ruletables,
    };
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
        let dag = PDAG::from_vecvec(dag);
        let cpdag = PDAG::from_vecvec(cpdag);

        assert_eq!((1.0, 2), parent_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), parent_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), ancestor_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), ancestor_aid(&cpdag, &dag));
        assert_eq!((1.0, 2), oset_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), oset_aid(&cpdag, &dag));
    }

    #[test]
    pub fn reachability_algos_agree() {
        let reps = 30;
        (0..reps).for_each(|_| {
            let pdag = PDAG::random_pdag(0.5, 100);
            let t = [0];
            let adjust = gensearch(&pdag, ruletables::Parents {}, t.iter(), false);

            let d_expected = descendants(&pdag, t.iter());
            let pd_expected = possible_descendants(&pdag, t.iter());
            let (nam_expected, nva_expected) = get_nam_nva(&pdag, &t, adjust.clone());

            let (pd, nam, nva) = super::get_pd_nam_nva(&pdag, &t, adjust.clone());
            assert_eq!(pd_expected, pd);
            assert_eq!(nam_expected, nam);
            assert_eq!(nva_expected, nva);

            let (pd, nam) = super::get_pd_nam(&pdag, &t);
            assert_eq!(nam_expected, nam);
            assert_eq!(pd_expected, pd);

            let nam = super::get_nam(&pdag, &t);
            assert_eq!(nam_expected, nam);

            let (d, pd, nam) = super::get_d_pd_nam(&pdag, &t);
            assert_eq!(d_expected, d);
            assert_eq!(pd_expected, pd);
            assert_eq!(nam_expected, nam);

            let ivb = super::get_invalid_unblocked(&pdag, &t, adjust.clone());
            assert!(ivb.is_subset(&nva_expected));
        });
    }
}
