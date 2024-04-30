library(opendp)
enable_features("contrib", "floating-point")

test_that("make_randomized_response_bool", {
  meas <- make_randomized_response_bool(0.75)

  expect_type(meas(arg = TRUE), "logical")
  expect_equal(meas(d_in = 1L), 1.0986122886681098)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("then_laplace", {
  space <- c(atom_domain(.T = "i32"), absolute_distance(.T = "i32"))
  meas <- space |> then_laplace(1.)
  expect_type(meas(arg = 0L), "integer")
  expect_equal(meas(d_in = 1L), 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
  expect_error(meas(0L), "numeric attr not allowed; Did you mean 'arg='?")

  space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
  (space |> then_laplace(1.))(arg = 0.)

  space <- c(vector_domain(atom_domain(.T = "f64")), l1_distance(.T = "f64"))
  (space |> then_laplace(1.))(arg = c(0., 1.))

  space <- c(vector_domain(atom_domain(.T = "int")), l1_distance(.T = "int"))
  (space |> then_laplace(1.))(arg = c(0L, 1L))
})

test_that("make_laplace", {
  meas <- make_laplace(atom_domain(.T = "i32"), absolute_distance(.T = "i32"), 1.)
  expect_type(meas(arg = 0L), "integer")
  expect_equal(meas(d_in = 1L), 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
  expect_error(meas(0L), "numeric attr not allowed; Did you mean 'arg='?")
})

test_that("make_geometric", {
  space <- c(atom_domain(.T = "i32"), absolute_distance(.T = "i32"))
  meas <- space |> then_geometric(1., bounds = c(0L, 2L))
  expect_type(meas(arg = 0L), "integer")
  expect_equal(meas(d_in = 1L), 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("make_laplace", {
  space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
  meas <- space |> then_laplace(1., k = -40L)
  expect_type(meas(arg = 0), "double")
  expect_equal(meas(d_in = 1), 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("make_discrete_laplace", {
  space <- c(atom_domain(.T = "i32"), absolute_distance(.T = "i32"))
  meas <- space |> then_laplace(1.)
  expect_type(meas(arg = 0L), "integer")
  expect_equal(meas(d_in = 1L), 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_gaussian_curve", {
  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
  meas <- make_zCDP_to_approxDP(input_space |> then_gaussian(4.))
  curve <- meas(d_in = 1.)
  expect_equal(curve(delta = 0.), Inf)
  expect_equal(curve(delta = 1e-3), 0.6880024554878086)
  expect_equal(curve(delta = 1.), 0.)

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(4.))(d_in = 0.0)
  expect_equal(curve(delta = 0.0), 0.0)
  expect_error(curve(delta = -0.0))

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(0.))(d_in = 1.0)
  expect_equal(curve(delta = 0.0), Inf)
  expect_equal(curve(delta = 0.1), Inf)

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(0.))(d_in = 0.0)
  expect_equal(curve(delta = 0.0), 0.0)
  expect_equal(curve(delta = 0.1), 0.0)
})

test_that("test_gaussian_search", {
  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))

  make_smd_gauss <- function(scale, delta) {
    make_fix_delta(make_zCDP_to_approxDP(input_space |> then_gaussian(scale)), delta)
  }

  fixed_meas <- make_smd_gauss(1., 1e-5)
  fixed_meas(d_in = 1.)

  scale <- binary_search_param(
    function(s) make_smd_gauss(s, 1e-5),
    d_in = 1., d_out = c(1., 1e-5)
  )
  expect_equal(make_smd_gauss(scale, 1e-5)(d_in = 1.)[[1]], 1.)
})

test_that("test_laplace", {
  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
  meas <- input_space |> then_laplace(10.5)
  meas(arg = 100.)
  expect_lt(meas(d_in = 1.), 0.096)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_vector_laplace", {
  input_space <- c(vector_domain(atom_domain(.T = f64)), l1_distance(.T = f64))
  meas <- input_space |> then_laplace(scale = 10.5)
  meas(arg = c(80., 90., 100.))
  expect_lt(meas(d_in = 1.), 1.3)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_gaussian_smoothed_max_divergence", {
  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
  meas <- make_zCDP_to_approxDP(input_space |> then_gaussian(scale = 10.5))
  meas(arg = 100.)

  epsilon <- meas(d_in = 1.)(delta = 0.000001)
  expect_gt(epsilon, 0.4)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_gaussian_zcdp", {
  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
  meas <- input_space |> then_gaussian(scale = 1.5, .MO = "ZeroConcentratedDivergence<f64>")
  meas(arg = 100.)

  expect_lt(meas(d_in = 1.), 0.223)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_vector_gaussian", {
  delta <- 0.000001
  input_space <- c(vector_domain(atom_domain(.T = f64)), l2_distance(.T = f64))
  meas <- make_fix_delta(
    make_zCDP_to_approxDP(
      input_space |> then_gaussian(scale = 10.5)
    ), delta
  )
  meas(arg = c(80., 90., 100.))
  expect_lt(meas(d_in = 1.)[[1]], 0.6)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_geometric", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = i32))
  meas <- input_space |> then_geometric(scale = 2., bounds = c(1L, 10L))
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  meas <- input_space |> then_geometric(scale = 2.)
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_discrete_laplace", {
  meas <- make_laplace(atom_domain(.T = i32), absolute_distance(.T = i32), scale = 2.)
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_vector_discrete_laplace_cks20", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l1_distance(.T = i32))
  meas <- input_space |> then_laplace(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_geometric", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = i32))
  meas <- input_space |> then_geometric(scale = 2., bounds = c(1L, 10L))
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_vector_discrete_laplace", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l1_distance(.T = i32))
  meas <- input_space |> then_laplace(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_discrete_gaussian", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = i32))
  meas <- input_space |> then_gaussian(scale = 2.)
  meas(arg = 100L)
  expect_equal(meas(d_in = 1L), 0.125)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_vector_discrete_gaussian", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = f64))
  meas <- input_space |> then_gaussian(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1.), 0.125)
  expect_gt(meas(d_in = 1.), 0.124)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_laplace_threshold", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  meas <- input_space |>
    then_count_by(.MO = "L1Distance<f64>", .TV = f64) |>
    then_base_laplace_threshold(scale = 2., threshold = 28.)

  meas(arg = c(rep("CAT_A", each = 20), rep("CAT_B", each = 10)))

  expect_lte(meas(d_in = 1L)[[1]], 1.0)

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_randomized_response", {
  meas <- make_randomized_response(categories = c("A", "B", "C", "D"), prob = 0.75)
  meas(arg = "A")
  expect_lte(meas(d_in = 1L), log(9.))
  expect_gt(meas(d_in = 1L), log(8.999))

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_randomized_response_bool", {
  meas <- make_randomized_response_bool(prob = 0.75)
  meas(arg = TRUE)
  expect_lte(meas(d_in = 1L), log(3.))
  expect_gt(meas(d_in = 1L), log(2.999))

  expect_error(meas(), "expected exactly one of attr, arg or d_in")
})

test_that("test_gaussian", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = f64))
  expect_equal(class((input_space |> then_gaussian(1.))(arg = 1L)), "integer") # nolint: expect_s3_class_linter.
  # Both expect_s3_class and expect_s4_class failed.
  # Not sure which we are using, and which we expect to pass.

  input_space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = 1.)

  input_space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = c(1L, 2L, 3L))

  input_space <- c(vector_domain(atom_domain(.T = f64)), l2_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = c(1., 2., 3.))
})
