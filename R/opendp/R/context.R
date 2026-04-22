#' @include mod.R typing.R
NULL

`%||%` <- function(x, y) {
  if (is.null(x)) y else x
}

is_query <- function(x) inherits(x, "opendp_query")
is_context <- function(x) inherits(x, "opendp_context")
is_partial_chain <- function(x) inherits(x, "partial_chain")
is_measurement_like <- function(x) inherits(x, c("measurement", "odometer"))

chain_output_measure <- function(x) {
  if (inherits(x, "measurement") || inherits(x, "odometer")) {
    return(x("output_measure"))
  }
  stop("expected a measurement or odometer", call. = FALSE)
}

new_partial_chain <- function(partial, auto_name = "param") {
  attr(partial, "auto_name") <- auto_name
  class(partial) <- c("partial_chain", "function")
  partial
}

#' @export
print.partial_chain <- function(x, ...) {
  cat(toString(x, ...))
}

#' @export
toString.partial_chain <- function(x, ...) {
  paste0("PartialChain(auto = ", attr(x, "auto_name"), ")")
}

#' Sentinel for Context API parameter search.
#'
#' Pass `auto()` to a `then_*` constructor inside a query to ask the Context API
#' to solve for the missing numeric parameter at release time.
#'
#' @concept context
#' @return An auto sentinel.
#' @export
auto <- function() {
  structure(list(), class = "opendp_auto")
}

is_auto <- function(x) inherits(x, "opendp_auto")

count_auto_args <- function(args) {
  sum(vapply(args, is_auto, logical(1)))
}

replace_auto_args <- function(args, value) {
  lapply(args, function(arg) {
    if (is_auto(arg)) value else arg
  })
}

get_auto_name <- function(args) {
  matches <- names(args)[vapply(args, is_auto, logical(1))]
  if (length(matches) == 0) {
    return("param")
  }
  matches[[1]]
}

compose_partial_chain <- function(lhs, constructor, args, log_) {
  new_partial_chain(
    function(param) {
      base_lhs <- lhs(param)
      rhs <- constructor(base_lhs, args)
      make_chain_dyn(rhs, base_lhs, log_)
    },
    auto_name = attr(lhs, "auto_name") %||% "param"
  )
}

clone_query <- function(query, chain = query$chain, wrap_release = query$wrap_release) {
  new_query(
    chain = chain,
    output_measure = query$output_measure,
    d_in = query$d_in,
    d_out = query$d_out,
    context = query$context,
    wrap_release = wrap_release
  )
}

new_query <- function(chain, output_measure, d_in = NULL, d_out = NULL,
                      context = NULL, wrap_release = NULL) {
  self <- new.env(parent = emptyenv())
  self$chain <- chain
  self$output_measure <- output_measure
  self$d_in <- d_in
  self$d_out <- d_out
  self$context <- context
  self$wrap_release <- wrap_release
  class(self) <- "opendp_query"
  self
}

#' Construct a query chain.
#'
#' This is most commonly created via `query()` on a context object, but it may
#' also be used stand-alone to build a transformation or measurement to release
#' against explicit `data`.
#'
#' @concept context
#' @param chain An initial metric space or transformation.
#' @param output_measure How privacy loss is measured on the output of the query.
#' @param d_in Upper bound on the input dataset distance.
#' @param d_out Upper bound on the per-query privacy loss.
#' @param context Optional parent context.
#' @return Query
#' @export
Query <- function(chain, output_measure, d_in = NULL, d_out = NULL, context = NULL) {
  new_query(chain, output_measure, d_in = d_in, d_out = d_out, context = context)
}

#' @export
print.opendp_query <- function(x, ...) {
  cat(toString(x, ...))
}

#' @export
toString.opendp_query <- function(x, ...) {
  context <- if (is.null(x$context)) {
    ""
  } else {
    paste0(",\n  context        = ", toString(x$context))
  }
  paste0(
    "Query(\n",
    "  chain          = ", toString(x$chain), ",\n",
    "  output_measure = ", toString(x$output_measure), ",\n",
    "  d_in           = ", deparse(x$d_in), ",\n",
    "  d_out          = ", deparse(x$d_out), context, "\n",
    ")"
  )
}

