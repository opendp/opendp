library(opendp)
enable_features("contrib", "idealized-numerics")

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

  space <- c(atom_domain(.T = "f64", nan = FALSE), absolute_distance(.T = "f64"))
  (space |> then_laplace(1.))(arg = 0.)

  space <- c(vector_domain(atom_domain(.T = "f64", nan = FALSE)), l1_distance(.T = "f64"))
  (space |> then_laplace(1.))(arg = c(0., 1.))

  space <- c(vector_domain(atom_domain(.T = "int", nan = FALSE)), l1_distance(.T = "int"))
  (space |> then_laplace(1.))(arg = c(0L, 1L))
})

test_that("make_laplace_int", {
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
})

test_that("make_laplace_float", {
  space <- c(atom_domain(.T = "f64", nan = FALSE), absolute_distance(.T = "f64"))
  meas <- space |> then_laplace(1., k = -40L)
  expect_type(meas(arg = 0), "double")
  expect_equal(meas(d_in = 1), 1.0)
})

test_that("make_discrete_laplace", {
  space <- c(atom_domain(.T = "i32"), absolute_distance(.T = "i32"))
  meas <- space |> then_laplace(1.)
  expect_type(meas(arg = 0L), "integer")
  expect_equal(meas(d_in = 1L), 1.0)
})

test_that("test_gaussian_curve", {
  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
  meas <- make_zCDP_to_approxDP(input_space |> then_gaussian(4.))
  curve <- meas(d_in = 1.)
  expect_equal(curve(delta = 0.), Inf)
  # see cdp_delta for formula of 0.688...
  expect_equal(curve(delta = 1e-3), 0.6880024554878085)
  expect_equal(curve(delta = 1.), 0.)

  # see cdp_delta for formula of 0.1508...
  expect_equal(curve(epsilon = 0.), 0.1508457845622862)
  expect_equal(curve(epsilon = 0.6880024554878085), 1e-3)

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(4.))(d_in = 0.0)
  expect_equal(curve(delta = 0.0), 0.0)
  expect_equal(curve(delta = -0.0), 0.0)
  expect_equal(curve(epsilon = -0.0), 0.0)

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(0.))(d_in = 1.0)
  expect_equal(curve(delta = 0.0), Inf)
  expect_equal(curve(delta = 0.1), Inf)
  expect_equal(curve(epsilon = 0.0), 1.0)
  expect_equal(curve(epsilon = 0.1), 1.0)

  curve <- make_zCDP_to_approxDP(input_space |> then_gaussian(0.))(d_in = 0.0)
  expect_equal(curve(delta = 0.0), 0.0)
  expect_equal(curve(delta = 0.1), 0.0)
  expect_equal(curve(epsilon = 0.0), 0.0)
  expect_equal(curve(epsilon = 0.1), 0.0)
})

test_that("test_gaussian_search", {
  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))

  make_approx_gauss <- function(scale, delta) {
    make_fix_delta(make_zCDP_to_approxDP(input_space |> then_gaussian(scale)), delta)
  }

  fixed_meas <- make_approx_gauss(1., 1e-5)
  fixed_meas(d_in = 1.)

  scale <- binary_search_param(
    function(s) make_approx_gauss(s, 1e-5),
    d_in = 1., d_out = c(1., 1e-5)
  )
  expect_equal(make_approx_gauss(scale, 1e-5)(d_in = 1.)[[1]], 1.)
})

test_that("test_laplace", {
  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
  meas <- input_space |> then_laplace(10.5)
  meas(arg = 100.)
  expect_lt(meas(d_in = 1.), 0.096)
})

test_that("test_vector_laplace", {
  input_space <- c(vector_domain(atom_domain(.T = f64, nan = FALSE)), l1_distance(.T = f64))
  meas <- input_space |> then_laplace(scale = 10.5)
  meas(arg = c(80., 90., 100.))
  expect_lt(meas(d_in = 1.), 1.3)
})

test_that("test_gaussian_smoothed_max_divergence", {
  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
  meas <- make_zCDP_to_approxDP(input_space |> then_gaussian(scale = 10.5))
  meas(arg = 100.)

  epsilon <- meas(d_in = 1.)(delta = 0.000001)
  expect_gt(epsilon, 0.4)
})

