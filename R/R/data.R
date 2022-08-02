#' Convert a data slice to an object
#'
#' @useDynLib opendp slice_as_object__wrapper
slice_as_object <- function(data) {
  .Call(slice_as_object__wrapper, data, PACKAGE = 'opendp')
}