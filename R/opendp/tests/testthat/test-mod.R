test_that("binary search", {
  enable_features("contrib")
  s_vec <- c(vector_domain(atom_domain(.T = "float", nan = FALSE)), symmetric_distance())
  t_sum <- s_vec |> then_clamp(c(0., 1.)) |> then_sum()

  m_sum <- binary_search_chain(\(s) t_sum |> then_laplace(s), d_in = 1L, d_out = 1., .T = "float")
})
