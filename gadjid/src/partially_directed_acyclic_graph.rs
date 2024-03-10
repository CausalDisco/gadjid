// SPDX-License-Identifier: MPL-2.0
//! Defines the PDAG struct that is a supertype of DAGs and CPDAGs.

use core::panic;
use rand::distributions::Distribution;
use rustc_hash::FxHashMap;
use std::{error::Error, fmt};

use crate::{
    ascending_list_utils::ascending_lists_first_shared_element,
    graph_loading::edgelist::{ColumnMajorOrder, Edgelist, RowMajorOrder},
};

/// PDAG edge enum defined from a graph traversal perspective.
///
/// If traversing from some node `X` along edge `e` to a node of interest `Y` ,
/// defines `e` as the direction it has to `Y`.
///
/// Examples:
///
/// When traversing from X to its child Y, `X -> Y`, we have `e` = `Incoming`.
///
/// It can be instructive to think of associating the edge and node of interest with
/// a parenthesis, like `X (-> Y)`, making it clear that the edge is `Incoming`.
///
/// In the case of `X (<- Y)` <=> `(Y ->) X`, the edge would be `Outgoing`.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Edge {
    /// An auxiliary edge type used to indicate that a search was rooted at this node.
    Init,
    /// -> An edge that points into the node it is associated with
    Incoming,
    /// <- An edge that points away from the node it is associated with
    Outgoing,
    /// -- An undirected edge
    Undirected,
}

/// Represents a partially directed acyclic graph (PDAG). Internally, stores an adjacency matrix encoded in a
/// CSR-like format.
#[derive(Debug, PartialEq, Eq)]
pub struct PDAG {
    /// Codes which edges correspond to which node
    /// i.e. len |V|+1 (first entry always 0, last entry always 2*|E|)
    /// `node_edge_ranges[i]` is the index of the first edge attached to node i, and
    /// `node_edge_ranges[i+1]-1` is the index of the last edge attached to node i.
    pub node_edge_ranges: Vec<usize>,

    /// Holds the number of incoming edges for each node, len is |V|. Because the neighbourhoods are sorted by
    /// incoming, then undirected, then outgoing, we can infer the different types of edges by looking at the element
    /// number of the edge in the neighbourhood.
    pub node_in_out_degree: Vec<(usize, usize)>,

    /// For some node holds all the nodes attached to it.
    /// The len is 2*|E| because we store both X->Y and Y<-X.
    /// If there are N neighbors for node i, of which P are incoming, U are undirected and C are outgoing.
    /// then P + U + C = N, and the first P elements of the neighbourhood are the incoming neighbors,
    /// the next U elements are the undirected neighbors, and the last C elements are the outgoing neighbors.
    pub neighbourhoods: Vec<usize>,

    /// The number of nodes in the graph
    pub n_nodes: usize,

    /// The number of directed edges in the graph
    pub n_directed_edges: usize,

    /// The number of undirected edges in the graph
    pub n_undirected_edges: usize,

    /// The type of the PDAG
    pub pdag_type: Structure,
}

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
/// The type that the PDAG has been determined to be.
pub enum Structure {
    /// The PDAG contains no undirected edges and is acyclic, so it is a DAG.
    DAG,
    /// The graph contains directed and undirected edges and no directed cycles. 
    /// It is however not guaranteed to be a CPDAG.
    CPDAG,
}


/// Will display the adjacency matrix of the PDAG, encoded as row-to-column adjacency matrix.
impl fmt::Display for PDAG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut adjacency = vec![vec![0; self.n_nodes]; self.n_nodes];

        #[allow(clippy::needless_range_loop)]
        for node in 0..self.n_nodes {
            for child in self.children_of(node).iter().copied() {
                adjacency[node][child] = 1;
            }
            for undirected in self.adjacent_undirected_of(node).iter().copied() {
                adjacency[node][undirected] = 2;
            }
        }

        for row in adjacency {
            writeln!(f)?;
            for val in row {
                write!(f, "{} ", val)?;
            }
        }
        Ok(())
    }
}

