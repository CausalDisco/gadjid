// SPDX-License-Identifier: MPL-2.0
#![warn(missing_docs)]
//! Python-wrappers for the rust gadjid (Graph Adjustment Identification Distance) library.

use ::gadjid::graph_operations::ancestor_aid as rust_ancestor_aid;
use ::gadjid::graph_operations::oset_aid as rust_oset_aid;
use ::gadjid::graph_operations::parent_aid as rust_parent_aid;
use ::gadjid::graph_operations::shd as rust_shd;
use ::gadjid::graph_operations::sid as rust_sid;
use ::gadjid::EdgelistIterator;
use ::gadjid::PDAG;
use anyhow::bail;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

mod numpy_ndarray_handler;
mod scipy_sparse_handler;
use numpy_ndarray_handler::try_from as try_from_dense;
use scipy_sparse_handler::try_from as try_from_sparse;

/// Adjustment Identification Distance: A 𝚐𝚊𝚍𝚓𝚒𝚍 for Causal Structure Learning
///
/// For details, see the arXiv preprint at https://doi.org/10.48550/arXiv.2402.08616
/// The source code is available at https://github.com/CausalDisco/gadjid
///
/// Adjacency matrices are accepted as either int8 numpy ndarrays
/// or int8 scipy sparse matrices in CSR or CSC format.
/// (The entry in row s and column t codes whether Xₛ → Xₜ.)
///
/// Example:
///
/// ```python
/// from gadjid import example, ancestor_aid, oset_aid, parent_aid, shd
/// import numpy as np
///
/// example.run_parent_aid()
///
/// Gtrue = np.array([
///     [0, 1, 1, 1, 1],
///     [0, 0, 1, 1, 1],
///     [0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0]
/// ], dtype=np.int8)
/// Gguess = np.array([
///     [0, 0, 1, 1, 1],
///     [1, 0, 1, 1, 1],
///     [0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0],
///     [0, 0, 0, 0, 0]
/// ], dtype=np.int8)
///
/// print(ancestor_aid(Gtrue, Gguess))
/// print(shd(Gtrue, Gguess))
/// ```
#[pymodule]
fn gadjid(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::ancestor_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::oset_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::parent_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::shd, m)?)?;
    m.add_function(wrap_pyfunction!(crate::sid, m)?)?;
    Ok(())
}

const ROW_TO_COL: &str = "row->col";
const COL_TO_ROW: &str = "col->row";

fn edge_direction_semantics_is_row_to_col(edge_direction: &str) -> PyResult<bool> {
    match edge_direction {
        ROW_TO_COL => Ok(true),
        COL_TO_ROW => Ok(false),
        _ => Err(PyErr::new::<PyTypeError, _>(format!(
            "edge_direction argument must be either a string containing (exactly) '{}' or '{}'",
            ROW_TO_COL, COL_TO_ROW
        ))),
    }
}

/// Ancestor Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn ancestor_aid(
    g_true: &PyAny,
    g_guess: &PyAny,
    edge_semantics: &str,
) -> PyResult<(f64, usize)> {
    let row_to_col = edge_direction_semantics_is_row_to_col(edge_semantics)?;
    let graph_truth = graph_from_pyobject(g_true, row_to_col)?;
    let graph_guess = graph_from_pyobject(g_guess, row_to_col)?;
    let (normalized_distance, n_errors) = rust_ancestor_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Optimal Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn oset_aid(g_true: &PyAny, g_guess: &PyAny, edge_semantics: &str) -> PyResult<(f64, usize)> {
    let row_to_col = edge_direction_semantics_is_row_to_col(edge_semantics)?;
    let graph_truth = graph_from_pyobject(g_true, row_to_col)?;
    let graph_guess = graph_from_pyobject(g_guess, row_to_col)?;
    let (normalized_distance, n_errors) = rust_oset_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Parent Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn parent_aid(
    g_true: &PyAny,
    g_guess: &PyAny,
    edge_semantics: &str,
) -> PyResult<(f64, usize)> {
    let row_to_col = edge_direction_semantics_is_row_to_col(edge_semantics)?;
    let graph_truth = graph_from_pyobject(g_true, row_to_col)?;
    let graph_guess = graph_from_pyobject(g_guess, row_to_col)?;
    let (normalized_distance, n_errors) = rust_parent_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Structural Hamming Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn shd(g_true: &PyAny, g_guess: &PyAny, edge_semantics: &str) -> PyResult<(f64, usize)> {
    let row_to_col = edge_direction_semantics_is_row_to_col(edge_semantics)?;
    let graph_truth = graph_from_pyobject(g_true, row_to_col)?;
    let graph_guess = graph_from_pyobject(g_guess, row_to_col)?;
    let (normalized_distance, n_errors) = rust_shd(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Structural Identification Distance between two DAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn sid(
    g_true: &PyAny,
    g_guess: &PyAny,
    edge_semantics: &str,
) -> anyhow::Result<(f64, usize)> {
    let row_to_col = edge_direction_semantics_is_row_to_col(edge_semantics)?;
    let dag_truth = graph_from_pyobject(g_true, row_to_col)?;
    let dag_guess = graph_from_pyobject(g_guess, row_to_col)?;
    let (normalized_distance, n_errors) = rust_sid(&dag_truth, &dag_guess)?;
    Ok((normalized_distance, n_errors))
}

/// Load a graph from a 2D numpy or scipy sparse matrix.
/// Will load a matrix into a PDAG, automatically loading into a DAG and checking
/// acyclicity. If undirected edges present, assumes that it encodes as valid CPDAG
fn graph_from_pyobject(ob: &PyAny, is_row_to_col: bool) -> anyhow::Result<PDAG> {
    // first try to load as np dense matrix
    match try_from_dense(ob, is_row_to_col) {
        Ok(load_result) => Ok(load_result),
        Err(e1) => match try_from_sparse(ob, is_row_to_col) {
            Ok(graph) => Ok(graph),
            Err(e2) => {
                let msg = format!(
                    "Errors occured when loading adjacency matrix. Did not succeed trying to load data
as np ndarray or scipy sparse matrix.
\nAttempt to load from numpy ndarray:\n\"{}\"
\nAttempt to load from scipy sparse :\n\"{}\"", e1, e2);
                anyhow::bail!(msg)
            }
        },
    }
}

/// Helper to avoid repetition, used by the numpy and scipy sparse loading files.
/// Will load an edgelist iterator into a PDAG, automatically loading into a DAG and checking
/// acyclicity. If undirected edges present, assumes that it encodes as valid CPDAG
pub(crate) fn graph_from_iterator(
    iterator: impl Iterator<Item = (usize, usize, i8)>,
    row_to_col: bool,
    graph_size: usize,
) -> anyhow::Result<PDAG> {
    match row_to_col {
        true => match PDAG::try_from_row_major(EdgelistIterator::into_row_major_edgelist(
            iterator, graph_size,
        )) {
            Ok(pdag) => Ok(pdag),
            Err(err) => match err {
                ::gadjid::LoadError::NotAcyclic => bail!(err),
            },
        },
        // we have a col-to-row matrix
        false => match PDAG::try_from_col_major(EdgelistIterator::into_column_major_edgelist(
            iterator, graph_size,
        )) {
            Ok(pdag) => Ok(pdag),
            Err(err) => match err {
                ::gadjid::LoadError::NotAcyclic => bail!(err),
            },
        },
    }
}
