import numpy as np


FROM_ROW_TO_COL = "from row to column"
FROM_COL_TO_ROW = "from column to row"


def make_dag(size, density, seed) -> np.ndarray:
    np.random.seed(seed)
    dense: np.ndarray = np.random.binomial(
        1, density, size=(size, size)
    ).astype(np.int8)
    # fill lower triangle+diagonal with zeros
    dense = np.triu(dense, 1)
    perm = np.random.permutation(size)
    return dense[perm, :][:, perm]
