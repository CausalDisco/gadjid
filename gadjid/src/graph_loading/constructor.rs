// SPDX-License-Identifier: MPL-2.0
//! Holds functions for easy construction of Edgelist<IterationOrder> from various sources.

use std::{
    iter::{repeat, Enumerate, FlatMap, Map, Repeat, Zip},
    marker::PhantomData,
    vec::IntoIter,
};

use super::edgelist::{ColumnMajorOrder, Edgelist, IterationLayoutTag, RowMajorOrder};

/// Iterator adaptor so we can turn a triplet-iterator into order-type-annotated edge-list iterator
pub trait EdgelistIterator<I>: Iterator<Item = (usize, usize, i8)> + Sized
where
    I: Iterator<Item = (usize, usize, i8)>,
{
    /// Converts a triple-iterator into an EdgelistIterator.
    /// Assumes that the iterator yields edges in `(row, column, edgetype)` order, with `row`
    /// varying the slowest. Will panic otherwise.
    fn into_row_major_edgelist(self, size: usize) -> Edgelist<RowMajorOrder, I>;
    /// Converts a triple-iterator into an EdgelistIterator.
    /// Assumes that the iterator yields edges in `(column, row, edgetype)` order, with `column`
    /// varying the slowest. Will panic otherwise.
    fn into_column_major_edgelist(self, size: usize) -> Edgelist<ColumnMajorOrder, I>;
}

// Implement for all relevant Iterators that we want to turn into EdgelistIterator
impl<I: Iterator<Item = (usize, usize, i8)>> EdgelistIterator<I> for I {
    fn into_row_major_edgelist(self, size: usize) -> Edgelist<RowMajorOrder, I> {
        Edgelist {
            layout_tag: PhantomData::<RowMajorOrder> {},
            size,
            iterator: self,
            previous_index: None,
        }
    }
    fn into_column_major_edgelist(self, size: usize) -> Edgelist<ColumnMajorOrder, I> {
        Edgelist {
            layout_tag: PhantomData::<ColumnMajorOrder> {},
            size,
            iterator: self,
            previous_index: None,
        }
    }
}

/// Long type annotation, necessary to make the compiler happy
type ConversionFromVecVecToTriple = FlatMap<
    Enumerate<IntoIter<Vec<i8>>>,
    Map<
        Enumerate<Zip<Repeat<usize>, IntoIter<i8>>>,
        fn((usize, (usize, i8))) -> (usize, usize, i8),
    >,
    fn(
        (usize, Vec<i8>),
    ) -> Map<
        Enumerate<Zip<Repeat<usize>, IntoIter<i8>>>,
        fn((usize, (usize, i8))) -> (usize, usize, i8),
    >,
>;
impl<T: IterationLayoutTag> Edgelist<T, ConversionFromVecVecToTriple> {
    /// Creates an `Edgelist` from a vector of vectors.
    /// The calling function will automatically infer the generic `IterationLayoutTag`.
    /// If using `RowMajorOrder`, it would load an adjacency matrix as it is visually
    /// shown in code.
    ///
    /// For example, the matrix `Adj`
    /// ```text
    ///       0 1 1
    /// Adj = 0 0 1
    ///       0 0 0
    /// ```
    /// is represented by loading this `Vec<Vec<i8>>` as `RowMajorOrder`:
    /// ```text
    /// vec![
    ///     vec![0, 1, 1],
    ///     vec![0, 0, 1],
    ///     vec![0, 0, 0],
    /// ]
    /// ````
    /// and `transpose(Adj)` is represented by loading it as `ColumnMajorOrder`.
    pub fn from_vecvec(
        vecvec: Vec<Vec<i8>>,
    ) -> Edgelist<T, impl Iterator<Item = (usize, usize, i8)>> {
        let size = vecvec.len();
        assert!(size == vecvec[0].len(), "adjacency matrix must be square");

        // ugly but necessary type annotations
        type OrderConverter = fn((usize, (usize, i8))) -> (usize, usize, i8);
        type EnumZip = Enumerate<Zip<std::iter::Repeat<usize>, std::vec::IntoIter<i8>>>;

        fn give_index((row, val): (usize, Vec<i8>)) -> Map<EnumZip, OrderConverter> {
            repeat(row).zip(val).enumerate().map(reorder)
        }

        fn reorder((col, (row, val)): (usize, (usize, i8))) -> (usize, usize, i8) {
            (row, col, val)
        }

        let flattened_matrix = vecvec.into_iter().enumerate().flat_map(give_index);

        Edgelist {
            layout_tag: PhantomData::<T> {},
            size,
            iterator: flattened_matrix,
            previous_index: None,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn load_row_major() {
        // matrix we are simulating is
        // 0 1 1
        // 0 0 1
        // 0 0 0
        let vec = vec![(0, 1, 1), (0, 2, 1), (1, 2, 1)];
        let len = vec.len();
        let iter = super::EdgelistIterator::into_row_major_edgelist(vec.into_iter(), len);

        iter.for_each(drop);
    }
    #[test]
    #[should_panic]
    fn fail_load_row_major() {
        // matrix we are simulating is
        // 0 1 1
        // 0 0 1
        // 0 0 0

        // but we are yielding in wrong order
        let vec = vec![(1, 2, 1), (0, 1, 1), (0, 2, 1)];
        let len = vec.len();
        let iter = super::EdgelistIterator::into_row_major_edgelist(vec.into_iter(), len);

        iter.for_each(drop);
    }
    #[test]
    fn load_col_major() {
        // matrix we are simulating is
        // 0 1 1
        // 0 0 1
        // 0 0 0
        let vec = vec![(1, 0, 1), (2, 0, 1), (2, 1, 1)];
        let len = vec.len();
        let iter = super::EdgelistIterator::into_column_major_edgelist(vec.into_iter(), len);

        iter.for_each(drop);
    }
    #[test]
    #[should_panic]
    fn fail_load_col_major() {
        // matrix we are simulating is
        // 0 1 1
        // 0 0 1
        // 0 0 0

        // but we are yielding in wrong order
        let vec = vec![(2, 0, 1), (1, 0, 1), (2, 1, 1)];
        let len = vec.len();
        let iter = super::EdgelistIterator::into_column_major_edgelist(vec.into_iter(), len);

        iter.for_each(drop);
    }
}
