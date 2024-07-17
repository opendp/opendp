# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Internal function. Retrieve the type descriptor string of an AnyObject.
#'
#' @concept data
#' @param this A pointer to the AnyObject.
#' @return str
object_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("object_type", "data", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "data__object_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Internal function. Use an SMDCurve to find epsilon at a given `delta`.
#'
#' @concept data
#' @param curve The SMDCurve.
#' @param delta What to fix delta to compute epsilon.
#' @return Epsilon at a given `delta`.
smd_curve_epsilon <- function(
  curve,
  delta
) {
  # Standardize type arguments.
  .T.delta <- get_atom(object_type(curve))

  log <- new_constructor_log("smd_curve_epsilon", "data", new_hashtab(
    list("curve", "delta"),
    list(curve, delta)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.delta, inferred = rt_infer(delta))

  # Call wrapper function.
  output <- .Call(
    "data__smd_curve_epsilon",
    curve, delta, rt_parse(.T.delta),
    log, PACKAGE = "opendp")
  output
}
