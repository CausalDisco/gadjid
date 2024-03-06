// SPDX-License-Identifier: MPL-2.0
#![warn(missing_docs)]
//! gadjid -  Graph Adjustment Identification Distance library
mod ascending_list_utils;
mod graph_loading;
pub mod graph_operations;
mod partially_directed_acyclic_graph;

pub use graph_loading::constructor::EdgelistIterator;
pub use partially_directed_acyclic_graph::LoadError;
pub use partially_directed_acyclic_graph::PDAG;

#[cfg(test)]
#[allow(non_snake_case)]
pub(crate) mod test {
    use rand::{Rng, SeedableRng};
    use rustc_hash::FxHashSet;
    use std::hash::{Hash, Hasher};

    use crate::{
        graph_operations::{self, gensearch, get_nam, ruletables},
        PDAG,
    };

    pub fn load_pdag_from_mtx(full_path: &str) -> PDAG {
        // read the mtx file
        let mtx = std::fs::read_to_string(full_path).unwrap();

        let mut lines = mtx.lines();

        // skipping first line of mtx format that give metadata like dimensions
        lines.next();

        let dims = lines
            .next()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<&str>>();
        let rows = dims[0].parse::<usize>().unwrap();
        let cols = dims[1].parse::<usize>().unwrap();

        // allocate matrix for the adjacency matrix
        let mut adj = vec![vec![0; cols]; rows];

        // and fill it with the edges from the mtx file
        for line in lines {
            let mut iter = line.split_whitespace();

            let i = iter.next().unwrap().parse::<usize>().unwrap();
            let j = iter.next().unwrap().parse::<usize>().unwrap();
            let edgetype = iter.next();
            match edgetype {
                // in DAG format, there are only tuples of coordinates, no edge types
                None => {
                    adj[i - 1][j - 1] = 1;
                }
                // in CPDAG format, there are is a third entry for the edge type
                Some(s) => {
                    let edge_code = s.parse::<i8>().unwrap();
                    adj[i - 1][j - 1] = edge_code;
                }
            }
        }

        PDAG::from_vecvec(adj)
    }

