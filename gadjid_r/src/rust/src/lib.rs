// SPDX-License-Identifier: MPL-2.0
//! R-wrappers for the rust gadjid (Graph Adjustment Identification Distance) library.

use anyhow::{bail, Result};
use extendr_api::prelude::*;

use ::gadjid::graph_operations::ancestor_aid as rust_ancestor_aid;
use ::gadjid::graph_operations::oset_aid as rust_oset_aid;
use ::gadjid::graph_operations::parent_aid as rust_parent_aid;
use ::gadjid::graph_operations::shd as rust_shd;
use ::gadjid::graph_operations::sid as rust_sid;
use ::gadjid::EdgelistIterator;
use ::gadjid::PDAG;

extendr_module! {
    mod gadjid;
    fn ancestor_aid;
    fn oset_aid;
    fn parent_aid;
    fn shd;
    fn sid;
}

const ROW_TO_COL: &str = "from row to column";
const COL_TO_ROW: &str = "from column to row";

fn edge_direction_is_row_to_col(edge_direction: &str) -> Result<bool> {
    match edge_direction {
        ROW_TO_COL => Ok(true),
        COL_TO_ROW => Ok(false),
        _ => bail!(format!(
            r#"edge_direction string argument must be either "{}" or "{}""#,
            ROW_TO_COL, COL_TO_ROW
        )),
    }
}

/// Ancestor Adjustment Identification Distance between two DAG / CPDAG adjacency matrices
///
/// Computes the ancestor adjustment intervention distance between the true `g_true` DAG or CPDAG and an estimated `g_guess` DAG or CPDAG.
///
/// For details see Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616} \cr
/// The source code is available at [github.com/CausalDisco/gadjid](https://github.com/CausalDisco/gadjid)
///
/// Graph inputs are accepted as adjacency matrices of type double.
/// An adjacency matrix for a DAG may only contain 0s and 1s.
/// An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s.
/// DAG and CPDAG inputs are validated for acyclicity.
/// However, for CPDAG inputs, __the user needs to ensure the adjacency
/// matrix indeed codes a valid CPDAG (instead of just a PDAG)__.
///
/// If `edge_direction="from row to column"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘r → c’;
/// if `edge_direction="from column to row"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘c → r’;
/// for either setting of `edge_direction`,
/// a `2` in row `r` and column `c` codes an undirected edge ‘r – c’
/// (an additional `2` in row `c` and column `r` is ignored;
/// one of the two entries is sufficient to code an undirected edge).
///
/// @param g_true Adjacency matrix of the true graph
/// @param g_guess Adjacency matrix of the guess graph
/// @param edge_direction either "from row to column" or "from column to row"
///
/// @return 2-element vector of type double \cr c(normalized error in \[0,1\], total number of errors)
///
/// @examples
/// full <- rbind(c(0, 1, 1, 1),
///               c(0, 0, 1, 1),
///               c(0, 0, 0, 1),
///               c(0, 0, 0, 0))
/// chain <- rbind(c(0, 1, 0, 0),
///                c(0, 0, 1, 0),
///                c(0, 0, 0, 1),
///                c(0, 0, 0, 0))
/// identical(ancestor_aid(full, chain, "from row to column"), c(0/12, 0))
///
/// @references
/// L Henckel, T Würtzen, S Weichwald.
/// "Adjustment Identification Distance: A gadjid for Causal Structure Learning."
/// Proceedings of the 40th Conference on Uncertainty in Artificial Intelligence (UAI), 2024.
/// \doi{doi:10.48550/arXiv.2402.08616}
///
/// @export
#[extendr]
fn ancestor_aid(g_true: RMatrix<f64>, g_guess: RMatrix<f64>, edge_direction: &str) -> Result<Robj> {
    let g_true = graph_from_rmatrix(&g_true, edge_direction)?;
    let g_guess = graph_from_rmatrix(&g_guess, edge_direction)?;
    let aid = rust_ancestor_aid(&g_true, &g_guess);
    Ok(r!([aid.0, aid.1 as f64]))
}

