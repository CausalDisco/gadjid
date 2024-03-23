# SPDX-License-Identifier: MPL-2.0
"""Run a quick example."""

# wall time
from time import perf_counter as timer

import numpy as np

from gadjid import parent_aid


rng = np.random.default_rng(0)

ROW_TO_COL = "from row to col"

def random_dag(size, probability):
    """Draw adjacency matrix of a random DAG."""
    adj = rng.binomial(1, probability, size=(size, size)).astype(np.int8)
    adj = np.triu(adj, 1)
    perm = rng.permutation(size)
    return adj[perm, :][:, perm]


def run_parent_aid(size=500, probability=0.1):
    """Compare two DAGs using parent_aid."""
    print(
        f"\nCalculating the Parent-AID between {size}-node DAGs "
        f"with {100 * (probability):.0f}% of all possible edges\n\n"
        f"    >>> parent_aid(random_dag({size}, {probability}), "
        f"random_dag({size}, {probability}))"
    )

    DAGa = random_dag(size, probability)
    DAGb = random_dag(size, probability)

    tic = timer()
    print(f"    {parent_aid(DAGa, DAGb, edge_direction=ROW_TO_COL)}")
    toc = timer() - tic
    print(
        f"\ntook {toc:.3f} seconds.\n\n"
        "Compare this to the runtime for calculating the SID in R via \n\n    "
        'R -e "library(SID); '
        f"s <- structIntervDist(randomDAG({size}, {probability:.2f}), "
        f'randomDAG({size}, {probability:.2f}));"\n'
    )
