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
    /// Non-Causal walk that has not been blocked
    NON_CAUSAL_OPEN,
    /// Initial status
    Init,
}

/// Returns possible children of the node `v` and the shared edge. `v (-> c)` or `v (-- c)`. See the [`Edge`] enum for a more detailed explanation of this notation.
/// Will not return treatment nodes.
fn get_next_steps(graph: &PDAG, t: &[usize], v: usize) -> Vec<(Edge, usize)> {
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
}

/// Checks amenability of a (CP)DAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
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
    enum WalkStatus {
        /// Descendant / Directed (always amenable)
        D,
        /// Possible Descendant / Partially Directed, Amenable (starts T→)
        PD_AM,
        /// Possible Descendant / Partially Directed, Not Amenable (starts T—)
        PD_NAM,
        /// Initial status
        Init,
    }

    let mut desc = FxHashSet::from_iter(t.iter().copied());
    let mut poss_desc = desc.clone();
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            WalkStatus::PD_NAM => {
                not_amenable.insert(node);
                poss_desc.insert(node);
            }
            WalkStatus::PD_AM => {
                poss_desc.insert(node);
            }
            WalkStatus::D => {
                poss_desc.insert(node);
                desc.insert(node);
            }
            _ => (),
        }

        for (move_on_by, w) in get_next_steps(graph, t, node) {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::D)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_NAM)),
                    _ => None,
                },
                WalkStatus::PD_AM | WalkStatus::PD_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => Some((move_on_by, w, walkstatus)),
                    _ => None,
                },
                WalkStatus::D => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::D)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_AM)),
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

/// Checks amenability of a (CP)DAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Returns tuple of:<br>
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
pub fn get_pd_nam(graph: &PDAG, t: &[usize]) -> (FxHashSet<usize>, FxHashSet<usize>) {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum WalkStatus {
        /// Possible Descendant / Partially Directed, Amenable (starts T→)
        PD_AM,
        /// Possible Descendant / Partially Directed, Not Amenable (starts T—)
        PD_NAM,
        /// Initial status
        Init,
    }

    let mut poss_de = FxHashSet::from_iter(t.iter().copied());
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            WalkStatus::PD_NAM => {
                not_amenable.insert(node);
                poss_de.insert(node);
            }
            // any other PD walk
            WalkStatus::PD_AM => {
                poss_de.insert(node);
            }
            _ => (),
        }

        for (move_on_by, w) in get_next_steps(graph, t, node) {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::PD_AM)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_NAM)),
                    _ => None,
                },
                WalkStatus::PD_AM | WalkStatus::PD_NAM => match move_on_by {
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

/// Checks amenability of a CPDAG relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y.
///
/// Returns set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
///
/// Follows Algorithm 2 in https://doi.org/10.48550/arXiv.2402.08616
pub fn get_nam(graph: &PDAG, t: &[usize]) -> FxHashSet<usize> {
    let mut not_amenable = FxHashSet::<usize>::default();

    let mut visited = FxHashSet::<usize>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v)));

    while let Some((arrived_by, node)) = to_visit_stack.pop() {
        visited.insert(node);
        match arrived_by {
            Edge::Init => {
                graph
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
                get_next_steps(graph, t, node)
                    .into_iter()
                    .for_each(|(move_on_by, w)| {
                        if !visited.contains(&w) {
                            to_visit_stack.push((move_on_by, w));
                        }
                    });
            }
        }
    }
    not_amenable
}

fn get_next_steps_conditioned(
    graph: &PDAG,
    t: &[usize],
    arrived_by: Edge,
    v: usize,
    node_is_adjustment: bool,
) -> Vec<(Edge, usize, bool)> {
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
}

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Returns tuple of:<br>
/// - Set PD of possible descendants of T in G
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
///   This includes all NAM, so NAM is a subset NVA.
pub fn get_pd_nam_nva(
    graph: &PDAG,
    t: &[usize],
    z: &FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>, FxHashSet<usize>) {
    let mut poss_de = FxHashSet::from_iter(t.iter().copied());
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

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
            WalkStatus::NON_CAUSAL_OPEN => {
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

        for (move_on_by, w, blocked) in
            get_next_steps_conditioned(graph, t, arrived_by, node, node_is_adjustment)
        {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::PD_OPEN_AM)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_OPEN_NAM)),
                    _ => None,
                },
                WalkStatus::PD_OPEN_AM | WalkStatus::PD_BLOCK_AM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_AM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_AM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_NAM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_NAM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                WalkStatus::NON_CAUSAL_OPEN if !blocked => {
                    Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
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

    (poss_de, not_amenable, not_vas)
}

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Follows Algorithm 3 in https://doi.org/10.48550/arXiv.2402.08616
///
/// Returns tuple of:<br>
/// - Set NAM (Not AMenable) of nodes Y \notin T in G such that G is not amenable relative to (T, Y)
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
///   This includes all NAM, so NAM is a subset NVA.
#[cfg(test)]
pub fn get_nam_nva(
    graph: &PDAG,
    t: &[usize],
    z: &FxHashSet<usize>,
) -> (FxHashSet<usize>, FxHashSet<usize>) {
    let mut not_amenable = FxHashSet::<usize>::default();
    let mut not_vas = z.clone();

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => {
                not_amenable.insert(node);
                // we want the property that not_amenable is a subset of not_vas
                // so, if we insert a node into not_amenable, we also insert it into not_vas
                not_vas.insert(node);
            }
            WalkStatus::NON_CAUSAL_OPEN | WalkStatus::PD_BLOCK_AM => {
                not_vas.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in
            get_next_steps_conditioned(graph, t, arrived_by, node, node_is_adjustment)
        {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming => Some((move_on_by, w, WalkStatus::PD_OPEN_AM)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN)),
                    Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_OPEN_NAM)),
                    _ => None,
                },
                WalkStatus::PD_OPEN_AM | WalkStatus::PD_BLOCK_AM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_AM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_AM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                WalkStatus::PD_OPEN_NAM | WalkStatus::PD_BLOCK_NAM => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK_NAM)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN_NAM) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                WalkStatus::NON_CAUSAL_OPEN if !blocked => {
                    Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
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

    (not_amenable, not_vas)
}

