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

test_that("nonfinite bounds reach Rust validation", {
  expect_error(binary_search(function(x) {
    stop("must not evaluate", call. = FALSE)
  }, bounds = list(0., NaN)), "search bounds must be finite and not NaN")
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

test_that("type inference preserves integer fallback after ordinary R errors", {
  predicate <- function(x) {
    stopifnot(is.integer(x))
    x >= 5L
  }
  expect_identical(binary_search(predicate), 5L)
})

test_that("type inference permits a non-type error at zero", {
  predicate <- function(x) {
    if (x <= 0) stop("nonpositive parameter", call. = FALSE)
    x >= 5
  }
  expect_identical(binary_search(predicate), 5.)
})

test_that("type inference retries with integers on a structured type error", {
  predicate <- function(x) {
    if (!is.integer(x)) {
      stop(errorCondition("expected integer", variant = "Type", class = "opendp_error")) # nolint: condition_call_linter.
    }
    if (x == 0L) stop("zero is unavailable", call. = FALSE)
    x >= 5L
  }
  expect_identical(binary_search(predicate), 5L)
})


test_that("search recovers from Rust-generated overflow", {
  enable_features("contrib")
  sum_trans <- make_sum(
    vector_domain(atom_domain(.T = i32, bounds = c(0L, 100L))),
    symmetric_distance()
  )
  target <- sum_trans(d_in = 5L)
  predicate <- function(x) sum_trans(d_in = x) <= target
  expect_identical(binary_search(predicate, bounds = c(0L, 100000000L)), 5L)
})
