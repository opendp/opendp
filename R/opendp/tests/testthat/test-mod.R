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
  expect_s3_class(error, "condition")
  expect_match(conditionMessage(error), "boom happened")
})

test_that("search does not mask a terminal error after initial recovery", {
  terminal <- simpleError("same callback failure")
  original <- simpleError("same callback failure")
  result <- tryCatch(binary_search(function(x) {
    if (x <= 0L) stop(original) # nolint: condition_call_linter.
    if (x < 16L) return(FALSE)
    stop(terminal) # nolint: condition_call_linter.
  }, .T = "int"), error = identity)
  expect_identical(result, terminal)
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

test_that("absent and narrow evaluable regions preserve the initial condition", {
  original <- simpleError("original failure")
  later <- simpleError("original failure")
  for (narrow in c(FALSE, TRUE)) {
    caught <- tryCatch(binary_search(function(x) {
      if (narrow && x == 7L) return(TRUE)
      if (x == 0L) stop(original) # nolint: condition_call_linter.
      stop(later) # nolint: condition_call_linter.
    }, .T = "int"), error = identity)
    expect_identical(caught, original)
  }
})

test_that("callback headers stay generic after anchoring", {
  original <- simpleError("[Overflow] : not an OpenDP-generated error")
  caught <- tryCatch(binary_search(function(x) {
    if (x == 0L) return(FALSE)
    stop(original) # nolint: condition_call_linter.
  }, .T = "int"), error = identity)
  expect_identical(caught, original)
})

test_that("tolerated Search conditions do not mask an absent Boolean boundary", {
  swallowed <- tryCatch(binary_search(\(x) FALSE, bounds = c(0L, 1L)), error = identity)
  expect_identical(swallowed$variant, "Search")
  caught <- tryCatch(binary_search(function(x) {
    if (x == .Machine$integer.max) return(FALSE)
    stop(swallowed) # nolint: condition_call_linter.
  }, .T = "int"), error = identity)
  expect_identical(caught$variant, "Search")
  expect_match(conditionMessage(caught), "unable to infer bounds")
  expect_false(identical(caught, swallowed))
})

test_that("unknown structured callback variants retain terminal condition identity", {
  original <- structure(
    list(message = "unknown variant", call = NULL, variant = "UnknownVariant"),
    class = c("opendp_error", "error", "condition")
  )
  caught <- tryCatch(binary_search(function(x) {
    if (x == 0L) return(FALSE)
    stop(original) # nolint: condition_call_linter.
  }, .T = "int"), error = identity)
  expect_identical(caught, original)
})
