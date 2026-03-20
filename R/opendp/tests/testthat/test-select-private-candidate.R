library(opendp)
enable_features("contrib", "private-selection-v2")

test_that("make_select_private_candidate supports new distribution API", {
  space <- c(vector_domain(atom_domain(.T = "f64")), symmetric_distance())

  m_count <- space |>
    then_count() |>
    then_laplace(scale = 2.0)

  m_sum <- space |>
    then_impute_constant(0.0) |>
    then_clamp(c(0.0, 100.0)) |>
    then_sum() |>
    then_laplace(scale = 200.0)

  m_scored_candidate <- make_composition(c(m_count, m_sum))

  m_threshold <- make_select_private_candidate(
    m_scored_candidate,
    mean = 100.0,
    threshold = 23.0
  )

  release <- m_threshold(arg = c(10.0, 12.0, 15.0))
  expect_true(is.list(release) || is.null(release))
  expect_equal(m_threshold(d_in = 1L), 2 * m_scored_candidate(d_in = 1L))

  m_best <- make_select_private_candidate(
    m_scored_candidate,
    mean = 2.0,
    distribution = "negative_binomial",
    eta = 0.5
  )

  release_best <- m_best(arg = c(10.0, 12.0, 15.0))
  expect_true(is.list(release_best) || is.null(release_best))
})