new_context <- function(accountant, queryable, d_in, d_mids = NULL, d_out = NULL,
                        query_space = NULL) {
  self <- new.env(parent = emptyenv())
  self$accountant <- accountant
  self$queryable <- queryable
  self$d_in <- d_in
  self$d_mids <- d_mids
  self$d_out <- d_out
  self$d_mids_consumed <- list()
  self$query_space <- query_space
  class(self) <- "opendp_context"
  self
}

#' @export
print.opendp_context <- function(x, ...) {
  cat(toString(x, ...))
}

#' @export
toString.opendp_context <- function(x, ...) {
  paste0(
    "Context(\n",
    "  accountant = ", toString(x$accountant), ",\n",
    "  d_in       = ", deparse(x$d_in), ",\n",
    "  d_mids     = ", deparse(x$d_mids), ",\n",
    "  d_out      = ", deparse(x$d_out), "\n",
    ")"
  )
}

context_eval <- function(context, measurement) {
  answer <- context$queryable(query = measurement)

  consumed <- if (is.null(context$d_mids)) {
    measurement(d_in = context$d_in)
  } else {
    next_mid <- context$d_mids[[1]]
    context$d_mids <- context$d_mids[-1]
    next_mid
  }
  context$d_mids_consumed <- c(context$d_mids_consumed, list(consumed))
  answer
}

space_from_accountant <- function(accountant) {
  list(accountant("input_domain"), accountant("input_metric"))
}

is_atomic_distance <- function(x) {
  is.atomic(x) && is.null(dim(x)) && length(x) == 1
}

is_unbounded_loss <- function(d_out) {
  identical(d_out, Inf) || identical(d_out, c(Inf, Inf)) || identical(d_out, c(Inf, 1))
}

normalize_measure_distance <- function(dist) {
  if (length(dist) == 1) {
    dist[[1]]
  } else {
    dist
  }
}

measure_type_string <- function(measure) {
  rt_to_string(get_type(measure))
}

cast_measure <- function(chain, to_measure = NULL, d_to = NULL) {
  if (is.null(to_measure) || !is_measurement_like(chain)) {
    return(chain)
  }
  if (measure_equal(chain_output_measure(chain), to_measure)) {
    return(chain)
  }

  from_to <- c(
    measure_type_string(chain_output_measure(chain)),
    measure_type_string(to_measure)
  )

  if (identical(from_to, c("MaxDivergence", "Approximate<MaxDivergence>"))) {
    return(make_approximate(chain))
  }
  if (identical(from_to, c("ZeroConcentratedDivergence", "Approximate<ZeroConcentratedDivergence>"))) {
    return(make_approximate(chain))
  }
  if (identical(from_to, c("MaxDivergence", "ZeroConcentratedDivergence"))) {
    return(make_pureDP_to_zCDP(chain))
  }
  if (identical(from_to, c("ZeroConcentratedDivergence", "Approximate<MaxDivergence>")) ||
      identical(from_to, c("Approximate<ZeroConcentratedDivergence>", "Approximate<MaxDivergence>"))) {
    return(make_fix_delta(make_zCDP_to_approxDP(chain), d_to[[2]]))
  }

  stop(
    "Unable to cast measure from ", from_to[[1]], " to ", from_to[[2]],
    call. = FALSE
  )
}

