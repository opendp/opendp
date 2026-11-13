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

test_that("Rust error text survives releasing its FFI allocation", {
  for (i in 1:20) {
    expect_error(
      atom_domain(bounds = c(1., 0.)),
      "\\[MakeDomain\\] : lower bound .* may not be greater than upper bound"
    )
  }
  gc()
})

test_that("owned string and boolean results are copied before release", {
  domain <- atom_domain(.T = i32)
  ident <- make_identity(domain, discrete_distance())
  for (i in 1:20) {
    expect_match(toString(domain), "AtomDomain")
    expect_true(ident(d_in = 1L, d_out = 1L))
    expect_false(ident(d_in = 1L, d_out = 0L))
    expect_equal(ident(arg = 42L), 42L)
  }
  gc()
})

test_that("temporary AnyObjects are scoped across reentrant callbacks and errors", {
  domain <- atom_domain(.T = i32)
  ident <- make_identity(domain, discrete_distance())
  nested <- new_function(function_ = function(arg) ident(arg = arg), .TO = i32)
  for (i in 1:20) {
    expect_equal(nested(arg = 7L), 7L)
    expect_error(ident(d_in = 1L, d_out = -1L))
    expect_equal(ident(arg = 8L), 8L)
  }
  gc()
})

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
