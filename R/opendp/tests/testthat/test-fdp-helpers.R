test_that("privacy curve profile", {
  # Profile: epsilon → delta relationship, with inverse delta → epsilon
  p_profile <- privacy_curve(profile = \(eps) if (eps < 0.5) 1. else 1e-8)
  # Direct: epsilon < 0.5 gives delta = 1.0
  expect_equal(p_profile(epsilon = 0.499), 1.)
  # Inverse: finding epsilon where profile(epsilon) = 1e-8 gives epsilon ≥ 0.5
  expect_equal(p_profile(delta = 1e-8), 0.5)
  # Relationship: if epsilon increases from 0.499 to 0.5, delta decreases from 1.0 to 1e-8
  expect_gt(p_profile(epsilon = 0.499), p_profile(epsilon = 0.501))
})

test_that("privacy curve tradeoff", {
  # Tradeoff: alpha (accuracy) → beta (failure probability), inversely related
  p_tradeoff <- privacy_curve(tradeoff = \(alpha) 1. - alpha)
  # At alpha = 0.3, beta = 1 - 0.3 = 0.7
  expect_equal(p_tradeoff(alpha = 0.3), 0.7)
  # At alpha = 0.7, beta = 1 - 0.7 = 0.3
  expect_equal(p_tradeoff(alpha = 0.7), 0.3)
  # Relationship: as alpha increases, beta decreases
  expect_lt(p_tradeoff(alpha = 0.7), p_tradeoff(alpha = 0.3))
})

test_that("privacy curve approxdp", {
  # ApproxDP: discrete (epsilon, delta) pairs
  pairs <- list(c(0., 1.), c(1., 0.1), c(2., 0.01))
  p_pairs <- privacy_curve(approxdp = pairs)
  # Direct lookup: epsilon = 1.0 gives delta = 0.1
  expect_equal(p_pairs(epsilon = 1.0), 0.1)
  # Direct lookup: epsilon = 2.0 gives delta = 0.01
  expect_equal(p_pairs(epsilon = 2.0), 0.01)
  # Inverse lookup: delta = 0.01 should give epsilon = 2.0
  expect_equal(p_pairs(delta = 0.01), 2.0)
})

test_that("privacy curve gdp", {
  # GDP (Gaussian DP): relationship between epsilon, delta, and mu
  p_mu <- privacy_curve(mu = 1.0)
  # For mu=1.0, at epsilon = 0, delta should be 0 (no privacy loss means no failure)
  expect_equal(p_mu(epsilon = 0.), 0.382925038)
  # As epsilon increases, delta decreases
  expect_gt(p_mu(epsilon = 1.0), p_mu(epsilon = 2.0))
  # Inverse relationship: as delta increases, required epsilon decreases
  expect_gt(p_mu(delta = 0.01), p_mu(delta = 0.5))
})

test_that("constructor argument validation is enforced", {
  expect_error(
    privacy_curve(),
    "expected exactly one of profile, tradeoff, approxdp or mu"
  )
  expect_error(
    privacy_curve(profile = \(eps) eps, approxdp = list(c(0., 1.))),
    "expected exactly one of profile, tradeoff, approxdp or mu"
  )
})
