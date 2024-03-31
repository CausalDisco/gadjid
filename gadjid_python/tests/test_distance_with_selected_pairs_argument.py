# SPDX-License-Identifier: MPL-2.0
from utils import FROM_COL_TO_ROW, FROM_ROW_TO_COL, make_dag

import gadjid


# This does not validate the correctness of the implementation, but it checks
# that the # selective_pairs argument on the python-end is correctly passed to
# the rust-backend.


# The correctness is validated by the snapshot tests in Rust.
def compare_selective_pairs_to_all_pairs():
    fn = [gadjid.parent_aid, gadjid.ancestor_aid, gadjid.oset_aid]
    fns = [
        gadjid.parent_aid_selected_pairs,
        gadjid.ancestor_aid_selected_pairs,
        gadjid.oset_aid_selected_pairs,
    ]

    exps = 10
    for exp in range(exps):
        size = 10
        # make 2 random dags:
        truth_dag = make_dag(size, density=0.5, seed=exp)
        guess_dag = make_dag(size, density=0.5, seed=exp + exps)

        for f, fs in zip(fn, fns):
            assert f(
                truth_dag, guess_dag, edge_direction=FROM_ROW_TO_COL
            ) == fs(
                truth_dag.T,
                guess_dag.T,
                treatments=list(range(size)),
                effects=list(range(size)),
                edge_direction=FROM_COL_TO_ROW,
            )


# Check that, if any of the two sets of selected pairs is empty,
# the distance is zero.
def one_empty_leads_to_zero_distance():
    fns = [
        gadjid.parent_aid_selected_pairs,
        gadjid.ancestor_aid_selected_pairs,
        gadjid.oset_aid_selected_pairs,
    ]

    exps = 10
    for exp in range(exps):
        size = 10
        # make 2 random dags:
        truth_dag = make_dag(size, density=0.5, seed=exp)
        guess_dag = make_dag(size, density=0.5, seed=exp + exps)

        for f in fns:
            assert f(
                truth_dag.T,
                guess_dag.T,
                treatments=[],
                effects=list(range(size)),
                edge_direction=FROM_COL_TO_ROW,
            ) == (0.0, 0)

            assert f(
                truth_dag.T,
                guess_dag.T,
                treatments=list(range(size)),
                effects=[],
                edge_direction=FROM_COL_TO_ROW,
            ) == (0.0, 0)


if __name__ == "__main__":
    compare_selective_pairs_to_all_pairs()
    one_empty_leads_to_zero_distance()
    print("all tests passed")