impl PDAG {
    /// Given a node, return all nodes reachable by an incoming edge. Nodes will be returned in sorted
    /// ascending order
    pub fn parents_of(&self, node: usize) -> &[usize] {
        let start = self.node_edge_ranges[node];
        let end = self.node_edge_ranges[node + 1];
        let nb = &self.neighbourhoods[start..end];

        // take the first #in_degree elements
        let parent_end = self.node_in_out_degree[node].0;
        &nb[..parent_end]
    }
    /// Given a node, return all nodes reachable by an outgoing edge. Nodes will be returned in sorted
    /// ascending order
    pub fn children_of(&self, node: usize) -> &[usize] {
        let start = self.node_edge_ranges[node];
        let end = self.node_edge_ranges[node + 1];
        let nb = &self.neighbourhoods[start..end];

        // take the last #out_degree elements
        let child_start = nb.len() - self.node_in_out_degree[node].1;

        &nb[child_start..]
    }

    /// Given a node, return all nodes reachable via an undirected edge. Nodes will be returned in sorted
    /// ascending order
    pub fn adjacent_undirected_of(&self, node: usize) -> &[usize] {
        let start = self.node_edge_ranges[node];
        let end = self.node_edge_ranges[node + 1];
        let nb = &self.neighbourhoods[start..end];

        // take from the first #in_degree to the last #out_degree elements
        let parents_end = self.node_in_out_degree[node].0;
        let children_start = nb.len() - self.node_in_out_degree[node].1;

        &nb[parents_end..children_start]
    }

    /// Given a node, return all nodes reachable in one step along a possibly incoming edge (undirected or incoming).
    /// Not yielded in any particular order.
    pub fn possible_parents_of(&self, node: usize) -> &[usize] {
        let start = self.node_edge_ranges[node];
        let end = self.node_edge_ranges[node + 1];
        let nb = &self.neighbourhoods[start..end];

        // take from the start to the last #out_degree elements
        let children_start = nb.len() - self.node_in_out_degree[node].1;

        &nb[..children_start]
    }

    /// Given a node, return all nodes reachable in one step along a possibly outgoing edge (undirected or outgoing).
    /// Not yielded in any particular order.
    pub fn possible_children_of(&self, node: usize) -> &[usize] {
        let start = self.node_edge_ranges[node];
        let end = self.node_edge_ranges[node + 1];
        let nb = &self.neighbourhoods[start..end];

        // take from the first #in_degree to the end
        let parents_end = self.node_in_out_degree[node].0;

        &nb[parents_end..]
    }
}

#[derive(Debug)]
/// Error that can occur when loading a PDAG from an adjacency matrix.
pub enum LoadError {
    /// The adjacency matrix does not represent a PDAG because it contains a cycle.
    NotAcyclic,
}

impl Error for LoadError {}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::NotAcyclic => write!(f, "Graph is not acyclic"),
        }
    }
}

impl PDAG {
    // TODO: from_row_major and from_col_major are very similar, unify as much as possible for clarity

