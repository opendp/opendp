test_that("user-defined transformation works in R", {
  enable_features("contrib", "honest-but-curious")

  trans <- make_user_transformation(
    input_domain = vector_domain(atom_domain(.T = i32)),
    input_metric = symmetric_distance(),
    output_domain = vector_domain(atom_domain(.T = i32)),
    output_metric = symmetric_distance(),
    function_ = \(arg) rep(arg + 1L, 2),
    stability_map = \(d_in) d_in * 2L
  )

  expect_equal(trans(arg = c(1L, 2L)), c(2L, 3L, 2L, 3L))
  expect_equal(trans(d_in = 1L), 2L)
})

test_that("nullable Rust diagnostics are copied before their error is freed", {
  enable_features("contrib")
  # Zero counts produce NaN interpolation, whose integer cast has no message.
  quantiles <- make_quantiles_from_counts(c(0L, 1L, 2L), 0.5)
  caught <- tryCatch(quantiles(arg = c(0L, 0L)), error = identity)
  expect_identical(caught$variant, "FailedCast")
  expect_null(caught$raw_message)
  gc()
  expect_match(conditionMessage(caught), "FailedCast")
  expect_type(caught$backtrace, "character")
})

test_that("empty or malformed callback messages become canonical FFI errors", {
  enable_features("contrib", "honest-but-curious")
  for (message in list(NULL, character(), NA_character_, 1L)) {
    original <- structure(list(message = message, call = NULL), class = c("error", "condition"))
    postprocess <- new_function(function_ = \(.arg) stop(original), .TO = i32) # nolint: condition_call_linter.
    caught <- tryCatch(postprocess(arg = 1L), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "FFI")
    expect_identical(caught$raw_message, "")
    expect_match(conditionMessage(caught), "^[[]FFI[]] : ")
  }
})

test_that("scalar extrinsic callback results release their temporary reference", {
  enable_features("contrib", "honest-but-curious")
  released <- FALSE
  finalize <- function(.payload) released <<- TRUE
  domain <- user_domain("ScalarTemporary", member = \(.arg) TRUE)
  metric <- user_distance("ScalarTemporary")
  trans <- make_user_transformation(domain, metric, domain, metric, identity, function(.d_in) {
    payload <- new.env(parent = emptyenv())
    reg.finalizer(payload, finalize)
    structure(1L, payload = payload)
  })
  expect_true(trans(d_in = 0L, d_out = 2L))
  gc()
  expect_true(released)
})

test_that("user-defined measurement supports typed and extrinsic outputs", {
  enable_features("contrib", "honest-but-curious")

  typed_meas <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(.arg) 23L,
    privacy_map = \(.d_in) 0.,
    .TO = i32
  )

  expect_equal(typed_meas(arg = 1L), 23L)
  expect_equal(typed_meas(d_in = 200L), 0.)

  extrinsic_meas <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(arg) list(value = arg, tag = "release"),
    privacy_map = \(.d_in) 0.
  )

  expect_equal(extrinsic_meas(arg = 7L)$value, 7L)
  expect_equal(extrinsic_meas(arg = 7L)$tag, "release")
})

test_that("new function and privacy profile work in R", {
  enable_features("contrib", "honest-but-curious")

  postprocess <- new_function(
    function_ = \(arg) arg[[1]] / arg[[2]],
    .TO = f64
  )
  expect_equal(postprocess(arg = list(12., 100.)), 0.12)

  profile <- privacy_profile(log_delta = \(epsilon) ifelse(epsilon < 0.5, 0., -10000.))
  expect_equal(profile(epsilon = 0.499), 1.)
  expect_equal(profile(log_delta = -10000.), 0.5)
  expect_equal(profile(log_delta_at = 0.5), -10000.)
  expect_gt(profile(epsilon = 0.5), 0.)

  slack_profile <- privacy_profile(
    log_delta = \(epsilon) ifelse(epsilon < 2., 0., -Inf), delta_slack = 0.1
  )
  expect_equal(slack_profile(epsilon = 2.), 0.1)
  expect_equal(slack_profile(delta = 0.1), 2.)
  expect_equal(slack_profile(delta = 0.09), Inf)
})

