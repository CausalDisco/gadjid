// SPDX-License-Identifier: MPL-2.0
//! Implements the Optimal Adjustment Intervention Distance (Oset-AID) algorithm

use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{
    graph_operations::{
        get_d_pd_nam, get_invalidly_un_blocked, get_parents, get_pd_nam, get_proper_ancestors,
    },
    PDAG,
};

/// This oset function takes in a precomputed t_descendants set.
/// Returns the optimal adjustment set of the provided treatments.
pub fn optimal_adjustment_set_given_descendants(
    dag: &PDAG,
    treatments: &[usize],
    responses: &[usize],
    t_descendants: &FxHashSet<usize>,
) -> FxHashSet<usize> {
    let response_ancestors = get_proper_ancestors(dag, treatments.iter(), responses.iter());
    let response_and_anc_hash = FxHashSet::from_iter(response_ancestors);
    let causal_nodes = response_and_anc_hash.intersection(t_descendants);
    let causal_nodes_parents = get_parents(dag, causal_nodes);
    FxHashSet::from_iter(causal_nodes_parents.difference(t_descendants).copied())
}

/// Computes the oset adjustment intervention distance
/// between an estimated `guess` DAG or CPDAG and the true `truth` DAG or CPDAG
/// (a PDAG is used for internal representation, but every PDAG is assumed either a DAG or a CPDAG
///  currently distances between general PDAGs are not implemented)
/// Returns a tuple of (normalized error (in \[0,1]), total number of errors)
pub fn oset_aid(truth: &PDAG, guess: &PDAG) -> (f64, usize) {
    assert!(
        guess.n_nodes == truth.n_nodes,
        "both graphs must contain the same number of nodes"
    );
    assert!(guess.n_nodes >= 2, "graph must contain at least 2 nodes");

    let _ = rayon::ThreadPoolBuilder::new().num_threads(num_cpus::get_physical()).build_global();

    let verifier_mistakes_found = (0..guess.n_nodes)
        .into_par_iter()
        .map(|treatment| {
            // precomputed once for each T because we use it for the optimal adjustment set.
            let (t_desc_in_guess, claim_possible_effect, nam_in_guess) =
                get_d_pd_nam(guess, &[treatment]);

            let (t_poss_desc_in_truth, nam_in_true) = get_pd_nam(truth, &[treatment]);

            let mut mistakes = 0;
            for y in 0..guess.n_nodes {
                if y == treatment {
                    continue; // this case is always correct
                }
                // if y is not claimed to be effect of t based on the guess graph
                if !claim_possible_effect.contains(&y) {
                    // but possibly a descendant of t in the truth graph.
                    if t_poss_desc_in_truth.contains(&y) {
                        // the causal order might be wrong, so
                        // we count a mistake
                        mistakes += 1;
                    }
                } else {
                    let y_nam_in_guess = nam_in_guess.contains(&y);
                    let y_nam_in_true = nam_in_true.contains(&y);

                    // if they disagree on amenability:
                    if y_nam_in_guess != y_nam_in_true {
                        mistakes += 1;
                    }
                    // if they agree on amenability and y is amenable, we need to find the adjustment set
                    else if !y_nam_in_guess {
                        // this oset function uses the precomputed t_desc_in_guess
                        let o_set_adjustment = optimal_adjustment_set_given_descendants(
                            guess,
                            &[treatment],
                            &[y],
                            &t_desc_in_guess,
                        );

                        // if the o-set from the guess graph is not valid in the truth graph (by blocking too much or too little)
                        if get_invalidly_un_blocked(truth, &[treatment], &o_set_adjustment)
                            .contains(&y)
                        {
                            // we count a mistake
                            mistakes += 1;
                        }
                    }
                }
            }

            mistakes
        })
        .sum();

    let n = guess.n_nodes;
    let comparisons = n * n - n;
    (
        verifier_mistakes_found as f64 / comparisons as f64,
        verifier_mistakes_found,
    )
}

#[cfg(test)]
pub fn optimal_adjustment_set(
    dag: &PDAG,
    treatments: &[usize],
    responses: &[usize],
) -> FxHashSet<usize> {
    let t_descendants = crate::graph_operations::get_descendants(dag, treatments.iter());
    optimal_adjustment_set_given_descendants(dag, treatments, responses, &t_descendants)
}

#[cfg(test)]
mod test {
    use rand::SeedableRng;
    use rustc_hash::FxHashSet;

    use crate::PDAG;

    use super::{optimal_adjustment_set, oset_aid};

    #[test]
    fn property_equal_dags_zero_distance() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 2..40 {
            for _rep in 0..2 {
                let dag = PDAG::random_dag(0.5, n, &mut rng);
                assert_eq!(
                    (0.0, 0),
                    oset_aid(&dag, &dag),
                    "oset_aid between same dags of size {n} must be zero, dag: {}",
                    dag
                );
            }
        }
    }

    #[test]
    #[ignore]
    fn random_inputs_no_crash() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 2..40 {
            for _rep in 0..2 {
                let dag1 = PDAG::random_dag(1.0, n, &mut rng);
                let dag2 = PDAG::random_dag(1.0, n, &mut rng);
                oset_aid(&dag1, &dag2);
            }
        }
    }

    #[test]
    fn o_set() {
        // 0 -> 1 --> 2 ---> 3 <----7
        //      |     |      |
        //      v     v      v
        //      4 <-- 5 <--- 6

        let v_dag = vec![
            vec![0, 1, 0, 0, 0, 0, 0, 0], //
            vec![0, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[1], &[5])
        );
        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[0, 2], &[4])
        );
        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[0, 2], &[6])
        );
        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[1], &[6])
        );
        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[2], &[5])
        );
        assert_eq!(
            FxHashSet::from_iter([7]),
            optimal_adjustment_set(&dag, &[2], &[6])
        );
        assert_eq!(
            FxHashSet::from_iter([2]),
            optimal_adjustment_set(&dag, &[7], &[5])
        );

        //      _-> 1 -_
        //     /        \
        //    /          \
        //   /            v
        // 0 <- 4 -> 5 -> 2 -> 3
        //   \                ^
        //    \              /
        //     v            /
        //      6 ------> 7

        let v_dag = vec![
            vec![0, 1, 0, 0, 0, 0, 1, 0], //
            vec![0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
        ];

        let dag = PDAG::from_row_to_column_vecvec(v_dag);

        assert_eq!(
            FxHashSet::from_iter([5]),
            FxHashSet::from_iter(optimal_adjustment_set(&dag, &[0], &[3]))
        );
        assert_eq!(
            FxHashSet::from_iter([1, 7]),
            FxHashSet::from_iter(optimal_adjustment_set(&dag, &[5], &[3]))
        );
    }
}
