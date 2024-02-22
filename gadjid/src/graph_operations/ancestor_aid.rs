// SPDX-License-Identifier: MPL-2.0
//! Implements the Ancestor Adjustment Intervention Distance (Ancestor-AID) algorithm

use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{
    graph_operations::{get_nam, get_nam_nvas, possible_descendants},
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
            let nam_in_guess = if matches!(
                guess.pdag_type,
                crate::partially_directed_acyclic_graph::Structure::CPDAG
            ) {
                get_nam(guess, &[treatment])
            } else {
                FxHashSet::<usize>::default()
            };

            // --- this function differs from parent_aid.rs only in the imports and from here
            let ruletable = crate::graph_operations::ruletables::ancestors::AncestorsRuletable {};
            // gensearch yield_starting_vertices 'false' because Ancestors(T)\T is the adjustment set
            let ancestor_adjustment = crate::graph_operations::gensearch::gensearch(
                guess,
                ruletable,
                [treatment].iter(),
                false,
            );

            let claim_possible_effect = possible_descendants(guess, [treatment].iter());

            // now we take a look at the nodes in the true graph for which the adj.set. was not valid.
            let (nam_in_true, nvas_in_true) =
                get_nam_nvas(truth, &[treatment], ancestor_adjustment);
            // --- to here
            let t_poss_desc_in_truth = possible_descendants(truth, [treatment].iter());

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
                    // if y is not amenable in guess
                    if nam_in_guess.contains(&y) {
                        // but it is amenable in truth
                        if !nam_in_true.contains(&y) {
                            // we count a mistake
                            mistakes += 1;
                        }
                    }
                    // if we reach this point, y has a VAS in guess
                    // now, if the adjustment set is not valid in truth
                    // (either because the pair (t,y) is not amenable or because the VAS is not valid)
                    else if nvas_in_true.contains(&y) {
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
mod tests {
    use std::io::Write;

    use crate::{graph_operations::ancestor_aid, PDAG};

    #[test]
    fn property_equal_dags_zero_distance() {
        for n in 2..40 {
            for _rep in 0..2 {
                let dag = PDAG::random_dag(0.5, n);
                assert_eq!(
                    (0.0, 0),
                    ancestor_aid(&dag, &dag),
                    "ancestor_aid between same dags of size {n} must be zero, dag: {}",
                    dag
                );
                print!(".");
                let _ = std::io::stdout().flush();
            }
        }
    }

    #[test]
    fn random_inputs_no_crash() {
        for n in 2..40 {
            for _rep in 0..2 {
                let dag1 = PDAG::random_dag(1.0, n);
                let dag2 = PDAG::random_dag(1.0, n);
                ancestor_aid(&dag1, &dag2);
                let _ = std::io::stdout().flush();
            }
        }
    }
}
