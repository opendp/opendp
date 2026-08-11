# Compatibility target for Polars' unstable binary DSL serialization:
# r-polars 1.7.0 / Python Polars 1.36.1 / Rust Polars 0.52.0
# Rust revision: 2a151c10fa76790711c2f75e6d012dd69c627ddd
# DSL schema: 4aade69a4a8aba464f5bb77824e189450cfdd5ba9f38a843b2282b1fea4a071d

.assert_r_polars_compatibility <- function() {
  if (!requireNamespace("polars", quietly = TRUE)) {
    stop("r-polars 1.7.0 is required for Polars interoperability.", call. = FALSE)
  }
  installed <- as.character(utils::packageVersion("polars"))
  if (!identical(installed, "1.7.0")) {
    stop(
      "This OpenDP build expects r-polars 1.7.0 / Polars 1.36.1. ",
      "The installed r-polars version is ", installed, ".",
      call. = FALSE
    )
  }
  engine <- get("PY_VERSION", envir = asNamespace("polars"))
  schema_hash <- get("DSL_SCHEMA_HASH_CURRENT", envir = asNamespace("polars"))
  expected_hash <-
    "4aade69a4a8aba464f5bb77824e189450cfdd5ba9f38a843b2282b1fea4a071d"
  if (!identical(engine, "1.36.1") || !identical(schema_hash, expected_hash)) {
    stop(
      "This OpenDP build expects r-polars 1.7.0 / Polars 1.36.1 ",
      "with DSL schema ", expected_hash, ".",
      call. = FALSE
    )
  }
}

.opendp_polars_plugin_path <- function() {
  dll <- getLoadedDLLs()[["opendp"]]
  if (is.null(dll)) {
    stop("the OpenDP native library is not loaded", call. = FALSE)
  }
  normalizePath(dll[["path"]], mustWork = TRUE)
}

.deserialize_opendp_expr <- function(data) {
  tryCatch(
    polars::pl$deserialize_expr(data),
    error = function(err) {
      if (grepl("unknown variant `FfiPlugin`", conditionMessage(err),
                fixed = TRUE)) {
        stop(
          "OpenDP plugin expressions require r-polars 1.7.0 built with ",
          "the Polars `ffi_plugin` feature.",
          call. = FALSE
        )
      }
      stop(err)
    }
  )
}

#' Construct a domain for an r-polars Series
#'
#' @param name Column name.
#' @param element_domain OpenDP domain for elements in the column.
#' @return A Polars Series domain.
#' @export
series_domain <- function(name, element_domain) {
  if (!is.character(name) || length(name) != 1L || is.na(name) ||
      !nzchar(name)) {
    stop("name must be one non-empty string", call. = FALSE)
  }
  log_ <- new_constructor_log(
    "series_domain",
    "domains",
    new_hashtab(
      list("name", "element_domain"),
      list(name, element_domain)
    )
  )
  .Call(
    "domains__r_polars_series_domain",
    name,
    element_domain,
    log_,
    PACKAGE = "opendp"
  )
}

#' Construct a domain for an r-polars LazyFrame
#'
#' @param series_domains A list of Polars Series domains.
#' @return A Polars LazyFrame domain.
#' @export
lazyframe_domain <- function(series_domains) {
  if (!is.list(series_domains)) {
    stop("series_domains must be a list", call. = FALSE)
  }
  log_ <- new_constructor_log(
    "lazyframe_domain",
    "domains",
    new_hashtab(list("series_domains"), list(series_domains))
  )
  .Call(
    "domains__r_polars_lazyframe_domain",
    series_domains,
    log_,
    PACKAGE = "opendp"
  )
}

