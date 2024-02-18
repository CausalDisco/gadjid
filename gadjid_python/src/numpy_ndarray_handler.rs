// SPDX-License-Identifier: MPL-2.0
use gadjid::PDAG;
use numpy::ndarray::ArrayView2;
use numpy::PyReadonlyArray2;
use pyo3::PyAny;

use crate::graph_from_iterator;

/// Load a DAG from a numpy ndarray
pub fn try_from(ob: &PyAny) -> anyhow::Result<PDAG> {
    let ndarray = ob.extract::<PyReadonlyArray2<i8>>()?;
    let shape = ndarray.shape();
    let graph_size = shape[0];
    anyhow::ensure!(shape[0] == shape[1], "Matrix must be square");
    anyhow::ensure!(graph_size > 0, "Matrix must be non-empty");

    // determine iteration order
    let row_major = ndarray.is_c_contiguous();
    // check if array is contiguous and then get slice
    if let Ok(slice) = ndarray.as_slice() {
        graph_from_slice(slice, row_major, graph_size)
    }
    // otherwise load from view
    else {
        // we will load views as if they are row-major iterators,
        // as we use .indexed_iter() which is row-major.
        // https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#method.indexed_iter
        graph_from_view(ndarray.as_array(), true, graph_size)
    }
}

/// Load a DAG from an slice of i8s, by wrapping to [`Edgelist`]
fn graph_from_slice(slice: &[i8], row_major: bool, graph_size: usize) -> anyhow::Result<PDAG> {
    let iterator = slice.iter().enumerate().map(move |(ind, val)| {
        (
            ind / graph_size,
            ind - (ind / graph_size) * graph_size,
            *val,
        )
    });

    graph_from_iterator(iterator, row_major, graph_size)
}

/// Load a DAG from a numpy ndarray view, by wrapping to [`SquareMatrixIterator`]
fn graph_from_view(
    view: ArrayView2<i8>,
    row_major: bool,
    graph_size: usize,
) -> anyhow::Result<PDAG> {
    println!("Loading from view");
    let iterator = view
        .indexed_iter()
        .map(move |((row, col), val)| (row, col, *val));

    graph_from_iterator(iterator, row_major, graph_size)
}
