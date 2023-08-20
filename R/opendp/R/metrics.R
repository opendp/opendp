# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Construct an instance of the `AbsoluteDistance` metric.
#'
#' [absolute_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.absolute_distance.html)
#'
#' @param .T undocumented
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
#'
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
#'
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
#'
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
#'
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
#' @param .T undocumented
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
#' @param .T undocumented
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


#' Construct an instance of the `LInfDiffDistance` metric.
#'
#' [linf_diff_distance in Rust documentation.](https://docs.rs/opendp/latest/opendp/metrics/fn.linf_diff_distance.html)
#'
#' @param .T undocumented
#' @export
linf_diff_distance <- function(
    .T
) {
    # Standardize type arguments.
    .T <- rt_parse(type_name = .T)

    log <- new_constructor_log("linf_diff_distance", "metrics", new_hashtab(
        list("T"),
        list(.T)
    ))

    # Call wrapper function.
    output <- .Call(
        "metrics__linf_diff_distance",
        .T,
        log, PACKAGE = "opendp")
    output
}


#' Debug a `metric`.
#'
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


#' Construct an instance of the `SymmetricDistance` metric.
#'
#'
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
