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

        assert!(g_true.n_nodes == g_guess.n_nodes, "Graphs have different number of nodes");
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

        // sampling 5 ts without replacement
        let mut ts: [usize; 5]  = [0; 5];
        (0..5).for_each(|i| {
            let mut t = rng.gen_range(0u32..g_true.n_nodes as u32) as usize;
            while ts.contains(&t) {
                t = rng.gen_range(0u32..g_true.n_nodes as u32) as usize;
            }
            ts[i as usize] = t;
        });

        let y: usize = rng.gen_range(0u32..g_true.n_nodes as u32) as usize;

        let zs: Vec<Vec<usize>> = ts
            .iter()
            .map(|t| {
                

                // choosing a random adjustment set uniformly between 4 variants:
                // 1) the parents of t
                // 2) the ancestors of t
                // 3) the non-descendants of t
                // 4) a random set of size between 1 and |V|-6, disjoint from all T and Y
                match rng.gen_range(1u8..=4u8) {
                    1 => g_guess.parents_of(*t).to_vec(),
                    2 => {
                        // ancestor adjustment
                        let ruletable =
                        crate::graph_operations::ruletables::ancestors::AncestorsRuletable {};
                        let ancestor_adjustment = crate::graph_operations::gensearch::gensearch(
                            &g_guess,
                            ruletable,
                            [*t].iter(),
                            // yield_starting_vertices 'false' because Ancestors(T)\T is the adjustment set
                            false,
                        )
                        .iter()
                        .copied()
                        .collect::<Vec<usize>>();
                        let mut ancestor_adjustment = Vec::from_iter(ancestor_adjustment);
                        ancestor_adjustment.sort();
                        ancestor_adjustment
                    }
                    3 => {
                        // the non-descendants, which are the complement of the possible descendants
                        let possdesc =
                            graph_operations::possible_descendants(&g_guess, [*t].iter());
                        (0..g_guess.n_nodes)
                            .filter(|x| !possdesc.contains(x))
                            .collect::<Vec<usize>>()
                    }
                    4 => {
                        // fully random adjustment set of size between 1 and |V|-6
                        let adj_size =
                            rng.gen_range(1u32..=g_guess.n_nodes as u32 - 6u32) as usize;

                        // sampling zs without replacement from the set of all nodes except y and the ts
                        let mut adj_set: Vec<usize> = vec![];
                        (0..=adj_size).for_each(|_| {
                            let mut sample = rng.gen_range(0u32..g_true.n_nodes as u32) as usize;
                            while adj_set.contains(&sample) || ts.contains(&sample) || y == sample {
                                sample = rng.gen_range(0u32..g_true.n_nodes as u32) as usize;
                            }
                            adj_set.push(sample);
                        });
                        adj_set
                    }
                    _ => unreachable!("num is in {{1, 2, 3, 4}}"),
                }
            })
            .collect();

        // parents_of returns a slice, (defined .iter() order), so we don't need to stabilize with sort.
        let pa_true_1st_T = g_true.parents_of(ts[0]).to_vec();

        // below, we sort results because the order of the elements in the hashsets is not defined and we want fully matching snapshots
        let mut an_true_1st_T: Vec<usize> = graph_operations::ancestors(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        an_true_1st_T.sort();
        let mut ch_true_1st_T: Vec<usize> = graph_operations::children(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        ch_true_1st_T.sort();
        let mut de_true_1st_T: Vec<usize> = graph_operations::descendants(&g_true, [ts[0]].iter())
            .iter()
            .copied()
            .collect();
        de_true_1st_T.sort();
        let mut poss_de_true_1st_T: Vec<usize> =
            graph_operations::possible_descendants(&g_true, [ts[0]].iter())
                .iter()
                .copied()
                .collect();
        poss_de_true_1st_T.sort();
        let mut prop_an_true_1st_T_and_1st_Y: Vec<usize> =
            graph_operations::proper_ancestors(&g_true, [ts[0]].iter(), [y].iter())
                .iter()
                .copied()
                .collect();
        prop_an_true_1st_T_and_1st_Y.sort();

        // for each we pair each t from ts with the random adjustment set z from zs, and compute the NAM and NVA sets.
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
                // we have to sort as we don't know the order of the elements in the sets after .collect()
                nam.sort();
                nva.sort();
                (nam, nva)
            })
            .unzip();

        let ancestor_aid = graph_operations::ancestor_aid(&g_true, &g_guess);
        let oset_aid = graph_operations::oset_aid(&g_true, &g_guess);
        let parent_aid = parent_aid(&g_true, &g_guess);
        let shd = graph_operations::shd(&g_true, &g_guess);

        Testcase {
            g_true: g_true_name.to_string(),
            g_guess: g_guess_name.to_string(),
            ts: ts.to_vec(),
            y,
            zs,
            pa_true_1st_T,
            an_true_1st_T,
            ch_true_1st_T,
            de_true_1st_T,
            poss_de_true_1st_T,
            prop_an_true_1st_T_and_1st_Y,
            nams,
            nvas,
            ancestor_aid,
            oset_aid,
            parent_aid,
            shd,
        }
    }

    /// Stores the result of loading the two graphs and computing various graph operations on them.
    #[derive(serde::Serialize)]
    pub struct Testcase {
        g_true: String,
        g_guess: String,
        ts: Vec<usize>,
        y: usize,
        zs: Vec<Vec<usize>>,
        nams: Vec<Vec<usize>>,
        nvas: Vec<Vec<usize>>,
        pa_true_1st_T: Vec<usize>,
        an_true_1st_T: Vec<usize>,
        ch_true_1st_T: Vec<usize>,
        de_true_1st_T: Vec<usize>,
        poss_de_true_1st_T: Vec<usize>,
        prop_an_true_1st_T_and_1st_Y: Vec<usize>,
        ancestor_aid: (f64, usize),
        oset_aid: (f64, usize),
        parent_aid: (f64, usize),
        shd: (f64, usize),
    }

    #[test]
    fn small_dag_snapshot() {
        for (left, right) in (1..=5).map(|x| (2 * x - 1, 2 * x)) {
            insta::assert_yaml_snapshot!(
                format!("small-DAG{}-vs-DAG{}", left, right),
                test(
                    &format!("200{:0>2}.DAG-10", left),
                    &format!("200{:0>2}.DAG-10", right)
                )
            );
        }
    }

    #[test]
    fn small_cpdag_snapshot() {
        for (left, right) in (1..=5).map(|x| (2 * x - 1, 2 * x)) {
            insta::assert_yaml_snapshot!(
                format!("small-CPDAG{}-vs-CPDAG{}", left, right),
                test(
                    &format!("200{:0>2}.CPDAG-10", left),
                    &format!("200{:0>2}.CPDAG-10", right)
                )
            );
        }
    }

    #[test]
    fn big_dag_snapshot() {
        for (left, right) in (1..=5).map(|x| (2 * x - 1, 2 * x)) {
            insta::assert_yaml_snapshot!(
                format!("big-DAG{}-vs-DAG{}", left, right),
                test(
                    &format!("100{:0>2}.DAG-100", left),
                    &format!("100{:0>2}.DAG-100", right)
                )
            );
        }
    }

    #[test]
    fn big_cpdag_snapshot() {
        for (left, right) in (1..=5).map(|x| (2 * x - 1, 2 * x)) {
            insta::assert_yaml_snapshot!(
                format!("big-CPDAG{}-vs-CPDAG{}", left, right),
                test(
                    &format!("100{:0>2}.CPDAG-100", left),
                    &format!("100{:0>2}.CPDAG-100", right)
                )
            );
        }
    }
}
