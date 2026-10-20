test_that("binary search", {
  enable_features("contrib")
  s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
  t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()

  m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1., .T = "float")
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
