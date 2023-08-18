# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Internal function. Free the memory associated with `this`, a string.
#' Used to clean up after the type getter functions.
#'
#' @param this undocumented
extrinsic_object_free <- function(
    this
) {
    # No type arguments to standardize.
    log <- new_constructor_log("extrinsic_object_free", "data", new_hashtab(
        list("this"),
        list(this)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = ExtrinsicObject, inferred = rt_infer(this))

    # Call wrapper function.
    output <- .Call(
        "data__extrinsic_object_free",
        this,
        log, PACKAGE = "opendpbase")
    output
}


#' Internal function. Populate the buffer behind `ptr` with `len` random bytes
#' sampled from a cryptographically secure RNG.
#'
#' @param ptr undocumented
#' @param len undocumented
#' @return bool
fill_bytes <- function(
    ptr,
    len
) {
    # No type arguments to standardize.
    log <- new_constructor_log("fill_bytes", "data", new_hashtab(
        list("ptr", "len"),
        list(ptr, unbox2(len))
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = u8, inferred = rt_infer(ptr))
    rt_assert_is_similar(expected = usize, inferred = rt_infer(len))

    # Call wrapper function.
    output <- .Call(
        "data__fill_bytes",
        ptr, len,
        log, PACKAGE = "opendpbase")
    output
}


#' Internal function. Retrieve the type descriptor string of an AnyObject.
#'
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
        log, PACKAGE = "opendpbase")
    output
}


#' Internal function. Use an SMDCurve to find epsilon at a given `delta`.
#'
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
        log, PACKAGE = "opendpbase")
    output
}


#' Internal function. Convert the AnyObject to a string representation.
#'
#' @param this The AnyObject to convert to a string representation.
#' @return str
to_string <- function(
    this
) {
    # No type arguments to standardize.
    log <- new_constructor_log("to_string", "data", new_hashtab(
        list("this"),
        list(this)
    ))

    # Call wrapper function.
    output <- .Call(
        "data__to_string",
        this,
        log, PACKAGE = "opendpbase")
    output
}
