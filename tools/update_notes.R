# This file is adapted from https://github.com/yutannihilation/string2path, under MIT license
# Based on https://github.com/yutannihilation/string2path/blob/main/update_authors.R

library(RcppTOML)
library(stringr)

## Update inst/AUTHORS

vendor_path <- "vendor"
manifests <- list.files(vendor_path, pattern = "Cargo.toml", recursive = TRUE)

l <- lapply(manifests, \(x) RcppTOML::parseTOML(file.path(vendor_path, x))$package)

names <- vapply(l, \(x) x[["name"]], FUN.VALUE = character(1L))
versions <- vapply(l, \(x) toString(x[["version"]]), FUN.VALUE = character(1L))

authors <- vapply(l, \(x) {
  # Remove email addresses
  authors <- stringr::str_remove(x[["authors"]], "\\s+<.+>")
  paste(authors, collapse = ", ")
}, FUN.VALUE = character(1L))

# TODO: handle cases where license field is not present, but license file is present
licenses <- vapply(l, \(x) {
  if ("license" %in% x) { x[["license"]] } else { "see license file" }
}, FUN.VALUE = character(1L))

dir.create("R/opendp/inst", showWarnings = FALSE)

cat("The authors of the dependency Rust crates:

", file = "R/opendp/inst/AUTHORS")

authors_flattened <- vapply(stringr::str_split(authors, ",\\s+"), \(x) {
  paste(x, collapse = "\n  ")
}, FUN.VALUE = character(1L))

cat(paste(
  names, " (version ", versions, "):\n  ",
  authors_flattened,
  "\n",
  sep = "",
  collapse = "\n"
), file = "R/opendp/inst/AUTHORS", append = TRUE)

## Update LICENSE.note

cat("This package contains the Rust source code of the dependencies in src/vendor.tar.xz
The authorships and the licenses are listed below.

===============================

", file = "R/opendp/LICENSE.note")

cat(paste(
  "Name:    ", names,    "\n",
  "Files:   vendor/", names,    "/*\n",
  "Authors: ", authors,  "\n",
  "License: ", licenses, "\n",
  sep = "",
  collapse = "\n------------------------------\n\n"
), file = "R/opendp/LICENSE.note", append = TRUE)
