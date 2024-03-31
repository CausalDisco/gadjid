# SPDX-License-Identifier: MPL-2.0
import gadjid
from utils import FROM_ROW_TO_COL, FROM_COL_TO_ROW, make_dag


def compare_selective_pairs_to_all_pairs():
    exps = 10
    for exp in range(exps):
        size = 10

        # make 2 random dags:
        truth_dag = make_dag(size, density=0.5, seed=exp)
        guess_dag = make_dag(size, density=0.5, seed=exp + exps)

        assert gadjid.parent_aid(
            truth_dag, guess_dag, edge_direction=FROM_ROW_TO_COL
        ) == gadjid.parent_aid_selective_pairs(
            truth_dag.T, guess_dag.T, treatments=list(range(size)), effects=list(range(size)), edge_direction=FROM_COL_TO_ROW
        )

def one_empty_leads_to_zero_distance():
    exps = 10
    for exp in range(exps):
        size = 10
        # make 2 random dags:
        truth_dag = make_dag(size, density=0.5, seed=exp)
        guess_dag = make_dag(size, density=0.5, seed=exp + exps)

        assert gadjid.parent_aid_selective_pairs(
            truth_dag.T, guess_dag.T, treatments=[], effects=list(range(size)), edge_direction=FROM_COL_TO_ROW
        ) == (0.0, 0)
        
        assert gadjid.parent_aid_selective_pairs(
            truth_dag.T, guess_dag.T, treatments=list(range(size)), effects = [], edge_direction=FROM_COL_TO_ROW
        ) == (0.0, 0)

if __name__ == "__main__":
    compare_selective_pairs_to_all_pairs()
    one_empty_leads_to_zero_distance()
    print("all tests passed")