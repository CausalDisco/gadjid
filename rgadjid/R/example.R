library(rextendr)
document()
library(rgadjid)

guess_data <- c(0, 0, 1, 0)
truth_data <- c(0, 1, 0, 0)

#can use just a vector (or even vector of vectors) or 'matrix'
g <- guess_data
t <- matrix(data = truth_data, nrow=2)

children_of_first_node(g)
children_of_first_node(t)

class(t)
class(g)


#the dimensionality is inferred (and asserted to be square)
sid(t, g)
shd(t, g)
parent_aid(t,g)
ancestor_aid(t,g)
class(oset_aid(t,g))
