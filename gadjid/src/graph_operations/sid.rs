// SPDX-License-Identifier: MPL-2.0
//! Implements Structural Intervention Distance between two DAGs

use std::{error::Error, fmt::Display};

use crate::graph_operations::parent_aid;
use crate::partially_directed_acyclic_graph::Structure::DAG;
use crate::PDAG;

#[derive(Debug)]
/// Errors that can occur when computing SID
pub enum SIDError {
    /// The truth graph is not a DAG
    TruthNotDAG,
    ///
    GuessNotDAG,
    /// The two input graphs are not the same size
    NotSameSize,
}

impl Display for SIDError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SIDError::TruthNotDAG => write!(
                f,
                "Truth graph is not a DAG. Use `parent_aid` if you want to pass a CPDAG"
            ),
            SIDError::GuessNotDAG => write!(
                f,
                "Guess graph is not a DAG. Use `parent_aid` if you want to pass a CPDAG"
            ),
            SIDError::NotSameSize => write!(f, "The two input graphs are not the same size"),
        }
    }
}

impl Error for SIDError {}

/// Structural Intervention Distance between DAGs.
/// Will return error if either graph is not a DAG.
pub fn sid(truth: &PDAG, guess: &PDAG) -> Result<(f64, usize), SIDError> {
    if !matches!(truth.pdag_type, DAG) {
        return Err(SIDError::TruthNotDAG);
    }
    if !matches!(guess.pdag_type, DAG) {
        return Err(SIDError::GuessNotDAG);
    }
    if truth.n_nodes != guess.n_nodes {
        return Err(SIDError::NotSameSize);
    }

    Ok(parent_aid(truth, guess))
}