test_that("test_gaussian_zcdp", {
  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
  meas <- input_space |> then_gaussian(scale = 1.5, .MO = "ZeroConcentratedDivergence")
  meas(arg = 100.)

  expect_lt(meas(d_in = 1.), 0.223)
})

test_that("test_vector_gaussian", {
  delta <- 0.000001
  input_space <- c(vector_domain(atom_domain(.T = f64, nan = FALSE)), l2_distance(.T = f64))
  meas <- make_fix_delta(
    make_zCDP_to_approxDP(
      input_space |> then_gaussian(scale = 10.5)
    ), delta
  )
  meas(arg = c(80., 90., 100.))
  expect_lt(meas(d_in = 1.)[[1]], 0.6)
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
})

test_that("test_discrete_laplace", {
  meas <- make_laplace(atom_domain(.T = i32), absolute_distance(.T = i32), scale = 2.)
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)
})

test_that("test_vector_discrete_laplace_cks20", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l1_distance(.T = i32))
  meas <- input_space |> then_laplace(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)
})

test_that("test_geometric", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = i32))
  meas <- input_space |> then_geometric(scale = 2., bounds = c(1L, 10L))
  meas(arg = 100L)
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)
})

test_that("privacy profile and guarantee are distinct", {
  enable_features("honest-but-curious")
  profile <- privacy_profile(log_delta = \(epsilon) -epsilon)
  guarantee <- privacy_guarantee(profile = profile)

  expect_s3_class(profile, "privacy_profile")
  expect_s3_class(guarantee, "privacy_guarantee")
  expect_false(inherits(profile, "privacy_guarantee"))
  expect_equal(guarantee(delta = 1e-7), profile(delta = 1e-7))
})

test_that("privacy guarantees preserve exact approximate-DP certificates", {
  enable_features("honest-but-curious")
  for (delta in c(0., 5e-324, 0.1, 1.)) {
    guarantee <- privacy_guarantee(approx_dp = c(2., delta))
    expect_identical(guarantee(epsilon = 2.), delta)
    expect_identical(guarantee(epsilon = Inf), delta)
    expect_identical(guarantee(delta = delta), if (delta == 1.) 0. else 2.)
    expect_identical(guarantee(delta = 1.), 0.)
  }
  guarantee <- privacy_guarantee(approx_dp = c(2., 0.1))
  expect_identical(guarantee(delta = 0.1 - .Machine$double.eps * 0.1), Inf)
})

test_that("privacy guarantees preserve both representations across pointer round-trips", {
  enable_features("honest-but-curious")
  profile <- privacy_profile(log_delta = \(eps) ifelse(eps < 2., 0., -Inf))
  profile_copy <- opendp:::new_privacy_profile_internal(profile("ptr"))
  combined <- privacy_guarantee(approx_dp = c(1., 0.1), profile = profile_copy)
  reverse <- opendp:::`_privacy_guarantee_with_approx_dp`(privacy_guarantee(profile = profile), 1., 0.1)
  roundtrip <- opendp:::new_privacy_guarantee_internal(combined("ptr"))
  rm(profile, profile_copy, combined)
  invisible(gc())
  for (guarantee in list(reverse, roundtrip)) {
    expect_identical(guarantee(epsilon = 1.), 0.1)
    expect_identical(guarantee(epsilon = 2.), 0.)
    expect_identical(guarantee(delta = 0.1), 1.)
    expect_identical(guarantee(delta = 0.), 2.)
  }
})

test_that("privacy guarantee builders do not require honest-but-curious", {
  enable_features("honest-but-curious")
  profile <- privacy_profile(log_delta = \(epsilon) -epsilon)
  old_options <- options(opendp_features = list("contrib"))
  tryCatch({
    for (guarantee in list(
      privacy_guarantee(approx_dp = c(1., 0.1)),
      privacy_guarantee(profile = profile),
      privacy_guarantee(approx_dp = c(1., 0.1), profile = profile)
    )) {
      expect_gte(guarantee(epsilon = 1.), 0.)
      expect_gte(guarantee(delta = 0.1), 0.)
    }
  }, finally = options(old_options))
})