test_that("privacy profiles call vector forward callbacks once per batch", {
  enable_features("contrib", "honest-but-curious")
  for (is_log in c(FALSE, TRUE)) {
    calls <- list()
    callback <- function(values) {
      expect_type(values, "double")
      calls[[length(calls) + 1L]] <<- values
      if (is_log) -values else exp(-values)
    }
    profile <- if (is_log) privacy_profile(log_delta = callback) else privacy_profile(delta = callback)
    expect_length(calls, 0)
    expect_equal(profile(epsilon = 1.), exp(-1.))
    expect_identical(calls, list(1.))
    expect_equal(profile(epsilon = c(0., 1., 1.)), c(1., exp(-1.), exp(-1.)))
    expect_length(calls, 2)
    expect_identical(calls[[2]], c(0., 1., 1.))
    expect_equal(profile(log_delta_at = c(0., 1., 1.)), c(0., -1., -1.))
    expect_length(calls, 3)
    expect_equal(profile(epsilon = numeric()), numeric())
  }
})

test_that("privacy profiles generically invert scalar and vector budgets", {
  enable_features("contrib", "honest-but-curious")
  calls <- 0L
  forward <- function(values) {
    stopifnot(is.double(values), length(values) == 1L)
    calls <<- calls + 1L
    -values
  }
  profile <- privacy_profile(log_delta = forward)
  expect_equal(profile(log_delta = -10000.), 10000.)
  log_budgets <- c(-Inf, -10000., -2., -2., -1., 0.)
  expect_equal(profile(log_delta = log_budgets), -log_budgets)
  expect_identical(profile(log_delta = log_budgets), vapply(log_budgets, function(value) profile(log_delta = value), numeric(1)))
  budgets <- c(0., 0.25, 0.25, 1.)
  expect_equal(profile(delta = budgets), c(Inf, -log(0.25), -log(0.25), 0.))
  expect_identical(profile(delta = budgets), vapply(budgets, function(value) profile(delta = value), numeric(1)))
  slack <- privacy_profile(log_delta = forward, delta_slack = 0.25)
  expect_equal(slack(delta = c(0.1, 0.25, 0.25, 1.)), c(Inf, Inf, Inf, 0.))
  before <- calls
  expect_equal(slack(delta = c(0.1, 1.)), c(Inf, 0.))
  expect_equal(slack(delta = numeric()), numeric())
  expect_equal(slack(log_delta = numeric()), numeric())
  expect_identical(calls, before)
})

test_that("vector privacy profiles validate ordering and callback lengths", {
  enable_features("contrib", "honest-but-curious")
  profile <- privacy_profile(log_delta = function(.values) stop("must not evaluate unsorted inputs", call. = FALSE))
  expect_error(profile(epsilon = c(1., 0.)), "nondecreasing")
  expect_error(profile(log_delta_at = c(1., 0.)), "nondecreasing")
  expect_error(profile(delta = c(0.2, 0.1)), "nondecreasing")
  expect_error(profile(log_delta = c(-1., -2.)), "nondecreasing")
  for (size in c(0L, 1L, 3L)) {
    callback <- function(.values) rep(0., size)
    profile <- privacy_profile(log_delta = callback)
    expect_error(profile(epsilon = c(0., 1.)), "output length")
    expect_error(profile(log_delta_at = c(0., 1.)), "output length")
    if (size != 1L) {
      # Generic inversion checks singleton forward callback outputs.
      expect_error(profile(delta = c(0.1, 0.2)), "output length")
      expect_error(profile(log_delta = c(-2., -1.)), "output length")
    }
  }
  expect_error(privacy_profile(delta = function(.values) numeric())(epsilon = 1.), "output length")
  expect_error(privacy_profile(), "exactly one")
  expect_error(privacy_profile(delta = identity, log_delta = identity), "exactly one")
  original <- simpleError("vector callback failure")
  fail <- function(.values) stop(original) # nolint: condition_call_linter.
  profile <- privacy_profile(log_delta = fail)
  expect_identical(tryCatch(profile(epsilon = c(0., 1.)), error = identity), original)
  expect_identical(tryCatch(profile(delta = c(0.1, 0.2)), error = identity), original)
})

test_that("deprecated privacy profile adapts scalar callbacks for vector queries", {
  enable_features("contrib", "honest-but-curious")
  calls <- numeric()
  scalar <- function(value) {
    expect_length(value, 1)
    calls <<- c(calls, value)
    exp(-value)
  }
  expect_warning({
    profile <- new_privacy_profile(scalar)
  }, "vector-valued")
  expect_equal(profile(epsilon = c(0., 1., 1.)), c(1., exp(-1.), exp(-1.)))
  expect_identical(calls, c(0., 1., 1.))
  expect_equal(profile(delta = 0.5), log(2.))
})