translate_measure_distance <- function(d_from, from_measure, to_measure, alpha = NULL) {
  if (measure_equal(from_measure, to_measure)) {
    return(d_from)
  }

  from_to <- c(measure_type_string(from_measure), measure_type_string(to_measure))
  constant <- 1.0

  if (identical(from_to, c("MaxDivergence", "Approximate<MaxDivergence>"))) {
    return(c(d_from, 0.0))
  }

  if (identical(from_to, c("ZeroConcentratedDivergence", "MaxDivergence"))) {
    space <- list(atom_domain(.T = "f64", nan = FALSE), absolute_distance(.T = "f64"))
    scale <- binary_search_param(
      function(scale) make_pureDP_to_zCDP(space |> then_laplace(scale)),
      d_in = constant,
      d_out = d_from,
      .T = "float"
    )
    return((space |> then_laplace(scale))(d_in = constant))
  }

  if (identical(from_to, c("Approximate<MaxDivergence>", "ZeroConcentratedDivergence"))) {
    space <- list(atom_domain(.T = "i32"), absolute_distance(.T = "f64"))
    scale <- binary_search_param(
      function(scale) {
        make_fix_delta(make_zCDP_to_approxDP(space |> then_gaussian(scale)), d_from[[2]])
      },
      d_in = constant,
      d_out = d_from,
      .T = "float"
    )
    return((space |> then_gaussian(scale))(d_in = constant))
  }

  if (identical(from_to, c("Approximate<MaxDivergence>", "Approximate<ZeroConcentratedDivergence>"))) {
    epsilon <- d_from[[1]]
    delta <- d_from[[2]]
    if (is.null(alpha) || alpha < 0 || alpha >= 1) {
      stop("alpha must be in [0, 1)", call. = FALSE)
    }
    delta_zcdp <- delta * (1 - alpha)
    delta_inf <- delta * alpha
    rho <- translate_measure_distance(
      c(epsilon, delta_zcdp),
      from_measure,
      zero_concentrated_divergence()
    )
    return(c(rho, delta_inf))
  }

  stop(
    "Unable to translate distance from ", from_to[[1]], " to ", from_to[[2]],
    call. = FALSE
  )
}

normalize_compositor <- function(domain, privacy_unit, privacy_loss,
                                 split_evenly_over = NULL, split_by_weights = NULL) {
  input_metric <- privacy_unit[[1]]
  d_in <- privacy_unit[[2]]
  output_measure <- privacy_loss[[1]]
  d_out <- privacy_loss[[2]]

  if (!is.null(split_evenly_over) && !is.null(split_by_weights)) {
    stop("Cannot specify both split_evenly_over and split_by_weights", call. = FALSE)
  }

  if (!is.null(split_evenly_over) || !is.null(split_by_weights)) {
    if (length(d_out) != 1) {
      stop(
        "Static split privacy losses are only implemented for scalar losses in the R Context API.",
        call. = FALSE
      )
    }

    weights <- if (!is.null(split_evenly_over)) {
      rep(d_out, split_evenly_over)
    } else {
      as.numeric(split_by_weights) * d_out
    }

    scale <- binary_search_param(
      function(scale) {
        make_adaptive_composition(
          input_domain = domain,
          input_metric = input_metric,
          output_measure = output_measure,
          d_in = d_in,
          d_mids = weights * scale
        )
      },
      d_in = d_in,
      d_out = d_out,
      .T = "float"
    )

    d_mids <- as.list(weights * scale)
    accountant <- make_adaptive_composition(
      input_domain = domain,
      input_metric = input_metric,
      output_measure = output_measure,
      d_in = d_in,
      d_mids = weights * scale
    )
    return(list(accountant, d_mids, NULL))
  }

  odometer <- make_fully_adaptive_composition(
    input_domain = domain,
    input_metric = input_metric,
    output_measure = output_measure
  )

  if (is_unbounded_loss(d_out)) {
    return(list(odometer, NULL, NULL))
  }

  accountant <- make_privacy_filter(odometer, d_in = d_in, d_out = d_out)
  list(accountant, NULL, d_out)
}

partial_chain_fix <- function(chain, d_in, d_out, output_measure = NULL, bounds = NULL, .T = NULL) {
  param <- binary_search_param(
    function(param) cast_measure(chain(param), output_measure, d_out),
    d_in = d_in,
    d_out = d_out,
    bounds = bounds,
    .T = .T
  )
  fixed <- cast_measure(chain(param), output_measure, d_out)
  attr(fixed, "param") <- param
  fixed
}

