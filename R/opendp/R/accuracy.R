# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Convert a desired `accuracy` (tolerance) into a discrete gaussian noise scale at a statistical significance level `alpha`.
#'
#' [accuracy_to_discrete_gaussian_scale in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.accuracy_to_discrete_gaussian_scale.html)
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/v0.12.0/proofs/rust/src/accuracy/accuracy_to_discrete_gaussian_scale.pdf)
#'
#' @concept accuracy
#' @param accuracy Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `accuracy` and `alpha`
#' @export
accuracy_to_discrete_gaussian_scale <- function(
  accuracy,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = accuracy)

  log <- new_constructor_log("accuracy_to_discrete_gaussian_scale", "accuracy", new_hashtab(
    list("accuracy", "alpha", "T"),
    list(accuracy, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(accuracy))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__accuracy_to_discrete_gaussian_scale",
    accuracy, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a desired `accuracy` (tolerance) into a discrete Laplacian noise scale at a statistical significance level `alpha`.
#'
#' [accuracy_to_discrete_laplacian_scale in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.accuracy_to_discrete_laplacian_scale.html)
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/v0.12.0/proofs/rust/src/accuracy/accuracy_to_discrete_laplacian_scale.pdf)
#'
#' @concept accuracy
#' @param accuracy Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `accuracy` and `alpha`
#' @return Discrete laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
#' @export
accuracy_to_discrete_laplacian_scale <- function(
  accuracy,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = accuracy)

  log <- new_constructor_log("accuracy_to_discrete_laplacian_scale", "accuracy", new_hashtab(
    list("accuracy", "alpha", "T"),
    list(accuracy, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(accuracy))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__accuracy_to_discrete_laplacian_scale",
    accuracy, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a desired `accuracy` (tolerance) into a gaussian noise scale at a statistical significance level `alpha`.
#'
#' [accuracy_to_gaussian_scale in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.accuracy_to_gaussian_scale.html)
#'
#' @concept accuracy
#' @param accuracy Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `accuracy` and `alpha`
#' @export
accuracy_to_gaussian_scale <- function(
  accuracy,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = accuracy)

  log <- new_constructor_log("accuracy_to_gaussian_scale", "accuracy", new_hashtab(
    list("accuracy", "alpha", "T"),
    list(accuracy, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(accuracy))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__accuracy_to_gaussian_scale",
    accuracy, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a desired `accuracy` (tolerance) into a Laplacian noise scale at a statistical significance level `alpha`.
#'
#' [accuracy_to_laplacian_scale in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.accuracy_to_laplacian_scale.html)
#'
#' @concept accuracy
#' @param accuracy Desired accuracy. A tolerance for how far values may diverge from the input to the mechanism.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `accuracy` and `alpha`
#' @return Laplacian noise scale that meets the `accuracy` requirement at a given level-`alpha`.
#' @export
accuracy_to_laplacian_scale <- function(
  accuracy,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = accuracy)

  log <- new_constructor_log("accuracy_to_laplacian_scale", "accuracy", new_hashtab(
    list("accuracy", "alpha", "T"),
    list(accuracy, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(accuracy))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__accuracy_to_laplacian_scale",
    accuracy, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a discrete gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
#'
#' [discrete_gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.discrete_gaussian_scale_to_accuracy.html)
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/v0.12.0/proofs/rust/src/accuracy/discrete_gaussian_scale_to_accuracy.pdf)
#'
#' @concept accuracy
#' @param scale Gaussian noise scale.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `scale` and `alpha`
#' @export
discrete_gaussian_scale_to_accuracy <- function(
  scale,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = scale)

  log <- new_constructor_log("discrete_gaussian_scale_to_accuracy", "accuracy", new_hashtab(
    list("scale", "alpha", "T"),
    list(scale, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__discrete_gaussian_scale_to_accuracy",
    scale, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
#'
#' \eqn{\alpha = P[Y \ge accuracy]}, where \eqn{Y = | X - z |}, and \eqn{X \sim \mathcal{L}_{Z}(0, scale)}.
#' That is, \eqn{X} is a discrete Laplace random variable and \eqn{Y} is the distribution of the errors.
#'
#' This function returns a float accuracy.
#' You can take the floor without affecting the coverage probability.
#'
#' [discrete_laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.discrete_laplacian_scale_to_accuracy.html)
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/v0.12.0/proofs/rust/src/accuracy/discrete_laplacian_scale_to_accuracy.pdf)
#'
#' @concept accuracy
#' @param scale Discrete Laplacian noise scale.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `scale` and `alpha`
#' @export
discrete_laplacian_scale_to_accuracy <- function(
  scale,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = scale)

  log <- new_constructor_log("discrete_laplacian_scale_to_accuracy", "accuracy", new_hashtab(
    list("scale", "alpha", "T"),
    list(scale, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__discrete_laplacian_scale_to_accuracy",
    scale, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
#'
#' [gaussian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.gaussian_scale_to_accuracy.html)
#'
#' @concept accuracy
#' @param scale Gaussian noise scale.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `scale` and `alpha`
#' @export
gaussian_scale_to_accuracy <- function(
  scale,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = scale)

  log <- new_constructor_log("gaussian_scale_to_accuracy", "accuracy", new_hashtab(
    list("scale", "alpha", "T"),
    list(scale, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__gaussian_scale_to_accuracy",
    scale, alpha, .T,
    log, PACKAGE = "opendp")
  output
}


#' Convert a Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.
#'
#' [laplacian_scale_to_accuracy in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/accuracy/fn.laplacian_scale_to_accuracy.html)
#'
#' @concept accuracy
#' @param scale Laplacian noise scale.
#' @param alpha Statistical significance, level-`alpha`, or (1. - `alpha`)100% confidence. Must be within (0, 1].
#' @param .T Data type of `scale` and `alpha`
#' @export
laplacian_scale_to_accuracy <- function(
  scale,
  alpha,
  .T = NULL
) {
  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = scale)

  log <- new_constructor_log("laplacian_scale_to_accuracy", "accuracy", new_hashtab(
    list("scale", "alpha", "T"),
    list(scale, alpha, .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "accuracy__laplacian_scale_to_accuracy",
    scale, alpha, .T,
    log, PACKAGE = "opendp")
  output
}
