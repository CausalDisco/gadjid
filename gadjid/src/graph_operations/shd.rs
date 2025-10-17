// SPDX-License-Identifier: MPL-2.0
//! Implements structural hamming distance

use rayon::prelude::*;

use crate::{
    ascending_list_utils::{ascending_lists_set_symmetric_difference, ascending_lists_set_union},
    PDAG,
};

/// Generalized Structural hamming distance between two simple graphs. Returns a tuple of
/// (normalized error (in \[0,1]), total number of errors)
// this can be generalised to different graphs with different types of edges
// using generics, as we don't care about incoming/outgoing/parent/child semantics here
pub fn shd(g_truth: &PDAG, g_guess: &PDAG) -> (f64, usize) {
    assert_eq!(g_truth.n_nodes, g_guess.n_nodes, "graph size mismatch");
    if g_truth.n_nodes == 1 {
        return (0f64, 0);
    }

    crate::rayon::build_global();

    let dist = (0..g_truth.n_nodes)
        .into_par_iter()
        .map(|node| {
            let truth_children = g_truth
                .children_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);
            let truth_parents = g_truth
                .parents_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);
            let truth_undirected = g_truth
                .adjacent_undirected_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);

            let guess_children = g_guess
                .children_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);
            let guess_parents = g_guess
                .parents_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);
            let guess_undirected = g_guess
                .adjacent_undirected_of(node)
                .iter()
                .copied()
                .filter(|e| e < &node);

            let children_symdif =
                ascending_lists_set_symmetric_difference(truth_children, guess_children);
            let parents_symdif =
                ascending_lists_set_symmetric_difference(truth_parents, guess_parents);
            let undirected_symdif =
                ascending_lists_set_symmetric_difference(truth_undirected, guess_undirected);

            let distinct_children_and_parents =
                ascending_lists_set_union(children_symdif.into_iter(), parents_symdif.into_iter());
            let union = ascending_lists_set_union(
                distinct_children_and_parents.into_iter(),
                undirected_symdif.into_iter(),
            );
            union.len()
        })
        .sum();
    // there are |V|*(|V|-1)/2  unordered pairs of nodes
    let comparisons = g_truth.n_nodes * (g_truth.n_nodes - 1) / 2;
    (dist as f64 / comparisons as f64, dist)
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;

    use crate::PDAG;

    use super::shd;

    /// Structural hamming distance between two adjacency matrices, ignores diagonal. Only used for the tests.
    /// This function works directly on the adjacency matrix representation.
    /// If applied on PDAG, either both or none of the matrices should double code the undirected edges as 2 2's.
    /// This is in contrast to the crate-public SHD defined for structs that `impl TraverseGraph`,
    /// in which case we are lenient with double-coding because the adjacency matrix will be
    /// compressed to the same final representation.
    fn shd_from_adjacency(g_truth: &[Vec<i8>], g_guess: &[Vec<i8>]) -> (f64, usize) {
        assert_eq!(
            g_truth.len(),
            g_guess.len(),
            "matrix outer dimension mismatch"
        );

        if g_truth.len() == 1 {
            return (0f64, 0);
        }

        let dim = g_truth.len();
        let mut dist = 0;

        for i in 0..dim {
            assert_eq!(
                g_truth[i].len(),
                g_guess[i].len(),
                "matrix dimension mismatch at inner index {i}"
            );

            for j in (i + 1)..dim {
                // (i,j) walks through upper triangle
                let upper_triangle_match = g_truth[i][j] == g_guess[i][j];
                if !upper_triangle_match {
                    // if upper triangle doesn't match, we don't need to inspect lower triangle
                    dist += 1;
                } else {
                    // otherwise, we need to check if the lower triangle matches
                    let lower_triangle_match = g_truth[j][i] == g_guess[j][i];
                    if !lower_triangle_match {
                        dist += 1;
                    }
                }
            }
        }

        let comparisons = g_truth.len() * (g_truth.len() - 1) / 2;
        (dist as f64 / comparisons as f64, dist)
    }

    #[test]
    fn property_equal_dags_zero_distance() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 2..40 {
            let dag = PDAG::random_dag(0.5, n, &mut rng);
            assert_eq!(
                (0.0, 0),
                shd(&dag, &dag),
                "dist between same dags of size {n} must be zero"
            );
        }
    }

    #[test]
    fn structural_hamming_distance_dag() {
        let g_truth = vec![vec![0]];
        let g_guess = vec![vec![0]];

        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (0f64, 0));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (0f64, 0));

        let g_truth = vec![
            vec![0, 1], //
            vec![0, 0],
        ];
        let g_guess = vec![
            vec![0, 0], //
            vec![0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );
        assert_eq!(shd(&d_truth, &d_guess), (1f64, 1));

        // 0 -> 1
        let g_truth = vec![
            vec![0, 1], //
            vec![0, 0],
        ];
        // 0 <- 1
        let g_guess = vec![
            vec![0, 0], //
            vec![1, 0],
        ];

        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (1f64, 1));

        let g_truth = vec![
            vec![0, 1, 1], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        let g_guess = vec![
            vec![0, 1, 1], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (0f64, 0));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (0f64, 0));

        let g_truth = vec![
            vec![0, 1, 0, 1], //
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];
        let g_guess = vec![
            vec![0, 1, 0, 1], //
            vec![0, 0, 1, 0],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64 / 6f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (1f64 / 6f64, 1));
    }

    #[test]
    fn structural_hamming_distance_pdag() {
        let g_truth = vec![vec![0]];
        let g_guess = vec![vec![0]];

        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (0f64, 0));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (0f64, 0));

        let g_truth = vec![
            vec![0, 2], //
            vec![0, 0],
        ];
        let g_guess = vec![
            vec![0, 0], //
            vec![0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );
        assert_eq!(shd(&d_truth, &d_guess), (1f64, 1));

        // 0 -> 1
        let g_truth = vec![
            vec![0, 2], //
            vec![0, 0],
        ];
        // 0 <- 1
        let g_guess = vec![
            vec![0, 0], //
            vec![1, 0],
        ];

        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (1f64, 1));

        let g_truth = vec![
            vec![0, 2, 1], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        let g_guess = vec![
            vec![0, 2, 1], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (0f64, 0));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );
        assert_eq!(shd(&d_truth, &d_guess), (0f64, 0));

        let g_truth = vec![
            vec![0, 2, 1], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        let g_guess = vec![
            vec![0, 1, 2], //
            vec![0, 0, 2],
            vec![0, 0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64, 3));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );
        assert_eq!(shd(&d_truth, &d_guess), (1f64, 3));

        let g_truth = vec![
            vec![0, 2, 0, 1], //
            vec![0, 0, 1, 1],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];
        let g_guess = vec![
            vec![0, 2, 0, 1], //
            vec![0, 0, 1, 0],
            vec![0, 0, 0, 0],
            vec![0, 2, 0, 0],
        ];
        assert_eq!(shd_from_adjacency(&g_truth, &g_guess), (1f64 / 6f64, 1));
        let (d_truth, d_guess) = (
            PDAG::from_row_to_column_vecvec(g_truth),
            PDAG::from_row_to_column_vecvec(g_guess),
        );

        assert_eq!(shd(&d_truth, &d_guess), (1f64 / 6f64, 1));
    }
}
