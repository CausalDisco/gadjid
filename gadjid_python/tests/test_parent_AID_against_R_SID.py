# SPDX-License-Identifier: MPL-2.0
from pathlib import Path

import numpy as np

from gadjid import parent_aid


TESTGRAPHS_DIR = Path(__file__).parent.parent.parent / "testgraphs"


def load_trlpt(name):
    trplt = np.loadtxt(
        TESTGRAPHS_DIR / f"100-node-DAG-{name:.0f}.mtx",
        skiprows=2,
    )
    size = 100
    adj = np.zeros((size, size)).astype(np.int8)
    for i, j in trplt:
        adj[int(i) - 1, int(j) - 1] = 1
    return adj


def test_parent_AID_against_R_SID():
    testcases = np.loadtxt(
        TESTGRAPHS_DIR / "SID-100-node-DAGs.csv",
        delimiter=",",
        skiprows=1,
    )

    for true_name, guess_name, _, rsid in testcases:
        true_name = int(true_name)
        guess_name = int(guess_name)
        Gtrue = load_trlpt(true_name)
        Gguess = load_trlpt(guess_name)
        sid = parent_aid(Gtrue, Gguess)
        assert sid[1] == int(rsid), (
            f"failed for sid({true_name}, {guess_name}):"
            f" {sid[1]} vs {int(rsid)}"
        )

