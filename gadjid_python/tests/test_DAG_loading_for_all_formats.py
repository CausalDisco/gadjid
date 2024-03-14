# SPDX-License-Identifier: MPL-2.0
import numpy as np
import scipy

import gadjid as aid


def make_dag(size, density, seed) -> np.ndarray:
    np.random.seed(seed)
    dense: np.ndarray = np.random.binomial(
        1, density, size=(size, size)).astype(np.int8)
    # fill lower triangle+diagonal with zeros
    dense = np.triu(dense, 1)
    perm = np.random.permutation(size)
    return dense[perm, :][:, perm]


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
            current_result = aid.shd(matrix, matrices[0])
            assert (
                last_result is None or last_result == current_result
            ), f"failed for {names[i]}"
            last_result = current_result

if __name__ == "__main__":
    test_DAG_loading_for_all_formats()