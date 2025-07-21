# SPDX-License-Identifier: MPL-2.0

cpdag10 <- matrix(0, nrow = 10, ncol = 10)
cpdag10_entries <- cbind(row = c(7, 1, 1, 2, 3, 4, 4, 3, 2, 3, 4),
                         col = c(2, 5, 6, 6, 6, 6, 7, 8, 10, 10, 10),
                         val = c(2, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1))
cpdag10[as.matrix(cpdag10_entries[, 1:2])] <- cpdag10_entries[, 3]

dag17 <- matrix(0, nrow = 10, ncol = 10)
dag17_entries <- cbind(row = c(4, 1, 3, 4, 6, 7, 8, 10, 1, 4, 1, 2, 3, 4, 6, 7, 8, 9, 10, 1, 3, 4, 1, 3, 4, 6, 1, 3, 4, 6, 7, 10, 1, 2, 3, 4, 6, 7, 8, 10, 1, 3, 4, 6, 7),
                       col = c(1, 2, 2, 2, 2, 2, 2, 2, 3, 3, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10))
dag17[as.matrix(dag17_entries[, 1:2])] <- 1

dag18 <- matrix(0, nrow = 10, ncol = 10)
dag18_entries <- cbind(row = c(9, 1, 7, 10, 5, 3, 4, 2, 8),
                       col = c(1, 3, 4, 5, 6, 7, 8, 9, 10))
dag18[as.matrix(dag18_entries[, 1:2])] <- 1

cpdag19 <- matrix(0, nrow = 10, ncol = 10)
cpdag19_entries <- cbind(row = c(9, 7, 2, 1, 6, 9, 3, 6, 3),
                         col = c(1, 2, 4, 5, 5, 7, 8, 8, 10),
                         val = c(2, 2, 2, 1, 1, 2, 2, 2, 2))
cpdag19[as.matrix(cpdag19_entries[, 1:2])] <- cpdag19_entries[, 3]

test_that("ancestor_aid", {
    expect_equal(ancestor_aid(cpdag10, dag17, "from row to column"), c(23/90, 23))
    expect_equal(ancestor_aid(cpdag10, dag17, "from row to column"),
                 ancestor_aid(t(cpdag10), t(dag17), "from column to row"))
    expect_equal(ancestor_aid(dag17, dag18, "from row to column"), c(67/90, 67))
    expect_equal(ancestor_aid(dag18, t(cpdag19), "from column to row"), c(68/90, 68))
})

test_that("oset_aid", {
    expect_equal(oset_aid(cpdag10, dag17, "from row to column"), c(23/90, 23))
    expect_equal(oset_aid(cpdag10, dag17, "from row to column"),
                 oset_aid(t(cpdag10), t(dag17), "from column to row"))
    expect_equal(oset_aid(dag17, dag18, "from row to column"), c(63/90, 63))
    expect_equal(oset_aid(dag18, t(cpdag19), "from column to row"), c(69/90, 69))
})

test_that("parent_aid", {
    expect_equal(parent_aid(cpdag10, dag17, "from row to column"), c(23/90, 23))
    expect_equal(parent_aid(cpdag10, dag17, "from row to column"),
                 parent_aid(t(cpdag10), t(dag17), "from column to row"))
    expect_equal(parent_aid(dag17, dag18, "from row to column"), c(85/90, 85))
    expect_equal(parent_aid(dag18, t(cpdag19), "from column to row"), c(68/90, 68))
})

test_that("shd", {
    expect_equal(shd(cpdag10, dag17), c(40/45, 40))
    expect_equal(shd(cpdag10, dag17), shd(t(cpdag10), t(dag17)))
    expect_equal(shd(dag17, dag18), c(40/45, 40))
    expect_equal(shd(dag18, t(cpdag19)), c(15/45, 15))
})

test_that("sid", {
    expect_error(sid(cpdag10, dag17, "from row to column"))
    expect_equal(sid(dag17, dag18, "from row to column"), c(85/90, 85))
    expect_equal(sid(dag17, dag18, "from row to column"),
                 sid(t(dag17), t(dag18), "from column to row"))
    expect_equal(sid(dag18, t(dag17), "from column to row"), c(47/90, 47))
})
