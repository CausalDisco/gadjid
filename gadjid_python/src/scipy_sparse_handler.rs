// SPDX-License-Identifier: MPL-2.0

use anyhow::bail;
use gadjid::PDAG;
use numpy::PyReadonlyArray1;
use pyo3::{prelude::PyAnyMethods, Bound, PyAny};
use std::slice::Iter;

use crate::graph_from_iterator;

/// Encodes sparse matrix in CSR/CSC format.
struct CSMatrix<'a> {
    shape: usize,
    indptr: &'a [i32],
    indices: Iter<'a, i32>,
    data: Iter<'a, i8>,
    state: i32,
    current_outer_dim: usize,
}

impl Iterator for CSMatrix<'_> {
    // yields (outer_idx, inner_idx, value)
    type Item = (usize, usize, i8);

    fn next(&mut self) -> Option<Self::Item> {
        if let (Some(inner_idx), Some(value)) = (self.indices.next(), self.data.next()) {
            self.state += 1;
            // advance next outer_dim
            if self.current_outer_dim < self.shape {
                while self.state > self.indptr[self.current_outer_dim + 1] {
                    self.current_outer_dim += 1
                }
            }
            Some((self.current_outer_dim, *inner_idx as usize, *value))
        } else {
            None
        }
    }
}

/// Load a PDAG from a scipy sparse matrix in csr or csc format.
pub fn try_from(ob: &Bound<'_, PyAny>, row_to_col: bool) -> anyhow::Result<PDAG> {
    // get the encoding format
    let format = ob.getattr("format")?;
    let format = format.extract::<String>()?;
    let format = format.as_str();

    // determine whether the matrix is row or column major
    let row_major_iteration = match format {
        // Compressed Sparse Row matrix
        "csr" => true,
        // Compressed Sparse Column matrix
        "csc" => false,
        // will panic later otherwise
        _ => false,
    };

    // load as row-major if it's in row-major order and we want to interpret it as row_to_col,
    // or if it's in column-major order and we want to interpret it as col_to_row (so transpose by interpreting as row-major)
    let interpret_as_row_major = row_to_col == row_major_iteration;

    // get the shape to make sure it is square and for later CSR / CSC iteration
    let shape = ob.getattr("shape")?;
    let shape = shape.extract::<(usize, usize)>()?;
    anyhow::ensure!(shape.0 == shape.1, "Matrix must be square");

    if format == "csr" || format == "csc" {
        graph_from_csc_or_csr(ob, interpret_as_row_major, shape.0)
    } else {
        bail!("Unsupported sparse matrix format received: '{:?}'. The package currently only supports 'csr' and 'csc'.", format);
    }
}

fn graph_from_csc_or_csr(
    ob: &Bound<'_, PyAny>,
    interpret_as_row_major: bool,
    shape: usize,
) -> anyhow::Result<PDAG> {
    // these explanations assume a csr matrix
    // element at index `r` and `r+1` hold the indices of the first (inclusive) and last
    // (exclusive) nonzero entries in row `r`
    let indptr = ob.getattr("indptr")?;
    let indptr = indptr.extract::<PyReadonlyArray1<i32>>()?;
    let indptr = indptr.as_slice()?;

    // element at index `i` holds the column index `c` of the i-th nonzero entry
    let indices = ob.getattr("indices")?;
    let indices = indices.extract::<PyReadonlyArray1<i32>>()?;
    let indices = indices.as_slice()?;

    // element at index `i` holds the value `v` of the i-th nonzero entry
    let data = ob.getattr("data")?;
    let data = data.extract::<PyReadonlyArray1<i8>>()?;
    let data = data.as_slice()?;

    // So, relating this all to the source matrix M, we have M[r,c]=v
    let iterator = CSMatrix {
        shape,
        indptr,
        indices: indices.iter(),
        data: data.iter(),
        state: 0,
        current_outer_dim: 0,
    };

    graph_from_iterator(iterator, interpret_as_row_major, shape)
}
