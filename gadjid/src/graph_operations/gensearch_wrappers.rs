// SPDX-License-Identifier: MPL-2.0
//! Holds functions that simplify calls to the generalized search algorithm [`gensearch`].
//! The real logic is happening in the intersection of `gensearch` and the chosen `ruletables`.
// (The latter of which also holds the relevant tests.)

use crate::{sets::NodeSet, PDAG};

use super::ruletables::{proper_ancestors::ProperAncestors, Parents};

#[cfg(test)]
use super::ruletables::{Ancestors, Children, Descendants};

/// Gets all ancestors of a set of nodes. Will also return the starting nodes.
#[cfg(test)]
pub fn get_ancestors<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> NodeSet {
    let ruletable = Ancestors {};
    // gensearch yield_starting_vertices 'true' because $a \in Ancestors(a)$
    crate::graph_operations::gensearch(dag, ruletable, starting_vertices, true)
}

/// Gets the union of children of each node. This is more efficient than calling `children_of` for each node and then joining the results.
#[cfg(test)]
pub fn get_children<'a>(dag: &PDAG, starting_vertices: impl Iterator<Item = &'a usize>) -> NodeSet {
    let ruletable = Children {};
    // gensearch yield_starting_vertices 'false' because $a \notin Children(a)$
    crate::graph_operations::gensearch(dag, ruletable, starting_vertices, false)
}

/// Gets all descendants of a set of nodes. Will also return the starting nodes.
#[cfg(test)]
pub fn get_descendants<'a>(
    dag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> NodeSet {
    let start: Vec<usize> = starting_vertices.copied().collect();
    let ruletable = Descendants {};
    // gensearch yield_starting_vertices 'true' because $a \in Descendants(a)$
    crate::graph_operations::gensearch(dag, ruletable, start.iter(), true)
}

/// Gets the union of parents of each node. This is more efficient than calling `parents_of` for each node and then joining the results.
pub fn get_parents<'a>(dag: &PDAG, starting_vertices: impl Iterator<Item = &'a usize>) -> NodeSet {
    let ruletable = Parents {};
    // gensearch yield_starting_vertices 'false' because $a \notin Parents(a)$
    crate::graph_operations::gensearch(dag, ruletable, starting_vertices, false)
}

/// Gets all proper ancestors of responses given them and the treatments
pub fn get_proper_ancestors<'a>(
    dag: &PDAG,
    treatments: impl Iterator<Item = &'a usize>,
    responses: impl Iterator<Item = &'a usize>,
) -> NodeSet {
    let treatment_hashset = NodeSet::from_iter(treatments.copied());
    let ruletable = ProperAncestors {
        treatments: treatment_hashset,
    };
    // gensearch yield_starting_vertices 'true' because $a \in ProperAncestors(a)$
    crate::graph_operations::gensearch(dag, ruletable, responses, true)
}
