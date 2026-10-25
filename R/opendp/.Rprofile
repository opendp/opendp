if (!nzchar(Sys.getenv("OPENDP_LIB_DIR"))) {
  repo_lib_dirs <- c(
    normalizePath("../../rust/target/debug", mustWork = FALSE),
    normalizePath("../../rust/target/release", mustWork = FALSE)
  )
  existing_lib_dir <- Filter(
    function(path) file.exists(file.path(path, "libopendp.a")),
    repo_lib_dirs
  )
  if (length(existing_lib_dir) > 0L) {
    Sys.setenv(OPENDP_LIB_DIR = existing_lib_dir[[1L]])
  }
}
