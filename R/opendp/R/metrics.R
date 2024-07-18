# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Construct an instance of the `AbsoluteDistance` metric.
#'
#' [absolute_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.absolute_distance.html)
#'
#' @concept metrics
#' @param .T undocumented
#' @return Metric
#' @export
absolute_distance <- function(
  .T
) {
  # Standardize type arguments.
  .T <- rt_parse(type_name = .T)

  log <- new_constructor_log("absolute_distance", "metrics", new_hashtab(
    list("T"),
    list(.T)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__absolute_distance",
    .T,
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `ChangeOneDistance` metric.
#'
#' @concept metrics
#'
#' @return Metric
#' @export
change_one_distance <- function(

) {
  # No type arguments to standardize.
  log <- new_constructor_log("change_one_distance", "metrics", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__change_one_distance",
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `DiscreteDistance` metric.
#'
#' @concept metrics
#'
#' @return Metric
#' @export
discrete_distance <- function(

) {
  # No type arguments to standardize.
  log <- new_constructor_log("discrete_distance", "metrics", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__discrete_distance",
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `HammingDistance` metric.
#'
#' @concept metrics
#'
#' @return Metric
#' @export
hamming_distance <- function(

) {
  # No type arguments to standardize.
  log <- new_constructor_log("hamming_distance", "metrics", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__hamming_distance",
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `InsertDeleteDistance` metric.
#'
#' @concept metrics
#'
#' @return Metric
#' @export
insert_delete_distance <- function(

) {
  # No type arguments to standardize.
  log <- new_constructor_log("insert_delete_distance", "metrics", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__insert_delete_distance",
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `L1Distance` metric.
#'
#' [l1_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.l1_distance.html)
#'
#' @concept metrics
#' @param .T undocumented
#' @return Metric
#' @export
l1_distance <- function(
  .T
) {
  # Standardize type arguments.
  .T <- rt_parse(type_name = .T)

  log <- new_constructor_log("l1_distance", "metrics", new_hashtab(
    list("T"),
    list(.T)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__l1_distance",
    .T,
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `L2Distance` metric.
#'
#' [l2_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.l2_distance.html)
#'
#' @concept metrics
#' @param .T undocumented
#' @return Metric
#' @export
l2_distance <- function(
  .T
) {
  # Standardize type arguments.
  .T <- rt_parse(type_name = .T)

  log <- new_constructor_log("l2_distance", "metrics", new_hashtab(
    list("T"),
    list(.T)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__l2_distance",
    .T,
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `LInfDistance` metric.
#'
#' [linf_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.linf_distance.html)
#'
#' @concept metrics
#' @param monotonic set to true if non-monotonicity implies infinite distance
#' @param .T The type of the distance.
#' @return Metric
#' @export
linf_distance <- function(
  .T,
  monotonic = FALSE
) {
  # Standardize type arguments.
  .T <- rt_parse(type_name = .T)

  log <- new_constructor_log("linf_distance", "metrics", new_hashtab(
    list("monotonic", "T"),
    list(unbox2(monotonic), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = bool, inferred = rt_infer(monotonic))

  # Call wrapper function.
  output <- .Call(
    "metrics__linf_distance",
    monotonic, .T,
    log, PACKAGE = "opendp")
  output
}


#' Debug a `metric`.
#'
#' @concept metrics
#' @param this The metric to debug (stringify).
#' @return str
#' @export
metric_debug <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("metric_debug", "metrics", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__metric_debug",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the distance type of a `metric`.
#'
#' @concept metrics
#' @param this The metric to retrieve the distance type from.
#' @return str
#' @export
metric_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("metric_distance_type", "metrics", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__metric_distance_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Get the type of a `metric`.
#'
#' @concept metrics
#' @param this The metric to retrieve the type from.
#' @return str
#' @export
metric_type <- function(
  this
) {
  # No type arguments to standardize.
  log <- new_constructor_log("metric_type", "metrics", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__metric_type",
    this,
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `PartitionDistance` metric.
#'
#' [partition_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.partition_distance.html)
#'
#' @concept metrics
#' @param metric The metric used to compute distance between partitions.
#' @return Metric
#' @export
partition_distance <- function(
  metric
) {
  # No type arguments to standardize.
  log <- new_constructor_log("partition_distance", "metrics", new_hashtab(
    list("metric"),
    list(metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__partition_distance",
    metric,
    log, PACKAGE = "opendp")
  output
}


#' Construct an instance of the `SymmetricDistance` metric.
#'
#' @concept metrics
#'
#' @return Metric
#' @export
symmetric_distance <- function(

) {
  # No type arguments to standardize.
  log <- new_constructor_log("symmetric_distance", "metrics", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "metrics__symmetric_distance",
    log, PACKAGE = "opendp")
  output
}


#' Construct a new UserDistance.
#' Any two instances of an UserDistance are equal if their string descriptors are equal.
#' Requires `honest-but-curious`: The OpenDP Library cannot verify that a user-defined metric forms a metric space when paired with any given domain.
#'
#' @concept metrics
#' @param descriptor A string description of the metric.
#' @return Metric
#' @export
user_distance <- function(
  descriptor
) {
  assert_features("honest-but-curious")

  # No type arguments to standardize.
  log <- new_constructor_log("user_distance", "metrics", new_hashtab(
    list("descriptor"),
    list(unbox2(descriptor))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = String, inferred = rt_infer(descriptor))

  # Call wrapper function.
  output <- .Call(
    "metrics__user_distance",
    descriptor,
    log, PACKAGE = "opendp")
  output
}
