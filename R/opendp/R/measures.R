# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Construct an instance of the `FixedSmoothedMaxDivergence` measure.
#'
#' [fixed_smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.fixed_smoothed_max_divergence.html)
#'
#' @param .T undocumented
#' @export
fixed_smoothed_max_divergence <- function(
    .T
) {
    # Standardize type arguments.
    .T <- rt_parse(type_name = .T)

    log <- new_constructor_log("fixed_smoothed_max_divergence", "measures", new_hashtab(
        list("T"),
        list(.T)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__fixed_smoothed_max_divergence",
        .T,
        log, PACKAGE = "opendp")
    output
}


#' Construct an instance of the `MaxDivergence` measure.
#'
#' [max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.max_divergence.html)
#'
#' @param .T undocumented
#' @export
max_divergence <- function(
    .T
) {
    # Standardize type arguments.
    .T <- rt_parse(type_name = .T)

    log <- new_constructor_log("max_divergence", "measures", new_hashtab(
        list("T"),
        list(.T)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__max_divergence",
        .T,
        log, PACKAGE = "opendp")
    output
}


#' Debug a `measure`.
#'
#' @param this The measure to debug (stringify).
#' @return str
#' @export
measure_debug <- function(
    this
) {
    # No type arguments to standardize.
    log <- new_constructor_log("measure_debug", "measures", new_hashtab(
        list("this"),
        list(this)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__measure_debug",
        this,
        log, PACKAGE = "opendp")
    output
}


#' Get the distance type of a `measure`.
#'
#' @param this The measure to retrieve the distance type from.
#' @return str
#' @export
measure_distance_type <- function(
    this
) {
    # No type arguments to standardize.
    log <- new_constructor_log("measure_distance_type", "measures", new_hashtab(
        list("this"),
        list(this)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__measure_distance_type",
        this,
        log, PACKAGE = "opendp")
    output
}


#' Get the type of a `measure`.
#'
#' @param this The measure to retrieve the type from.
#' @return str
#' @export
measure_type <- function(
    this
) {
    # No type arguments to standardize.
    log <- new_constructor_log("measure_type", "measures", new_hashtab(
        list("this"),
        list(this)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__measure_type",
        this,
        log, PACKAGE = "opendp")
    output
}


#' Construct an instance of the `SmoothedMaxDivergence` measure.
#'
#' [smoothed_max_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.smoothed_max_divergence.html)
#'
#' @param .T undocumented
#' @export
smoothed_max_divergence <- function(
    .T
) {
    # Standardize type arguments.
    .T <- rt_parse(type_name = .T)

    log <- new_constructor_log("smoothed_max_divergence", "measures", new_hashtab(
        list("T"),
        list(.T)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__smoothed_max_divergence",
        .T,
        log, PACKAGE = "opendp")
    output
}


#' Construct an instance of the `ZeroConcentratedDivergence` measure.
#'
#' [zero_concentrated_divergence in Rust documentation.](https://docs.rs/opendp/latest/opendp/measures/fn.zero_concentrated_divergence.html)
#'
#' @param .T undocumented
#' @export
zero_concentrated_divergence <- function(
    .T
) {
    # Standardize type arguments.
    .T <- rt_parse(type_name = .T)

    log <- new_constructor_log("zero_concentrated_divergence", "measures", new_hashtab(
        list("T"),
        list(.T)
    ))

    # Call wrapper function.
    output <- .Call(
        "measures__zero_concentrated_divergence",
        .T,
        log, PACKAGE = "opendp")
    output
}
