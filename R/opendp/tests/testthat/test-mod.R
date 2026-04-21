test_that("binary search", {
  enable_features("contrib")
  s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
  t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()

  m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1., .T = "float")
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