test_that("privacy guarantees propagate representation errors", {
  enable_features("honest-but-curious")
  original <- simpleError("profile failure")
  fail <- function(.values) stop(original) # nolint: condition_call_linter.
  profile <- privacy_profile(log_delta = fail)
  for (guarantee in list(
    privacy_guarantee(profile = profile),
    privacy_guarantee(approx_dp = c(1., 0.1), profile = profile)
  )) {
    expect_identical(tryCatch(guarantee(epsilon = 1.), error = identity), original)
    expect_identical(tryCatch(guarantee(delta = 0.1), error = identity), original)
  }
})

test_that("privacy guarantee construction validates representations", {
  enable_features("honest-but-curious")
  expect_error(privacy_guarantee(), "at least one")
  expect_error(privacy_guarantee(approx_dp = 1.), "numeric pair")
  expect_error(privacy_guarantee(profile = identity), "privacy_profile")
  expect_error(privacy_guarantee(approx_dp = c(-1., 0.)), "epsilon")
  expect_error(privacy_guarantee(approx_dp = c(Inf, 0.)), "finite")
  expect_error(privacy_guarantee(approx_dp = c(1., 1.1)), "delta")
  empty <- opendp:::`_new_privacy_guarantee`()
  expect_error(empty(epsilon = 0.), "no representation")
  expect_error(empty(delta = 1.), "no representation")
})

test_that("privacy guarantee f-DP reporting supports scalars and vectors", {
  profile <- privacy_profile(log_delta = function(epsilon) rep(-Inf, length(epsilon)))
  for (guarantee in list(
    privacy_guarantee(approx_dp = c(0., 0.)),
    privacy_guarantee(profile = profile),
    privacy_guarantee(approx_dp = c(0., 0.), profile = profile)
  )) {
    expect_equal(guarantee(alpha = 0.3), 0.7)
    expect_equal(guarantee(beta = 0.7), 0.3)
    expect_equal(guarantee(alpha = c(0., 0.25, 0.25, 1.)), c(1., 0.75, 0.75, 0.))
    expect_equal(guarantee(beta = c(0.25, 0.5, 0.5, 1.)), c(0.75, 0.5, 0.5, 0.))
    expect_equal(guarantee(alpha = numeric()), numeric())
    expect_equal(guarantee(beta = numeric()), numeric())
    expect_lt(guarantee(beta = 0.), 1.)
    expect_gt(guarantee(beta = 0.), 1. - .Machine$double.eps)
    expect_equal(guarantee(beta = 1.), 0.)
    expect_error(guarantee(alpha = c(0.5, 0.25)), "nondecreasing")
    expect_error(guarantee(beta = c(0.5, 0.25)), "nondecreasing")
    for (bad in list(-0.1, 1.1, NaN, Inf)) {
      expect_error(guarantee(alpha = bad))
      expect_error(guarantee(beta = bad))
    }
  }
})

test_that("f-DP reporting propagates source profile failures", {
  failure <- privacy_guarantee(
    approx_dp = c(0., 0.),
    profile = privacy_profile(log_delta = function(epsilon) stop("profile failed", call. = FALSE))
  )
  expect_error(failure(alpha = 0.5), "profile failed")
  expect_error(failure(beta = 0.5), "profile failed")
  for (output in list(numeric(), c(0., 0.), NaN, 0.1)) {
    invalid <- privacy_guarantee(
      approx_dp = c(0., 0.),
      profile = privacy_profile(log_delta = function(epsilon) output)
    )
    expect_error(invalid(alpha = 0.5))
    expect_error(invalid(beta = 0.5))
  }
})

test_that("privacy guarantee zCDP representation", {
  guarantee <- privacy_guarantee(zCDP = 0.5)
  expect_gt(guarantee(delta = 0.1), 0.)
  expect_lt(guarantee(epsilon = 1.), 1.)

  approximate <- privacy_guarantee(zCDP = 0., zCDP_delta = 0.2)
  expect_equal(approximate(epsilon = Inf), 0.2)
})