    /// Creates a PDAG from a adjacency matrix traversed in row-major order.
    ///
    /// If there is an undirected edge between node i and j, the edgelist may yield
    /// (i, j, 2) and (j, i, 2). Yielding only one is also fine, but yielding
    /// (i, j, 1) and (j, i, 1), or (i, j, 1) and (j, i, 2) will cause a panic.
    pub fn try_from_row_major<I>(edgelist: Edgelist<RowMajorOrder, I>) -> Result<PDAG, LoadError>
    where
        I: Iterator<Item = (usize, usize, i8)>,
    {
        let matrix_size = edgelist.size;
        // incoming edges will be encountered in order lexicographically sorted by (inner_idx, outer_idx),
        let mut incomings: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        // outgoing edges will be encountered in order (outer_idx, inner_idx),
        // meaning they all outgoing edges from some node will lie contiguously,
        // so we can store them in a vector and keep track of node indices
        let mut outgoings = vec![];
        // Assuming that only few nodes have undirected edges, we can save some memory by
        // only allocating the vector if we actually need it.
        let mut undirected: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        let mut node_io_degree = vec![(0, 0); matrix_size];
        // accessing the slice is faster than accessing the vector directly
        let node_io_degree_slice = node_io_degree.as_mut_slice();
        let mut node_undirected_degree = vec![0; matrix_size];
        let node_undirected_degree_slice = node_undirected_degree.as_mut_slice();

        let mut node_edge_ranges = vec![0; matrix_size + 1];
        let node_edge_ranges_slice = node_edge_ranges.as_mut_slice();

        for (outer_idx, inner_idx, val) in edgelist {
            // verify that no edges are self-looping
            if outer_idx == inner_idx {
                panic!("found unexpected self-looping edge '{val}' at position ({outer_idx}, {inner_idx})")
            }

            match val {
                1 => {
                    incomings.entry(inner_idx).or_default().push(outer_idx);
                    outgoings.push(inner_idx);
                    let (in_deg, out_deg) = node_io_degree_slice[inner_idx];
                    node_io_degree_slice[inner_idx] = (in_deg + 1, out_deg);

                    let (in_deg, out_deg) = node_io_degree_slice[outer_idx];
                    node_io_degree_slice[outer_idx] = (in_deg, out_deg + 1);
                }
                2 => {
                    undirected.entry(inner_idx).or_default().push(outer_idx);
                    undirected.entry(outer_idx).or_default().push(inner_idx);

                    node_undirected_degree_slice[inner_idx] += 1;
                    node_undirected_degree_slice[outer_idx] += 1;
                }
                _ => panic!("Found value '{val}' in adjacency matrix at position ({}, {}), expected to see only 0's, 1's or 2's for PDAG.", outer_idx, inner_idx)
            }
        }

        // the neighboorhood list will be at least as long as the number of directed edges twice. It
        // might be longer if there are additionally undirected edges.
        let mut neighbourhoods = Vec::with_capacity(outgoings.len() * 2);
        let mut outgoing_iter = outgoings.into_iter();
        let mut n_edges = 0;
        let mut n_directed_edges = 0;
        let mut n_undirected_edges = 0;
        // assertion for the compiler in case it could help:
        assert!(node_io_degree_slice.len() == matrix_size);
        for i in 0..matrix_size {
            let mut nb = vec![];

            // adding all the incoming edges and counting
            nb.extend(incomings.remove(&i).unwrap_or_default().iter().copied());
            let n_in = nb.len();

            // adding the undirected edges. We have to sort these here and then dedup them.
            // They might have been double-counted as incoming and outgoing edges, but we only want them
            // once, because we are lenient with a double-coding of undirected edges.
            if let Some(mut vec) = undirected.remove(&i) {
                vec.sort_unstable();
                vec.dedup();
                nb.extend(vec.into_iter());
            }
            let n_undirected = nb.len() - n_in;

            // adding all the outgoing edges and counting
            nb.extend(outgoing_iter.by_ref().take(node_io_degree_slice[i].1));

            n_undirected_edges += n_undirected;
            n_directed_edges += nb.len() - n_undirected;

            if !nb.is_empty() {
                // Now we want to ensure that the matrix passed represents a simple graph.
                // We compare all endpoint nodes between pairs of (incoming, undirected, outgoing) and
                // check that they are distinct.
                // The fact that all the edges are sorted by endpoint node makes this a bit easier,
                // allowing us to scan throught the lists only once without having to sort them first.

                let incomings = &nb[..n_in];
                let undirected = &nb[n_in..n_in + n_undirected];
                let outgoings = &nb[n_in + n_undirected..];

                if let Some(val) = ascending_lists_first_shared_element(incomings, undirected) {
                    panic!(
                        "Graph not simple: found both edge {val}->{i} and edge {val}--{i} in adjacency matrix",
                    );
                }
                if let Some(val) = ascending_lists_first_shared_element(outgoings, undirected) {
                    panic!(
                        "Graph not simple: found both edge {i}->{val} and edge {i}--{val} in adjacency matrix",
                    );
                }
                if let Some(val) = ascending_lists_first_shared_element(incomings, outgoings) {
                    panic!(
                        "Graph not simple: found both edge {val}->{i} and edge {i}->{val} in adjacency matrix",
                    );
                }
            }

            // we fill in the node_edge_ranges vector with the size of the neighbourhood
            node_edge_ranges_slice[i + 1] = n_edges + nb.len();
            n_edges += nb.len();

            // finally, we add the constructed neighbourhood to the neighbourhoods list and continue
            neighbourhoods.extend(nb.into_iter());
        }

        n_directed_edges /= 2;
        n_undirected_edges /= 2;

        let mut pdag = PDAG {
            node_edge_ranges,
            node_in_out_degree: node_io_degree,
            neighbourhoods,
            n_nodes: matrix_size,
            n_directed_edges,
            n_undirected_edges,
            // does not matter what we put here as it will always be overwritten
            pdag_type: Structure::DAG,
        };

        if has_cycle(&pdag) {
            return Err(LoadError::NotAcyclic);
        }

        if pdag.n_undirected_edges == 0 {
            pdag.pdag_type = Structure::DAG;
        } else {
            pdag.pdag_type = Structure::CPDAG;
        }

        Ok(pdag)
    }