append_to_query <- function(query, constructor, args, log_) {
  auto_count <- count_auto_args(args)
  if (auto_count > 1) {
    stop("At most one auto() may be unresolved at a time.", call. = FALSE)
  }

  chain <- query$chain
  if (is_partial_chain(chain)) {
    if (auto_count > 0) {
      stop("At most one auto() may be unresolved at a time.", call. = FALSE)
    }
    return(clone_query(query, chain = compose_partial_chain(chain, constructor, args, log_)))
  }

  if (auto_count == 0) {
    rhs <- constructor(chain, args)
    return(clone_query(query, chain = make_chain_dyn(rhs, chain, log_)))
  }

  auto_name <- get_auto_name(args)
  new_chain <- new_partial_chain(
    function(param) {
      rhs <- constructor(chain, replace_auto_args(args, param))
      make_chain_dyn(rhs, chain, log_)
    },
    auto_name = auto_name
  )
  clone_query(query, chain = new_chain)
}

opendp_then <- function(lhs, constructor, args, log_) {
  auto_count <- count_auto_args(args)
  if (auto_count > 1) {
    stop("At most one auto() may be unresolved at a time.", call. = FALSE)
  }

  if (is_query(lhs)) {
    return(append_to_query(lhs, constructor, args, log_))
  }

  if (is_partial_chain(lhs)) {
    if (auto_count > 0) {
      stop("At most one auto() may be unresolved at a time.", call. = FALSE)
    }
    return(compose_partial_chain(lhs, constructor, args, log_))
  }

  if (auto_count > 0) {
    stop("auto() may only be used inside the Context API.", call. = FALSE)
  }

  rhs <- constructor(lhs, args)
  make_chain_dyn(rhs, lhs, log_)
}

infer_domain <- function(rtype) {
  if (!inherits(rtype, "runtime_type")) {
    rtype <- rt_parse(rtype)
  }

  if (is.null(rtype$args)) {
    atom <- rt_to_string(rtype)
    if (atom %in% c("f32", "f64")) {
      return(atom_domain(.T = atom, nan = FALSE))
    }
    return(atom_domain(.T = atom))
  }
  if (rtype$origin == "Vec") {
    return(vector_domain(infer_domain(rtype$args[[1]])))
  }
  if (rtype$origin == "HashMap") {
    return(map_domain(infer_domain(rtype$args[[1]]), infer_domain(rtype$args[[2]])))
  }
  if (rtype$origin == "Option") {
    return(option_domain(infer_domain(rtype$args[[1]])))
  }

  stop("unrecognized carrier type: ", rt_to_string(rtype), call. = FALSE)
}

#' Construct a domain from a carrier type or public example.
#'
#' @concept context
#' @param .T Carrier type or example.
#' @param infer If `TRUE`, infer the type from a public example.
#' @return Domain
#' @export
domain_of <- function(.T, infer = FALSE) {
  if (inherits(.T, "domain")) {
    return(.T)
  }
  rtype <- if (infer) rt_infer(.T) else rt_parse(.T)
  infer_domain(rtype)
}

#' Construct a metric from a metric type.
#'
#' @concept context
#' @param .M Metric type or existing metric.
#' @return Metric
#' @export
metric_of <- function(.M) {
  if (inherits(.M, "metric")) {
    return(.M)
  }
  rtype <- rt_parse(.M)
  origin <- rtype$origin
  if (origin == "AbsoluteDistance") {
    return(absolute_distance(.T = rt_to_string(rtype$args[[1]])))
  }
  if (origin == "L1Distance") {
    return(l1_distance(.T = rt_to_string(rtype$args[[1]])))
  }
  if (origin == "L2Distance") {
    return(l2_distance(.T = rt_to_string(rtype$args[[1]])))
  }
  if (origin == "HammingDistance") {
    return(hamming_distance())
  }
  if (origin == "SymmetricDistance") {
    return(symmetric_distance())
  }
  if (origin == "InsertDeleteDistance") {
    return(insert_delete_distance())
  }
  if (origin == "ChangeOneDistance") {
    return(change_one_distance())
  }
  if (origin == "DiscreteDistance") {
    return(discrete_distance())
  }
  stop("unrecognized metric: ", rt_to_string(rtype), call. = FALSE)
}