test_that("user-defined callback errors surface cleanly", {
  enable_features("contrib", "honest-but-curious")

  postprocess <- new_function(
    function_ = \(arg) stop("boom from postprocess", call. = FALSE),
    .TO = f64
  )

  meas <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(arg) stop("boom from release", call. = FALSE),
    privacy_map = \(.d_in) 0.,
    .TO = i32
  )

  meas_with_bad_map <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(.arg) 23L,
    privacy_map = \(.d_in) stop("boom from privacy map", call. = FALSE),
    .TO = i32
  )
  queryable <- new_queryable(\(.query) stop("boom from transition", call. = FALSE), .Q = i32, .A = i32)
  for (invoke in list(
    \() postprocess(arg = 1.),
    \() meas(arg = 1L),
    \() meas_with_bad_map(d_in = 1L),
    \() queryable(query = 1L)
  )) {
    caught <- tryCatch(invoke(), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "FFI")
    expect_match(caught$raw_message, "^boom from ")
    expect_match(conditionMessage(caught), "^[[]FFI[]] : boom from ")
  }
})

# These closures raise genuine Rust errors, not user-authored variant headers.
opendp_callback_failures <- function() {
  sum_trans <- make_sum(
    vector_domain(atom_domain(.T = i32, bounds = c(0L, 100L))),
    symmetric_distance()
  )
  measurement <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = identity,
    privacy_map = \(.d_in) NaN,
    .TO = i32
  )
  zcdp <- make_pureDP_to_zCDP(measurement)
  list(
    Overflow = \() sum_trans(d_in = 100000000L),
    NumericError = \() zcdp(d_in = 1L),
    MetricSpace = \() make_laplace(atom_domain(.T = f64), absolute_distance(f64), scale = 1.)
  )
}

test_that("structured OpenDP errors survive callback and transition boundaries", {
  enable_features("contrib", "honest-but-curious")

  failures <- opendp_callback_failures()
  for (variant in names(failures)) {
    fail <- failures[[variant]]
    original <- tryCatch(fail(), error = identity)
    expect_identical(class(original), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(original$variant, variant)
    expect_null(original$call)
    expect_type(original$raw_message, "character")
    expect_type(original$backtrace, "character")
    expect_match(conditionMessage(original), paste0("^[[]", variant, "[]] : "))

    postprocess <- new_function(function_ = \(.arg) fail(), .TO = i32)
    transitions <- list(
      \(.query) fail(),
      \(.query, .is_internal) fail()
    )
    calls <- c(
      list(\() postprocess(arg = 1L)),
      lapply(transitions, function(transition) {
        queryable <- new_queryable(transition, .Q = i32, .A = i32)
        \() queryable(query = 1L)
      })
    )
    for (invoke in calls) {
      caught <- tryCatch(invoke(), error = identity)
      expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
      expect_identical(caught$variant, variant)
      expect_identical(caught$raw_message, original$raw_message)
      expect_type(caught$backtrace, "character")
      expect_match(conditionMessage(caught), paste0("^[[]", variant, "[]] : "))
    }
  }
})

test_that("callback text cannot impersonate structured OpenDP errors", {
  enable_features("contrib", "honest-but-curious")

  for (message in c(
    "[Overflow] : callback failure",
    "[NumericError] : callback failure",
    "Error: [Overflow] : callback failure",
    "Error in base::eval(\"quoted : call\") : [Overflow] : callback failure",
    "\n [Overflow] : multiline\n[NumericError] : nested",
    "prefix Overflow: message",
    "Overflow: message",
    "[UnknownVariant] : message",
    "[OverflowX] : message"
  )) {
    original <- simpleError(message)
    postprocess <- new_function(
      function_ = \(.arg) stop(original), # nolint: condition_call_linter.
      .TO = i32
    )
    caught <- tryCatch(postprocess(arg = 1L), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "FFI")
    expect_identical(caught$raw_message, message)
  }

  # Keep call. = TRUE to exercise namespace qualifiers and quoted colons.
  postprocess <- new_function(
    function_ = \(.arg) base::eval(quote(stop("[Overflow] : callback failure"))), # nolint: condition_call_linter.
    .TO = i32
  )
  expect_identical(tryCatch(postprocess(arg = 1L), error = identity)$variant, "FFI")

  for (variant in list(NULL, NA_character_, 1L, c("Overflow", "NumericError"), "UnknownVariant")) {
    malformed <- structure(
      list(message = "[Overflow] : malformed", call = NULL, variant = variant),
      class = c("opendp_error", "error", "condition")
    )
    postprocess <- new_function(function_ = \(.arg) stop(malformed), .TO = i32) # nolint: condition_call_linter.
    caught <- tryCatch(postprocess(arg = 1L), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "FFI")
    expect_identical(caught$raw_message, malformed$message)
  }
})

test_that("extrinsic comparisons return equality and both orderings", {
  enable_features("contrib", "honest-but-curious")
  domain <- user_domain("ComparisonDomain", member = \(.x) TRUE)
  metric <- user_distance("ComparisonDistance")
  trans <- make_user_transformation(
    input_domain = domain, input_metric = metric,
    output_domain = domain, output_metric = metric,
    function_ = identity, stability_map = identity
  )
  expect_true(trans(d_in = 1L, d_out = 1L))
  expect_true(trans(d_in = 1L, d_out = 2L))
  expect_false(trans(d_in = 2L, d_out = 1L))
})

test_that("incomparable extrinsic distances propagate an ordinary FFI failure", {
  enable_features("contrib", "honest-but-curious")
  domain <- user_domain("ComparisonDomain", member = \(.x) TRUE)
  metric <- user_distance("ComparisonDistance")
  trans <- make_user_transformation(
    input_domain = domain, input_metric = metric,
    output_domain = domain, output_metric = metric,
    function_ = identity, stability_map = identity
  )
  caught <- tryCatch(trans(d_in = NaN, d_out = 1L), error = identity)
  expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(caught$variant, "FFI")
  expect_match(conditionMessage(caught), "objects are not comparable")
})

test_that("user-defined callbacks surface conversion failures cleanly", {
  enable_features("contrib", "honest-but-curious")

  postprocess <- new_function(
    function_ = \(.arg) "not an integer",
    .TO = i32
  )
  expect_error(
    postprocess(arg = 1L),
    "FailedFunction|integer"
  )

  meas <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(.arg) "not an integer",
    privacy_map = \(.d_in) 0.,
    .TO = i32
  )
  expect_error(
    meas(arg = 1L),
    "FailedFunction|integer"
  )
})