/// Validate Z as adjustment set relative to (T, Y) for a given set T of treatment
/// nodes and all possible Y in G.
///
/// Returns tuple of:<br>
/// - Set NVA (Not Validly Adjusted) of nodes Y \notin T in G such that Z is not a valid adjustment set for (T, Y) in G.
/// Here, amenability (condition 1.) is not verified, that is, NVA is not a superset of NAM;
/// instead, NVA contains Y for which condition 2. or 3.
/// of the modified adjustment criterion for walk-based verification
/// in https://doi.org/10.48550/arXiv.2402.08616 are violated
pub fn get_invalidly_un_blocked(
    graph: &PDAG,
    t: &[usize],
    z: &FxHashSet<usize>,
) -> FxHashSet<usize> {
    #[allow(non_camel_case_types)]
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    enum WalkStatus {
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

    let mut visited = FxHashSet::<(Edge, usize, WalkStatus)>::default();
    let mut to_visit_stack = Vec::from_iter(t.iter().map(|v| (Edge::Init, *v, WalkStatus::Init)));

    while let Some((arrived_by, node, walkstatus)) = to_visit_stack.pop() {
        visited.insert((arrived_by, node, walkstatus));

        match walkstatus {
            // when the node is reached on a causal path but blocked, or an unblocked non-causal path
            WalkStatus::PD_BLOCK | WalkStatus::NON_CAUSAL_OPEN => {
                ivb.insert(node);
            }
            _ => (),
        }
        let node_is_adjustment = z.contains(&node);

        for (move_on_by, w, blocked) in
            get_next_steps_conditioned(graph, t, arrived_by, node, node_is_adjustment)
        {
            let next = match walkstatus {
                WalkStatus::Init => match move_on_by {
                    Edge::Incoming | Edge::Undirected => Some((move_on_by, w, WalkStatus::PD_OPEN)),
                    Edge::Outgoing => Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN)),
                    _ => None,
                },
                WalkStatus::PD_OPEN | WalkStatus::PD_BLOCK => match move_on_by {
                    Edge::Incoming | Edge::Undirected => match blocked {
                        false => Some((move_on_by, w, walkstatus)),
                        true => Some((move_on_by, w, WalkStatus::PD_BLOCK)),
                    },
                    Edge::Outgoing if !blocked && matches!(walkstatus, WalkStatus::PD_OPEN) => {
                        Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
                    }
                    _ => None,
                },
                WalkStatus::NON_CAUSAL_OPEN if !blocked => {
                    Some((move_on_by, w, WalkStatus::NON_CAUSAL_OPEN))
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

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rustc_hash::FxHashSet;

    use crate::graph_operations::{
        ancestor_aid, gensearch, get_descendants, get_nam_nva, get_possible_descendants, oset_aid,
        parent_aid, ruletables,
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
    #[ignore]
    pub fn reachability_algos_agree() {
        let reps = 30;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        (0..reps).for_each(|_rep| {
            let pdag = PDAG::random_pdag(0.5, 100, &mut rng);
            let t = [0];
            let adjust = gensearch(&pdag, ruletables::Parents {}, t.iter(), false);

            let d_expected = get_descendants(&pdag, t.iter());
            let pd_expected = get_possible_descendants(&pdag, t.iter());
            let (nam_expected, nva_expected) = get_nam_nva(&pdag, &t, &adjust);

            #[cfg(test)]
            assert!(d_expected.is_subset(&pd_expected));
            #[cfg(test)]
            assert!(nam_expected.is_subset(&pd_expected));
            #[cfg(test)]
            assert!(nam_expected.is_subset(&nva_expected));

            let (d, pd, nam) = super::get_d_pd_nam(&pdag, &t);
            assert_eq!(d_expected, d);
            assert_eq!(pd_expected, pd);
            assert_eq!(nam_expected, nam);

            let (pd, nam) = super::get_pd_nam(&pdag, &t);
            assert_eq!(nam_expected, nam);
            assert_eq!(pd_expected, pd);

            let nam = super::get_nam(&pdag, &t);
            assert_eq!(nam_expected, nam);

            let (pd, nam, nva) = super::get_pd_nam_nva(&pdag, &t, &adjust);
            assert_eq!(pd_expected, pd);
            assert_eq!(nam_expected, nam);
            assert_eq!(nva_expected, nva);

            let ivb = super::get_invalidly_un_blocked(&pdag, &t, &adjust);
            assert!(ivb.is_subset(&nva_expected));
            assert_eq!(nva_expected, &ivb | &nam_expected);
        });
    }
}
