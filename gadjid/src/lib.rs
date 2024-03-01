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
mod test {
    use std::hash::{Hash, Hasher};

    use rand::{Rng, SeedableRng};
    use rustc_hash::FxHashSet;

    use crate::{
        graph_operations::{self, parent_aid},
        PDAG,
    };

    fn load_pdag_from_mtx(full_path: &str) -> PDAG {
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

            // for undirected edges in CPDAGs:
            // let edge_type = iter.next().unwrap().parse::<i8>().unwrap();

            adj[i - 1][j - 1] = 1;
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
            &testgraphs
                .join(format!("{}.mtx", g_true_name))
                .to_str()
                .unwrap(),
        );
        let g_guess = load_pdag_from_mtx(
            &testgraphs
                .join(format!("{}.mtx", g_guess_name))
                .to_str()
                .unwrap(),
        );

        // get deterministic seed by hashing the two graph names
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        g_true_name.hash(&mut hasher);
        g_guess_name.hash(&mut hasher);
        let seed = hasher.finish();

        // using rand_chacha to sample nodes with seed because it is reproducible across platforms
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

        let t = rng.gen_range(0..g_true.n_nodes as usize);
        let mut y = rng.gen_range(0..g_true.n_nodes as usize);
        if y == t {
            y = (y + 1) % g_true.n_nodes as usize;
        }
        let z = g_guess.parents_of(t);

        let pa_true_T = g_true.parents_of(t as usize).iter().copied().collect();
        let an_true_T = graph_operations::ancestors(&g_true, [t].iter());
        let ch_true_T = graph_operations::children(&g_true, [t].iter());
        let de_true_T = graph_operations::descendants(&g_true, [t].iter());
        let poss_de_true_T = graph_operations::possible_descendants(&g_true, [t].iter());
        let prop_an_true_Y = graph_operations::proper_ancestors(&g_true, [t].iter(), [y].iter());

        let (nam, nva) =
            graph_operations::get_nam_nva(&g_true, &[t], FxHashSet::from_iter(z.iter().copied()));

        let shd = graph_operations::shd(&g_true, &g_guess);

        let sid = graph_operations::sid(&g_true, &g_guess).unwrap();

        let oset_aid = graph_operations::oset_aid(&g_true, &g_guess);

        let parent_aid = parent_aid(&g_true, &g_guess);

        let ancestor_aid = graph_operations::ancestor_aid(&g_true, &g_guess);

        Testcase {
            g_true: g_true_name.to_string(),
            g_guess: g_guess_name.to_string(),
            t,
            y,
            z: z.to_vec(),
            pa_true_T,
            an_true_T,
            ch_true_T,
            de_true_T,
            poss_de_true_T,
            prop_an_true_Y,
            nam,
            nva,
            shd,
            sid,
            oset_aid,
            parent_aid,
            ancestor_aid,
        }
    }

    /// Stores the result of loading the two graphs and computing various graph operations on them.
    #[derive(serde::Serialize)]
    pub struct Testcase {
        g_guess: String,
        g_true: String,
        t: usize,
        y: usize,
        z: Vec<usize>,
        pa_true_T: FxHashSet<usize>,
        an_true_T: FxHashSet<usize>,
        ch_true_T: FxHashSet<usize>,
        de_true_T: FxHashSet<usize>,
        poss_de_true_T: FxHashSet<usize>,
        prop_an_true_Y: FxHashSet<usize>,
        nam: FxHashSet<usize>,
        nva: FxHashSet<usize>,
        shd: (f64, usize),
        sid: (f64, usize),
        oset_aid: (f64, usize),
        parent_aid: (f64, usize),
        ancestor_aid: (f64, usize),
    }

    #[test]
    fn snapshot_tests() {
        // will add CPDAGs, too
        insta::assert_yaml_snapshot!("DAG1-vs-DAG2", test("10001.DAG-100", "10002.DAG-100"));
        insta::assert_yaml_snapshot!("DAG3-vs-DAG4", test("10003.DAG-100", "10004.DAG-100"));
        insta::assert_yaml_snapshot!("DAG5-vs-DAG6", test("10005.DAG-100", "10006.DAG-100"));
        insta::assert_yaml_snapshot!("DAG7-vs-DAG8", test("10007.DAG-100", "10008.DAG-100"));
        insta::assert_yaml_snapshot!("DAG9-vs-DAG10", test("10009.DAG-100", "10010.DAG-100"));
    }
}
