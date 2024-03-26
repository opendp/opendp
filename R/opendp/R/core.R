# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Eval the `function` with `arg`.
#'
#' @concept core
#' @param this Function to invoke.
#' @param arg Input data to supply to the measurement. A member of the measurement's input domain.
#' @param TI Input Type.
#' @return Any
#' @export
function_eval <- function(
  this,
  arg,
  TI = NULL
) {
  # Standardize type arguments.
  .T.arg <- parse_or_infer(TI, arg)

  log <- new_constructor_log("function_eval", "core", new_hashtab(
    list("this", "arg", "TI"),
    list(this, arg, TI)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__function_eval",
    this, arg, TI, rt_parse(.T.arg),
    log, PACKAGE = "opendp")
  output
}


#' Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
#'
#' @concept core
#' @param measurement Measurement to check the privacy relation of.
#' @param distance_in undocumented
#' @param distance_out undocumented
#' @return True indicates that the relation passed at the given distance.
#' @export
measurement_check <- function(
  measurement,
  distance_in,
  distance_out
) {
  # Standardize type arguments.
  .T.distance_in <- measurement_input_distance_type(measurement)
  .T.distance_out <- measurement_output_distance_type(measurement)

  log <- new_constructor_log("measurement_check", "core", new_hashtab(
    list("measurement", "distance_in", "distance_out"),
    list(measurement, distance_in, distance_out)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))
  rt_assert_is_similar(expected = .T.distance_out, inferred = rt_infer(distance_out))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_check",
    measurement, distance_in, distance_out, rt_parse(.T.distance_in), rt_parse(.T.distance_out),
    log, PACKAGE = "opendp")
  output
}


