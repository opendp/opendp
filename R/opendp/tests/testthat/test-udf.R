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
  expect_equal(profile(delta = 1e-8), 0.5)
})

test_that("user-defined callback errors surface cleanly", {
  enable_features("contrib", "honest-but-curious")

  postprocess <- new_function(
    function_ = \(arg) stop("boom from postprocess", call. = FALSE),
    .TO = f64
  )
  caught <- tryCatch(postprocess(arg = 1.), error = identity)
  expect_s3_class(caught, "opendp_error")
  expect_identical(caught$variant, "FFI")
  expect_identical(caught$raw_message, "boom from postprocess")

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
    "boom from release"
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
    "boom from privacy map"
  )
})

test_that("Rust overflow survives R callbacks and transitions", {
  enable_features("contrib", "honest-but-curious")
  sum_trans <- make_sum(
    vector_domain(atom_domain(.T = i32, bounds = c(0L, 100L))),
    symmetric_distance()
  )
  fail <- \(.arg) sum_trans(d_in = 100000000L)
  original <- tryCatch(fail(1L), error = identity)
  expect_identical(original$variant, "Overflow")
  fn <- new_function(function_ = fail, .TO = i32)
  q <- new_queryable(fail, .Q = i32, .A = i32)
  for (invoke in list(\() fn(arg = 1L), \() q(query = 1L))) {
    caught <- tryCatch(invoke(), error = identity)
    expect_s3_class(caught, "opendp_error")
    expect_identical(caught$variant, "Overflow")
    expect_identical(caught$raw_message, original$raw_message)
  }
})

test_that("untyped callback messages are not parsed as variants", {
  enable_features("contrib", "honest-but-curious")
  message <- "[Overflow] : callback \u03bb"
  fn <- new_function(function_ = \(.arg) stop(message, call. = FALSE), .TO = i32)
  caught <- tryCatch(fn(arg = 1L), error = identity)
  expect_identical(caught$variant, "FFI")
  expect_identical(caught$raw_message, message)
})

test_that("nullable Rust diagnostics survive error cleanup and callbacks", {
  enable_features("contrib", "honest-but-curious")
  quantiles <- make_quantiles_from_counts(c(0L, 1L, 2L), 0.5)
  caught <- tryCatch(quantiles(arg = c(0L, 0L)), error = identity)
  expect_identical(caught$variant, "FailedCast")
  expect_null(caught$raw_message)
  gc()
  expect_match(conditionMessage(caught), "FailedCast")

  fn <- new_function(function_ = \(.arg) quantiles(arg = c(0L, 0L)), .TO = i32)
  caught <- tryCatch(fn(arg = 1L), error = identity)
  expect_identical(caught$variant, "FailedCast")
  expect_null(caught$raw_message)
})

test_that("extrinsic comparison results and failures cross the callback boundary", {
  enable_features("contrib", "honest-but-curious")
  domain <- user_domain("ComparisonDomain", member = \(.x) TRUE)
  metric <- user_distance("ComparisonDistance")
  trans <- make_user_transformation(
    input_domain = domain, input_metric = metric,
    output_domain = domain, output_metric = metric,
    function_ = identity, stability_map = identity
  )
  expect_true(trans(d_in = 1L, d_out = 1L))
  expect_false(trans(d_in = 2L, d_out = 1L))
  caught <- tryCatch(trans(d_in = NaN, d_out = 1L), error = identity)
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
    "integer"
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
    "integer"
  )
})

test_that("callbacks can retain borrowed opaque queryables", {
  enable_features("contrib", "honest-but-curious")
  for (i in 1:5) {
    meas <- make_randomized_response_bool(prob = 0.75)
    adaptive <- make_adaptive_composition(
      meas("input_domain"), meas("input_metric"), meas("output_measure"),
      d_in = 1L, d_mids = c(2., 2.)
    )
    saved <- NULL
    post <- new_function(function_ = function(q) {
      saved <<- q
      1L
    }, .TO = i32)
    chained <- make_chain_pm(post, adaptive)
    expect_equal(chained(arg = TRUE), 1L)
    rm(chained, post, adaptive)
    gc()
    expect_type(saved(query = meas), "logical")
    expect_type(saved(query = meas), "logical")
    expect_error(saved(query = meas))
    rm(saved, meas)
    gc()
  }
})

test_that("extrinsic descriptors and callback results release their R references", {
  enable_features("contrib", "honest-but-curious")
  released <- 0L
  on_finalize <- function(e) released <<- released + 1L
  for (i in 1:20) local({
    # Avoid a cross-language cycle through the environment's parent or callback.
    value <- new.env(parent = emptyenv())
    reg.finalizer(value, on_finalize)
    metric <- user_distance("LifetimeTest", descriptor = list(value = value))
    expect_identical(metric("descriptor")$value, value)
    callback <- function(x) value
    environment(callback) <- list2env(list(value = value), parent = baseenv())
    fn <- new_function(function_ = callback, .TO = "ExtrinsicObject")
    expect_identical(fn(arg = 1L), value)
  })
  gc()
  gc()
  expect_equal(released, 20L)
})