#' Build a metric space from a carrier type.
#'
#' @concept context
#' @param .T Carrier type or public example.
#' @param .M Optional metric type.
#' @param infer If `TRUE`, infer the carrier type from the example.
#' @return A metric space as `(domain, metric)`.
#' @export
space_of <- function(.T, .M = NULL, infer = FALSE) {
  rtype <- if (infer) rt_infer(.T) else rt_parse(.T)
  domain <- infer_domain(rtype)

  if (is.null(.M)) {
    if (rtype$origin == "Vec") {
      .M <- symmetric_distance()
    } else if (is.null(rtype$args) && rt_to_string(rtype) %in% c("f32", "f64", "i8", "i16", "i32", "i64", "u32", "u64", "usize")) {
      .M <- absolute_distance(.T = rt_to_string(rtype))
    } else {
      stop("no default metric for domain ", rt_to_string(get_type(domain)), call. = FALSE)
    }
  } else {
    .M <- metric_of(.M)
  }

  list(domain, .M)
}

#' Construct a privacy loss from common parameters.
#'
#' @concept context
#' @param epsilon Parameter for pure DP.
#' @param delta Parameter for approximate DP.
#' @param rho Parameter for zCDP.
#' @return A pair of `(measure, loss)`.
#' @export
loss_of <- function(epsilon = NULL, delta = NULL, rho = NULL) {
  warn_range <- function(name, value, info_level, warn_level) {
    if (value > warn_level) {
      warning(
        name, " should be less than or equal to ", warn_level,
        if (info_level != warn_level) {
          paste0(", and is typically less than or equal to ", info_level)
        } else {
          ""
        },
        call. = FALSE
      )
    } else if (value > info_level) {
      message(name, " is typically less than or equal to ", info_level)
    }
  }

  if ((is.null(epsilon) && is.null(rho)) || (!is.null(epsilon) && !is.null(rho))) {
    stop("Either epsilon or rho must be specified, and they are mutually exclusive.", call. = FALSE)
  }

  if (!is.null(epsilon)) {
    warn_range("epsilon", epsilon, 1, 5)
    measure <- max_divergence()
    loss <- as.numeric(epsilon)
  } else {
    warn_range("rho", rho, 0.25, 0.5)
    measure <- zero_concentrated_divergence()
    loss <- as.numeric(rho)
  }

  if (is.null(delta)) {
    return(list(measure, loss))
  }

  warn_range("delta", delta, 1e-6, 1e-6)
  list(approximate(measure), c(loss, delta))
}

#' Construct a privacy unit from common parameters.
#'
#' @concept context
#' @param contributions Max contributed records.
#' @param changes Max changed records.
#' @param absolute Max absolute influence on scalar aggregates.
#' @param l1 Max l1 influence on vector aggregates.
#' @param l2 Max l2 influence on vector aggregates.
#' @param local Set to `TRUE` for local DP.
#' @param identifier Not yet supported in the R Context API.
#' @param ordered Choose ordered dataset metrics.
#' @param .U Optional distance type.
#' @return A pair of `(metric, d_in)`.
#' @export
unit_of <- function(contributions = NULL, changes = NULL, absolute = NULL, l1 = NULL, l2 = NULL,
                    local = NULL, identifier = NULL, ordered = FALSE, .U = NULL) {
  values <- list(
    contributions = contributions,
    changes = changes,
    absolute = absolute,
    l1 = l1,
    l2 = l2,
    local = local
  )
  if (sum(vapply(values, function(v) !is.null(v), logical(1))) != 1) {
    stop("Must specify exactly one distance.", call. = FALSE)
  }

  if (!is.null(identifier)) {
    stop("identifier-based privacy units are not yet implemented in the R Context API.", call. = FALSE)
  }

  if (isTRUE(local)) {
    if (!is.null(identifier) || ordered || !is.null(.U)) {
      stop('"local" must be the only parameter', call. = FALSE)
    }
    return(list(discrete_distance(), 1L))
  }

  if (ordered && is.null(contributions) && is.null(changes)) {
    stop('"ordered" is only valid with "changes" or "contributions"', call. = FALSE)
  }

  if (!is.null(contributions)) {
    metric <- if (ordered) insert_delete_distance() else symmetric_distance()
    return(list(metric, contributions))
  }
  if (!is.null(changes)) {
    metric <- if (ordered) hamming_distance() else change_one_distance()
    return(list(metric, changes))
  }
  if (!is.null(absolute)) {
    return(list(absolute_distance(.T = rt_to_string(parse_or_infer(.U, absolute))), absolute))
  }
  if (!is.null(l1)) {
    return(list(l1_distance(.T = rt_to_string(parse_or_infer(.U, l1))), l1))
  }
  if (!is.null(l2)) {
    return(list(l2_distance(.T = rt_to_string(parse_or_infer(.U, l2))), l2))
  }

  stop("No matching metric found", call. = FALSE)
}

