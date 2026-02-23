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
  log_ <- new_constructor_log("object_type", "data", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "data__object_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Internal function. Use a PrivacyProfile to find epsilon at a given `epsilon`.
#'
#' @concept data
#' @param curve The PrivacyProfile.
#' @param epsilon What to fix epsilon to compute delta.
#' @return Delta at a given `epsilon`.
privacy_profile_delta <- function(
  curve,
  epsilon
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("privacy_profile_delta", "data", new_hashtab(
    list("curve", "epsilon"),
    list(curve, unbox2(epsilon))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(epsilon))

  # Call wrapper function.
  output <- .Call(
    "data__privacy_profile_delta",
    curve, epsilon,
    log_, PACKAGE = "opendp")
  output
}


#' Internal function. Use an PrivacyProfile to find epsilon at a given `delta`.
#'
#' @concept data
#' @param profile The PrivacyProfile.
#' @param delta What to fix delta to compute epsilon.
#' @return Epsilon at a given `delta`.
privacy_profile_epsilon <- function(
  profile,
  delta
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("privacy_profile_epsilon", "data", new_hashtab(
    list("profile", "delta"),
    list(profile, unbox2(delta))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(delta))

  # Call wrapper function.
  output <- .Call(
    "data__privacy_profile_epsilon",
    profile, delta,
    log_, PACKAGE = "opendp")
  output
}
