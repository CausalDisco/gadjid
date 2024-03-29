// SPDX-License-Identifier: MPL-2.0
//! This module contains the Edgelist struct, which is an iterator over the edges of a graph.

/// An iterator over the edges of a graph, yielding `(from, to, edgetype)` tuples.

/// Example yield: `(4, 7, 1)`, which is to be interpreted as `4 -> 7`.

/// Will skip over all 0's in the inner iterator, yielding only nonzero entries.
/// Will panic during load if the inner iterator yields edges in a non-row-by-row or non-column-by-column order.
pub struct Edgelist<Order: IterationLayoutTag, I>
where
    I: Iterator<Item = (usize, usize, i8)>,
{
    /// Holds the order layout. Used only for strong typing.
    pub layout_tag: std::marker::PhantomData<Order>,
    /// |V| where V is the set of vertices.
    pub size: usize,
    /// The iterator over the entries of the matrix, yielding (row, column, value) tuples.
    pub iterator: I,
    /// The index of the last yielded entry. Used to check order.
    pub previous_index: Option<(usize, usize)>,
}

impl<Order, I> Edgelist<Order, I>
where
    Order: IterationLayoutTag,
    I: Iterator<Item = (usize, usize, i8)>,
{
    /// panic if receiving `next_index` having an earlier outer idx than `prev_index`
    /// OR if receiving `next_index` with an earlier-or-same inner idx given the same outer idx as
    /// `prev_index`
    fn order_check(prev_index: Option<(usize, usize)>, next_index: (usize, usize)) {
        if let Some((prev_outer, prev_inner)) = prev_index {
            let (next_outer, next_inner) = next_index;

            if next_outer < prev_outer || (next_outer == prev_outer && next_inner <= prev_inner) {
                panic!(
                    "Iterator yielded entries in wrong order. {}, prev (outer, inner) index:{:?}, next (outer, inner) index:{:?}",
                    std::any::type_name::<Self>(),
                    (prev_outer, prev_inner),
                    (next_outer, next_inner)
                );
            }
        }
    }
}

// Iterator so we can iterate over the [`Edgelist`] skipping zero entries, and panicking on order violation
impl<Order, I> Iterator for Edgelist<Order, I>
where
    Order: IterationLayoutTag,
    I: Iterator<Item = (usize, usize, i8)>,
{
    type Item = (usize, usize, i8);
    fn next(&mut self) -> Option<Self::Item> {
        for val in self.iterator.by_ref() {
            match val {
                // skip 0 entries
                (_, _, 0) => {
                    continue;
                }
                // yield non-zero entries
                (_, _, _) => {
                    // panic if order is violated
                    (Self::order_check)(self.previous_index, (val.0, val.1));
                    // record previous yield index
                    self.previous_index = Some((val.0, val.1));
                    return Some(val);
                }
            }
        }
        None
    }
}

/// Trait to tag [`Edgelist`] with the iteration order and layout in which it yields edges.
/// Implemented by [`RowMajorOrder`] and [`ColumnMajorOrder`]. There is no functionality
/// associated with this trait, it is purely for strong type checking.
pub trait IterationLayoutTag: Sized {}

/// This indicates that the outer index is the row, and the inner index is the column. The
/// column index will vary the fastest. The iterator will yield triples of `(row, column, edge_type)`.
pub struct RowMajorOrder;

/// This indicates that the outer index is the column, and the inner index is the row. The
/// row index will vary the fastest. The iterator will yield triples of `(column, row, edge_type)`.
pub struct ColumnMajorOrder;

// There is no actual functionality associated with the trait implementation, it is purely for strong type checking.
impl IterationLayoutTag for RowMajorOrder {}
impl IterationLayoutTag for ColumnMajorOrder {}
