opendp::enable_features("contrib", "honest-but-curious")

test_that("tuple gc", {
  input_space <- c(vector_domain(atom_domain(.T = f64, nan = FALSE)), symmetric_distance())

  gctorture(TRUE)
  # node reuse depends on heap layout
  for (i in 1:30) {
    clamper <- input_space |> then_clamp(c(3., 7.))
    gctorture(FALSE)
    expect_equal(clamper(arg = c(-100., 5., 100.)), c(3., 5., 7.))
    gctorture(TRUE)
  }
  gctorture(FALSE)
})
