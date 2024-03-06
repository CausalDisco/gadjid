// SPDX-License-Identifier: MPL-2.0
//! Implements the generalized graph search algorithm and other search algorithms using it.
use crate::{
    graph_operations::ruletables::RuleTable, partially_directed_acyclic_graph::Edge, PDAG,
};
use rustc_hash::FxHashSet;

/// General reachability graph search algorithm, Algorithm 6 in https://doi.org/10.48550/arXiv.2211.16468
pub fn gensearch<'a>(
    dag: &PDAG,
    ruletable: impl RuleTable,
    starting_vertices: impl Iterator<Item = &'a usize>,
    yield_starting_vertices: bool,
) -> FxHashSet<usize> {
    // Holds the edge traversed to get to some node and the node itself
    let mut to_visit_stack = Vec::<(Edge, usize)>::new();

    let mut result = FxHashSet::default();

    for s in starting_vertices {
        to_visit_stack.push((Edge::Init, *s));
        if yield_starting_vertices {
            result.insert(*s);
        }
    }

    // initialize all edges to visited=false for incoming and outgoing
    let mut visited_in = FxHashSet::default();
    let mut visited_out = FxHashSet::default();

    while let Some((current_edge, current_node)) = to_visit_stack.pop() {
        match current_edge {
            Edge::Incoming => {
                visited_in.insert(current_node);
            }
            Edge::Outgoing => {
                visited_out.insert(current_node);
            }
            _ => (),
        }

        for (next_edge, is_incoming) in [(Edge::Incoming, true), (Edge::Outgoing, false)] {
            let neighborhood: &[usize] = match next_edge {
                Edge::Incoming => dag.children_of(current_node),
                Edge::Outgoing => dag.parents_of(current_node),
                _ => unreachable!(),
            };

            for next_node in neighborhood.iter().copied() {
                let (continue_to_next, yield_next) =
                    ruletable.lookup(&current_edge, &current_node, &next_edge, &next_node);
                if continue_to_next
                    && (is_incoming && !visited_in.contains(&next_node)
                        || !is_incoming && !visited_out.contains(&next_node))
                {
                    to_visit_stack.push((next_edge, next_node));
                }
                if yield_next {
                    result.insert(next_node);
                }
            }
        }
    }

    result
}
