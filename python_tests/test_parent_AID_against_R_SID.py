# SPDX-License-Identifier: MPL-2.0
import numpy as np

from gadjid import parent_aid


def load_trlpt(name):
    trplt = np.loadtxt(
        "testgraphs/" f"{name:.0f}.DAG-100.mtx",
        skiprows=2,
    )
    size = 100
    adj = np.zeros((size, size)).astype(np.int8)
    for i, j in trplt:
        adj[int(i) - 1, int(j) - 1] = 1
    return adj


def run_test():
    try:
        testcases = np.loadtxt(
            "testgraphs/SID.DAG-100.csv",
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

        print("test_AID_against_R-SID - ok")
    except AssertionError as e:
        print("test_AID_against_R-SID - failed: ", e)


if __name__ == "__main__":
    print("running")
    run_test()