    /// Creates a PDAG from a adjacency matrix traversed in column-major order.
    ///
    /// If there is an undirected edge between node i and j, the edgelist may yield
    /// (i, j, 2) and (j, i, 2). Yielding only one is also fine, but yielding
    /// (i, j, 1) and (j, i, 1), or (i, j, 1) and (j, i, 2) will cause a panic.
    pub fn try_from_col_major<I>(edgelist: Edgelist<ColumnMajorOrder, I>) -> Result<PDAG, LoadError>
    where
        I: Iterator<Item = (usize, usize, i8)>,
    {
        let matrix_size = edgelist.size;
        // outgoing_ edges will be encountered in order lexicographically sorted by (inner_idx, outer_idx),
        let mut outgoings_: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        // outgoing edges will be encountered in order (outer_idx, inner_idx),
        // meaning they all outgoing edges from some node will lie contiguously,
        // so we can store them in a vector and keep track of node indices
        let mut incomings_ = vec![];
        // Assuming that only few nodes have undirected edges, we can save some memory by
        // only allocating the vector if we actually need it.
        let mut undirected: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        let mut node_io_degree = vec![(0, 0); matrix_size];
        // accessing the slice is faster than accessing the vector directly
        let node_io_degree_slice = node_io_degree.as_mut_slice();
        let mut node_undirected_degree = vec![0; matrix_size];
        let node_undirected_degree_slice = node_undirected_degree.as_mut_slice();

        let mut node_edge_ranges = vec![0; matrix_size + 1];
        let node_edge_ranges_slice = node_edge_ranges.as_mut_slice();

        for (outer_idx, inner_idx, val) in edgelist {
            // verify that no edges are self-looping
            if outer_idx == inner_idx {
                panic!("found unexpected self-looping edge '{val}' at position ({outer_idx}, {inner_idx})")
            }

            match val {
                1 => {
                    outgoings_.entry(inner_idx).or_default().push(outer_idx);
                    incomings_.push(inner_idx);
                    let (in_deg, out_deg) = node_io_degree_slice[inner_idx];
                    node_io_degree_slice[inner_idx] = (in_deg, out_deg + 1);

                    let (in_deg, out_deg) = node_io_degree_slice[outer_idx];
                    node_io_degree_slice[outer_idx] = (in_deg + 1, out_deg);
                }
                2 => {
                    undirected.entry(inner_idx).or_default().push(outer_idx);
                    undirected.entry(outer_idx).or_default().push(inner_idx);

                    node_undirected_degree_slice[inner_idx] += 1;
                    node_undirected_degree_slice[outer_idx] += 1;
                }
                _ => panic!("Found value '{val}' in adjacency matrix at position ({}, {}), expected to see only 0's, 1's or 2's for PDAG.", outer_idx, inner_idx),
            }
        }

        // the neighboorhood list will be at least as long as the number of directed edges twice. It
        // might be longer if there are additionally undirected edges.
        let mut neighbourhoods = Vec::with_capacity(incomings_.len() * 2);
        let mut incomings_iter = incomings_.into_iter();
        let mut n_edges = 0;
        let mut n_directed_edges = 0;
        let mut n_undirected_edges = 0;
        // assertion for the compiler in case it could help:
        assert!(node_io_degree_slice.len() == matrix_size);
        for i in 0..matrix_size {
            let mut nb = vec![];

            // adding all the incoming edges and counting
            nb.extend(incomings_iter.by_ref().take(node_io_degree_slice[i].0));
            let n_in = nb.len();

            // adding the undirected edges. We have to sort these here and then dedup them.
            // They might have been double-counted as incoming and outgoing edges, but we only want them
            // once, because we are lenient with a double-coding of undirected edges.
            if let Some(mut vec) = undirected.remove(&i) {
                vec.sort_unstable();
                vec.dedup();
                nb.extend(vec.into_iter());
            }

            let n_undirected = nb.len() - n_in;

            // adding all the outgoing edges and counting
            nb.extend(outgoings_.remove(&i).unwrap_or_default().iter().copied());

            n_undirected_edges += n_undirected;
            n_directed_edges += nb.len() - n_undirected;

            if !nb.is_empty() {
                // Now we want to ensure that the matrix passed represents a simple graph.
                // We compare all endpoint nodes between pairs of (incoming, undirected, outgoing) and
                // check that they are distinct.
                // The fact that all the edges are sorted by endpoint node makes this a bit easier,
                // allowing us to scan throught the lists only once without having to sort them first.

                // One could also compare all 3 at once for a slight speedup (at most x2)
                // but I found it very difficult to make it elegant. There would be many more cases to
                // consider, more (constant time) overhead for bookkeeping. I think it's not worth it.

                let incomings = &nb[..n_in];
                let undirected = &nb[n_in..n_in + n_undirected];
                let outgoings = &nb[n_in + n_undirected..];

                if let Some(val) = ascending_lists_first_shared_element(incomings, undirected) {
                    panic!(
                        "Graph not simple: found both edge {val}->{i} and edge {val}--{i} in adjacency matrix",
                    );
                }
                if let Some(val) = ascending_lists_first_shared_element(outgoings, undirected) {
                    panic!(
                        "Graph not simple: found both edge {i}->{val} and edge {i}--{val} in adjacency matrix",
                    );
                }
                if let Some(val) = ascending_lists_first_shared_element(incomings, outgoings) {
                    panic!(
                        "Graph not simple: found both edge {val}->{i} and edge {i}->{val} in adjacency matrix",
                    );
                }
            }

            // we fill in the node_edge_ranges vector with the size of the neighbourhood
            node_edge_ranges_slice[i + 1] = n_edges + nb.len();
            n_edges += nb.len();

            // finally, we add the constructed neighbourhood to the neighbourhoods list and continue
            neighbourhoods.extend(nb.into_iter());
        }

        n_directed_edges /= 2;
        n_undirected_edges /= 2;

        let mut pdag = PDAG {
            node_edge_ranges,
            node_in_out_degree: node_io_degree,
            neighbourhoods,
            n_nodes: matrix_size,
            n_directed_edges,
            n_undirected_edges,
            // does not matter what we put here as it will always be overwritten
            pdag_type: Structure::DAG,
        };

        if has_cycle(&pdag) {
            return Err(LoadError::NotAcyclic);
        }

        if pdag.n_undirected_edges == 0 {
            pdag.pdag_type = Structure::DAG;
        } else {
            pdag.pdag_type = Structure::CPDAG;
        }

        Ok(pdag)
    }

