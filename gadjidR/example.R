# This function will build the R wrapper functions for gadjid from Rust.
# Requires that the current working directory is gadjid/gadjidR
rextendr::document()

# Import freshly built library
library(gadjidR)

# Show summary
?gadjidR

# 1D vector will be interpreted as contiguous rows.
# The graph size is inferred to be sqrt(len).
truth_data <- c(0, 1, 0, 0)
# [ [0, 1]
#   [0, 0] ] 

# Also possible to nest vectors like this:
guess_data <- c(c(0, 0), c(1, 0))
# [ [0, 0]
#   [1, 0] ] 

# Can represent adjacency matrices as a vector, a vector of vectors or 'matrix'
t_adj <- truth_data
# This is also valid:
# t_adj <- matrix(data = truth_data, nrow=2)
g_adj <- guess_data

# Defining the two valid edge_direction constants for convenience:
ROW_TO_COL <- "from row to col"
COL_TO_ROW <- "from col to row"

# sanity check loading of the adjacency matrices:
children_of_first_node(g_adj, edge_direction = COL_TO_ROW)
children_of_first_node(t_adj, edge_direction = ROW_TO_COL)

sid(t_adj, g_adj, edge_direction = ROW_TO_COL)
shd(t_adj, g_adj)
parent_aid(t_adj, g_adj, edge_direction = ROW_TO_COL)
ancestor_aid(t_adj, g_adj, edge_direction = ROW_TO_COL)
oset_aid(t_adj, g_adj, edge_direction = ROW_TO_COL)