test_that("test_vector_discrete_laplace", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l1_distance(.T = i32))
  meas <- input_space |> then_laplace(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1L), 0.5)
  expect_gt(meas(d_in = 1L), 0.49999)
})

test_that("test_discrete_gaussian", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = i32))
  meas <- input_space |> then_gaussian(scale = 2.)
  meas(arg = 100L)
  expect_equal(meas(d_in = 1L), 0.125)
})

test_that("test_vector_discrete_gaussian", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = f64))
  meas <- input_space |> then_gaussian(scale = 2.)
  meas(arg = c(100L, 10L, 12L))
  expect_lte(meas(d_in = 1.), 0.125)
  expect_gt(meas(d_in = 1.), 0.124)
})

test_that("test_laplace_threshold", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  meas <- input_space |> then_count_by() |> then_laplace_threshold(scale = 2., threshold = 28L)

  meas(arg = c(rep("CAT_A", each = 20), rep("CAT_B", each = 10)))

  expect_lte(meas(d_in = 1L)[[1]], 1.0)
})

test_that("test_randomized_response", {
  meas <- make_randomized_response(categories = c("A", "B", "C", "D"), prob = 0.75)
  meas(arg = "A")
  expect_lte(meas(d_in = 1L), log(9.))
  expect_gt(meas(d_in = 1L), log(8.999))
})

test_that("test_randomized_response_bool", {
  meas <- make_randomized_response_bool(prob = 0.75)
  meas(arg = TRUE)
  expect_lte(meas(d_in = 1L), log(3.))
  expect_gt(meas(d_in = 1L), log(2.999))
})

test_that("test_gaussian", {
  input_space <- c(atom_domain(.T = i32), absolute_distance(.T = f64))
  expect_equal(class((input_space |> then_gaussian(1.))(arg = 1L)), "integer") # nolint: expect_s3_class_linter.
  # Both expect_s3_class and expect_s4_class failed.
  # Not sure which we are using, and which we expect to pass.

  input_space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = 1.)

  input_space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = c(1L, 2L, 3L))

  input_space <- c(vector_domain(atom_domain(.T = f64, nan = FALSE)), l2_distance(.T = f64))
  (input_space |> then_gaussian(1.))(arg = c(1., 2., 3.))
})


test_that("test_randomized_response_bitvec", {
  xx <- raw(3)
  xx[1] <- charToRaw("A")
  xx[2] <- as.raw(1L)
  input_space <- c(bitvector_domain(max_weight = 4L, size = 24L), discrete_distance())
  m_rr <- input_space |> then_randomized_response_bitvec(f = 0.95)
  # epsilon is 2 * m * ln((2 - f) / f)
  # where m = 4 and f = .95
  expect_equal(m_rr(d_in = 1L), 0.8006676684558611)
  m_rr(arg = xx)
})

test_that("privacy guarantee RDP representation", {
  batches <- list()
  guarantee <- privacy_guarantee(renyiDP = function(orders) {
    batches[[length(batches) + 1L]] <<- orders
    0.5 * orders
  })
  expect_gt(guarantee(delta = 0.1), 0.)
  expect_lt(guarantee(epsilon = 1.), 1.)

  approximate <- privacy_guarantee(
    renyiDP = \(orders) rep(0., length(orders)),
    renyiDP_delta = 0.2
  )
  expect_equal(approximate(epsilon = Inf), 0.2)
  expect_gt(length(batches), 0L)
  expect_true(all(lengths(batches) == 1L))

  for (output in list(numeric(), c(0., 0.), NaN, -1.)) {
    invalid <- privacy_guarantee(renyiDP = \(orders) output)
    caught <- tryCatch(invalid(epsilon = 1.), error = identity)
    expect_s3_class(caught, "opendp_error")
    expect_identical(caught$variant, if (anyNA(output)) "NumericError" else "FailedMap")
  }
  failing <- privacy_guarantee(
    approx_dp = c(1., 0.1),
    renyiDP = function(orders) stop("RDP failed", call. = FALSE)
  )
  caught <- tryCatch(failing(epsilon = 1.), error = identity)
  expect_s3_class(caught, "opendp_error")
  expect_identical(caught$variant, "FFI")
  expect_identical(caught$raw_message, "RDP failed")
})