    /// Creates a PDAG from a row-major encoded adjacency matrix. 
    /// An entry of 1 at position `[i,j]` indicates a directed edge `i -> j`, 
    /// the opposite of how [`from_col_to_row_vecvec`] does it.
    /// An entry of 2 at position `[i,j]` and/or `[j,i]` indicates an undirected edge between `i` and `j`.
    pub fn from_row_to_col_vecvec(dense: Vec<Vec<i8>>) -> Self {
        let edgelist = Edgelist::from_vecvec(dense);
        let mut pdag = PDAG::try_from_row_major(edgelist).unwrap();

        // TODO: CPDAGness check
        if pdag.n_undirected_edges > 0 {
            pdag.pdag_type = Structure::CPDAG
        } else {
            pdag.pdag_type = Structure::DAG
        }
        pdag
    }

    /// Creates a PDAG from a row_major adjacency matrix. 
    /// An entry of 1 at position `[i,j]` indicates a directed edge `j -> i`, 
    /// the opposite of how [`from_row_to_col_vecvec`] does it.
    /// An entry of 2 at position `[i,j]` and/or `[j,i]` indicates an undirected edge between `i` and `j`.
    pub fn from_col_to_row_vecvec(vecvec: Vec<Vec<i8>>) -> Self {
        let edgelist = Edgelist::from_vecvec(vecvec);
        let mut pdag = PDAG::try_from_col_major(edgelist).unwrap();

        // TODO: CPDAGness check
        if pdag.n_undirected_edges > 0 {
            pdag.pdag_type = Structure::CPDAG
        } else {
            pdag.pdag_type = Structure::DAG
        }
        pdag
    }