test_that("callback userdata is released when later argument conversion raises", {
  # The native fixture calls the same converters as generated wrappers, then
  # raises before a Rust call. Source headers are absent in installed-only tests.
  src <- test_path("..", "..", "src")
  skip_if_not(file.exists(file.path(src, "opendp.h")), "requires generated source headers")
  build <- tempfile("callback-scope-")
  dir.create(build)
  on.exit(unlink(build, recursive = TRUE), add = TRUE)
  file.copy(test_path("..", "native", "callback-scope.c"), build)
  file.copy(file.path(src, c("convert.h", "ownership.h", "opendp.h")), build)
  dll <- file.path(build, paste0("callback-scope", .Platform$dynlib.ext))
  output <- system2(file.path(R.home("bin"), "R"),
    c("CMD", "SHLIB", shQuote(file.path(build, "callback-scope.c")), "-o", shQuote(dll)),
    stdout = TRUE, stderr = TRUE
  )
  expect_null(attr(output, "status"), info = paste(output, collapse = "\n"))
  fixture <- dyn.load(dll)
  on.exit(dyn.unload(dll), add = TRUE, after = FALSE)
  invoke <- getNativeSymbolInfo("test_callback_scope", fixture)$address
  scope <- getNativeSymbolInfo("opendp_with_scope", "opendp")$address
  unregistered <- getNativeSymbolInfo("test_forget_unregistered", fixture)$address
  forget <- getNativeSymbolInfo("opendp_forget_resource", "opendp")$address
  expect_error(.Call(unregistered, scope, forget), "unregistered OpenDP resource")
  released <- 0L
  on_finalize <- function(e) released <<- released + 1L
  for (transition in c(FALSE, TRUE)) {
    converter <- getNativeSymbolInfo(
      if (transition) "sexp_to_transitionfn" else "sexp_to_callbackfn", "opendp"
    )$address
    for (i in 1:10) local({
      value <- new.env(parent = emptyenv())
      reg.finalizer(value, on_finalize)
      callback <- function(arg) value
      environment(callback) <- list2env(list(value = value), parent = baseenv())
      expect_error(.Call(invoke, callback, transition, scope, converter), "later argument conversion failed")
    })
  }
  gc()
  gc()
  expect_equal(released, 20L)

  # Retained callback entry points must work after their creating scope ends.
  direct <- getNativeSymbolInfo("test_callback_without_wrapper", fixture)$address
  to_object <- getNativeSymbolInfo("sexp_to_anyobjectptr", "opendp")$address
  object_free <- getNativeSymbolInfo("opendp_data__object_free", "opendp")$address
  error_free <- getNativeSymbolInfo("opendp_core___error_free", "opendp")$address
  function_free <- getNativeSymbolInfo("opendp_core___function_free", "opendp")$address
  runtime_type <- getFromNamespace("rt_parse", "opendp")("i32")
  for (transition in c(FALSE, TRUE)) {
    converter <- getNativeSymbolInfo(
      if (transition) "sexp_to_transitionfn" else "sexp_to_callbackfn", "opendp"
    )$address
    constructor <- getNativeSymbolInfo(
      if (transition) "opendp_core__new_queryable" else "opendp_core__new_function", "opendp"
    )$address
    evaluate <- getNativeSymbolInfo(
      if (transition) "opendp_core__queryable_eval" else "opendp_core__function_eval", "opendp"
    )$address
    called <- 0L
    callback <- function(arg, is_internal = FALSE) {
      called <<- called + 1L
      expect_identical(arg, 7L)
      if (transition) expect_false(is_internal)
      8L
    }
    invoke_direct <- function(fn) {
      .Call(
        direct, fn, transition, scope, converter, constructor,
        evaluate, function_free, to_object, object_free, error_free, runtime_type
      )
    }
    expect_true(invoke_direct(callback))
    expect_equal(called, 1L)
    expect_false(invoke_direct(\(arg) stop("callback failed", call. = FALSE)))
  }
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

test_that("extrinsic descriptor access requires honest-but-curious", {
  original_features <- getOption("opendp_features")
  on.exit(options(opendp_features = original_features), add = TRUE)
  enable_features("honest-but-curious")

  domain_descriptor <- new.env(parent = emptyenv())
  domain_descriptor$kind <- "test"
  domain <- user_domain(
    identifier = "AccessDomain",
    member = \(x) is.integer(x),
    descriptor = domain_descriptor
  )
  metric <- user_distance(
    identifier = "AccessDistance",
    descriptor = list(kind = "test")
  )
  measure <- user_divergence(
    identifier = "AccessDivergence",
    descriptor = list(kind = "test")
  )
  internal_measure <- getFromNamespace("_extrinsic_divergence", "opendp")(
    identifier = "InternalAccessDivergence",
    descriptor = list(kind = "test")
  )

  disable_features("honest-but-curious")
  for (obj in list(domain, metric, measure, internal_measure)) {
    expect_error(obj("descriptor"), "honest-but-curious")
  }
  expect_true(domain(member = 2L))

  enable_features("honest-but-curious")
  expect_identical(domain("descriptor"), domain_descriptor)
  for (obj in list(domain, metric, measure, internal_measure)) {
    expect_identical(obj("descriptor")$kind, "test")
  }
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
    "boom from member"
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
