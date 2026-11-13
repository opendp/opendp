opendp::enable_features("contrib", "honest-but-curious")

test_that("temporary tuple and measurement arrays survive repeated FFI calls", {
  input_space <- c(vector_domain(atom_domain(.T = f64, nan = FALSE)), symmetric_distance())
  meas <- input_space |> then_count() |> then_laplace(scale = 1.)
  for (i in 1:20) {
    clamper <- input_space |> then_clamp(c(0., 1.))
    expect_equal(clamper(arg = c(-1., 2.)), c(0., 1.))
    expect_error(input_space |> then_clamp(c(1., 0.)))
    composed <- make_composition(list(meas, meas))
    expect_length(composed(arg = c(0., 1.)), 2)
  }
  gc()
})