    /// Creates a random DAG with the given edge density and size.
    pub fn random_dag(edge_density: f64, graph_size: usize, mut rng: impl rand::RngCore) -> PDAG {
        assert!(graph_size > 0, "Graph size must be larger than 0");
        assert!(
            (0.0..=1.0).contains(&edge_density),
            "edge probability must be in [0, 1]"
        );
        let edge_dist = rand::distributions::Bernoulli::new(edge_density).unwrap();

        let mut adjacency = vec![vec![0; graph_size]; graph_size];
        let permutation = rand::seq::index::sample(&mut rng, graph_size, graph_size);
        for y in 0..graph_size {
            for x in y + 1..graph_size {
                adjacency[permutation.index(x)][permutation.index(y)] =
                    if edge_dist.sample(&mut rng) { 1 } else { 0 };
            }
        }

        PDAG::from_row_to_col_vecvec(adjacency)
    }

    /// Creates a random vecvec of a PDAG with random edges with the given edge density and size.
    pub fn _random_pdag_vecvec(
        edge_density: f64,
        graph_size: usize,
        mut rng: impl rand::RngCore,
    ) -> Vec<Vec<i8>> {
        assert!(graph_size > 0, "Graph size must be larger than 0");
        assert!(
            (0.0..=1.0).contains(&edge_density),
            "edge probability must be in [0, 1]"
        );
        let edge_dist = rand::distributions::Bernoulli::new(edge_density).unwrap();
        // P(edge between X and Y is directed) given that there is an edge between X and Y
        let p_directedness = 0.8;
        let directionality_dist = rand::distributions::Bernoulli::new(p_directedness).unwrap();
        let mut adjacency = vec![vec![0; graph_size]; graph_size];
        let permutation = rand::seq::index::sample(&mut rng, graph_size, graph_size);
        for y in 0..graph_size {
            for x in y + 1..graph_size {
                adjacency[permutation.index(x)][permutation.index(y)] =
                    if edge_dist.sample(&mut rng) {
                        if directionality_dist.sample(&mut rng) {
                            1
                        } else {
                            2
                        }
                    } else {
                        0
                    };
            }
        }
        adjacency
    }

    /// Creates a random PDAG with random edges with the given edge density and size.
    pub fn random_pdag(edge_density: f64, graph_size: usize, mut rng: impl rand::RngCore) -> PDAG {
        PDAG::from_row_to_col_vecvec(PDAG::_random_pdag_vecvec(
            edge_density,
            graph_size,
            &mut rng,
        ))
    }
}

