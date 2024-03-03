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
    use std::hash::{Hash, Hasher};

    use rand::{Rng, SeedableRng};
    use rustc_hash::FxHashSet;

    use crate::{
        graph_operations::{self, parent_aid},
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
            match edgetype{
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

        let ts: Vec<usize> = (0..5).map(|_| rng.gen_range(0..g_true.n_nodes)).collect();
        let ys: Vec<usize> = (0..5).map(|_| rng.gen_range(0..g_true.n_nodes)).collect();
        let zs: Vec<Vec<usize>> = ts.iter().map(|t| g_guess.parents_of(*t).to_vec()).collect();

        // below, we sort results because the order of the elements in the sets is not deterministic and we want matching snapshots
        let pa_true_T = g_true.parents_of(ts[0] as usize).to_vec();
        let mut an_true_T: Vec<usize> = graph_operations::ancestors(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        an_true_T.sort();
        let mut ch_true_T: Vec<usize> = graph_operations::children(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        ch_true_T.sort();
        let mut de_true_T: Vec<usize> = graph_operations::descendants(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        de_true_T.sort();
        let mut poss_de_true_T: Vec<usize> =
            graph_operations::possible_descendants(&g_true, [ts[0]].iter())
                .iter()
                .copied()
                .collect();
        poss_de_true_T.sort();
        let mut prop_an_true_Y: Vec<usize> =
            graph_operations::proper_ancestors(&g_true, [ts[0]].iter(), [ys[0]].iter())
                .iter()
                .copied()
                .collect();
        prop_an_true_Y.sort();

        let (nams, nvas): (Vec<Vec<usize>>, Vec<Vec<usize>>) = ts
            .iter()
            .zip(zs.iter())
            .map(|(t, z)| {
                let (nam, nva) = graph_operations::get_nam_nva(
                    &g_true,
                    &[*t],
                    FxHashSet::from_iter(z.iter().copied()),
                );
                let mut nam: Vec<usize> = nam.iter().copied().collect();
                let mut nva: Vec<usize> = nva.iter().copied().collect();
                nam.sort();
                nva.sort();
                (nam, nva)
            })
            .unzip();

        let shd = graph_operations::shd(&g_true, &g_guess);

        let oset_aid = graph_operations::oset_aid(&g_true, &g_guess);

        let parent_aid = parent_aid(&g_true, &g_guess);

        let ancestor_aid = graph_operations::ancestor_aid(&g_true, &g_guess);

        Testcase {
            g_true: g_true_name.to_string(),
            g_guess: g_guess_name.to_string(),
            ts,
            ys,
            zs,
            pa_true_1st_T: pa_true_T,
            an_true_1st_T: an_true_T,
            ch_true_1st_T: ch_true_T,
            de_true_1st_T: de_true_T,
            poss_de_true_1st_T: poss_de_true_T,
            prop_an_true_1st_T_and_1st_Y: prop_an_true_Y,
            nams,
            nvas,
            shd,
            oset_aid,
            parent_aid,
            ancestor_aid,
        }
    }

    /// Stores the result of loading the two graphs and computing various graph operations on them.
    #[derive(serde::Serialize)]
    pub struct Testcase {
        g_true: String,
        g_guess: String,
        ts: Vec<usize>,
        ys: Vec<usize>,
        zs: Vec<Vec<usize>>,
        nams: Vec<Vec<usize>>,
        nvas: Vec<Vec<usize>>,
        pa_true_1st_T: Vec<usize>,
        an_true_1st_T: Vec<usize>,
        ch_true_1st_T: Vec<usize>,
        de_true_1st_T: Vec<usize>,
        poss_de_true_1st_T: Vec<usize>,
        prop_an_true_1st_T_and_1st_Y: Vec<usize>,
        shd: (f64, usize),
        oset_aid: (f64, usize),
        parent_aid: (f64, usize),
        ancestor_aid: (f64, usize),
    }

    #[test]
    fn small_cpdag_snapshot() {
        insta::assert_yaml_snapshot!("small-CPDAG1-vs-CPDAG2", test("20001.CPDAG-10", "20002.CPDAG-10"));
        insta::assert_yaml_snapshot!("small-CPDAG3-vs-CPDAG4", test("20003.CPDAG-10", "20004.CPDAG-10"));
        insta::assert_yaml_snapshot!("small-CPDAG5-vs-CPDAG6", test("20005.CPDAG-10", "20006.CPDAG-10"));
        insta::assert_yaml_snapshot!("small-CPDAG7-vs-CPDAG8", test("20007.CPDAG-10", "20008.CPDAG-10"));
        insta::assert_yaml_snapshot!("small-CPDAG9-vs-CPDAG10", test("20009.CPDAG-10", "20010.CPDAG-10"));
    }
    #[test]
    fn small_dag_snapshot() {
        insta::assert_yaml_snapshot!("small-DAG1-vs-DAG2", test("20001.DAG-10", "20002.DAG-10"));
        insta::assert_yaml_snapshot!("small-DAG3-vs-DAG4", test("20003.DAG-10", "20004.DAG-10"));
        insta::assert_yaml_snapshot!("small-DAG5-vs-DAG6", test("20005.DAG-10", "20006.DAG-10"));
        insta::assert_yaml_snapshot!("small-DAG7-vs-DAG8", test("20007.DAG-10", "20008.DAG-10"));
        insta::assert_yaml_snapshot!("small-DAG9-vs-DAG10", test("20009.DAG-10", "20010.DAG-10"));
    }
    #[test]
    fn big_dag_snapshot() {
        insta::assert_yaml_snapshot!("big-DAG1-vs-DAG2", test("10001.DAG-100", "10002.DAG-100"));
        insta::assert_yaml_snapshot!("big-DAG3-vs-DAG4", test("10003.DAG-100", "10004.DAG-100"));
        insta::assert_yaml_snapshot!("big-DAG5-vs-DAG6", test("10005.DAG-100", "10006.DAG-100"));
        insta::assert_yaml_snapshot!("big-DAG7-vs-DAG8", test("10007.DAG-100", "10008.DAG-100"));
        insta::assert_yaml_snapshot!("big-DAG9-vs-DAG10", test("10009.DAG-100", "10010.DAG-100"));
    }

    #[test]
    fn big_cpdag_snapshot() {
        insta::assert_yaml_snapshot!("big-CPDAG1-vs-CPDAG2", test("10001.CPDAG-100", "10002.CPDAG-100"));
        insta::assert_yaml_snapshot!("big-CPDAG3-vs-CPDAG4", test("10003.CPDAG-100", "10004.CPDAG-100"));
        insta::assert_yaml_snapshot!("big-CPDAG5-vs-CPDAG6", test("10005.CPDAG-100", "10006.CPDAG-100"));
        insta::assert_yaml_snapshot!("big-CPDAG7-vs-CPDAG8", test("10007.CPDAG-100", "10008.CPDAG-100"));
        insta::assert_yaml_snapshot!("big-CPDAG9-vs-CPDAG10", test("10009.CPDAG-100", "10010.CPDAG-100"));
    }
}
