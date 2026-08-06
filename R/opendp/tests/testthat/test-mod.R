test_that("binary search", {
  enable_features("contrib")
  s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
  t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()

  m_sum <- binary_search_chain(
    \(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1., .T = "float"
  )
  expect_lte(m_sum(d_in = 1L), 1.)
})

test_that("binary search honors explicit integer type without bounds", {
  expect_equal(binary_search(\(x) x > 5L, .T = "int"), 6L)
  expect_equal(binary_search(\(x) x < 5L, .T = "int"), 4L)
})

test_that("binary search supports one-sided bounds", {
  expect_equal(binary_search(\(x) x <= -5L, list(-10L, NULL)), -5L)
  expect_equal(binary_search(\(x) x <= -5L, list(NULL, -1L)), -5L)
  expect_error(
    binary_search(\(x) x <= -5L, list(0L, NULL)),
    "decision boundary is below the lower bound"
  )
  expect_error(
    binary_search(\(x) x <= -5L, list(NULL, -10L)),
    "decision boundary is above the upper bound"
  )
  expect_error(
    binary_search(\(x) x <= -5L, c(-10L, NULL)),
    paste0("use list", "\\(lower, NULL\\)")
  )
})

test_that("Rust type failures retain their canonical error variant", {
  for (search in list(binary_search, getFromNamespace("exponential_bounds_search", "opendp"))) {
    caught <- tryCatch(search(\(.x) TRUE, .T = "bool"), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "Type")
  }
})

test_that("exponential search propagates late callback errors", {
  error <- tryCatch(
    binary_search(
      function(x) {
        if (identical(x, 16L)) stop("boom happened", call. = FALSE)
        FALSE
      },
      .T = "int"
    ),
    error = identity
  )
  expect_identical(class(error), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(error$variant, "FFI")
  expect_identical(error$raw_message, "boom happened")
})

test_that("search does not mask a terminal error after initial recovery", {
  result <- tryCatch(binary_search(function(x) {
    if (x <= 0L) stop("initial callback failure", call. = FALSE)
    if (x < 16L) return(FALSE)
    stop("terminal callback failure", call. = FALSE)
  }, .T = "int"), error = identity)
  expect_identical(class(result), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(result$variant, "FFI")
  expect_identical(result$raw_message, "terminal callback failure")
})

test_that("nonfinite explicit bounds are rejected before evaluation", {
  for (bound in c(Inf, -Inf, NaN)) {
    expect_error(binary_search(function(x) {
      stop("must not evaluate", call. = FALSE)
    }, bounds = list(0., bound)), "search bounds must be finite and not NaN")
  }
})

test_that("floating-point aliases to idealized-numerics", {
  disable_features("floating-point", "idealized-numerics")

  expect_warning(
    enable_features("floating-point"),
    "\"floating-point\" is deprecated. Use \"idealized-numerics\" instead."
  )
  expect_warning(
    assert_features("floating-point"),
    "\"floating-point\" is deprecated. Use \"idealized-numerics\" instead."
  )
  expect_true("idealized-numerics" %in% getOption("opendp_features"))
  expect_false("floating-point" %in% getOption("opendp_features"))

  disable_features("idealized-numerics")
})

test_that("initial recovery tries the other permitted direction", {
  for (type in c("int", "float")) {
    zero <- if (type == "int") 0L else 0.
    predicate <- function(x) {
      if (x == 0) stop("zero is unavailable", call. = FALSE)
      x <= -5
    }
    expect_equal(binary_search(predicate, .T = type, return_sign = TRUE), c(-5, -1))
    expect_equal(binary_search(predicate, bounds = list(NULL, zero)), -5)
    bounds <- getFromNamespace("exponential_bounds_search", "opendp")(predicate, .T = type)
    expect_false(identical(predicate(bounds[[1]]), predicate(bounds[[2]])))
    seen <- numeric()
    expect_error(binary_search(function(x) {
      seen <<- c(seen, x)
      predicate(x)
    }, bounds = list(zero, NULL)), "below the lower bound")
    expect_true(all(seen >= 0))
  }
})

test_that("independent Rust FFI errors are not replaced with callback conditions", {
  enable_features("contrib", "honest-but-curious")
  callback_error <- tryCatch(binary_search(function(x) {
    if (x == 0L) return(FALSE)
    stop("callback failure", call. = FALSE)
  }, .T = "int"), error = identity)
  expect_identical(callback_error$variant, "FFI")
  expect_identical(callback_error$raw_message, "callback failure")
  # A real Rust null-pointer failure, independent of the callback.
  caught <- tryCatch(
    getFromNamespace("object_type", "opendp")(methods::new("externalptr")),
    error = identity
  )
  expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(caught$variant, "FFI")
  expect_match(caught$raw_message, "null pointer")
  expect_false(grepl("callback failure", caught$raw_message, fixed = TRUE))
})

test_that("initial invalid domains recover without caller bounds", {
  for (sign in c(-1L, 1L)) {
    for (polarity in c(FALSE, TRUE)) {
      predicate <- function(x) {
        x <- as.double(x) * sign
        if (x < 3 || x > 16) stop("initial region unavailable", call. = FALSE)
        (x >= 5) == polarity
      }
      expected <- sign * if (polarity) 5L else 4L
      expect_equal(binary_search(predicate, .T = "int"), expected)
      bounds <- if (sign > 0L) list(0L, NULL) else list(NULL, 0L)
      expect_equal(binary_search(predicate, bounds = bounds), expected)
    }
  }
})

test_that("absent and narrow evaluable regions preserve the initial diagnostic", {
  for (narrow in c(FALSE, TRUE)) {
    caught <- tryCatch(binary_search(function(x) {
      if (narrow && x == 7L) return(TRUE)
      if (x == 0L) stop("initial failure", call. = FALSE)
      stop("later failure", call. = FALSE)
    }, .T = "int"), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "FFI")
    expect_identical(caught$raw_message, "initial failure")
  }
})

test_that("callback headers stay generic after anchoring", {
  original <- simpleError("[Overflow] : not an OpenDP-generated error")
  caught <- tryCatch(binary_search(function(x) {
    if (x == 0L) return(FALSE)
    stop(original) # nolint: condition_call_linter.
  }, .T = "int"), error = identity)
  expect_identical(caught$variant, "FFI")
  expect_identical(caught$raw_message, conditionMessage(original))
})

test_that("tolerated callback errors do not mask an absent Boolean boundary", {
  swallowed <- tryCatch(binary_search(\(x) FALSE, bounds = c(0L, 1L)), error = identity)
  expect_identical(swallowed$variant, "Search")
  for (failure in list(swallowed, simpleError("tolerated callback failure"))) {
    caught <- tryCatch(binary_search(function(x) {
      if (x == .Machine$integer.max) return(FALSE)
      stop(failure) # nolint: condition_call_linter.
    }, .T = "int"), error = identity)
    expect_identical(class(caught), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
    expect_identical(caught$variant, "Search")
    expect_match(caught$raw_message, "unable to infer bounds")
  }
})