#' Get the function from a measurement.
#'
#' @concept core
#' @param this The measurement to retrieve the value from.
#' @return Function
#' @export
measurement_function <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_function", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_function",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input (carrier) data type of `this`.
#'
#' @concept core
#' @param this The measurement to retrieve the type from.
#' @return str
#' @export
measurement_input_carrier_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_input_carrier_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_carrier_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input distance type of `measurement`.
#'
#' @concept core
#' @param this The measurement to retrieve the type from.
#' @return str
#' @export
measurement_input_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_input_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_distance_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `measurement`.
#'
#' @concept core
#' @param this The measurement to retrieve the value from.
#' @return Domain
#' @export
measurement_input_domain <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_input_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_domain",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `measurement`.
#'
#' @concept core
#' @param this The measurement to retrieve the value from.
#' @return Metric
#' @export
measurement_input_metric <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_input_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_metric",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Invoke the `measurement` with `arg`. Returns a differentially private release.
#'
#' @concept core
#' @param this Measurement to invoke.
#' @param arg Input data to supply to the measurement. A member of the measurement's input domain.
#' @return Any
#' @export
measurement_invoke <- function(
  this,
  arg
) {
  # Standardize type arguments.
  .T.arg <- measurement_input_carrier_type(this)

  log <- new_constructor_log("measurement_invoke", "core", new_hashtab(
    list("this", "arg"),
    list(this, arg)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_invoke",
    this, arg, rt_parse(.T.arg),
    log, PACKAGE = "opendp")
  output
}


#' Use the `measurement` to map a given `d_in` to `d_out`.
#'
#' @concept core
#' @param measurement Measurement to check the map distances with.
#' @param distance_in Distance in terms of the input metric.
#' @return Any
#' @export
measurement_map <- function(
  measurement,
  distance_in
) {
  # Standardize type arguments.
  .T.distance_in <- measurement_input_distance_type(measurement)

  log <- new_constructor_log("measurement_map", "core", new_hashtab(
    list("measurement", "distance_in"),
    list(measurement, distance_in)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_map",
    measurement, distance_in, rt_parse(.T.distance_in),
    log, PACKAGE = "opendp")
  output
}


#' Get the output distance type of `measurement`.
#'
#' @concept core
#' @param this The measurement to retrieve the type from.
#' @return str
#' @export
measurement_output_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_output_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_output_distance_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the output domain from a `measurement`.
#'
#' @concept core
#' @param this The measurement to retrieve the value from.
#' @return Measure
#' @export
measurement_output_measure <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("measurement_output_measure", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_output_measure",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Invoke the `queryable` with `query`. Returns a differentially private release.
#'
#' @concept core
#' @param queryable Queryable to eval.
#' @param query Input data to supply to the measurement. A member of the measurement's input domain.
#' @return Any
#' @export
queryable_eval <- function(
  queryable,
  query
) {
  # Standardize type arguments.
  .T.query <- queryable_query_type(queryable)

  log <- new_constructor_log("queryable_eval", "core", new_hashtab(
    list("queryable", "query"),
    list(queryable, query)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.query, inferred = rt_infer(query))

  # Call wrapper function.
  output <- .Call(
    "core__queryable_eval",
    queryable, query, rt_parse(.T.query),
    log, PACKAGE = "opendp")
  output
}


#' Get the query type of `queryable`.
#'
#' @concept core
#' @param this The queryable to retrieve the query type from.
#' @return str
#' @export
queryable_query_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("queryable_query_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__queryable_query_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Check the privacy relation of the `measurement` at the given `d_in`, `d_out`
#'
#' @concept core
#' @param transformation undocumented
#' @param distance_in undocumented
#' @param distance_out undocumented
#' @return True indicates that the relation passed at the given distance.
#' @export
transformation_check <- function(
  transformation,
  distance_in,
  distance_out
) {
  # Standardize type arguments.
  .T.distance_in <- transformation_input_distance_type(transformation)
  .T.distance_out <- transformation_output_distance_type(transformation)

  log <- new_constructor_log("transformation_check", "core", new_hashtab(
    list("transformation", "distance_in", "distance_out"),
    list(transformation, distance_in, distance_out)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))
  rt_assert_is_similar(expected = .T.distance_out, inferred = rt_infer(distance_out))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_check",
    transformation, distance_in, distance_out, rt_parse(.T.distance_in), rt_parse(.T.distance_out),
    log, PACKAGE = "opendp")
  output
}


#' Get the function from a transformation.
#'
#' @concept core
#' @param this The transformation to retrieve the value from.
#' @return Function
#' @export
transformation_function <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_function", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_function",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input (carrier) data type of `this`.
#'
#' @concept core
#' @param this The transformation to retrieve the type from.
#' @return str
#' @export
transformation_input_carrier_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_input_carrier_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_carrier_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input distance type of `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the type from.
#' @return str
#' @export
transformation_input_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_input_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_distance_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the value from.
#' @return Domain
#' @export
transformation_input_domain <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_input_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_domain",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the value from.
#' @return Metric
#' @export
transformation_input_metric <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_input_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_metric",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Invoke the `transformation` with `arg`. Returns a differentially private release.
#'
#' @concept core
#' @param this Transformation to invoke.
#' @param arg Input data to supply to the transformation. A member of the transformation's input domain.
#' @return Any
#' @export
transformation_invoke <- function(
  this,
  arg
) {
  # Standardize type arguments.
  .T.arg <- transformation_input_carrier_type(this)

  log <- new_constructor_log("transformation_invoke", "core", new_hashtab(
    list("this", "arg"),
    list(this, arg)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_invoke",
    this, arg, rt_parse(.T.arg),
    log, PACKAGE = "opendp")
  output
}


#' Use the `transformation` to map a given `d_in` to `d_out`.
#'
#' @concept core
#' @param transformation Transformation to check the map distances with.
#' @param distance_in Distance in terms of the input metric.
#' @return Any
#' @export
transformation_map <- function(
  transformation,
  distance_in
) {
  # Standardize type arguments.
  .T.distance_in <- transformation_input_distance_type(transformation)

  log <- new_constructor_log("transformation_map", "core", new_hashtab(
    list("transformation", "distance_in"),
    list(transformation, distance_in)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_map",
    transformation, distance_in, rt_parse(.T.distance_in),
    log, PACKAGE = "opendp")
  output
}


#' Get the output distance type of `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the type from.
#' @return str
#' @export
transformation_output_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_output_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_distance_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the output domain from a `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the value from.
#' @return Domain
#' @export
transformation_output_domain <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_output_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_domain",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the output domain from a `transformation`.
#'
#' @concept core
#' @param this The transformation to retrieve the value from.
#' @return Metric
#' @export
transformation_output_metric <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("transformation_output_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_metric",
    this,
    log, PACKAGE = "opendp")
  output
}
