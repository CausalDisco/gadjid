// SPDX-License-Identifier: MPL-2.0

use gadjid::PDAG;
use numpy::ndarray::ArrayView2;
use numpy::{PyReadonlyArray2, PyUntypedArrayMethods};
use pyo3::{prelude::PyAnyMethods, Bound, PyAny};

use crate::graph_from_iterator;

/// Load a PDAG from a numpy ndarray
pub fn try_from(ob: &Bound<'_, PyAny>, row_to_col: bool) -> anyhow::Result<PDAG> {
    let ndarray = ob.extract::<PyReadonlyArray2<i8>>()?;
    let shape = ndarray.shape();
    let graph_size = shape[0];
    anyhow::ensure!(shape[0] == shape[1], "Matrix must be square");
    anyhow::ensure!(graph_size > 0, "Matrix must be non-empty");

    // determine iteration order
    let row_major_iteration = ndarray.is_c_contiguous();
    // load as row-major if it's in row-major order and we want to interpret it as row_to_col,
    // or if it's in column-major order and we want to interpret it as col_to_row (so transpose by interpreting as row-major)
    let interpret_as_row_major = row_to_col == row_major_iteration;
    // check if array is contiguous and then get slice
    if let Ok(slice) = ndarray.as_slice() {
        graph_from_slice(slice, interpret_as_row_major, graph_size)
    }
    // otherwise load from view
    else {
        // views are row-major iterators,
        // as we use .indexed_iter() which is row-major.
        // https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.indexed_iter
        // so the load order is solely determined by the 'is_row_to_col' flag
        graph_from_view(ndarray.as_array(), row_to_col, graph_size)
    }
}

/// Load a PDAG from an slice of i8s
fn graph_from_slice(
    slice: &[i8],
    interpret_as_row_major: bool,
    graph_size: usize,
) -> anyhow::Result<PDAG> {
    let iterator = slice.iter().enumerate().map(move |(ind, val)| {
        (
            ind / graph_size,
            ind - (ind / graph_size) * graph_size,
            *val,
        )
    });

    graph_from_iterator(iterator, interpret_as_row_major, graph_size)
}

/// Load a PDAG from a numpy ndarray view
fn graph_from_view(
    view: ArrayView2<i8>,
    row_to_col: bool,
    graph_size: usize,
) -> anyhow::Result<PDAG> {
    let iterator = view
        .indexed_iter()
        .map(move |((row, col), val)| (row, col, *val));

    graph_from_iterator(iterator, row_to_col, graph_size)
}
