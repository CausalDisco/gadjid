// SPDX-License-Identifier: MPL-2.0
#![warn(missing_docs)]
//! gadjid -  Graph Adjustment Identification Distance library

mod ascending_list_utils;
mod graph_loading;
mod partially_directed_acyclic_graph;

pub mod graph_operations;

pub use graph_loading::constructor::EdgelistIterator;
pub use partially_directed_acyclic_graph::LoadError;
pub use partially_directed_acyclic_graph::PDAG;

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use rand::{Rng, SeedableRng};
    use rustc_hash::{FxHashSet, FxHasher};
    use std::hash::{Hash, Hasher};

    use crate::{
        graph_operations::{
            ancestor_aid, gensearch, get_nam, get_nam_nva, get_possible_descendants, get_proper_ancestors, optimal_adjustment_set, oset_aid, parent_aid, ruletables, shd
        },
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

    fn hashset_to_sorted_vec<V: std::cmp::Ord + Copy>(set: &FxHashSet<V>) -> Vec<V> {
        let mut vec = Vec::from_iter(set.iter().copied());
        vec.sort();
        vec
    }

    fn get_nva_sorted_vec(graph: &PDAG, t: &[usize], z: &FxHashSet<usize>) -> Vec<usize> {
        let (_, nva) = get_nam_nva(graph, t, z);
        hashset_to_sorted_vec(&nva)
    }

    /// Takes two names, like `g_true_name="DAG1"` and `g_guess_name="DAG2"` and returns a Testcase,
    /// loading from the corresponding `../testgraphs/{g_true_name}.mtx` files
    fn test(g_true_name: &str, g_guess_name: &str) -> Testcase {
        // anchors at parent directory of Cargo.toml
        let mut testgraphs = std::path::PathBuf::new();
        testgraphs.push("..");
        testgraphs.push("testgraphs");

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
        assert!(g_true.n_nodes >= 7,
             "graphs must have at least 7 nodes to run tests, we need distinct 5 T and 1 Y and at least 1 Z");

        // get deterministic seed by hashing the two graph names using the fx algorithm
        // (should not rely upon std::collections::hash_map::DefaultHasher::new() over releases
        // as its internal algorithm is not specified, cf. https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html)
        let mut hasher = FxHasher::default();
        g_true_name.hash(&mut hasher);
        g_guess_name.hash(&mut hasher);
        let seed = hasher.finish();

        // using rand_chacha to sample nodes with seed because it is reproducible across platforms
        // this is recommended by the rand crate docs on portability, see
        // https://rust-random.github.io/rand/rand/rngs/struct.SmallRng.html
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

        let mut indices = Vec::from_iter(0..g_true.n_nodes);
        rand::seq::SliceRandom::shuffle(&mut *indices, &mut rng);
        let indices = indices;

        // determining a single reponse node y
        let y = indices[0];

        // determining the size of both the treatment set 'ts' and the random adjustment set 'random_adj'
        let t_size = rng.gen_range(1..=(g_guess.n_nodes - 2) as u32) as usize;
        let random_z_size = rng.gen_range(1..=(g_guess.n_nodes - t_size - 1) as u32) as usize;

        // getting the treatment nodes
        let mut t = indices[1..t_size + 1].to_vec();
        t.sort();
        // getting the adjustment set nodes
        let mut random_z = indices[1 + t_size..1 + t_size + random_z_size as usize].to_vec();
        random_z.sort();

        let oset_for_t_onto_y_in_g_guess = {
            let t_descendants = gensearch(&g_guess, crate::graph_operations::ruletables::Descendants {}, t.iter(), false);
            optimal_adjustment_set(&g_guess, &t, &[y], &t_descendants)
        };

        Testcase {
            g_true: g_true_name.to_string(),
            g_guess: g_guess_name.to_string(),
            ancestor_aid: ancestor_aid(&g_true, &g_guess),
            oset_aid: oset_aid(&g_true, &g_guess),
            parent_aid: parent_aid(&g_true, &g_guess),
            shd: shd(&g_true, &g_guess),
            t: t.clone(),
            y,
            z: random_z.clone(),
            possible_descendants_of_t_in_g_guess: hashset_to_sorted_vec(&get_possible_descendants(
                &g_guess,
                t.iter(),
            )),
            not_amenable_in_g_guess_wrt_t: hashset_to_sorted_vec(&get_nam(&g_guess, &t)),
            proper_ancestors_of_y_in_g_guess_wrt_t: hashset_to_sorted_vec(&get_proper_ancestors(
                &g_guess,
                t.iter(),
                [y].iter(),
            )),
            oset_for_t_onto_y_in_g_guess: hashset_to_sorted_vec(
                &oset_for_t_onto_y_in_g_guess,
            ),
            not_validly_adjusted_for_in_g_guess_by_parents_of_t: get_nva_sorted_vec(
                &g_guess,
                &t,
                &gensearch(&g_guess, ruletables::Parents {}, t.iter(), false),
            ),
            not_validly_adjusted_for_in_g_guess_by_oset_for_t_onto_y: get_nva_sorted_vec(
                &g_guess,
                &t,
                &oset_for_t_onto_y_in_g_guess,
            ),
            not_validly_adjusted_for_in_g_guess_by_empty_set: get_nva_sorted_vec(
                &g_guess,
                &t,
                &FxHashSet::default(),
            ),
            not_validly_adjusted_for_in_g_guess_by_z: get_nva_sorted_vec(
                &g_guess,
                &t,
                &FxHashSet::from_iter(random_z),
            ),
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
        t: Vec<usize>,
        /// the single effect node considered in the test
        y: usize,
        /// the random adjustment set drawn from the remaining nodes not in t or y
        z: Vec<usize>,
        /// the possible descendants of t in g_guess
        possible_descendants_of_t_in_g_guess: Vec<usize>,
        /// the nodes onto which the effect of t is not amenable to adjustment-set identification in g_guess
        not_amenable_in_g_guess_wrt_t: Vec<usize>,
        /// the proper ancestors of y in g_guess, w.r.t. the set t
        proper_ancestors_of_y_in_g_guess_wrt_t: Vec<usize>,
        /// the optimal adjustment set in g_guess, w.r.t. the effect of t onto y
        oset_for_t_onto_y_in_g_guess: Vec<usize>,
        /// the set of nodes for which the effect of t onto those nodes is not validly adjusted for in g_guess
        /// by the parents of t in g_guess
        not_validly_adjusted_for_in_g_guess_by_parents_of_t: Vec<usize>,
        /// the set of nodes for which the effect of t onto those nodes is not validly adjusted for in g_guess
        /// by the optimal adjustment set for t onto y in g_guess
        not_validly_adjusted_for_in_g_guess_by_oset_for_t_onto_y: Vec<usize>,
        /// the set of nodes for which the effect of t onto those nodes is not validly adjusted for in g_guess
        /// by the empty set
        not_validly_adjusted_for_in_g_guess_by_empty_set: Vec<usize>,
        /// the set of nodes for which the effect of t onto those nodes is not validly adjusted for in g_guess
        /// by the (randomly drawn) set z
        not_validly_adjusted_for_in_g_guess_by_z: Vec<usize>,
    }

    #[test]
    fn insta_snapshots_small() {
        // loops through (1, 2), (2, 3), ..., (9, 10), (10, 1) and creates snapshots for each pair
        for (true_id, guess_id) in (1..=10).map(|x| (x, (x % 10) + 1)) {
            let g_true = &format!("200{:0>2}.DAG-10", true_id);
            let g_guess = &format!("200{:0>2}.DAG-10", guess_id);
            insta::assert_yaml_snapshot!(
                format!("small-DAG{:0>2}-vs-DAG{:0>2}", true_id, guess_id),
                test(g_true, g_guess)
            );
            let g_true = &format!("200{:0>2}.CPDAG-10", true_id);
            let g_guess = &format!("200{:0>2}.CPDAG-10", guess_id);
            insta::assert_yaml_snapshot!(
                format!("small-CPDAG{:0>2}-vs-CPDAG{:0>2}", true_id, guess_id),
                test(g_true, g_guess)
            );
        }
    }

    #[test]
    #[ignore]
    fn insta_snapshots_large() {
        // loops through (1, 2), (2, 3), ..., (9, 10), (10, 1) and creates snapshots for each pair
        for (true_id, guess_id) in (1..=10).map(|x| (x, (x % 10) + 1)) {
            let g_true = &format!("100{:0>2}.DAG-100", true_id);
            let g_guess = &format!("100{:0>2}.DAG-100", guess_id);
            insta::assert_yaml_snapshot!(
                format!("large-DAG{:0>2}-vs-DAG{:0>2}", true_id, guess_id),
                test(g_true, g_guess)
            );
            let g_true = &format!("100{:0>2}.CPDAG-100", true_id);
            let g_guess = &format!("100{:0>2}.CPDAG-100", guess_id);
            insta::assert_yaml_snapshot!(
                format!("large-CPDAG{:0>2}-vs-CPDAG{:0>2}", true_id, guess_id),
                test(g_true, g_guess)
            );
        }
    }
}
