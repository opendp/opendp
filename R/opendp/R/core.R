# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Eval the `function` with `arg`.
#'
#' @concept core
#' @param this Function to invoke.
#' @param arg Input data to supply to the measurement. A member of the measurement's input domain.
#' @param TI Input Type.
#' @export
function_eval <- function(
  this,
  arg,
  TI = NULL
) {
  # Standardize type arguments.
  .T.arg <- rt_canon(parse_or_infer(TI, arg))

  log_ <- new_constructor_log("function_eval", "core", new_hashtab(
    list("this", "arg", "TI"),
    list(this, arg, TI)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__function_eval",
    this, arg, TI, .T.arg,
    log_, PACKAGE = "opendp")
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
  .T.distance_in <- rt_canon(measurement_input_distance_type(measurement))
  .T.distance_out <- rt_canon(measurement_output_distance_type(measurement))

  log_ <- new_constructor_log("measurement_check", "core", new_hashtab(
    list("measurement", "distance_in", "distance_out"),
    list(measurement, distance_in, distance_out)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))
  rt_assert_is_similar(expected = .T.distance_out, inferred = rt_infer(distance_out))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_check",
    measurement, distance_in, distance_out, .T.distance_in, .T.distance_out,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_function", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_function",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_input_carrier_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_carrier_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_input_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_distance_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_input_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_domain",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_input_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_input_metric",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Invoke the `measurement` with `arg`. Returns a differentially private release.
#'
#' @concept core
#' @param this Measurement to invoke.
#' @param arg Input data to supply to the measurement. A member of the measurement's input domain.
#' @export
measurement_invoke <- function(
  this,
  arg
) {
  # Standardize type arguments.
  .T.arg <- rt_canon(measurement_input_carrier_type(this))

  log_ <- new_constructor_log("measurement_invoke", "core", new_hashtab(
    list("this", "arg"),
    list(this, arg)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_invoke",
    this, arg, .T.arg,
    log_, PACKAGE = "opendp")
  output
}


#' Use the `measurement` to map a given `d_in` to `d_out`.
#'
#' @concept core
#' @param measurement Measurement to check the map distances with.
#' @param distance_in Distance in terms of the input metric.
#' @export
measurement_map <- function(
  measurement,
  distance_in
) {
  # Standardize type arguments.
  .T.distance_in <- rt_canon(measurement_input_distance_type(measurement))

  log_ <- new_constructor_log("measurement_map", "core", new_hashtab(
    list("measurement", "distance_in"),
    list(measurement, distance_in)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_map",
    measurement, distance_in, .T.distance_in,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_output_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_output_distance_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("measurement_output_measure", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__measurement_output_measure",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a Function from a user-defined callback.
#' Can be used to build a postprocessor.
#'
#'
#' Required features: `contrib`, `honest-but-curious`
#'
#' [new_function in Rust documentation.](https://docs.rs/opendp/0.14.2-nightly.20260503.1/opendp/core/struct.Function.html)
#'
#' **Why honest-but-curious?:**
#'
#' An OpenDP `function` must satisfy two criteria.
#' These invariants about functions are necessary to show correctness of other algorithms.
#'
#' First, `function` must not use global state.
#' For instance, a postprocessor that accesses the system clock time
#' can be used to build a measurement that reveals elapsed execution time,
#' which escalates a side-channel vulnerability into a direct vulnerability.
#'
#' Secondly, `function` must only raise data-independent exceptions.
#' For instance, raising an exception with the value of a DP release will both
#' reveal the DP output and cancel the computation, potentially avoiding privacy accounting.
#'
#' @concept core
#' @param function_ A function mapping data to a value of type `TO`
#' @param .TO Output Type
#' @return Function
#' @export
new_function <- function(
  function_,
  .TO
) {
  assert_features("contrib", "honest-but-curious")

  # Standardize type arguments.
  .TO <- rt_parse(type_name = .TO)
  .T.function_ <- rt_canon(pass_through(.TO))

  log_ <- new_constructor_log("new_function", "core", new_hashtab(
    list("function", "TO"),
    list(function_, .TO)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__new_function",
    function_, .TO, .T.function_,
    log_, PACKAGE = "opendp")
  output
}


#' Construct a queryable from a user-defined transition function.
#'
#'
#' Required features: `contrib`
#'
#' @concept core
#' @param transition A transition function taking a reference to self, a query, and an internal/external indicator
#' @param .Q Query Type
#' @param .A Output Type
#' @export
new_queryable <- function(
  transition,
  .Q = "ExtrinsicObject",
  .A = "ExtrinsicObject"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .Q <- rt_parse(type_name = .Q, generics = list(".A"))
  .A <- rt_parse(type_name = .A, generics = list(".Q"))
  .Q <- rt_substitute(.Q, .A = .A)
  .A <- rt_substitute(.A, .Q = .Q)
  .T.transition <- rt_canon(pass_through(.A))

  log_ <- new_constructor_log("new_queryable", "core", new_hashtab(
    list("transition", "Q", "A"),
    list(transition, .Q, .A)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__new_queryable",
    transition, .Q, .A, .T.transition,
    log_, PACKAGE = "opendp")
  output
}


#' Get the input (carrier) data type of `this`.
#'
#' @concept core
#' @param this The odometer to retrieve the type from.
#' @return str
#' @export
odometer_input_carrier_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_input_carrier_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_input_carrier_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `odometer`.
#'
#' @concept core
#' @param this The odometer to retrieve the value from.
#' @return Domain
#' @export
odometer_input_domain <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_input_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_input_domain",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Get the input domain from a `odometer`.
#'
#' @concept core
#' @param this The odometer to retrieve the value from.
#' @return Metric
#' @export
odometer_input_metric <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_input_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_input_metric",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Invoke the `odometer` with `arg`. Returns a differentially private release.
#'
#' @concept core
#' @param this Odometer to invoke.
#' @param arg Input data to supply to the odometer. A member of the odometer's input domain.
#' @export
odometer_invoke <- function(
  this,
  arg
) {
  # Standardize type arguments.
  .T.arg <- rt_canon(odometer_input_carrier_type(this))

  log_ <- new_constructor_log("odometer_invoke", "core", new_hashtab(
    list("this", "arg"),
    list(this, arg)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_invoke",
    this, arg, .T.arg,
    log_, PACKAGE = "opendp")
  output
}


#' Get the output domain from a `odometer`.
#'
#' @concept core
#' @param this The odometer to retrieve the value from.
#' @return Measure
#' @export
odometer_output_measure <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_output_measure", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_output_measure",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Eval the odometer `queryable` with an invoke `query`.
#'
#' @concept core
#' @param queryable Queryable to eval.
#' @param query Invoke query to supply to the queryable.
#' @export
odometer_queryable_invoke <- function(
  queryable,
  query
) {
  # Standardize type arguments.
  .T.query <- rt_canon(odometer_queryable_invoke_type(queryable))

  log_ <- new_constructor_log("odometer_queryable_invoke", "core", new_hashtab(
    list("queryable", "query"),
    list(queryable, query)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.query, inferred = rt_infer(query))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_queryable_invoke",
    queryable, query, .T.query,
    log_, PACKAGE = "opendp")
  output
}


#' Get the invoke query type of an odometer `queryable`.
#'
#' @concept core
#' @param this The queryable to retrieve the query type from.
#' @return str
#' @export
odometer_queryable_invoke_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_queryable_invoke_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_queryable_invoke_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Retrieve the privacy loss of an odometer `queryable`.
#'
#' @concept core
#' @param queryable Queryable to eval.
#' @param d_in Maximum distance between adjacent inputs in the input domain.
#' @export
odometer_queryable_privacy_loss <- function(
  queryable,
  d_in
) {
  # Standardize type arguments.
  .T.d_in <- rt_canon(odometer_queryable_privacy_loss_type(queryable))

  log_ <- new_constructor_log("odometer_queryable_privacy_loss", "core", new_hashtab(
    list("queryable", "d_in"),
    list(queryable, d_in)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.d_in, inferred = rt_infer(d_in))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_queryable_privacy_loss",
    queryable, d_in, .T.d_in,
    log_, PACKAGE = "opendp")
  output
}


#' Get the map query type of an odometer `queryable`.
#'
#' @concept core
#' @param this The queryable to retrieve the query type from.
#' @return str
#' @export
odometer_queryable_privacy_loss_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("odometer_queryable_privacy_loss_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__odometer_queryable_privacy_loss_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Eval the `queryable` with `query`. Returns a differentially private release.
#'
#' @concept core
#' @param queryable Queryable to eval.
#' @param query The input to the queryable.
#' @export
queryable_eval <- function(
  queryable,
  query
) {
  # Standardize type arguments.
  .T.query <- rt_canon(queryable_query_type(queryable))

  log_ <- new_constructor_log("queryable_eval", "core", new_hashtab(
    list("queryable", "query"),
    list(queryable, query)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.query, inferred = rt_infer(query))

  # Call wrapper function.
  output <- .Call(
    "core__queryable_eval",
    queryable, query, .T.query,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("queryable_query_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__queryable_query_type",
    this,
    log_, PACKAGE = "opendp")
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
  .T.distance_in <- rt_canon(transformation_input_distance_type(transformation))
  .T.distance_out <- rt_canon(transformation_output_distance_type(transformation))

  log_ <- new_constructor_log("transformation_check", "core", new_hashtab(
    list("transformation", "distance_in", "distance_out"),
    list(transformation, distance_in, distance_out)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))
  rt_assert_is_similar(expected = .T.distance_out, inferred = rt_infer(distance_out))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_check",
    transformation, distance_in, distance_out, .T.distance_in, .T.distance_out,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_function", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_function",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_input_carrier_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_carrier_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_input_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_distance_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_input_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_domain",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_input_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_input_metric",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Invoke the `transformation` with `arg`. Returns a differentially private release.
#'
#' @concept core
#' @param this Transformation to invoke.
#' @param arg Input data to supply to the transformation. A member of the transformation's input domain.
#' @export
transformation_invoke <- function(
  this,
  arg
) {
  # Standardize type arguments.
  .T.arg <- rt_canon(transformation_input_carrier_type(this))

  log_ <- new_constructor_log("transformation_invoke", "core", new_hashtab(
    list("this", "arg"),
    list(this, arg)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.arg, inferred = rt_infer(arg))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_invoke",
    this, arg, .T.arg,
    log_, PACKAGE = "opendp")
  output
}


#' Use the `transformation` to map a given `d_in` to `d_out`.
#'
#' @concept core
#' @param transformation Transformation to check the map distances with.
#' @param distance_in Distance in terms of the input metric.
#' @export
transformation_map <- function(
  transformation,
  distance_in
) {
  # Standardize type arguments.
  .T.distance_in <- rt_canon(transformation_input_distance_type(transformation))

  log_ <- new_constructor_log("transformation_map", "core", new_hashtab(
    list("transformation", "distance_in"),
    list(transformation, distance_in)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.distance_in, inferred = rt_infer(distance_in))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_map",
    transformation, distance_in, .T.distance_in,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_output_distance_type", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_distance_type",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_output_domain", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_domain",
    this,
    log_, PACKAGE = "opendp")
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
  log_ <- new_constructor_log("transformation_output_metric", "core", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "core__transformation_output_metric",
    this,
    log_, PACKAGE = "opendp")
  output
}
