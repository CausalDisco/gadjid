// SPDX-License-Identifier: MPL-2.0
//! Algorithm for getting all possible descendants of a set of nodes

use rustc_hash::FxHashSet;

use crate::PDAG;

#[allow(unused)]
/// Gets all the possible descendants (reachable via combinations of (-- and ->) of a set of nodes.
/// The input nodes are also included in the output.
pub(crate) fn possible_descendants<'a>(
    pdag: &PDAG,
    starting_vertices: impl Iterator<Item = &'a usize>,
) -> FxHashSet<usize> {
    let mut to_visit_stack = Vec::from_iter(starting_vertices.copied());

    let mut result = FxHashSet::from_iter(to_visit_stack.iter().copied());

    let mut visited = FxHashSet::default();

    while let Some(current_node) = to_visit_stack.pop() {
        visited.insert(current_node);
        pdag.possible_children_of(current_node)
            .iter()
            .filter(|p| !visited.contains(p))
            .for_each(|p| {
                to_visit_stack.push(*p);
                result.insert(*p);
            });
    }

    result
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashSet;

    use crate::PDAG;

    #[test]
    pub fn test_possible_descendants() {
        // 0 -> 1 -- 2
        // |
        // 3
        let cpdag = vec![
            vec![0, 1, 0, 2], //
            vec![0, 0, 2, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
        ];
        let cpdag = PDAG::from_vecvec(cpdag);
        let result = super::possible_descendants(&cpdag, [0].iter());
        assert_eq!(result, FxHashSet::from_iter(vec![0, 1, 2, 3]));

        // 5 -> 4 -- 0 -> 1 -- 2
        //           |
        //           3
        let cpdag = vec![
            vec![0, 1, 0, 2, 2, 0], //
            vec![0, 0, 2, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 1, 0],
        ];
        let cpdag = PDAG::from_vecvec(cpdag);
        let result = super::possible_descendants(&cpdag, [4].iter());
        assert_eq!(result, FxHashSet::from_iter(vec![0, 1, 2, 3, 4]));
    }
}
