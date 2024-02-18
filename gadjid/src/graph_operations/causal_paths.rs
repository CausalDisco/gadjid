// SPDX-License-Identifier: MPL-2.0
//! Slow version of getting causal nodes in a PDAG

use crate::PDAG;

/// Gets all nodes on causal paths from treatment to response.
/// The treatment is excluded from the result.
/// Does not check whether the treatment and response are disjoint, but assumes they are.
pub fn causal_nodes(dag: &PDAG, t: &[usize], y: &[usize]) -> Vec<usize> {
    assert_ne!(t.len(), 0, "treatments is not empty");
    assert_ne!(y.len(), 0, "response is not empty");
    let response_ancestors = super::proper_ancestors(dag, t.iter(), y.iter());
    let treatment_descendants = super::descendants(dag, t.iter());
    (response_ancestors.intersection(&treatment_descendants))
        .copied()
        .collect()
}

/// Gets the first node on each causal path from treatment to response.
/// The treatment is excluded from the result.
/// Does not check whether the treatment and response are disjoint, but assumes they are.
pub fn causal_children(dag: &PDAG, t: &[usize], y: &[usize]) -> Vec<usize> {
    assert_ne!(t.len(), 0, "treatments is not empty");
    assert_ne!(y.len(), 0, "response is not empty");
    let response_ancestors = super::proper_ancestors(dag, t.iter(), y.iter());
    let treatment_children = super::children(dag, t.iter());
    (response_ancestors.intersection(&treatment_children))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::graph_operations::causal_paths::{causal_children, causal_nodes};
    use crate::PDAG;
    use std::collections::HashSet;

    #[test]
    fn causal_nodes_test() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_nodes(&dag, &[0], &[2]);
        let expected = HashSet::from([1, 2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[1], &[2]);
        let expected = HashSet::from([2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[2], &[1]);
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 3 and 0 -> 2 -> 3
        let v_dag = vec![
            vec![0, 1, 1, 0], //
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_nodes(&dag, &[0], &[3]);
        let expected = HashSet::from([1, 2, 3]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[1], &[3]);
        let expected = HashSet::from([3]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 2 -> 4 and 0 -> 3 <- 4
        let v_dag = vec![
            vec![0, 1, 0, 1, 0], //
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 1],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0],
        ];
        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_nodes(&dag, &[0], &[4]);
        let expected = HashSet::from([1, 2, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[0, 1], &[4]);
        let expected = HashSet::from([2, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[0], &[3]);
        let expected = HashSet::from([1, 2, 3, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[0, 2], &[3]);
        let expected = HashSet::from([3, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 2 ----> 3 <----7
        //      |    |       |
        //      v    v       v
        //      4    5       6

        let v_dag = vec![
            vec![0, 1, 0, 0, 0, 0, 0, 0], //
            vec![0, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_nodes(&dag, &[0], &[6]);
        let expected = HashSet::from([1, 2, 3, 6]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[0], &[7]);
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_nodes(&dag, &[0, 1], &[5]);
        let expected = HashSet::from([2, 5]);
        assert_eq!(expected, result.iter().copied().collect());
    }

    #[test]
    fn causal_children_test() {
        // 0 -> 1 -> 2
        let v_dag = vec![
            vec![0, 1, 0], //
            vec![0, 0, 1],
            vec![0, 0, 0],
        ];
        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_children(&dag, &[0], &[2]);
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[0], &[1]);
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[1], &[2]);
        let expected = HashSet::from([2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[2], &[1]);
        let expected = HashSet::from([]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 3 and 0 -> 2 -> 3
        let v_dag = vec![
            vec![0, 1, 1, 0], //
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 1],
            vec![0, 0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_children(&dag, &[0], &[3]);
        let expected = HashSet::from([1, 2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[0, 1], &[3]);
        let expected = HashSet::from([2, 3]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 2 -> 4 and 0 -> 3 <- 4
        let v_dag = vec![
            vec![0, 1, 0, 1, 0], //
            vec![0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 1],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0],
        ];
        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_children(&dag, &[0], &[4]);
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[0], &[3]);
        let expected = HashSet::from([1, 3]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[0, 1], &[4]);
        let expected = HashSet::from([2]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[0, 2], &[3]);
        let expected = HashSet::from([3, 4]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 -> 2 ----> 3 <----7
        //      |    |       |
        //      v    v       v
        //      4    5       6

        let v_dag = vec![
            vec![0, 1, 0, 0, 0, 0, 0, 0], //
            vec![0, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_children(&dag, &[0], &[6]);
        let expected = HashSet::from([1]);
        assert_eq!(expected, result.iter().copied().collect());

        // 0 -> 1 --> 2 ---> 3 <----7
        //      |     |      |
        //      v     v      v
        //      4 <-- 5 <--- 6

        let v_dag = vec![
            vec![0, 1, 0, 0, 0, 0, 0, 0], //
            vec![0, 0, 1, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 1, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 0],
        ];

        let dag = PDAG::from_vecvec(v_dag);

        let result = causal_children(&dag, &[2], &[4]);
        let expected = HashSet::from([3, 5]);
        assert_eq!(expected, result.iter().copied().collect());

        let result = causal_children(&dag, &[1], &[4]);
        let expected = HashSet::from([2, 4]);
        assert_eq!(expected, result.iter().copied().collect());
    }
}