test_that("new queryable supports user-defined transitions", {
  enable_features("contrib")

  queryable <- new_queryable(
    transition = \(query, is_internal = FALSE) {
      if (is_internal) {
        return(query)
      }
      query + 1L
    },
    .Q = i32,
    .A = i32
  )

  expect_equal(queryable(query = 2L), 3L)
})

test_that("library extension supporting elements have python-style parity", {
  enable_features("honest-but-curious")

  domain_descriptor <- list(kind = "bounded", limit = 3L)
  domain <- user_domain(
    identifier = "DemoDomain",
    member = function(x) is.integer(x) && length(x) == 1 && x >= 0L && x <= 3L,
    descriptor = domain_descriptor
  )

  expect_identical(domain("descriptor")$kind, "bounded")
  expect_identical(domain("descriptor")$limit, 3L)
  expect_true(domain(member = 2L))
  expect_false(domain(member = 5L))

  metric_descriptor <- list(inner = "count")
  metric <- user_distance(
    identifier = "DemoDistance",
    descriptor = metric_descriptor
  )

  expect_identical(metric("descriptor")$inner, "count")

  measure_descriptor <- list(curve = "demo")
  measure <- user_divergence(
    identifier = "DemoDivergence",
    descriptor = measure_descriptor
  )
  expect_identical(measure("descriptor")$curve, "demo")

  internal_measure <- getFromNamespace("_extrinsic_divergence", "opendp")(
    identifier = "InternalDivergence",
    descriptor = list(curve = "internal")
  )
  expect_identical(internal_measure("descriptor")$curve, "internal")
})

test_that("extrinsic domain member errors surface cleanly", {
  enable_features("honest-but-curious")

  domain <- user_domain(
    identifier = "ErrorDomain",
    member = \(x) stop("boom from member", call. = FALSE),
    descriptor = list(kind = "error")
  )

  expect_error(
    domain(member = 1L),
    "FailedFunction|boom from member"
  )
})

test_that("incomparable extrinsic descriptors fail cleanly", {
  enable_features("honest-but-curious")

  metric_a <- user_distance(
    identifier = "DistanceA",
    descriptor = environment()
  )
  metric_b <- user_distance(
    identifier = "DistanceB",
    descriptor = environment()
  )

  expect_error(
    metric_a == metric_b,
    "not comparable|FailedFunction|comparison"
  )
})