/// Returns true if the graph has a cycle, false otherwise.
/// An implementation of Kahn's algorithm for topological sorting.
pub fn has_cycle(graph: &PDAG) -> bool {
    let mut in_degree: Vec<usize> = graph.node_in_out_degree.iter().map(|x| x.0).collect();

    let mut stack = Vec::new();

    // Fill stack with all roots.

    // Assert for the compiler in case it helps:
    assert!(in_degree.len() == graph.n_nodes);
    #[allow(clippy::needless_range_loop)]
    for u in 0..graph.n_nodes {
        if in_degree[u] == 0 {
            stack.push(u);
        }
    }

    // no root node implies cycle
    if stack.is_empty() {
        return true;
    }

    // Initialize count of visited vertices to #root nodes
    let mut visited = stack.len();

    // One by one destack vertices from stack and enstack
    // adjacents if indegree of adjacent becomes 0
    while let Some(current) = stack.pop() {
        // Iterate through all child nodes v
        // of popped node and decrease their in-degree
        // by 1 (effectively removing edges from the graph)
        for v in graph.children_of(current).iter().copied() {
            in_degree[v] -= 1;

            // If in-degree becomes zero, add it to stack because it is now a root.
            if in_degree[v] == 0 {
                stack.push(v);

                // every time we find a node with in-degree 0, we increment #visited.
                // This should happen exactly |V| times.
                visited += 1;
            }
        }
    }
    // Check that we visited all nodes once and no more or less. More would imply a cycle.
    visited < graph.n_nodes
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use std::collections::HashSet;

    use crate::PDAG;

    #[test]
    #[should_panic]
    pub fn fail_if_not_simple() {
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1], //
            vec![1, 0],
        ];

        PDAG::from_row_to_col_vecvec(dense);
    }

    #[test]
    #[should_panic]
    pub fn fail_if_not_simple2() {
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1], //
            vec![2, 0],
        ];

        PDAG::from_row_to_col_vecvec(dense);
    }

    #[test]
    pub fn lenient_with_undirected() {
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 2, 0], //
            vec![2, 0, 2],
            vec![0, 0, 0],
        ];

        PDAG::from_row_to_col_vecvec(dense);
    }

    #[test]
    pub fn neighbourhood_query_some_undirected() {
        // 0--2
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 2], //
            vec![0, 0],
        ];

        let cpdag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(cpdag.n_nodes, 2);

        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(0).iter().copied()),
            HashSet::from([1])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(1).iter().copied()),
            HashSet::from([0])
        );

        // 0 -> 1 -- 2
        // |  /
        // v v
        //  3
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1, 0, 1], //
            vec![0, 0, 2, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];

        let cpdag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(cpdag.n_nodes, 4);
        assert_eq!(
            HashSet::from_iter(cpdag.children_of(0).iter().copied()),
            HashSet::from([1, 3])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.possible_children_of(1).iter().copied()),
            HashSet::from([2, 3])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(2).iter().copied()),
            HashSet::from([1])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.parents_of(0).iter().copied()),
            HashSet::from([])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.parents_of(1).iter().copied()),
            HashSet::from([0])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.parents_of(2).iter().copied()),
            HashSet::new()
        );
        assert_eq!(
            HashSet::from_iter(cpdag.parents_of(3).iter().copied()),
            HashSet::from([0, 1])
        );

        // 0 -- 1 -- 2 -- 0
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 2, 0], //
            vec![0, 0, 2],
            vec![2, 0, 0],
        ];

        let cpdag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(cpdag.n_nodes, 3);

        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(0).iter().copied()),
            HashSet::from([1, 2])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(1).iter().copied()),
            HashSet::from([0, 2])
        );
        assert_eq!(
            HashSet::from_iter(cpdag.adjacent_undirected_of(2).iter().copied()),
            HashSet::from([0, 1])
        );
        assert_eq!(HashSet::from_iter(cpdag.children_of(1)), HashSet::new());
    }

    #[test]
    pub fn neighbourhood_query_directed_only() {
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1], //
            vec![0, 0],
        ];

        let dag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(dag.n_nodes, 2);

        assert_eq!(
            HashSet::from_iter(dag.children_of(0).iter().copied()),
            HashSet::from([1])
        );
        assert_eq!(
            HashSet::from_iter(dag.children_of(1).iter().copied()),
            HashSet::new()
        );

        // 0 -> 1 -> 2
        // |  /
        // v v
        //  3
        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1, 0, 1], //
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];

        let dag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(dag.n_nodes, 4);
        assert_eq!(
            HashSet::from_iter(dag.children_of(0).iter().copied()),
            HashSet::from([1, 3])
        );
        assert_eq!(
            HashSet::from_iter(dag.children_of(1).iter().copied()),
            HashSet::from([2, 3])
        );
        assert_eq!(
            HashSet::from_iter(dag.children_of(2).iter().copied()),
            HashSet::from([])
        );
        assert_eq!(
            HashSet::from_iter(dag.parents_of(0).iter().copied()),
            HashSet::from([])
        );
        assert_eq!(
            HashSet::from_iter(dag.parents_of(1).iter().copied()),
            HashSet::from([0])
        );
        assert_eq!(
            HashSet::from_iter(dag.parents_of(2).iter().copied()),
            HashSet::from([1])
        );
        assert_eq!(
            HashSet::from_iter(dag.parents_of(3).iter().copied()),
            HashSet::from([0, 1])
        );

        let dense: Vec<Vec<i8>> = vec![
            vec![0, 1, 0], //
            vec![0, 0, 0],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_row_to_col_vecvec(dense);

        assert_eq!(dag.n_nodes, 3);

        assert_eq!(
            HashSet::from_iter(dag.children_of(0).iter().copied()),
            HashSet::from([1])
        );
        assert_eq!(
            HashSet::from_iter(dag.children_of(1).iter().copied()),
            HashSet::new()
        );
    }

    #[test]
    pub fn property_row_major_and_col_major_loading_equal() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for size in 3..40 {
            let edge_density = 0.5;
            let adjacency = PDAG::_random_pdag_vecvec(edge_density, size, &mut rng);

            // transpose the adjacency matrix
            let mut transpose_adjacency = vec![vec![0; size]; size];

            for (x, row) in transpose_adjacency.iter_mut().enumerate() {
                for (y, entry) in row.iter_mut().enumerate() {
                    *entry = adjacency[y][x];
                }
            }

            // construct the DAG from the original and transposed adjacency matrix
            let row_major_dag = PDAG::from_row_to_col_vecvec(adjacency);
            let col_major_dag = PDAG::from_col_to_row_vecvec(transpose_adjacency);

            // the final representations of the DAG should be 100% equal
            assert_eq!(row_major_dag, col_major_dag);
        }
    }

    #[test]
    pub fn random_pdags_no_failure_load() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 1..40 {
            PDAG::random_pdag(0.5, n, &mut rng);
        }
    }

    #[test]
    pub fn property_random_dags_acyclic() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 1..40 {
            PDAG::random_dag(0.5, n, &mut rng);
        }
    }

    #[test]
    pub fn sorted_return_values() {
        let dense_matrices: Vec<Vec<Vec<i8>>> = vec![
            //
            vec![
                vec![0, 2, 0, 1], //
                vec![0, 0, 2, 1],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
            ],
            //
            vec![
                vec![0, 1, 1, 2, 1], //
                vec![0, 0, 0, 0, 1],
                vec![0, 0, 0, 0, 0],
                vec![0, 1, 0, 0, 1],
                vec![0, 0, 2, 0, 0],
            ],
            vec![
                vec![0, 0, 0, 0, 0], //
                vec![1, 0, 0, 0, 0],
                vec![1, 1, 0, 0, 0],
                vec![1, 2, 2, 0, 0],
                vec![1, 1, 0, 1, 0],
            ],
        ];

        for (i, dense) in dense_matrices.iter().enumerate() {
            let cpdag = PDAG::from_row_to_col_vecvec(dense.clone());

            for n in 0..cpdag.n_nodes {
                let mut children = cpdag.children_of(n).to_vec();
                children.sort();
                assert_eq!(
                    children,
                    cpdag.children_of(n).to_vec(),
                    "cpdag {}: children of node {} are not sorted",
                    i,
                    n
                );
            }

            for n in 0..cpdag.n_nodes {
                let mut children = cpdag.parents_of(n).to_vec();
                children.sort();
                assert_eq!(
                    children,
                    cpdag.parents_of(n).to_vec(),
                    "cpdag {}: parents of node {} are not sorted",
                    i,
                    n
                );
            }
            for n in 0..cpdag.n_nodes {
                let mut siblings = cpdag.adjacent_undirected_of(n).to_vec();
                siblings.sort();
                assert_eq!(
                    siblings,
                    cpdag.adjacent_undirected_of(n).to_vec(),
                    "cpdag {}: parents of node {} are not sorted",
                    i,
                    n
                );
            }
        }
    }

    #[test]
    #[should_panic]
    fn cyclic_dag_fail_0() {
        let g_truth = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 1, 0],
        ];
        let _ = PDAG::from_row_to_col_vecvec(g_truth);
    }

    #[test]
    #[should_panic]
    fn cyclic_dag_fail_1() {
        let g_truth = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![1, 0, 0],
        ];
        let _ = PDAG::from_row_to_col_vecvec(g_truth);
    }

    #[test]
    #[should_panic]
    fn cyclic_dag_fail_2() {
        let g_truth = vec![
            vec![0, 1, 1], //
            vec![0, 0, 1],
            vec![1, 1, 0],
        ];
        let _ = PDAG::from_row_to_col_vecvec(g_truth);
    }
}
