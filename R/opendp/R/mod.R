normalize_features <- function(features) {
  normalized <- vapply(
    features,
    function(feature) {
      if (identical(feature, "floating-point")) {
        warning("\"floating-point\" is deprecated. Use \"idealized-numerics\" instead.", call. = FALSE)
        return("idealized-numerics")
      }
      feature
    },
    FUN.VALUE = character(1),
    USE.NAMES = FALSE
  )

  as.list(normalized)
}

assert_features <- function(...) {
  for (feature in normalize_features(list(...))) {
    if (!feature %in% getOption("opendp_features")) {
      stop("Attempted to use function that requires ", feature, " but ", feature, " is not enabled. See https://github.com/opendp/opendp/discussions/304, then call enable_features(\"", feature, "\")", call. = FALSE)
    }
  }
}

#' Enable features for the opendp package.
#'
#' See https://github.com/opendp/opendp/discussions/304 for available features.
#'
#' @concept mod
#' @param ... features to enable
#' @export
enable_features <- function(...) {
  options(opendp_features = union(getOption("opendp_features"), normalize_features(list(...))))
}

#' Disable features in the opendp package.
#'
#' @concept mod
#' @param ... features to disable
#' @export
disable_features <- function(...) {
  features <- getOption("opendp_features")
  options(opendp_features = features[!features %in% normalize_features(list(...))])
}

is_space <- function(x) {
  inherits(x, "list") && length(x) == 2 && inherits(x[[1]], "domain") && inherits(x[[2]], "metric")
}

output_domain <- function(x) {
  if (inherits(x, "transformation")) {
    return(x("output_domain"))
  }

  if (is_space(x)) {
    return(x[[1]])
  }

  stop("expected a transformation or metric space. Got ", class(x), call. = FALSE)
}

output_metric <- function(x) {
  if (inherits(x, "transformation")) {
    return(x("output_metric"))
  }

  if (is_space(x)) {
    return(x[[2]])
  }

  stop("expected a transformation or metric space", call. = FALSE)
}

make_chain_dyn <- function(rhs, lhs, log) {
  if (is_space(lhs)) {
    return(rhs)
  }
  if (inherits(rhs, "transformation")) {
    if (inherits(lhs, "transformation")) {
      return(new_transformation(make_chain_tt(rhs, lhs)("ptr"), log))
    }
    rhs <- rhs("function")
  }
  if (inherits(rhs, "measurement")) {
    return(new_measurement(make_chain_mt(rhs, lhs)("ptr"), log))
  }
  if (inherits(rhs, "opendp_function")) {
    return(new_measurement(make_chain_pm(rhs, lhs)("ptr"), log))
  }
  stop("cannot chain lhs and rhs: ", class(lhs), ", ", class(rhs), call. = FALSE)
}

