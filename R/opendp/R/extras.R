#' @include mod.R
NULL

supports_partial <- function(constructor) {
  arg_names <- names(formals(constructor))
  length(arg_names) > 1 && identical(arg_names[1:2], c("input_domain", "input_metric"))
}

#' Convert a `make_` constructor into a `then_` constructor.
#'
#' This mirrors the Python extras utility for constructors whose first two
#' arguments are `input_domain` and `input_metric`.
#'
#' @concept extras
#' @param constructor A constructor whose first two arguments are `input_domain` and `input_metric`.
#' @return A partial constructor suitable for `|>` chaining.
#' @export
to_then <- function(constructor) {
  if (!supports_partial(constructor)) {
    stop(
      "the first two arguments of the constructor must be input_domain and input_metric",
      call. = FALSE
    )
  }
  constructor_name <- deparse(substitute(constructor))
  then_name <- sub("^make_", "then_", constructor_name)

  function(lhs, ...) {
    rhs <- constructor(output_domain(lhs), output_metric(lhs), ...)
    make_chain_dyn(
      rhs,
      lhs,
      new_constructor_log(
        then_name,
        "extras",
        new_hashtab(list(), list())
      )
    )
  }
}

#' Compose a measurement with a postprocessing function.
#'
#' This provides Python-like ergonomics for chaining a pure function after a
#' measurement release.
#'
#' @concept extras
#' @param lhs A measurement to postprocess.
#' @param f A pure postprocessing function applied to the release.
#' @param .TO The runtime type of the postprocessed output.
#' @return A composed measurement.
#' @export
then_postprocess <- function(lhs, f, .TO = "ExtrinsicObject") {
  make_chain_pm(
    postprocess1 = new_function(f, .TO = .TO),
    measurement0 = lhs
  )
}
