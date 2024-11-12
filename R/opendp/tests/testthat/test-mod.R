test_that("binary search", {
  enable_features("contrib")
  s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
  t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()

  m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1., .T = "float")
})

test_that("modular metric constructors", {
  enable_features("contrib")

  m_abs <- absolute_distance(.T = "i32", modular = TRUE)
  m_l1 <- l1_distance(.T = "i32", modular = TRUE)
  m_l2 <- l2_distance(.T = "f64", modular = TRUE)

  expect_match(toString(m_abs), "modulo")
  expect_match(toString(m_l1), "modular")
  expect_match(toString(m_l2), "modular")
})

test_that("modular metrics can build noise mechanisms", {
  enable_features("contrib")

  meas_abs <- make_laplace(
    atom_domain(.T = "i32"),
    absolute_distance(.T = "i32", modular = TRUE),
    1.
  )
  expect_type(meas_abs(arg = 0L), "integer")
  expect_equal(meas_abs(d_in = 1L), 1.0)

  meas_vec <- make_gaussian(
    vector_domain(atom_domain(.T = "i32")),
    l2_distance(.T = "f64", modular = TRUE),
    2.
  )
  expect_length(meas_vec(arg = c(0L, 1L)), 2)
  expect_equal(meas_vec(d_in = 1.), 0.125)
})
