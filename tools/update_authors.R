rlang::check_installed("RcppTOML")

library(RcppTOML)

## Update inst/AUTHORS

vendor_path <- "vendor"
manifests <- list.files(vendor_path, pattern = "Cargo.toml", recursive = TRUE)

l <- lapply(manifests, \(x) RcppTOML::parseTOML(file.path(vendor_path, x))$package)

names <- vapply(l, \(x) x[["name"]], FUN.VALUE = character(1L))
versions <- vapply(l, \(x) x[["version"]], FUN.VALUE = character(1L))

authors <- vapply(l, \(x) {
  # Remove email addresses
  authors <- stringr::str_remove(x[["authors"]], "\\s+<.+>")
  paste(authors, collapse = ", ")
}, FUN.VALUE = character(1L))

licenses <- vapply(l, \(x) x[["license"]], FUN.VALUE = character(1L))

dir.create("R/opendpbase/inst", showWarnings = FALSE)

cat("The authors of the dependency Rust crates:

", file = "R/opendpbase/inst/AUTHORS")

authors_flattened <- vapply(stringr::str_split(authors, ",\\s+"), \(x) {
  paste(x, collapse = "\n  ")
}, FUN.VALUE = character(1L))

cat(paste(
  names, " (version ", versions, "):\n  ",
  authors_flattened,
  "\n",
  sep = "",
  collapse = "\n"
), file = "R/opendpbase/inst/AUTHORS", append = TRUE)

## Update LICENSE.note

cat("This package contains the Rust source code of the dependencies in src/rust/vendor.tar.xz
The authorships and the licenses are listed below. In summary, all libraries are
distributed either under the MIT license or under MIT/Apache-2.0 dual license.

===============================

", file = "R/opendpbase/LICENSE.note")

cat(paste(
  "Name:    ", names,    "\n",
  "Files:   vendor/", names,    "/*\n",
  "Authors: ", authors,  "\n",
  "License: ", licenses, "\n",
  sep = "",
  collapse = "\n------------------------------\n\n"
), file = "R/opendpbase/LICENSE.note", append = TRUE)