context_compositor <- function(data, privacy_unit, privacy_loss,
                               split_evenly_over = NULL, split_by_weights = NULL,
                               domain = NULL) {
  assert_features("contrib")

  if (is.null(domain)) {
    domain <- domain_of(data, infer = TRUE)
  }

  normalized <- normalize_compositor(
    domain = domain,
    privacy_unit = privacy_unit,
    privacy_loss = privacy_loss,
    split_evenly_over = split_evenly_over,
    split_by_weights = split_by_weights
  )

  accountant <- normalized[[1]]
  d_mids <- normalized[[2]]
  d_out <- normalized[[3]]
  queryable <- accountant(arg = data)

  new_context(
    accountant = accountant,
    queryable = queryable,
    d_in = privacy_unit[[2]],
    d_mids = d_mids,
    d_out = d_out
  )
}

#' Context API entry point.
#'
#' Use `Context$compositor(...)` to construct a context that tracks privacy
#' expenditure across releases.
#'
#' @concept context
#' @export
Context <- new.env(parent = emptyenv())
Context$compositor <- context_compositor

#' Start a query from a context.
#'
#' @concept context
#' @param x A context.
#' @param ... Optional privacy loss parameters passed to `loss_of()`.
#' @export
query <- function(x, ...) {
  UseMethod("query")
}

#' @export
query.opendp_context <- function(x, ...) {
  kwargs <- list(...)
  d_query <- NULL

  if (!is.null(x$d_mids)) {
    if (length(kwargs) > 0) {
      stop("Expected no privacy arguments for this context query.", call. = FALSE)
    }
    if (length(x$d_mids) == 0) {
      stop("Privacy allowance has been exhausted.", call. = FALSE)
    }
    d_query <- x$d_mids[[1]]
  } else if (length(kwargs) > 0) {
    observed <- do.call(loss_of, kwargs)
    observed_measure <- observed[[1]]
    d_query <- observed[[2]]
    expected_measure <- chain_output_measure(x$accountant)
    if (!measure_equal(observed_measure, expected_measure)) {
      msg <- paste0(
        "Expected output measure ", toString(expected_measure),
        " but got ", toString(observed_measure), "."
      )
      if (measure_type_string(expected_measure) == "Approximate<MaxDivergence>" && is.null(kwargs$delta)) {
        msg <- paste0(msg, " Consider setting delta = 0.0 in your query.")
      }
      stop(msg, call. = FALSE)
    }
  }

  chain <- x$query_space %||% space_from_accountant(x$accountant)
  new_query(
    chain = chain,
    output_measure = chain_output_measure(x$accountant),
    d_in = x$d_in,
    d_out = d_query,
    context = x
  )
}

#' Resolve a query into a concrete chain.
#'
#' @concept context
#' @param x A query.
#' @param allow_transformations If `TRUE`, allow unresolved transformations.
#' @param bounds Optional search bounds for `auto()`.
#' @param .T Parameter type, either `"float"` or `"int"`.
#' @export
resolve <- function(x, allow_transformations = FALSE, bounds = NULL, .T = NULL) {
  UseMethod("resolve")
}

#' @export
resolve.opendp_query <- function(x, allow_transformations = FALSE, bounds = NULL, .T = NULL) {
  chain <- x$chain
  if (is_partial_chain(chain)) {
    if (is.null(x$d_in) || is.null(x$d_out)) {
      stop("Cannot resolve auto() without both d_in and d_out.", call. = FALSE)
    }
    chain <- partial_chain_fix(
      chain,
      d_in = x$d_in,
      d_out = x$d_out,
      output_measure = x$output_measure,
      bounds = bounds,
      .T = .T
    )
  } else {
    chain <- cast_measure(chain, x$output_measure, x$d_out)
  }

  if (!allow_transformations && inherits(chain, "transformation")) {
    stop("Query is not yet a measurement or odometer.", call. = FALSE)
  }
  chain
}

