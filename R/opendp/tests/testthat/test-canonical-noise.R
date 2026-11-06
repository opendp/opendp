library(opendp)
enable_features("contrib")

test_that("canonical noise compiles a guarantee to a profile", {
  specification <- privacy_guarantee(approx_dp = c(0.7, 1e-7))
  measurement <- make_canonical_noise(
    atom_domain(.T = "f64", nan = FALSE), absolute_distance(.T = "f64"),
    d_in = 1., d_out = specification
  )
  output <- measurement(d_in = 1.)
  expect_s3_class(output, "privacy_guarantee")
  expect_equal(output(epsilon = 0.7), 1e-7)
  expect_equal(output(delta = 1e-7), 0.7)
  expect_equal(measurement(d_in = 0.)(epsilon = 0.), 0.)
  expect_type(measurement(arg = 0.), "double")
  cast <- make_multiDP_to_approxDP(measurement, delta = 1e-7)
  expect_equal(cast(d_in = 1.), list(0.7, 1e-7))
})

test_that("canonical noise preserves canonical R error transport", {
  domain <- atom_domain(.T = "f64", nan = FALSE)
  metric <- absolute_distance(.T = "f64")
  specification <- privacy_guarantee(approx_dp = c(0.7, 1e-7))
  error <- tryCatch(
    make_canonical_noise(domain, metric, -1., specification),
    error = identity
  )
  expect_identical(class(error), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(error$variant, "MakeMeasurement")

  # Existing custom-profile callbacks require this feature, not canonical noise.
  enable_features("honest-but-curious")
  specification <- privacy_guarantee(profile = privacy_profile(
    log_delta = function(epsilon) stop("canonical callback failed", call. = FALSE)
  ))
  error <- tryCatch(
    make_canonical_noise(domain, metric, 1., specification),
    error = identity
  )
  expect_identical(class(error), c("opendp_error", "error", "condition")) # nolint: expect_s3_class_linter.
  expect_identical(error$variant, "FFI")
  expect_match(conditionMessage(error), "canonical callback failed")
})
