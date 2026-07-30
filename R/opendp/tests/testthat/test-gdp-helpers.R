test_that("privacy guarantee Gaussian-DP representation", {
  guarantee <- privacy_guarantee(gaussianDP = 1.)

  expect_equal(guarantee(alpha = 0.), 1.)
  expect_equal(guarantee(alpha = 1.), 0.)
  expect_gte(guarantee(alpha = 0.5), 0.)
  expect_lte(guarantee(alpha = 0.5), 1.)
  expect_gte(guarantee(beta = 0.5), 0.)
  expect_lte(guarantee(beta = 0.5), 1.)
  expect_gte(guarantee(epsilon = 0.5), 0.)
  expect_lte(guarantee(epsilon = 0.5), 1.)
})