#' Release a query.
#'
#' @concept context
#' @param x A query.
#' @param data Optional data for stand-alone queries.
#' @param bounds Optional search bounds for `auto()`.
#' @param .T Parameter type, either `"float"` or `"int"`.
#' @export
release <- function(x, data = NULL, bounds = NULL, .T = NULL) {
  UseMethod("release")
}

#' @export
release.opendp_query <- function(x, data = NULL, bounds = NULL, .T = NULL) {
  if (!is.null(x$context) && !is.null(data)) {
    stop("Cannot specify data when the query is part of a context.", call. = FALSE)
  }

  measurement <- resolve(x, bounds = bounds, .T = .T)

  answer <- if (!is.null(x$context)) {
    context_eval(x$context, measurement)
  } else if (!is.null(data)) {
    measurement(arg = data)
  } else {
    stop("Cannot release query without data or context.", call. = FALSE)
  }

  if (!is.null(x$wrap_release)) {
    answer <- x$wrap_release(answer, measurement)
  }
  answer
}

#' Retrieve the discovered `auto()` parameter.
#'
#' @concept context
#' @param x A query.
#' @param allow_transformations If `TRUE`, allow unresolved transformations.
#' @param bounds Optional search bounds for `auto()`.
#' @param .T Parameter type, either `"float"` or `"int"`.
#' @export
param <- function(x, allow_transformations = FALSE, bounds = NULL, .T = NULL) {
  UseMethod("param")
}

#' @export
param.opendp_query <- function(x, allow_transformations = FALSE, bounds = NULL, .T = NULL) {
  resolved <- resolve(
    x,
    allow_transformations = allow_transformations,
    bounds = bounds,
    .T = .T
  )
  attr(resolved, "param")
}

#' Construct a nested compositor from a query.
#'
#' @concept context
#' @param x A query.
#' @param split_evenly_over Number of equal-budget child queries.
#' @param split_by_weights Relative weights for child queries.
#' @param d_out Optional output privacy loss.
#' @param output_measure Optional output privacy measure.
#' @param alpha Optional delta split parameter used for approx-zCDP conversion.
#' @export
compositor <- function(x, split_evenly_over = NULL, split_by_weights = NULL,
                       d_out = NULL, output_measure = NULL, alpha = NULL) {
  UseMethod("compositor")
}

compose_context_chain <- function(chain, d_in, d_out, output_measure,
                                  reservation_d_out = d_out,
                                  split_evenly_over = NULL, split_by_weights = NULL,
                                  alpha = NULL) {
  if (is_space(chain)) {
    input_domain <- chain[[1]]
    input_metric <- chain[[2]]
  } else if (inherits(chain, "transformation")) {
    input_domain <- chain("output_domain")
    input_metric <- chain("output_metric")
    d_in <- chain(d_in = d_in)
  } else {
    stop("Expected a metric space or transformation.", call. = FALSE)
  }

  privacy_loss <- list(output_measure, d_out)
  normalized <- normalize_compositor(
    domain = input_domain,
    privacy_unit = list(input_metric, d_in),
    privacy_loss = privacy_loss,
    split_evenly_over = split_evenly_over,
    split_by_weights = split_by_weights
  )

  base_accountant <- normalized[[1]]
  d_mids <- normalized[[2]]
  final_d_out <- normalized[[3]]
  context_d_out <- final_d_out

  accountant <- base_accountant
  if (inherits(chain, "transformation")) {
    accountant <- make_chain_dyn(accountant, chain, accountant("log"))
  }

  submission_accountant <- base_accountant
  if (inherits(base_accountant, "odometer")) {
    # Parent queryables accept measurement queries, so wrap unbounded child
    # compositors in a filter while preserving the raw odometer for subcontexts.
    submission_accountant <- make_privacy_filter(
      submission_accountant,
      d_in = d_in,
      d_out = reservation_d_out
    )
    context_d_out <- reservation_d_out
  }
  if (inherits(chain, "transformation")) {
    submission_accountant <- make_chain_dyn(
      submission_accountant,
      chain,
      submission_accountant("log")
    )
  }

  list(
    accountant = accountant,
    submission_accountant = submission_accountant,
    d_in = d_in,
    d_mids = d_mids,
    d_out = context_d_out,
    query_space = list(input_domain, input_metric)
  )
}

