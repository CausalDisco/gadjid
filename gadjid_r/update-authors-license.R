## Slightly adapted from https://github.com/yutannihilation/string2path/blob/main/update_authors.R
## which is due to Hiroaki Yutani and licensed under MIT license

rlang::check_installed("RcppTOML")
rlang::check_installed("stringr")

library(RcppTOML)


## Parse Cargo.toml files

VENDOR_PATH <- file.path("src", "rust", "vendor")
manifests <- list.files(VENDOR_PATH, pattern = "Cargo\\.toml", recursive = TRUE)

l <- lapply(manifests, \(x) {
  cat("parsing ", x, "\n")
  RcppTOML::parseTOML(file.path(VENDOR_PATH, x))$package
})

names <- vapply(l, function(x)
  x[["name"]], FUN.VALUE = character(1L))

idx <- names != "{}"
l <- l[idx]
names <- names[idx]

versions <- vapply(l, function(x)
  x[["version"]], FUN.VALUE = character(1L))

licenses <- vapply(l, function(x)
  x[["license"]], FUN.VALUE = character(1L))

# Manually curated authors where they are missing in their Cargo.toml file
missing_authors <- c(
  "crossbeam-deque" = "The Crossbeam Project Developers",
  "crossbeam-epoch" = "The Crossbeam Project Developers",
  "crossbeam-utils" = "The Crossbeam Project Developers"
)

authors <- vapply(seq_along(l), function(i) {
  a <- l[[i]][["authors"]]
  crate <- names[i]
  if (is.null(a) || length(a) == 0) {
    if (!is.null(missing_authors[[crate]])) {
      message(sprintf(
        "Author missing for crate '%s', using manual fill-in.",
        crate
      ))
      return(missing_authors[[crate]])
    } else {
      warning(sprintf(
        "Author missing for crate '%s' and no manual fill-in available!",
        crate
      ))
      return(NA_character_)
    }
  }
  a_clean <- stringr::str_remove(a, "\\s+<.+>")
  paste(a_clean, collapse = ", ")
}, FUN.VALUE = character(1L))


## Update inst/AUTHORS

dir.create("inst", showWarnings = FALSE)

cat("Authors of gadjid: Sebastian Weichwald, Theo Würtzen, Leonard Henckel

Authors of the dependency Rust crates:\n\n", file = "inst/AUTHORS")

cat(paste("- ", names, " (version ", versions, "): ",
    authors, "\n",
    sep = "", collapse = ""),
  file = "inst/AUTHORS",
  append = TRUE
)


## Update LICENSE.note

cat("This package contains the source code of the dependency Rust crates in src/rust/vendor.tar.xz
with licenses as listed below.

When Cargo (Rust’s build system and package manager) is not installed,
the pre-compiled binary will be downloaded.\n\n",
  file = "LICENSE.note"
)

cat(paste("Crate:   ", names, " (version ", versions, ")\n",
    "License: ", licenses, "\n",
    "Authors: ", authors, "\n",
    "Files:   src/rust/vendor.tar.xz:vendor/", names, "-", versions, "/*\n",
    sep = "", collapse = "\n"),
  file = "LICENSE.note",
  append = TRUE
)
