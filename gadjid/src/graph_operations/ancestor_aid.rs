// SPDX-License-Identifier: MPL-2.0
//! Implements the Ancestor Adjustment Intervention Distance (Ancestor-AID) algorithm

use rayon::prelude::*;

use crate::{
    graph_operations::{
        gensearch,
        reachability::{get_pd_nam, get_pd_nam_nva},
    },
    PDAG,
};

/// Computes the ancestor adjustment intervention distance
/// between an estimated `guess` DAG or CPDAG and the true `truth` DAG or CPDAG
/// (a PDAG is used for internal representation, but every PDAG is assumed either a DAG or a CPDAG
///  currently distances between general PDAGs are not implemented)
/// Returns a tuple of (normalized error (in \[0,1]), total number of errors)
// This function largely overlaps with parent_aid in parent_aid.rs; differences ---highlighted--- below
pub fn ancestor_aid(truth: &PDAG, guess: &PDAG) -> (f64, usize) {
    assert!(
        guess.n_nodes == truth.n_nodes,
        "both graphs must contain the same number of nodes"
    );
    assert!(guess.n_nodes >= 2, "graph must contain at least 2 nodes");

    let verifier_mistakes_found = (0..guess.n_nodes)
        .into_par_iter()
        .map(|treatment| {
            // --- this function differs from parent_aid.rs only in the imports and from here

            // ancestor adjustment
            let ruletable = crate::graph_operations::ruletables::Ancestors {};
            let adjustment_set = gensearch(
                // gensearch yield_starting_vertices 'false' because Ancestors(T)\T is the adjustment set
                guess,
                ruletable,
                [treatment].iter(),
                false,
            );

            // claim that all possible descendants could be affected by the treatment
            let (claim_possible_effect, nam_in_guess) = get_pd_nam(guess, &[treatment]);
            // --- to here

            // now we take a look at the nodes in the true graph for which the adj.set. was not valid.
            let (t_poss_desc_in_truth, nam_in_true, nva_in_true) =
                get_pd_nam_nva(truth, &[treatment], &adjustment_set);

            let mut mistakes = 0;
            for y in 0..truth.n_nodes {
                if y == treatment {
                    continue; // this case is always correct
                }
                // if y is not claimed to be effect of t based on the guess graph
                if !claim_possible_effect.contains(&y) {
                    // but possibly a descendant of t in the truth graph.
                    if t_poss_desc_in_truth.contains(&y) {
                        // the ancestral order might be wrong, so
                        // we count a mistake
                        mistakes += 1;
                    }
                } else {
                    let y_nam_in_guess = nam_in_guess.contains(&y);
                    let y_nam_in_true = nam_in_true.contains(&y);

                    #[allow(clippy::if_same_then_else)]
                    // if they disagree on amenability:
                    if y_nam_in_guess != y_nam_in_true {
                        mistakes += 1;
                    }
                    // if we reach this point, (t,y) is either amenable or non-amenable in both graphs;
                    // now, if it is amenable but the adjustment set is not valid in the true graph (only in the guess graph)
                    else if !y_nam_in_true && nva_in_true.contains(&y) {
                        // we count a mistake
                        mistakes += 1;
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
mod test {
    use rand::SeedableRng;

    use crate::PDAG;

    use super::ancestor_aid;

    #[test]
    fn property_equal_dags_zero_distance() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);
        for n in 2..40 {
            for _rep in 0..2 {
                let dag = PDAG::random_dag(0.5, n, &mut rng);
                assert_eq!(
                    (0.0, 0),
                    ancestor_aid(&dag, &dag),
                    "ancestor_aid between same dags of size {n} must be zero, dag: {}",
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
                ancestor_aid(&dag1, &dag2);
            }
        }
    }
}
