# SPDX-License-Identifier: MPL-2.0
import numpy as np
import scipy

import gadjid


def make_dag(size, density, seed) -> np.ndarray:
    np.random.seed(seed)
    dense: np.ndarray = np.random.binomial(
        1, density, size=(size, size)
    ).astype(np.int8)
    # fill lower triangle+diagonal with zeros
    dense = np.triu(dense, 1)
    perm = np.random.permutation(size)
    return dense[perm, :][:, perm]


ROW_TO_COL = "from row to column"
COL_TO_ROW = "from column to row"


def test_edge_direction_argument():
    exps = 10
    for exp in range(exps):
        size = 10

        # make 2 random dags:
        truth_dag = make_dag(size, 0.5, exp)
        guess_dag = make_dag(size, 0.5, exp + exps)

        # for all functions that take an edge_direction argument, check that
        # the result is the same for both edge directions
        assert gadjid.sid(
            truth_dag, guess_dag, edge_direction=ROW_TO_COL
        ) == gadjid.sid(truth_dag.T, guess_dag.T, edge_direction=COL_TO_ROW)
        assert gadjid.parent_aid(
            truth_dag, guess_dag, edge_direction=ROW_TO_COL
        ) == gadjid.parent_aid(
            truth_dag.T, guess_dag.T, edge_direction=COL_TO_ROW
        )
        assert gadjid.ancestor_aid(
            truth_dag, guess_dag, edge_direction=ROW_TO_COL
        ) == gadjid.ancestor_aid(
            truth_dag.T, guess_dag.T, edge_direction=COL_TO_ROW
        )
        assert gadjid.oset_aid(
            truth_dag, guess_dag, edge_direction=ROW_TO_COL
        ) == gadjid.oset_aid(
            truth_dag.T, guess_dag.T, edge_direction=COL_TO_ROW
        )


def test_DAG_loading_for_all_formats():
    reps = 10
    for rep in range(reps):
        size = 10

        # make random dag:
        dag = make_dag(size + 1, 0.5, seed=rep)
        dag_copy = dag[:size, :size].copy()
        dag_view = dag[:size, :size]

        # make every possible kind of adjacency matrix that
        # represents the same DAG:
        matrices = []
        matrices += [np.asfortranarray(dag_copy)]
        matrices += [np.ascontiguousarray(dag_copy)]
        matrices += [scipy.sparse.csr_matrix(dag_copy)]
        matrices += [scipy.sparse.csc_matrix(dag_copy)]
        matrices += [np.asfortranarray(dag_view)]
        matrices += [np.ascontiguousarray(dag_view)]

        names = [
            "fortran",
            "contiguous",
            "csr",
            "csc",
            "fortran-view",
            "contiguous-view",
        ]

        last_result = None
        for i, matrix in enumerate(matrices):
            current_result = gadjid.shd(matrix, matrices[0])
            assert (
                last_result is None or last_result == current_result
            ), f"failed for {names[i]}"
            last_result = current_result


if __name__ == "__main__":
    test_edge_direction_argument()
    test_DAG_loading_for_all_formats()
    print("all tests passed")
