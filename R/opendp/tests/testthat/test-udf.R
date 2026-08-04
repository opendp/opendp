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

  profile <- new_privacy_profile(\(epsilon) if (epsilon < 0.5) 1. else 1e-8)
  expect_equal(profile(epsilon = 0.499), 1.)
  expect_equal(profile(delta = 1.1e-8), 0.5)
})

test_that("user-defined callback errors surface cleanly", {
  enable_features("contrib", "honest-but-curious")

  postprocess <- new_function(
    function_ = \(arg) stop("boom from postprocess", call. = FALSE),
    .TO = f64
  )
  expect_error(
    postprocess(arg = 1.),
    "FailedFunction|boom from postprocess"
  )

  meas <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(arg) stop("boom from release", call. = FALSE),
    privacy_map = \(.d_in) 0.,
    .TO = i32
  )
  expect_error(
    meas(arg = 1L),
    "FailedFunction|boom from release"
  )

  meas_with_bad_map <- make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = \(.arg) 23L,
    privacy_map = \(.d_in) stop("boom from privacy map", call. = FALSE),
    .TO = i32
  )
  expect_error(
    meas_with_bad_map(d_in = 1L),
    "FailedFunction|boom from privacy map"
  )
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
      expect_s3_class(caught, "opendp_error")
      expect_identical(caught$variant, variant)
      expect_match(caught$raw_message, original$raw_message, fixed = TRUE)
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
    postprocess <- new_function(
      function_ = \(.arg) stop(message, call. = FALSE),
      .TO = i32
    )
    caught <- tryCatch(postprocess(arg = 1L), error = identity)
    expect_s3_class(caught, "opendp_error")
    expect_identical(caught$variant, "FFI")
    expect_match(caught$raw_message, message, fixed = TRUE)
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
    expect_identical(caught$variant, "FFI")
    expect_match(caught$raw_message, "malformed", fixed = TRUE)
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

test_that("extrinsic comparisons transport structured conditions, not headers", {
  enable_features("contrib", "honest-but-curious")

  registerS3method("Ops", "opendp_test_descriptor", function(e1, e2) {
    stop(attr(e1, "failure")) # nolint: condition_call_linter.
  })
  withr::defer(rm("Ops.opendp_test_descriptor", envir = asNamespace("base")$.__S3MethodsTable__.))
  failures <- lapply(opendp_callback_failures(), function(fail) tryCatch(fail(), error = identity))
  failures$FFI <- simpleError("[Overflow] : comparison spoof")
  for (variant in names(failures)) {
    left <- structure(list(1L), failure = failures[[variant]], class = "opendp_test_descriptor")
    right <- structure(list(2L), failure = failures[[variant]], class = "opendp_test_descriptor")
    domain <- user_domain("ComparisonDomain", member = \(.x) TRUE)
    metric <- user_distance("ComparisonDistance")
    trans <- make_user_transformation(
      input_domain = domain,
      input_metric = metric,
      output_domain = domain,
      output_metric = metric,
      function_ = identity,
      stability_map = \(.d_in) left
    )
    caught <- tryCatch(trans(d_in = left, d_out = right), error = identity)
    expect_s3_class(caught, "opendp_error")
    expect_identical(caught$variant, variant)
    expect_match(caught$raw_message, "overflows|NaN operand|non-nan|comparison spoof")
  }
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