#' Add grouped-data bounds to a Polars LazyFrame domain
#'
#' @param domain A Polars LazyFrame domain.
#' @param by Character vector of grouping columns. Pass `character(0)` for a
#'   margin over the whole frame (a single implicit group).
#' @param max_length Optional maximum records in any group.
#' @param max_groups Optional maximum number of groups.
#' @param invariant Optional public invariant of the grouping: `"keys"` if the
#'   set of group keys is public, or `"lengths"` if group keys and their sizes
#'   are public.
#' @return A Polars LazyFrame domain with the grouping margin.
#' @export
with_margin <- function(
    domain,
    by,
    max_length = NULL,
    max_groups = NULL,
    invariant = NULL) {
  if (!is.character(by) || anyNA(by)) {
    stop("by must be a character vector without missing values", call. = FALSE)
  }
  validate_bound <- function(value, name) {
    if (is.null(value)) {
      return()
    }
    if (!is.numeric(value) || length(value) != 1L || !is.finite(value) ||
        value < 0 || value != floor(value) || value > 2^32 - 1) {
      stop(name, " must be NULL or one non-negative uint32", call. = FALSE)
    }
  }
  validate_bound(max_length, "max_length")
  validate_bound(max_groups, "max_groups")
  if (!is.null(invariant) &&
      !(is.character(invariant) && length(invariant) == 1L &&
        invariant %in% c("keys", "lengths"))) {
    stop("invariant must be NULL, \"keys\", or \"lengths\"", call. = FALSE)
  }
  log_ <- new_constructor_log(
    "with_margin",
    "domains",
    new_hashtab(
      list("domain", "by", "max_length", "max_groups", "invariant"),
      list(domain, by, max_length, max_groups, invariant)
    )
  )
  .Call(
    "domains__r_polars_with_margin",
    domain,
    by,
    max_length,
    max_groups,
    invariant,
    log_,
    PACKAGE = "opendp"
  )
}

#' Construct a domain describing an expression's input columns
#'
#' @param series_domains A list of Polars Series domains describing the columns
#'   available to the expression.
#' @param by Grouping columns. `NULL` (the default) describes a row-by-row
#'   expression; a character vector (possibly `character(0)`) describes an
#'   aggregation over the given grouping columns.
#' @param max_length Optional maximum records in any group.
#' @param max_groups Optional maximum number of groups.
#' @param invariant Optional public invariant of the grouping: `"keys"` or
#'   `"lengths"`.
#' @return A Polars wild expression domain.
#' @export
wild_expr_domain <- function(
    series_domains,
    by = NULL,
    max_length = NULL,
    max_groups = NULL,
    invariant = NULL) {
  if (!is.list(series_domains)) {
    stop("series_domains must be a list", call. = FALSE)
  }
  if (!is.null(by) && (!is.character(by) || anyNA(by))) {
    stop("by must be NULL or a character vector without missing values",
         call. = FALSE)
  }
  validate_bound <- function(value, name) {
    if (is.null(value)) {
      return()
    }
    if (!is.numeric(value) || length(value) != 1L || !is.finite(value) ||
        value < 0 || value != floor(value) || value > 2^32 - 1) {
      stop(name, " must be NULL or one non-negative uint32", call. = FALSE)
    }
  }
  validate_bound(max_length, "max_length")
  validate_bound(max_groups, "max_groups")
  if (!is.null(invariant) &&
      !(is.character(invariant) && length(invariant) == 1L &&
        invariant %in% c("keys", "lengths"))) {
    stop("invariant must be NULL, \"keys\", or \"lengths\"", call. = FALSE)
  }
  if (is.null(by) && (!is.null(max_length) || !is.null(max_groups) ||
                      !is.null(invariant))) {
    stop("max_length, max_groups and invariant require a non-NULL `by`",
         call. = FALSE)
  }
  log_ <- new_constructor_log(
    "wild_expr_domain",
    "domains",
    new_hashtab(
      list("series_domains", "by", "max_length", "max_groups", "invariant"),
      list(series_domains, by, max_length, max_groups, invariant)
    )
  )
  .Call(
    "domains__r_polars_wild_expr_domain",
    series_domains,
    by,
    max_length,
    max_groups,
    invariant,
    log_,
    PACKAGE = "opendp"
  )
}

#' Round-trip a serialized r-polars expression through OpenDP
#'
#' This is a temporary integration endpoint for the matched r-polars 1.7.0
#' compatibility spike.
#'
#' @param data A raw vector returned by `Expr$meta$serialize()`.
#' @return Serialized expression bytes as a raw vector.
#' @export
opendp_roundtrip_expr <- function(data) {
  .assert_r_polars_compatibility()
  .Call("data__roundtrip_polars", data, "Expr", PACKAGE = "opendp")
}

#' Round-trip a serialized r-polars lazy frame through OpenDP
#'
#' This is a temporary integration endpoint for the matched r-polars 1.7.0
#' compatibility spike.
#'
#' @param data A raw vector returned by `LazyFrame$serialize()`.
#' @return Serialized logical-plan bytes as a raw vector.
#' @export
opendp_roundtrip_lazyframe <- function(data) {
  .assert_r_polars_compatibility()
  .Call("data__roundtrip_polars", data, "LazyFrame", PACKAGE = "opendp")
}

