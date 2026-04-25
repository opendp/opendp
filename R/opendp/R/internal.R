# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Construct a new ExtrinsicDistance.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `user_distance`.
#'
#' See `user_distance` for correct usage of this function.
#'
#' @concept internal
#' @param identifier A string description of the metric.
#' @param descriptor Additional constraints on the domain.
#' @return Metric
`_extrinsic_distance` <- function(
  identifier,
  descriptor = NULL
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("_extrinsic_distance", "internal", new_hashtab(
    list("identifier", "descriptor"),
    list(identifier, descriptor)
  ))

  # Call wrapper function.
  output <- .Call(
    "internal___extrinsic_distance",
    identifier, descriptor,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a new ExtrinsicDivergence, a privacy measure defined from a bindings language.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `user_divergence`.
#'
#' See `user_divergence` for correct usage and proof definition for this function.
#'
#' @concept internal
#' @param identifier A string description of the privacy measure.
#' @param descriptor Additional constraints on the privacy measure.
#' @return Measure
`_extrinsic_divergence` <- function(
  identifier,
  descriptor = NULL
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("_extrinsic_divergence", "internal", new_hashtab(
    list("identifier", "descriptor"),
    list(identifier, descriptor)
  ))

  # Call wrapper function.
  output <- .Call(
    "internal___extrinsic_divergence",
    identifier, descriptor,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a new ExtrinsicDomain.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `user_domain`.
#'
#' See `user_domain` for correct usage and proof definition for this function.
#'
#' @concept internal
#' @param identifier A string description of the data domain.
#' @param member A function used to test if a value is a member of the data domain.
#' @param descriptor Additional constraints on the domain.
#' @return Domain
`_extrinsic_domain` <- function(
  identifier,
  member,
  descriptor = NULL
) {
  # Standardize type arguments.
  .T.member <- rt_canon("bool")

  log_ <- new_constructor_log("_extrinsic_domain", "internal", new_hashtab(
    list("identifier", "member", "descriptor"),
    list(identifier, member, descriptor)
  ))

  # Call wrapper function.
  output <- .Call(
    "internal___extrinsic_domain",
    identifier, member, descriptor, .T.member,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a Measurement from user-defined callbacks.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `make_user_measurement`.
#'
#' See `make_user_measurement` for correct usage and proof definition for this function.
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AnyDomain`
#' * Output Type:    `AnyMetric`
#' * Input Metric:   `AnyMeasure`
#' * Output Measure: `AnyObject`
#'
#' @concept internal
#' @param input_domain A domain describing the set of valid inputs for the function.
#' @param input_metric The metric from which distances between adjacent inputs are measured.
#' @param output_measure The measure from which distances between adjacent output distributions are measured.
#' @param function_ A function mapping data from `input_domain` to a release of type `TO`.
#' @param privacy_map A function mapping distances from `input_metric` to `output_measure`.
#' @param .TO The data type of outputs from the function.
#' @return Measurement
`_make_measurement` <- function(
  input_domain,
  input_metric,
  output_measure,
  function_,
  privacy_map,
  .TO = "ExtrinsicObject"
) {
  # Standardize type arguments.
  .TO <- rt_parse(type_name = .TO)
  .T.function_ <- rt_canon(pass_through(.TO))
  .T.privacy_map <- rt_canon(measure_distance_type(output_measure))

  log_ <- new_constructor_log("_make_measurement", "internal", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "function", "privacy_map", "TO"),
    list(input_domain, input_metric, output_measure, function_, privacy_map, .TO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = "AnyDomain", inferred = rt_infer(input_domain))
  rt_assert_is_similar(expected = "AnyMetric", inferred = rt_infer(input_metric))
  rt_assert_is_similar(expected = "AnyMeasure", inferred = rt_infer(output_measure))

  # Call wrapper function.
  output <- .Call(
    "internal___make_measurement",
    input_domain, input_metric, output_measure, function_, privacy_map, .TO, .T.function_, .T.privacy_map,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a Transformation from user-defined callbacks.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `make_user_transformation`.
#'
#' See `make_user_transformation` for correct usage and proof definition for this function.
#'
#' @concept internal
#' @param input_domain A domain describing the set of valid inputs for the function.
#' @param input_metric The metric from which distances between adjacent inputs are measured.
#' @param output_domain A domain describing the set of valid outputs of the function.
#' @param output_metric The metric from which distances between outputs of adjacent inputs are measured.
#' @param function_ A function mapping data from `input_domain` to `output_domain`.
#' @param stability_map A function mapping distances from `input_metric` to `output_metric`.
#' @return Transformation
`_make_transformation` <- function(
  input_domain,
  input_metric,
  output_domain,
  output_metric,
  function_,
  stability_map
) {
  # Standardize type arguments.
  .T.function_ <- rt_canon(domain_carrier_type(output_domain))
  .T.stability_map <- rt_canon(metric_distance_type(output_metric))

  log_ <- new_constructor_log("_make_transformation", "internal", new_hashtab(
    list("input_domain", "input_metric", "output_domain", "output_metric", "function", "stability_map"),
    list(input_domain, input_metric, output_domain, output_metric, function_, stability_map)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = "AnyDomain", inferred = rt_infer(input_domain))
  rt_assert_is_similar(expected = "AnyMetric", inferred = rt_infer(input_metric))
  rt_assert_is_similar(expected = "AnyDomain", inferred = rt_infer(output_domain))
  rt_assert_is_similar(expected = "AnyMetric", inferred = rt_infer(output_metric))

  # Call wrapper function.
  output <- .Call(
    "internal___make_transformation",
    input_domain, input_metric, output_domain, output_metric, function_, stability_map, .T.function_, .T.stability_map,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a Function from a user-defined callback.
#' This is meant for internal use, as it does not require "honest-but-curious",
#' unlike `new_function`.
#'
#' See `new_function` for correct usage and proof definition for this function.
#'
#'
#' Required features: `contrib`
#'
#' [_new_pure_function in Rust documentation.](https://docs.rs/opendp/0.14.2-nightly.20260425.1/opendp/internal/fn._new_pure_function.html)
#'
#' @concept internal
#' @param function_ A function mapping data to a value of type `TO`
#' @param .TO Output Type
#' @return Function
`_new_pure_function` <- function(
  function_,
  .TO = "ExtrinsicObject"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TO <- rt_parse(type_name = .TO)
  .T.function_ <- rt_canon(pass_through(.TO))

  log_ <- new_constructor_log("_new_pure_function", "internal", new_hashtab(
    list("function", "TO"),
    list(function_, .TO)
  ))

  # Call wrapper function.
  output <- .Call(
    "internal___new_pure_function",
    function_, .TO, .T.function_,
    log_, PACKAGE = "opendp")
  output
}