#' new transformation
#'
#' @concept mod
#' @param ptr pointer to the transformation struct
#' @param log call history
new_transformation <- function(ptr, log) {
  transformation <- function(attr, arg, d_in, d_out) {
    if (missing(attr) + missing(arg) + missing(d_in) != 2) {
      stop("expected exactly one of attr, arg or d_in", call. = FALSE)
    }
    if (missing(d_in)) {
      if (!missing(d_out)) {
        stop("expected d_in when d_out is specified", call. = FALSE)
      }
    } else if (missing(d_out)) {
      return(transformation_map(ptr, d_in))
    } else {
      return(transformation_check(ptr, d_in, d_out))
    }
    if (!missing(arg)) {
      return(transformation_invoke(ptr, arg))
    }

    switch(attr,
      input_domain = transformation_input_domain(ptr),
      input_metric = transformation_input_metric(ptr),
      output_domain = transformation_output_domain(ptr),
      output_metric = transformation_output_metric(ptr),
      `function` = transformation_function(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(transformation) <- "transformation"
  transformation
}

#' @concept mod
#' @export
toString.transformation <- function(x, ...) {
  paste0(
    "Transformation(\n",
    "  input_domain=", toString(x("input_domain")), ",\n",
    "  input_metric=", toString(x("input_metric")), ",\n",
    "  output_domain=", toString(x("output_domain")), ",\n",
    "  output_metric=", toString(x("output_metric")), "\n",
    ")"
  )
}

#' @concept mod
#' @export
print.transformation <- function(x, ...) {
  cat(toString(x, ...))
}

#' new measurement
#'
#' @concept mod
#' @param ptr pointer to the measurement struct
#' @param log call history
new_measurement <- function(ptr, log) {
  measurement <- function(attr, arg, d_in, d_out) {
    if (missing(attr) + missing(arg) + missing(d_in) != 2) {
      stop("expected exactly one of attr, arg or d_in", call. = FALSE)
    }
    if (missing(d_in)) {
      if (!missing(d_out)) {
        stop("expected d_in when d_out is specified", call. = FALSE)
      }
    } else if (missing(d_out)) {
      return(measurement_map(ptr, d_in))
    } else {
      return(measurement_check(ptr, d_in, d_out))
    }
    if (!missing(arg)) {
      return(measurement_invoke(ptr, arg))
    }
    if (is.numeric(attr)) {
      stop("numeric attr not allowed; Did you mean 'arg='?", call. = FALSE)
    }
    switch(attr,
      input_domain = measurement_input_domain(ptr),
      input_metric = measurement_input_metric(ptr),
      output_measure = measurement_output_measure(ptr),
      `function` = measurement_function(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(measurement) <- "measurement"
  measurement
}


#' @concept mod
#' @export
toString.measurement <- function(x, ...) {
  paste0(
    "Measurement(\n",
    "  input_domain=", toString(x("input_domain")), ",\n",
    "  input_metric=", toString(x("input_metric")), ",\n",
    "  output_measure=", toString(x("output_measure")), "\n",
    ")"
  )
}

#' @concept mod
#' @export
print.measurement <- function(x, ...) {
  cat(toString(x, ...))
}

#' new odometer
#'
#' @concept mod
#' @param ptr pointer to the odometer struct
#' @param log call history
new_odometer <- function(ptr, log) {
  odometer <- function(attr, arg) {
    if (missing(attr) + missing(arg) != 1) {
      stop("expected exactly one of attr or arg", call. = FALSE)
    }
    if (!missing(arg)) {
      return(odometer_invoke(ptr, arg))
    }
    if (is.numeric(attr)) {
      stop("numeric attr not allowed; Did you mean 'arg='?", call. = FALSE)
    }
    switch(attr,
      input_domain = odometer_input_domain(ptr),
      input_metric = odometer_input_metric(ptr),
      output_measure = odometer_output_measure(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(odometer) <- "odometer"
  odometer
}

#' @concept mod
#' @export
toString.odometer <- function(x, ...) {
  paste0(
    "Odometer(\n",
    "  input_domain=", toString(x("input_domain")), ",\n",
    "  input_metric=", toString(x("input_metric")), ",\n",
    "  output_measure=", toString(x("output_measure")), "\n",
    ")"
  )
}

#' @concept mod
#' @export
print.odometer <- function(x, ...) {
  cat(toString(x, ...))
}

#' new domain
#'
#' @concept mod
#' @param ptr a pointer to a domain
#' @param log call history
new_domain <- function(ptr, log) {
  domain <- function(attr, member) {
    if (missing(attr) + missing(member) != 1) {
      stop("expected exactly one of attr or member", call. = FALSE)
    }

    if (!missing(member)) {
      return(`_member`(ptr, member))
    }

    switch(attr,
      debug = domain_debug(ptr),
      type = domain_type(ptr),
      carrier_type = domain_carrier_type(ptr),
      descriptor = `_extrinsic_domain_descriptor`(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(domain) <- "domain"
  domain
}

#' @concept mod
#' @export
toString.domain <- function(x, ...) {
  x("debug")
}

#' @concept mod
#' @export
print.domain <- function(x, ...) {
  cat(toString(x, ...))
}

#' new metric
#'
#' @concept mod
#' @param ptr a pointer to a metric
#' @param log call history
new_metric <- function(ptr, log) {
  metric <- function(attr) {
    switch(attr,
      debug = metric_debug(ptr),
      type = metric_type(ptr),
      distance_type = metric_distance_type(ptr),
      descriptor = `_extrinsic_metric_descriptor`(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(metric) <- "metric"
  metric
}

#' @concept mod
#' @export
toString.metric <- function(x, ...) {
  x("debug")
}

#' @concept mod
#' @export
print.metric <- function(x, ...) {
  cat(toString(x, ...))
}

#' new measure
#'
#' @concept mod
#' @param ptr a pointer to a measure
#' @param log call history
new_measure <- function(ptr, log) {
  measure <- function(attr) {
    switch(attr,
      debug = measure_debug(ptr),
      type = measure_type(ptr),
      distance_type = measure_distance_type(ptr),
      descriptor = `_extrinsic_measure_descriptor`(ptr),
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(measure) <- "measure"
  measure
}

#' @concept mod
#' @export
toString.measure <- function(x, ...) {
  x("debug")
}

#' @concept mod
#' @export
print.measure <- function(x, ...) {
  cat(toString(x, ...))
}

#' new function
#'
#' @concept mod
#' @param ptr a pointer to a function
#' @param log call history
new_function_internal <- function(ptr, log) {
  opendp_function <- function(attr, arg) {
    if (missing(attr) + missing(arg) != 1) {
      stop("expected exactly one of attr or arg", call. = FALSE)
    }

    if (!missing(arg)) {
      return(function_eval(ptr, arg))
    }

    switch(attr,
      json = jsonlite::toJSON(to_ast(log), pretty = TRUE),
      ptr = ptr,
      log = log,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(opendp_function) <- "opendp_function"
  opendp_function
}

#' new privacy profile
#'
#' @concept mod
#' @param ptr a pointer to a privacy profile
new_privacy_profile_internal <- function(ptr) {
  privacy_profile <- function(attr, epsilon, delta) {
    if (missing(attr) + missing(epsilon) + missing(delta) != 2) {
      stop("expected exactly one of attr, epsilon or delta", call. = FALSE)
    }

    if (!missing(epsilon)) {
      return(privacy_profile_delta(ptr, epsilon))
    }

    if (!missing(delta)) {
      return(privacy_profile_epsilon(ptr, delta))
    }

    switch(attr,
      ptr = ptr,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(privacy_profile) <- "privacy_profile"
  privacy_profile
}

#' new queryable
#'
#' @concept mod
#' @param ptr a pointer to a queryable
new_queryable_internal <- function(ptr) {
  queryable <- function(attr, query) {
    if (missing(attr) + missing(query) != 1) {
      stop("expected exactly one of attr or query", call. = FALSE)
    }

    if (!missing(query)) {
      return(queryable_eval(ptr, query))
    }

    switch(attr,
      ptr = ptr,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(queryable) <- "queryable"
  queryable
}


#' new odometer queryable
#'
#' @concept mod
#' @param ptr a pointer to an odometer queryable
new_odometer_queryable_internal <- function(ptr) {
  odometer_queryable <- function(attr, query, d_in) {
    if (missing(attr) + missing(query) + missing(d_in) != 2) {
      stop("expected exactly one of attr, query or d_in", call. = FALSE)
    }

    if (!missing(query)) {
      return(odometer_queryable_invoke(ptr, query))
    }
    if (!missing(d_in)) {
      return(odometer_queryable_privacy_loss(ptr, d_in))
    }

    switch(attr,
      ptr = ptr,
      stop("unrecognized attribute", call. = FALSE)
    )
  }
  class(odometer_queryable) <- "odometer_queryable"
  odometer_queryable
}


#' extract heterogeneously typed keys and values from a hashtab
#'
#' @concept mod
#' @param data a hashtab
#' @param type_name the expected runtime_type of the hashtab
#' @export
hashitems <- function(data, type_name) {
  if (!inherits(data, "hashtab")) {
    stop("Expected hashtab data, got ", class(data), call. = FALSE)
  }
  if (type_name$origin != "HashMap") {
    stop("Expected HashMap type_name, got ", type_name$origin, call. = FALSE)
  }

  keys <- vector(RUST_TO_R[[type_name$args[[1]]$origin]], utils::numhash(data))
  vals <- vector(RUST_TO_R[[type_name$args[[2]]$origin]], utils::numhash(data))
  idx <- 0
  utils::maphash(data, function(k, v) {
    idx <<- idx + 1
    keys[idx] <<- k
    vals[idx] <<- v
  })
  list(keys, vals)
}

#' create an instance of a hashtab from keys and values
#'
#' @concept mod
#' @param keys a vector of keys
#' @param vals a vector of values
#' @export
new_hashtab <- function(keys, vals) {
  if (length(keys) != length(vals)) stop("keys and vals must have the same length", call. = FALSE)
  h <- utils::hashtab()
  mapply(function(k, v) utils::sethash(h, k, v), keys, vals, SIMPLIFY = FALSE)
  h
}

to_str <- function(x, depth) UseMethod("to_str")

#' Convert a format-able value to a string representation
#'
#' @param x value to convert to string
#' @param depth offset from start of string
#' @concept mod
#' @export to_str.default
to_str.default <- function(x, depth) format(x)

#' Convert hashtab to a string representation
#'
#' @param x value to convert to string
#' @param depth offset from start of string
#' @concept mod
#' @export
to_str.hashtab <- function(x, depth = 0L) {
  spacer <- strrep("  ", depth)
  val <- "hashtab(\n"
  utils::maphash(x, function(k, v) {
    val <<- c(val, paste0(spacer, "  ", k, ": ", to_str(v, depth = depth + 1), ",\n"))
  })
  val <- c(val, spacer, ")")
  paste0(val, collapse = "") # nolint: paste_linter
}

CONSTRUCTOR_LOG_KEYS <- list("_type", "name", "module", "kwargs")
new_constructor_log <- function(name, module, kwargs) {
  new_hashtab(CONSTRUCTOR_LOG_KEYS, list(
    unbox2("constructor"),
    unbox2(name),
    unbox2(module),
    kwargs
  ))
}

PARTIAL_LOG_KEYS <- list("_type", "lhs", "rhs")
new_partial_log <- function(lhs, rhs) {
  new_hashtab(PARTIAL_LOG_KEYS, list(
    unbox2("partial_chain"),
    lhs,
    rhs
  ))
}


to_ast <- function(item) {
  if (inherits(item, "scalar")) {
    item
  } else if (inherits(item, c("transformation", "measurement", "domain", "metric", "measure", "function"))) {
    to_ast(item$log)
  } else if (inherits(item, "runtime_type")) {
    unbox2(rt_to_string(item))
  } else if (utils::is.hashtab(item)) {
    # TODO: can jsonlite even write non-string keys?
    out <- list()
    utils::maphash(item, function(k, v) {
      out[[k]] <<- to_ast(v)
    })
    out
  } else if (is.list(item)) {
    lapply(item, to_ast)
  } else if (inherits(item, c("numeric", "character", "integer", "logical"))) {
    list(`_type` = "list", `_items` = item)
  } else {
    item
  }
}


unbox2 <- function(x) {
  if (requireNamespace("jsonlite", quietly = TRUE)) {
    jsonlite::unbox(x)
  } else {
    x
  }
}


#' Find the highest-utility (`d_in`, `d_out`)-close Transformation or Measurement.
#'
#' Searches for the numeric parameter to `make_chain` that results in a computation
#' that most tightly satisfies `d_out` when datasets differ by at most `d_in`,
#' then returns the Transformation or Measurement corresponding to said parameter.
#'
#' See `binary_search_param` to retrieve the discovered parameter instead of the complete computation chain.
#'
#' @concept mod
#' @param make_chain a function that takes a number and returns a Transformation or Measurement
#' @param d_in how far apart input datasets can be
#' @param d_out how far apart output datasets or distributions can be
#' @param bounds a 2-tuple of the lower and upper bounds on the input of `make_chain`
#' @param .T type of argument to `make_chain`, either "float" or "int"
#' @return a Transformation or Measurement (chain) that is (`d_in`, `d_out`)-close.
#' @export
#' @examples
#' enable_features("contrib")
#' # create a sum transformation over the space of float vectors
#' s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
#' t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()
#'
#' # find a measurement that satisfies epsilon = 1 when datasets differ by at most one record
#' m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1.)
binary_search_chain <- function(make_chain, d_in, d_out, bounds = NULL, .T = NULL) {
  return(make_chain(binary_search_param(make_chain, d_in, d_out, bounds, .T)))
}


#' Solve for the ideal constructor argument to `make_chain`
#'
#' Searches for the numeric parameter to `make_chain` that results in a computation
#' that most tightly satisfies `d_out` when datasets differ by at most `d_in`.
#'
#' @concept mod
#' @param make_chain a function that takes a number and returns a Transformation or Measurement
#' @param d_in how far apart input datasets can be
#' @param d_out how far apart output datasets or distributions can be
#' @param bounds a 2-tuple of the lower and upper bounds on the input of `make_chain`
#' @param .T type of argument to `make_chain`, either "float" or "int"
#' @return the parameter to `make_chain` that results in a (`d_in`, `d_out`)-close Transformation or Measurement
#' @export
binary_search_param <- function(make_chain, d_in, d_out, bounds = NULL, .T = NULL) {
  return(binary_search(function(param) {
    make_chain(param)(d_in = d_in, d_out = d_out)
  }, bounds, .T))
}

.call_search_callback <- function(predicate, .T, call_expr, bounds = NULL) {
  parse_rust_error_message <- function(message) {
    first_line <- strsplit(message, "\n", fixed = TRUE)[[1]][1]
    parsed <- regexec("^\\[([^]]+)\\] : (.*)$", first_line)
    parts <- regmatches(first_line, parsed)[[1]]
    if (length(parts) == 3) {
      return(list(variant = parts[[2]], message = parts[[3]]))
    }
    list(variant = NULL, message = first_line)
  }

  state <- new.env(parent = emptyenv())
  state$first_error <- NULL
  state$had_success <- FALSE

  wrapped <- function(v) {
    tryCatch({
      out <- predicate(v)
      state$had_success <- TRUE
      out
    }, error = function(e) {
      if (is.null(state$first_error)) {
        state$first_error <- e
      }
      stop(conditionMessage(e), call. = FALSE)
    })
  }

  tryCatch(
    force(call_expr(wrapped)),
    error = function(e) {
      if (!is.null(state$first_error)) {
        if (is.null(bounds) && !isTRUE(state$had_success)) {
          center <- if (.T$origin %in% c("f32", "f64")) 0. else 0L
          msg <- "Predicate always fails. Example error at %s: %s"
          stop(sprintf(msg, center, conditionMessage(state$first_error)), call. = FALSE)
        }
        stop(conditionMessage(state$first_error), call. = FALSE)
      }
      rust_error <- parse_rust_error_message(conditionMessage(e))
      if (identical(rust_error$variant, "Search")) {
        stop(rust_error$message, call. = FALSE)
      }
      stop(conditionMessage(e), call. = FALSE)
    }
  )
}

.infer_search_type <- function(predicate, .T = NULL, bounds = NULL) {
  if (!is.null(bounds)) {
    if (inherits(bounds, "integer")) {
      return(rt_parse("int"))
    }
    if (inherits(bounds, "numeric")) {
      return(rt_parse("float"))
    }
    stop("bounds must be either float or int", call. = FALSE)
  }

  if (!is.null(.T)) {
    return(rt_parse(.T))
  }

  check_type <- function(v) {
    f <- try(predicate(v), TRUE)
    !inherits(f, "try-error")
  }

  if (check_type(0.)) {
    return(rt_parse("float"))
  }
  if (check_type(0L)) {
    return(rt_parse("int"))
  }
  stop("unable to infer type `.T`; pass the type `.T` or bounds", call. = FALSE)
}

#' Find the closest passing value to the decision boundary of `predicate`
#'
#' If bounds are not passed, conducts an exponential search.
#'
#' @concept mod
#' @param predicate a monotonic unary function from a number to a boolean
#' @param bounds a 2-tuple of the lower and upper bounds on the input of `make_chain`
#' @param .T type of argument to `predicate`, one of float or int
#' @param return_sign if True, also return the direction away from the decision boundary
#' @return the discovered parameter within the bounds
#' @export
binary_search <- function(predicate, bounds = NULL, .T = NULL, return_sign = FALSE) {
  .T <- .infer_search_type(predicate, .T, bounds)

  result <- .call_search_callback(
    predicate = predicate,
    .T = .T,
    bounds = bounds,
    call_expr = function(wrapped) {
      `_binary_search`(wrapped, bounds = bounds, .T = .T, return_sign = return_sign)
    }
  )

  if (return_sign) {
    return(c(result[[1]], result[[2]]))
  }
  result
}

exponential_bounds_search <- function(predicate, .T) {
  .T <- .infer_search_type(predicate, .T)

  result <- .call_search_callback(
    predicate = predicate,
    .T = .T,
    call_expr = function(wrapped) {
      `_exponential_bounds_search`(wrapped, .T = .T)
    }
  )

  if (is.null(result)) {
    return(NULL)
  }
  unlist(result, use.names = FALSE)
}