#' @export
compositor.opendp_query <- function(x, split_evenly_over = NULL, split_by_weights = NULL,
                                    d_out = NULL, output_measure = NULL, alpha = NULL) {
  requested_d_out <- d_out %||% x$d_out
  if (is.null(requested_d_out)) {
    stop("d_out is unknown. Please specify it in the query.", call. = FALSE)
  }

  output_measure <- output_measure %||% x$output_measure
  if (!measure_equal(output_measure, x$output_measure)) {
    requested_d_out <- translate_measure_distance(
      requested_d_out,
      x$output_measure,
      output_measure,
      alpha = alpha
    )
  }
  reservation_d_out <- x$d_out
  if (!is.null(reservation_d_out) && !measure_equal(output_measure, x$output_measure)) {
    reservation_d_out <- translate_measure_distance(
      reservation_d_out,
      x$output_measure,
      output_measure,
      alpha = alpha
    )
  }
  reservation_d_out <- reservation_d_out %||% requested_d_out

  wrap_release <- function(queryable, measurement) {
    metadata <- attr(measurement, "opendp_context_metadata")
    new_context(
      accountant = metadata$accountant,
      queryable = queryable,
      d_in = metadata$d_in,
      d_mids = metadata$d_mids,
      d_out = metadata$d_out,
      query_space = metadata$query_space
    )
  }

  if (is_partial_chain(x$chain)) {
    new_chain <- new_partial_chain(
      function(param) {
        concrete <- x$chain(param)
        meta <- compose_context_chain(
          concrete,
          d_in = x$d_in,
          d_out = requested_d_out,
          output_measure = output_measure,
          reservation_d_out = reservation_d_out,
          split_evenly_over = split_evenly_over,
          split_by_weights = split_by_weights,
          alpha = alpha
        )
        submission_accountant <- meta$submission_accountant %||% meta$accountant
        attr(submission_accountant, "opendp_context_metadata") <- meta
        submission_accountant
      },
      auto_name = attr(x$chain, "auto_name") %||% "param"
    )
    return(clone_query(x, chain = new_chain, wrap_release = wrap_release))
  }

  meta <- compose_context_chain(
    x$chain,
    d_in = x$d_in,
    d_out = requested_d_out,
    output_measure = output_measure,
    reservation_d_out = reservation_d_out,
    split_evenly_over = split_evenly_over,
    split_by_weights = split_by_weights,
    alpha = alpha
  )
  submission_accountant <- meta$submission_accountant %||% meta$accountant
  attr(submission_accountant, "opendp_context_metadata") <- meta
  clone_query(x, chain = submission_accountant, wrap_release = wrap_release)
}

#' Current privacy loss of a context.
#'
#' @concept context
#' @param x A context.
#' @export
current_privacy_loss <- function(x) {
  UseMethod("current_privacy_loss")
}

#' @export
current_privacy_loss.opendp_context <- function(x) {
  if (inherits(x$queryable, "odometer_queryable")) {
    return(x$queryable(d_in = x$d_in))
  }
  x$d_mids_consumed
}

#' Remaining privacy loss of a context.
#'
#' @concept context
#' @param x A context.
#' @export
remaining_privacy_loss <- function(x) {
  UseMethod("remaining_privacy_loss")
}

#' @export
remaining_privacy_loss.opendp_context <- function(x) {
  if (inherits(x$queryable, "odometer_queryable")) {
    if (is.null(x$d_out)) {
      stop("The privacy loss is unbounded.", call. = FALSE)
    }
    return(x$d_out - x$queryable(d_in = x$d_in))
  }
  x$d_mids
}