    /// Takes two names, like `g_true="DAG1"` and `g_guess="DAG2"` and returns a Testcase, loading from the corresponding `../testgraphs/{g_true}.mtx` files
    fn test(g_true_name: &str, g_guess_name: &str) -> Testcase {
        // get the root of the project
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        // get the parent directory of the project
        let root_parent = std::path::Path::new(&root).parent().unwrap();
        // get the child dir "testgraphs"
        let testgraphs = root_parent.join("testgraphs");

        // load the true and guess graphs
        let g_true = load_pdag_from_mtx(
            testgraphs
                .join(format!("{}.mtx", g_true_name))
                .to_str()
                .unwrap(),
        );
        let g_guess = load_pdag_from_mtx(
            testgraphs
                .join(format!("{}.mtx", g_guess_name))
                .to_str()
                .unwrap(),
        );

        assert!(
            g_true.n_nodes == g_guess.n_nodes,
            "Graphs have different number of nodes"
        );
        assert!(g_true.n_nodes >= 7, "graphs must have at least 7 nodes to run tests, we need (distinct) 5 T and 1 Y and at least 1 Z");

        // get deterministic seed by hashing the two graph names
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        g_true_name.hash(&mut hasher);
        g_guess_name.hash(&mut hasher);
        let seed = hasher.finish();

        // using rand_chacha to sample nodes with seed because it is reproducible across platforms
        // this is recommended mentioned by the rand crate docs on portability, see
        // https://rust-random.github.io/rand/rand/rngs/struct.SmallRng.html
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

        use rand::seq::SliceRandom;
        let mut indices = Vec::from_iter(0..g_true.n_nodes);
        indices.shuffle(&mut rng);

        // determining the single reponse node Y
        let y = indices[0];

        // determining the size of both the treatment set 'ts' and the random adjustment set 'random_adj'
        let ts_size = rng.gen_range(1..=g_guess.n_nodes as u32 - 2) as usize;
        let random_adj_size =
            rng.gen_range(1..=g_guess.n_nodes as u32 - (ts_size as u32) - 1) as usize;

        // getting the treatment nodes
        let ts = indices[1..ts_size + 1].to_vec();
        // getting the adjustment set nodes
        let random_adj = indices[1 + ts_size..1 + ts_size + random_adj_size as usize].to_vec();

        let possible_descendants = graph_operations::possible_descendants(&g_guess, ts.iter());
        let proper_ancestors = graph_operations::proper_ancestors(&g_guess, ts.iter(), [y].iter());

        // precomputing the adjustment sets for the NVA computation later:
        let empty_adj = FxHashSet::default();
        let pa_adj = gensearch(&g_guess, ruletables::Parents {}, ts.iter(), false);
        let anc_adj = gensearch(&g_guess, ruletables::Ancestors {}, ts.iter(), false);
        let nondesc_adj = {
            FxHashSet::from_iter((0..g_guess.n_nodes).filter(|x| !possible_descendants.contains(x)))
        };
        let oset_adj = {
            let t_descendants = gensearch(&g_guess, ruletables::Descendants {}, ts.iter(), false);
            crate::graph_operations::optimal_adjustment_set(&g_guess, &ts, &[y], &t_descendants)
        };

        Testcase {
            g_true: g_true_name.to_string(),
            g_guess: g_guess_name.to_string(),
            T: ts.to_vec(),
            ancestor_aid: graph_operations::ancestor_aid(&g_true, &g_guess),
            oset_aid: graph_operations::oset_aid(&g_true, &g_guess),
            parent_aid: graph_operations::parent_aid(&g_true, &g_guess),
            shd: graph_operations::shd(&g_true, &g_guess),
            NAM: {
                let mut nam = Vec::from_iter(get_nam(&g_guess, &ts));
                nam.sort();
                nam
            },
            Y: y,
            proper_ancestors: {
                let mut p = Vec::from_iter(proper_ancestors);
                p.sort();
                p
            },
            possible_descendants: {
                let mut p = Vec::from_iter(possible_descendants);
                p.sort();
                p
            },
            oset: {
                let mut o = Vec::from_iter(oset_adj.clone());
                o.sort();
                o
            },
            random_adj: random_adj.clone(),
            parent_adjustment_NVA: {
                let (_, nva) = graph_operations::get_nam_nva(&g_true, &ts, pa_adj);
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
            ancestor_adjustment_NVA: {
                let (_, nva) = graph_operations::get_nam_nva(&g_true, &ts, anc_adj);
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
            nondescendant_adjustment_NVA: {
                let (_, nva) = graph_operations::get_nam_nva(&g_true, &ts, nondesc_adj);
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
            oset_adjustment_NVA: {
                let (_, nva) = graph_operations::get_nam_nva(&g_true, &ts, oset_adj);
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
            empty_adjustment_NVA: {
                let (_, nva) = graph_operations::get_nam_nva(&g_true, &ts, empty_adj);
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
            random_Z_adjustment_NVA: {
                let (_, nva) =
                    graph_operations::get_nam_nva(&g_true, &ts, random_adj.into_iter().collect());
                let mut nva = Vec::from_iter(nva);
                nva.sort();
                nva
            },
        }
    }

    /// Stores the result of loading the two graphs and computing various graph operations on them.
    #[derive(serde::Serialize)]
    pub struct Testcase {
        g_true: String,
        g_guess: String,
        ancestor_aid: (f64, usize),
        oset_aid: (f64, usize),
        parent_aid: (f64, usize),
        shd: (f64, usize),
        T: Vec<usize>,
        /// the nodes that are not amenable to adjustment-set identification from the set T g_true
        NAM: Vec<usize>,
        /// the single treatment node considered in the test
        Y: usize,
        /// the random adjustment set drawn from the remaining nodes not in T or y
        random_adj: Vec<usize>,
        /// the proper ancestors of y in g_guess, w.r.t. the set T
        proper_ancestors: Vec<usize>,
        /// the optimal adjustment set in g_guess, w.r.t. the set T and Y
        oset: Vec<usize>,
        /// the possible descendanT of g_guess, w.r.t. the set T
        possible_descendants: Vec<usize>,
        /// the NVA set in g_true for the parent adjustment for T based on g_guess
        parent_adjustment_NVA: Vec<usize>,
        /// the NVA set in g_true for the ancestor adjustment for T based on g_guess
        ancestor_adjustment_NVA: Vec<usize>,
        /// the NVA set in g_true for the non-descendant adjustment for T based on g_guess
        nondescendant_adjustment_NVA: Vec<usize>,
        /// the NVA set in g_true for the optimal adjustment for T based on g_guess
        oset_adjustment_NVA: Vec<usize>,
        /// the NVA set in g_true for the empty adjustment for T
        empty_adjustment_NVA: Vec<usize>,
        /// the NVA set in g_true for a random adjustment for T
        random_Z_adjustment_NVA: Vec<usize>,
    }

    #[test]
    fn create_and_compare_snapshots() {
        // loops through (1, 2), (2, 3), ..., (9, 10), (10, 1) and creates snapshots for each pair
        for (true_id, guess_id) in (1..=10).map(|x| (x, ((x + 1) % 11) + 1)) {
            insta::assert_yaml_snapshot!(
                format!("small-DAG{}-vs-DAG{}", true_id, guess_id),
                test(
                    &format!("200{:0>2}.DAG-10", true_id),
                    &format!("200{:0>2}.DAG-10", guess_id)
                )
            );
            insta::assert_yaml_snapshot!(
                format!("small-CPDAG{}-vs-CPDAG{}", true_id, guess_id),
                test(
                    &format!("200{:0>2}.CPDAG-10", true_id),
                    &format!("200{:0>2}.CPDAG-10", guess_id)
                )
            );
            insta::assert_yaml_snapshot!(
                format!("big-DAG{}-vs-DAG{}", true_id, guess_id),
                test(
                    &format!("100{:0>2}.DAG-100", true_id),
                    &format!("100{:0>2}.DAG-100", guess_id)
                )
            );
            insta::assert_yaml_snapshot!(
                format!("big-CPDAG{}-vs-CPDAG{}", true_id, guess_id),
                test(
                    &format!("100{:0>2}.CPDAG-100", true_id),
                    &format!("100{:0>2}.CPDAG-100", guess_id)
                )
            );
        }
    }
}
