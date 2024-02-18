// SPDX-License-Identifier: MPL-2.0
#![warn(missing_docs)]
//! Python-wrappers for the rust gadjid (Graph Adjustment Identification Distance) library.

use ::gadjid::EdgelistIterator;
use ::gadjid::graph_operations::ancestor_aid as rust_an_aid;
use ::gadjid::graph_operations::oset_aid as rust_o_aid;
use ::gadjid::graph_operations::parent_aid as rust_pa_aid;
use ::gadjid::graph_operations::shd as rust_shd;
use ::gadjid::graph_operations::sid as rust_sid;
use ::gadjid::PDAG;
use anyhow::bail;
use pyo3::prelude::*;

mod numpy_ndarray_handler;
mod scipy_sparse_handler;
use numpy_ndarray_handler::try_from as try_from_dense;
use scipy_sparse_handler::try_from as try_from_sparse;

/// Python module for gadjid
#[pymodule]
fn gadjid(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(crate::parent_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::ancestor_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::oset_aid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::sid, m)?)?;
    m.add_function(wrap_pyfunction!(crate::shd, m)?)?;
    Ok(())
}

/// Structural hamming distance between two adjacency matrices for DAGs (sparse or dense)
#[pyfunction]
pub fn shd(true_adjacency: &PyAny, guess_adjacency: &PyAny) -> PyResult<(f64, usize)> {
    let graph_truth = graph_from_pyobject(true_adjacency)?;
    let graph_guess = graph_from_pyobject(guess_adjacency)?;
    let (normalized_distance, n_errors) = rust_shd(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Structural Identification Distance between two DAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn sid(true_adjacency: &PyAny, guess_adjacency: &PyAny) -> anyhow::Result<(f64, usize)> {
    let dag_truth = graph_from_pyobject(true_adjacency)?;
    let dag_guess = graph_from_pyobject(guess_adjacency)?;
    let (normalized_distance, n_errors) = rust_sid(&dag_truth, &dag_guess)?;
    Ok((normalized_distance, n_errors))
}

/// Parent Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn parent_aid(true_adjacency: &PyAny, guess_adjacency: &PyAny) -> PyResult<(f64, usize)> {
    let graph_truth = graph_from_pyobject(true_adjacency)?;
    let graph_guess = graph_from_pyobject(guess_adjacency)?;
    let (normalized_distance, n_errors) = rust_pa_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Ancestor Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn ancestor_aid(true_adjacency: &PyAny, guess_adjacency: &PyAny) -> PyResult<(f64, usize)> {
    let graph_truth = graph_from_pyobject(true_adjacency)?;
    let graph_guess = graph_from_pyobject(guess_adjacency)?;
    let (normalized_distance, n_errors) = rust_an_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Optimal Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[pyfunction]
pub fn oset_aid(true_adjacency: &PyAny, guess_adjacency: &PyAny) -> PyResult<(f64, usize)> {
    let graph_truth = graph_from_pyobject(true_adjacency)?;
    let graph_guess = graph_from_pyobject(guess_adjacency)?;
    let (normalized_distance, n_errors) = rust_o_aid(&graph_truth, &graph_guess);
    Ok((normalized_distance, n_errors))
}

/// Load a graph from a 2D numpy or scipy sparse matrix.
/// Will load a matrix into a PDAG, automatically loading into a DAG and checking
/// acyclicity. If undirected edges present, assumes that it encodes as valid CPDAG
fn graph_from_pyobject(ob: &PyAny) -> anyhow::Result<PDAG> {
    // first try to load as np dense matrix
    match try_from_dense(ob) {
        Ok(load_result) => Ok(load_result),
        Err(e1) => match try_from_sparse(ob) {
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
    row_major: bool,
    graph_size: usize,
) -> anyhow::Result<PDAG> {
    match row_major {
        true => match PDAG::try_from_row_major(EdgelistIterator::into_row_major_edgelist(
            iterator, graph_size,
        )) {
            Ok(pdag) => Ok(pdag),
            Err(err) => match err {
                ::gadjid::LoadError::NotAcyclic => bail!(err),
            },
        },
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
