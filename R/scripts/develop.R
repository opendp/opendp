
build_docs <- function() {
  # dlls must be built before documentation is built
  pkgbuild::compile_dll()
  devtools::document()
}

# build a package .tar file
build_package <- function() {
  # the tar command is faster than R's bundled tar, and permits 128-byte+ paths
  Sys.setenv(R_BUILD_TAR = "tar")
  devtools::build()
}

# install the opendp package, so that it persists between interpreter sessions
install_package <- function() {
  # moves the package to another location and then runs Makevars
  # TODO: this is broken. See README.md.
  Sys.setenv(R_BUILD_TAR = "tar")
  devtools::install()
}