/// Optimal Adjustment Identification Distance between two DAG / CPDAG adjacency matrices
///
/// Computes the optimal adjustment intervention distance between the true `g_true` DAG or CPDAG and an estimated `g_guess` DAG or CPDAG.
///
/// For details see Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616} \cr
/// The source code is available at [github.com/CausalDisco/gadjid](https://github.com/CausalDisco/gadjid)
///
/// Graph inputs are accepted as adjacency matrices of type double.
/// An adjacency matrix for a DAG may only contain 0s and 1s.
/// An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s.
/// DAG and CPDAG inputs are validated for acyclicity.
/// However, for CPDAG inputs, __the user needs to ensure the adjacency
/// matrix indeed codes a valid CPDAG (instead of just a PDAG)__.
///
/// If `edge_direction="from row to column"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘r → c’;
/// if `edge_direction="from column to row"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘c → r’;
/// for either setting of `edge_direction`,
/// a `2` in row `r` and column `c` codes an undirected edge ‘r – c’
/// (an additional `2` in row `c` and column `r` is ignored;
/// one of the two entries is sufficient to code an undirected edge).
///
/// @param g_true Adjacency matrix of the true graph
/// @param g_guess Adjacency matrix of the guess graph
/// @param edge_direction either "from row to column" or "from column to row"
///
/// @return 2-element vector of type double \cr c(normalized error in \[0,1\], total number of errors)
///
/// @examples
/// full <- rbind(c(0, 1, 1, 1),
///               c(0, 0, 1, 1),
///               c(0, 0, 0, 1),
///               c(0, 0, 0, 0))
/// chain <- rbind(c(0, 1, 0, 0),
///                c(0, 0, 1, 0),
///                c(0, 0, 0, 1),
///                c(0, 0, 0, 0))
/// identical(oset_aid(full, chain, "from row to column"), c(3/12, 3))
///
/// @references
/// L Henckel, T Würtzen, S Weichwald.
/// "Adjustment Identification Distance: A gadjid for Causal Structure Learning."
/// Proceedings of the 40th Conference on Uncertainty in Artificial Intelligence (UAI), 2024.
/// \doi{doi:10.48550/arXiv.2402.08616}
///
/// @export
#[extendr]
fn oset_aid(g_true: RMatrix<f64>, g_guess: RMatrix<f64>, edge_direction: &str) -> Result<Robj> {
    let g_true = graph_from_rmatrix(&g_true, edge_direction)?;
    let g_guess = graph_from_rmatrix(&g_guess, edge_direction)?;
    let aid = rust_oset_aid(&g_true, &g_guess);
    Ok(r!([aid.0, aid.1 as f64]))
}

/// Parent Adjustment Identification Distance between two DAG / CPDAG adjacency matrices
///
/// Computes the parent adjustment intervention distance between the true `g_true` DAG or CPDAG and an estimated `g_guess` DAG or CPDAG.
///
/// For details see Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616} \cr
/// The source code is available at [github.com/CausalDisco/gadjid](https://github.com/CausalDisco/gadjid)
///
/// Graph inputs are accepted as adjacency matrices of type double.
/// An adjacency matrix for a DAG may only contain 0s and 1s.
/// An adjacency matrix for a CPDAG may only contain 0s, 1s and 2s.
/// DAG and CPDAG inputs are validated for acyclicity.
/// However, for CPDAG inputs, __the user needs to ensure the adjacency
/// matrix indeed codes a valid CPDAG (instead of just a PDAG)__.
///
/// If `edge_direction="from row to column"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘r → c’;
/// if `edge_direction="from column to row"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘c → r’;
/// for either setting of `edge_direction`,
/// a `2` in row `r` and column `c` codes an undirected edge ‘r – c’
/// (an additional `2` in row `c` and column `r` is ignored;
/// one of the two entries is sufficient to code an undirected edge).
///
/// @param g_true Adjacency matrix of the true graph
/// @param g_guess Adjacency matrix of the guess graph
/// @param edge_direction either "from row to column" or "from column to row"
///
/// @return 2-element vector of type double \cr c(normalized error in \[0,1\], total number of errors)
///
/// @examples
/// full <- rbind(c(0, 1, 1, 1),
///               c(0, 0, 1, 1),
///               c(0, 0, 0, 1),
///               c(0, 0, 0, 0))
/// chain <- rbind(c(0, 1, 0, 0),
///                c(0, 0, 1, 0),
///                c(0, 0, 0, 1),
///                c(0, 0, 0, 0))
/// identical(parent_aid(full, chain, "from row to column"), c(4/12, 4))
///
/// @references
/// L Henckel, T Würtzen, S Weichwald.
/// "Adjustment Identification Distance: A gadjid for Causal Structure Learning."
/// Proceedings of the 40th Conference on Uncertainty in Artificial Intelligence (UAI), 2024.
/// \doi{doi:10.48550/arXiv.2402.08616}
///
/// @export
#[extendr]
fn parent_aid(g_true: RMatrix<f64>, g_guess: RMatrix<f64>, edge_direction: &str) -> Result<Robj> {
    let g_true = graph_from_rmatrix(&g_true, edge_direction)?;
    let g_guess = graph_from_rmatrix(&g_guess, edge_direction)?;
    let aid = rust_parent_aid(&g_true, &g_guess);
    Ok(r!([aid.0, aid.1 as f64]))
}

