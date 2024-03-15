use anyhow::ensure;
use ::gadjid::graph_operations::ancestor_aid as rust_an_aid;
use ::gadjid::graph_operations::oset_aid as rust_o_aid;
use ::gadjid::graph_operations::parent_aid as rust_pa_aid;
use ::gadjid::graph_operations::shd as rust_shd;
use ::gadjid::graph_operations::sid as rust_sid;
use extendr_api::prelude::*;
use gadjid::{self, EdgelistIterator, PDAG};

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod rgadjid;
    fn parent_aid;
    fn ancestor_aid;
    fn oset_aid;
    fn sid;
    fn shd;
    fn children_of_first_node;
}

#[allow(non_snake_case)]
/// Loads a `matrix` of i32 into a triplet iterator for use with gadjid
fn r_load_matrix(m: &Robj, row_to_col : bool) -> anyhow::Result<PDAG> {
    ensure!(m.is_real(), "expected real numbers!");
    let data = m.as_real_slice().expect("Could not get data as slice");
    let matrix_size = isqrt(data.len());
    ensure!(
        matrix_size * matrix_size == data.len(),
        "Matrix must be square"
    );
    let coord_iter = (0..matrix_size).flat_map(|i| (0..matrix_size).map(move |j| (i, j)));
    let triplet_iter = data
        .iter()
        .map(|f| *f as i8)
        .zip(coord_iter)
        .map(|(v, (i, j))| (i, j, v));
    graph_from_iterator(triplet_iter, row_to_col, matrix_size)
}

/// @export
/// Gets the children of the first node, sanity check for row/column major loading
#[extendr]
pub fn children_of_first_node(true_adjacency: Robj, edge_direction : &str) -> Vec<usize> {
    let row_to_col = edge_direction_is_row_to_col(edge_direction);
    let graph_truth = graph_from_robject(true_adjacency, row_to_col);
    graph_truth.children_of(0).iter().copied().collect()
}

/// floor of sqrt of usize
fn isqrt(n: usize) -> usize {
    let _ = n == 0 && return n;
    let mut s = (n as f64).sqrt() as usize;
    s = (s + n / s) >> 1;
    if s * s > n {
        s - 1
    } else {
        s
    }
}

#[allow(non_snake_case)]
fn graph_from_robject(robj: Robj, row_to_col : bool) -> PDAG {
    // holds a list of transformations that we want to apply to the object to get a PDAG.
    let mut g = [r_load_matrix].iter();

    let mut errors = Vec::new();
    loop {
        if let Some(f) = g.next() {
            let r = f(&robj, row_to_col);
            match r {
                Ok(g) => break g,
                Err(e) => {
                    errors.push(e);
                }
            }
        } else {
            panic!("Could not load adjacency matrix: {:?}", errors);
        }
    }
}

const ROW_TO_COL: &str = "from row to col";
const COL_TO_ROW: &str = "from col to row";

fn edge_direction_is_row_to_col(edge_direction: &str) -> bool {
    match edge_direction {
        ROW_TO_COL => true,
        COL_TO_ROW => false,
        _ => panic!(
            "edge_direction argument must be a string containing (exactly) either '{}' or '{}'",
            ROW_TO_COL, COL_TO_ROW
        ),
    }
}

/// @export
/// Ancestor Adjustment Identification Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[extendr]
pub fn ancestor_aid(true_adjacency: Robj, guess_adjacency: Robj, edge_direction : &str) -> List {
    let row_to_col = edge_direction_is_row_to_col(edge_direction);
    let graph_truth = graph_from_robject(true_adjacency, row_to_col);
    let graph_guess = graph_from_robject(guess_adjacency, row_to_col);
    let (normalized_distance, n_errors) = rust_an_aid(&graph_truth, &graph_guess);
    list!(normalized_distance, n_errors)
}

/// @export
/// Optimal Adjustment Intervention Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[extendr]
pub fn oset_aid(true_adjacency: Robj, guess_adjacency: Robj, edge_direction : &str) -> List {
    let row_to_col = edge_direction_is_row_to_col(edge_direction);
    let graph_truth = graph_from_robject(true_adjacency, row_to_col);
    let graph_guess = graph_from_robject(guess_adjacency, row_to_col);
    let (normalized_distance, n_errors) = rust_o_aid(&graph_truth, &graph_guess);
    list!(normalized_distance, n_errors)
}

/// @export
/// Parent Adjustment Intervention Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
#[extendr]
pub fn parent_aid(true_adjacency: Robj, guess_adjacency: Robj, edge_direction : &str) -> List {
    let row_to_col = edge_direction_is_row_to_col(edge_direction);
    let graph_truth = graph_from_robject(true_adjacency, row_to_col);
    let graph_guess = graph_from_robject(guess_adjacency, row_to_col);
    let (normalized_distance, n_errors) = rust_pa_aid(&graph_truth, &graph_guess);
    list!(normalized_distance, n_errors)
}

/// @export
/// Structural Hamming Distance between two DAG / CPDAG adjacency matrices (sparse or dense)
/// Does not take `edge_direction` argument, because SHD only considers the adjacency matrix,
/// irrespective of the edge direction interpretation.
#[extendr]
pub fn shd(true_adjacency: Robj, guess_adjacency: Robj, ) -> List {
    // set to 'true' as default, the edge direction does not matter for SHD
    let row_to_col = true;
    let graph_truth = graph_from_robject(true_adjacency, row_to_col);
    let graph_guess = graph_from_robject(guess_adjacency, row_to_col);
    let (normalized_distance, n_errors) = rust_shd(&graph_truth, &graph_guess);
    list!(normalized_distance, n_errors)
}
/// @export
/// Structural Intervention Distance between two DAG adjacency matrices (sparse or dense)
#[extendr]
pub fn sid(true_adjacency: Robj, guess_adjacency: Robj, edge_direction : &str) -> List {
    let row_to_col = edge_direction_is_row_to_col(edge_direction);
    let dag_truth = graph_from_robject(true_adjacency, row_to_col);
    let dag_guess = graph_from_robject(guess_adjacency, row_to_col);
    let sid_result = rust_sid(&dag_truth, &dag_guess);
    match sid_result {
        Ok((normalized_distance, n_errors)) => list!(normalized_distance, n_errors),
        Err(e) => panic!("SID error: {}", e),
    }
}


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
                ::gadjid::LoadError::NotAcyclic => anyhow::bail!(err),
            },
        },
        // we have a col-to-row matrix
        false => match PDAG::try_from_col_major(EdgelistIterator::into_column_major_edgelist(
            iterator, graph_size,
        )) {
            Ok(pdag) => Ok(pdag),
            Err(err) => match err {
                ::gadjid::LoadError::NotAcyclic => anyhow::bail!(err),
            },
        },
    }
}