#' Construct a differentially private length expression
#'
#' @param scale Optional noise scale.
#' @param signed If `TRUE`, the length is released as a signed integer, allowing
#'   unbiased (possibly negative) noise; otherwise it is an unsigned integer.
#' @return An ordinary r-polars expression.
#' @export
dp_len <- function(scale = NULL, signed = FALSE) {
  .assert_r_polars_compatibility()
  if (!is.null(scale) &&
      (!is.numeric(scale) || length(scale) != 1L || !is.finite(scale) ||
       scale < 0)) {
    stop("scale must be NULL or one finite non-negative number", call. = FALSE)
  }
  if (!is.logical(signed) || length(signed) != 1L || is.na(signed)) {
    stop("signed must be TRUE or FALSE", call. = FALSE)
  }
  data <- .Call(
    "data__make_r_polars_dp_len",
    scale,
    signed,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

#' Construct a differentially private bounded-sum expression
#'
#' @param expr An ordinary r-polars expression.
#' @param bounds Finite lower and upper bounds.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_sum <- function(expr, bounds, scale = NULL) {
  .assert_r_polars_compatibility()
  if (!is.numeric(bounds) || length(bounds) != 2L ||
      any(!is.finite(bounds))) {
    stop("bounds must contain two finite numbers", call. = FALSE)
  }
  if (!is.null(scale) &&
      (!is.numeric(scale) || length(scale) != 1L || !is.finite(scale) ||
       scale < 0)) {
    stop("scale must be NULL or one finite non-negative number", call. = FALSE)
  }
  expr <- polars::as_polars_expr(expr)
  data <- .Call(
    "data__make_r_polars_dp_sum",
    expr$meta$serialize(format = "binary"),
    as.numeric(bounds),
    scale,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

#' Construct a differentially private bounded-mean expression
#'
#' @param expr An ordinary r-polars expression.
#' @param bounds Finite lower and upper bounds.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_mean <- function(expr, bounds, scale = NULL) {
  .assert_r_polars_compatibility()
  if (!is.numeric(bounds) || length(bounds) != 2L ||
      any(!is.finite(bounds))) {
    stop("bounds must contain two finite numbers", call. = FALSE)
  }
  if (!is.null(scale) &&
      (!is.numeric(scale) || length(scale) != 1L || !is.finite(scale) ||
       scale < 0)) {
    stop("scale must be NULL or one finite non-negative number", call. = FALSE)
  }
  expr <- polars::as_polars_expr(expr)
  data <- .Call(
    "data__make_r_polars_dp_mean",
    expr$meta$serialize(format = "binary"),
    as.numeric(bounds),
    scale,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

.validate_candidates <- function(candidates) {
  if (!is.numeric(candidates) || length(candidates) == 0L ||
      any(!is.finite(candidates))) {
    stop("candidates must be a non-empty vector of finite numbers",
         call. = FALSE)
  }
}

.validate_optional_scale <- function(scale) {
  if (!is.null(scale) &&
      (!is.numeric(scale) || length(scale) != 1L || !is.finite(scale) ||
       scale < 0)) {
    stop("scale must be NULL or one finite non-negative number", call. = FALSE)
  }
}

#' Construct a differentially private median expression
#'
#' @param expr An ordinary r-polars expression.
#' @param candidates A non-empty numeric vector of candidate values to select
#'   from.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_median <- function(expr, candidates, scale = NULL) {
  .assert_r_polars_compatibility()
  .validate_candidates(candidates)
  .validate_optional_scale(scale)
  expr <- polars::as_polars_expr(expr)
  data <- .Call(
    "data__make_r_polars_dp_median",
    expr$meta$serialize(format = "binary"),
    as.numeric(candidates),
    scale,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

#' Construct a differentially private quantile expression
#'
#' @param expr An ordinary r-polars expression.
#' @param alpha The quantile to estimate, in `[0, 1]`. Use `0.5` for the median.
#' @param candidates A non-empty numeric vector of candidate values to select
#'   from.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_quantile <- function(expr, alpha, candidates, scale = NULL) {
  .assert_r_polars_compatibility()
  if (!is.numeric(alpha) || length(alpha) != 1L || !is.finite(alpha) ||
      alpha < 0 || alpha > 1) {
    stop("alpha must be one number between 0 and 1", call. = FALSE)
  }
  .validate_candidates(candidates)
  .validate_optional_scale(scale)
  expr <- polars::as_polars_expr(expr)
  data <- .Call(
    "data__make_r_polars_dp_quantile",
    expr$meta$serialize(format = "binary"),
    as.numeric(alpha),
    as.numeric(candidates),
    scale,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

.make_unary_scale_expr <- function(entry, expr, scale) {
  .assert_r_polars_compatibility()
  .validate_optional_scale(scale)
  expr <- polars::as_polars_expr(expr)
  data <- .Call(
    entry,
    expr$meta$serialize(format = "binary"),
    scale,
    .opendp_polars_plugin_path(),
    PACKAGE = "opendp"
  )
  .deserialize_opendp_expr(data)
}

#' Add differentially private noise to an expression
#'
#' The noise distribution is chosen from the privacy measure: Laplace under
#' pure differential privacy, Gaussian under zero-concentrated differential
#' privacy.
#'
#' @param expr An ordinary r-polars expression.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_noise <- function(expr, scale = NULL) {
  .make_unary_scale_expr("data__make_r_polars_dp_noise", expr, scale)
}

#' Construct a differentially private count expression (excluding nulls)
#'
#' @param expr An ordinary r-polars expression.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_count <- function(expr, scale = NULL) {
  .make_unary_scale_expr("data__make_r_polars_dp_count", expr, scale)
}

#' Construct a differentially private null-count expression
#'
#' @param expr An ordinary r-polars expression.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_null_count <- function(expr, scale = NULL) {
  .make_unary_scale_expr("data__make_r_polars_dp_null_count", expr, scale)
}

#' Construct a differentially private count-of-unique-elements expression
#'
#' @param expr An ordinary r-polars expression.
#' @param scale Optional noise scale.
#' @return An ordinary r-polars expression.
#' @export
dp_n_unique <- function(expr, scale = NULL) {
  .make_unary_scale_expr("data__make_r_polars_dp_n_unique", expr, scale)
}

.new_onceframe_release <- function(queryable) {
  release <- new.env(parent = emptyenv())
  release$collect <- function() {
    data <- .Call(
      "data__onceframe_collect_r_polars",
      queryable,
      PACKAGE = "opendp"
    )
    polars::pl$deserialize_df(data)
  }
  class(release) <- "opendp_onceframe_release"
  release
}

#' Construct a private r-polars LazyFrame measurement
#'
#' @param input_domain A Polars LazyFrame domain.
#' @param input_metric The dataset adjacency metric.
#' @param output_measure The privacy measure.
#' @param lazyframe An r-polars LazyFrame containing OpenDP expressions.
#' @param global_scale Optional global noise scale.
#' @param threshold Optional private-key release threshold.
#' @return An OpenDP measurement. Invoking it with an r-polars LazyFrame returns
#'   a single-use release object whose `collect()` method returns an r-polars
#'   DataFrame.
#' @export
make_private_lazyframe <- function(
    input_domain,
    input_metric,
    output_measure,
    lazyframe,
    global_scale = NULL,
    threshold = NULL) {
  .assert_r_polars_compatibility()
  if (!is.null(global_scale) &&
      (!is.numeric(global_scale) || length(global_scale) != 1L ||
       !is.finite(global_scale) || global_scale < 0)) {
    stop(
      "global_scale must be NULL or one finite non-negative number",
      call. = FALSE
    )
  }
  if (!is.null(threshold) &&
      (!is.numeric(threshold) || length(threshold) != 1L ||
       !is.finite(threshold) || threshold < 0 ||
       threshold != floor(threshold) || threshold > 2^32 - 1)) {
    stop("threshold must be NULL or one non-negative uint32", call. = FALSE)
  }
  lazyframe <- polars::as_polars_lf(lazyframe)
  log_ <- new_constructor_log(
    "make_private_lazyframe",
    "measurements",
    new_hashtab(
      list(
        "input_domain", "input_metric", "output_measure", "lazyframe",
        "global_scale", "threshold"
      ),
      list(
        input_domain, input_metric, output_measure, lazyframe,
        global_scale, threshold
      )
    )
  )
  measurement <- .Call(
    "measurements__make_private_r_polars_lazyframe",
    input_domain,
    input_metric,
    output_measure,
    lazyframe$serialize(format = "binary"),
    global_scale,
    if (is.null(threshold)) NULL else as.integer(threshold),
    log_,
    PACKAGE = "opendp"
  )

  private_measurement <- function(attr, arg, d_in, d_out) {
    if (!missing(arg)) {
      private_data <- polars::as_polars_lf(arg)
      queryable <- .Call(
        "core__invoke_r_polars_lazyframe",
        measurement,
        private_data$serialize(format = "binary"),
        PACKAGE = "opendp"
      )
      return(.new_onceframe_release(queryable))
    }
    if (!missing(d_in)) {
      if (missing(d_out)) {
        return(measurement(d_in = d_in))
      }
      return(measurement(d_in = d_in, d_out = d_out))
    }
    measurement(attr = attr)
  }
  class(private_measurement) <- "measurement"
  private_measurement
}