/// Structural Hamming Distance between two DAG / CPDAG adjacency matrices
///
/// Computes the structural Hamming distance between the true `g_true` PDAG and an estimated `g_guess` PDAG.
///
/// For details see Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616} \cr
/// The source code is available at [github.com/CausalDisco/gadjid](https://github.com/CausalDisco/gadjid)
///
/// Graph inputs are accepted as adjacency matrices of type double.
/// An adjacency matrix for a PDAG may only contain 0s, 1s and 2s.
/// PDAG are validated for acyclicity.
///
/// A `1` in row `r` and column `c` codes a directed edge and
/// a `1` in row `c` and column `r` codes a directed edge in reverse direction;
/// a `2` in row `r` and column `c` codes an undirected edge ‘r – c’
/// (an additional `2` in row `c` and column `r` is ignored;
/// one of the two entries is sufficient to code an undirected edge).
///
/// @param g_true Adjacency matrix of the true partially directed acyclic graph
/// @param g_guess Adjacency matrix of the guess partially directed acyclic graph
///
/// @return 2-element vector of type double \cr c(normalized error in \[0,1\], total number of errors)
///
/// @examples
/// full <- rbind(c(0, 1, 1, 1),
///               c(0, 0, 1, 1),
///               c(0, 0, 0, 1),
///               c(0, 0, 0, 0))
/// chain <- rbind(c(0, 1, 0, 0),
///                c(0, 0, 1, 0),
///                c(0, 0, 0, 1),
///                c(0, 0, 0, 0))
/// identical(shd(full, chain), c(3/6, 3))
///
/// @export
#[extendr]
fn shd(g_true: RMatrix<f64>, g_guess: RMatrix<f64>) -> Result<Robj> {
    let edge_direction = ROW_TO_COL;
    let g_true = graph_from_rmatrix(&g_true, edge_direction)?;
    let g_guess = graph_from_rmatrix(&g_guess, edge_direction)?;
    let shd = rust_shd(&g_true, &g_guess);
    Ok(r!([shd.0, shd.1 as f64]))
}

/// Structural Identification Distance between two DAG adjacency matrices
///
/// Computes the structural intervention distance (SID),
/// between the true `g_true` DAG and an estimated `g_guess` DAG.
///
/// Since the Parent-AID reduces to the SID in the special case of DAG inputs
/// and is efficiently implemented using reachability algorithms,
/// it offers a faster way to calculate the SID;
/// see also Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616}.
/// The example below can be compared to
/// ```R
/// library("SID")
/// system.time(structIntervDist(random_dag(20), random_dag(20)))
/// ```
///
/// For details see Henckel, Würtzen, Weichwald (2024) \doi{doi:10.48550/arXiv.2402.08616} \cr
/// The source code is available at [github.com/CausalDisco/gadjid](https://github.com/CausalDisco/gadjid)
///
/// Graph inputs are accepted as adjacency matrices of type double.
/// An adjacency matrix for a DAG may only contain 0s and 1s.
/// DAG inputs are validated for acyclicity.
///
/// If `edge_direction="from row to column"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘r → c’;
/// if `edge_direction="from column to row"`, then
/// a `1` in row `r` and column `c` codes a directed edge ‘c → r’.
///
/// @param g_true Adjacency matrix of the true directed acyclic graph
/// @param g_guess Adjacency matrix of the guess directed acyclic graph
/// @param edge_direction either "from row to column" or "from column to row"
///
/// @return 2-element vector of type double \cr c(normalized error in \[0,1\], total number of errors)
///
/// @examples
/// random_dag <- function(n, p=0.1) {
///     P <- sample(n)
///     m <- matrix(0, n, n)
///     m[upper.tri(m)] <- rbinom(n*(n-1)/2, 1, p)
///     m[P, P]
/// }
///
/// system.time(sid(random_dag(400), random_dag(400), "from row to column"))
///
/// @references
/// L Henckel, T Würtzen, S Weichwald.
/// "Adjustment Identification Distance: A gadjid for Causal Structure Learning."
/// Proceedings of the 40th Conference on Uncertainty in Artificial Intelligence (UAI), 2024.
/// \doi{doi:10.48550/arXiv.2402.08616}
///
/// J Peters,P Bühlmann.
/// "Structural intervention distance for evaluating causal graphs."
/// Neural Compututation 27(3), 771–799, 2015.
/// \doi{doi:10.1162/NECO_a_00708}
///
/// @export
#[extendr]
fn sid(g_true: RMatrix<f64>, g_guess: RMatrix<f64>, edge_direction: &str) -> Result<Robj> {
    let g_true = graph_from_rmatrix(&g_true, edge_direction)?;
    let g_guess = graph_from_rmatrix(&g_guess, edge_direction)?;
    let sid = rust_sid(&g_true, &g_guess)?;
    Ok(r!([sid.0, sid.1 as f64]))
}

/// Load a graph from a R matrix.
/// Will load a matrix into a PDAG, automatically loading into a DAG and checking
/// acyclicity. If undirected edges present, assumes that it encodes as valid CPDAG
fn graph_from_rmatrix(rmat: &RMatrix<f64>, edge_direction: &str) -> Result<PDAG> {
    let interpret_as_col_to_row = edge_direction_is_row_to_col(edge_direction)?;
    let graph_size = rmat.nrows();
    let iterator = rmat.data().iter().enumerate().map(|(ind, val)| {
        (
            ind / graph_size,
            ind - (ind / graph_size) * graph_size,
            *val as i8,
        )
    });
    // R matrices are in column-major order, so above iterator yields
    // (outer, inner) = (column, row) with outer varying the slowest
    // and what Edgelist yields is taken as (from, to)
    let graph = if interpret_as_col_to_row {
        PDAG::try_from_col_major(EdgelistIterator::into_column_major_edgelist(
            iterator, graph_size,
        ))?
    } else {
        PDAG::try_from_row_major(EdgelistIterator::into_row_major_edgelist(
            iterator, graph_size,
        ))?
    };
    Ok(graph)
}
