// SPDX-License-Identifier: MPL-2.0
//! Implements functions that take graphs, such as SHD, generalized search, ...
pub mod aid_utils;
pub mod ancestor_aid;
pub mod causal_paths;
pub mod gensearch;
pub mod oset_aid;
pub mod parent_aid;
pub mod possible_descendants;
pub mod ruletables;
pub mod shd;
pub mod sid;

pub use aid_utils::{get_nam, get_nam_nvas};
pub use ancestor_aid::ancestor_aid;
pub use causal_paths::causal_children;
pub use causal_paths::causal_nodes;
pub use oset_aid::oset_aid;
pub use parent_aid::parent_aid;
pub use possible_descendants::possible_descendants;
pub use ruletables::ancestors::ancestors;
pub use ruletables::children::children;
pub use ruletables::descendants::descendants;
pub use ruletables::parents::parents;
pub use ruletables::proper_ancestors::proper_ancestors;
pub use shd::shd;
pub use sid::sid;

#[cfg(test)]
mod tests {

    use crate::PDAG;

    #[test]
    pub fn nam_correctly_counted_as_mistake() {
        // this tests checks mistakes between the cpdag X - Y and dag X -> Y for all distances.

        let dag = vec![
            vec![0, 1], //
            vec![0, 0],
        ];
        let cpdag = vec![
            vec![0, 2], //
            vec![0, 0],
        ];
        let dag = PDAG::from_vecvec(dag);
        let cpdag = PDAG::from_vecvec(cpdag);

        assert_eq!((1.0, 2), crate::graph_operations::parent_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), crate::graph_operations::parent_aid(&cpdag, &dag));
        assert_eq!(
            (1.0, 2),
            crate::graph_operations::ancestor_aid(&dag, &cpdag)
        );
        assert_eq!(
            (1.0, 2),
            crate::graph_operations::ancestor_aid(&cpdag, &dag)
        );
        assert_eq!((1.0, 2), crate::graph_operations::oset_aid(&dag, &cpdag));
        assert_eq!((1.0, 2), crate::graph_operations::oset_aid(&cpdag, &dag));
    }
}
