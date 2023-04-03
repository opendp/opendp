library(opendp)
enable_features("contrib")

test_that("make_composition", {
  domain <- atom_domain(.T = "i32")
  metric <- absolute_distance(.T = "i32")

  meas_lap <- make_laplace(domain, metric, scale = 0.)
  meas <- make_composition(c(meas_lap, meas_lap))
  meas <- make_composition(c(meas_lap, meas))

  # recursive packing/unpacking of compositors/releases
  expect_equal(meas(arg = 1L), list(1L, list(1L, 1L)))
})

test_that("make_composition", {
  domain <- atom_domain(.T = "i32")
  metric <- absolute_distance(.T = "i32")

  meas_lap <- make_laplace(domain, metric, scale = 0.)
  meas <- make_composition(c(meas_lap, meas_lap))
  meas <- make_composition(c(meas_lap, meas))

  # recursive packing/unpacking of compositors/releases
  expect_equal(meas(arg = 1L), list(1L, list(1L, 1L)))
})

test_that("make_chain_mt", {
  domain <- vector_domain(atom_domain(bounds = list(0L, 2L)))
  metric <- symmetric_distance()

  trans_sum <- make_sum(domain, metric)
  meas_lap <- make_laplace(trans_sum("output_domain"), trans_sum("output_metric"), scale = 1.)

  meas <- make_chain_mt(meas_lap, trans_sum)

  expect_type(meas(arg = c(1L, 3L)), "integer")
  expect_equal(meas(d_in = 1L), 2)

  meas <- trans_sum |> then_laplace(scale = 1.)

  expect_type(meas(arg = c(1L, 3L)), "integer")
  expect_equal(meas(d_in = 1L), 2)

  meas <- make_sum(domain, metric) |> then_laplace(scale = 1.)

  expect_type(meas(arg = c(1L, 3L)), "integer")
  expect_equal(meas(d_in = 1L), 2)
})

test_that("make_chain_tt", {
  domain <- vector_domain(atom_domain(.T = i32))
  metric <- symmetric_distance()

  trans_clamp <- make_clamp(domain, metric, bounds = c(0L, 2L))
  trans_sum <- make_sum(trans_clamp("output_domain"), trans_clamp("output_metric"))

  trans <- make_chain_tt(trans_sum, trans_clamp)

  expect_equal(trans(arg = c(1L, 3L)), 3L)
  expect_equal(trans(d_in = 1L), 2L)

  trans <- trans_clamp |> then_sum()

  expect_equal(trans(arg = c(1L, 3L)), 3L)
  expect_equal(trans(d_in = 1L), 2L)

  meas <- c(domain, metric) |> then_clamp(c(0L, 2L)) |> then_sum() |> then_laplace(scale = 1.)

  expect_type(meas(arg = c(1L, 3L)), "integer")
  expect_equal(meas(d_in = 1L), 2)
})


test_that("make_fix_delta", {
  domain <- atom_domain(bounds = c(0L, 2L))
  metric <- absolute_distance(i32)

  meas_zCDP <- make_gaussian(domain, metric, scale = 1.)
  meas_εδ <- make_zCDP_to_approxDP(meas_zCDP)

  curve <- meas_εδ(d_in = 1L)

  expect_equal(curve(delta = 1e-7), 5.6708588355)

  meas_fεδ <- make_fix_delta(meas_εδ, 1e-7)
  expect_equal(meas_fεδ(d_in = 1L), list(5.6708588355, 1e-7))
})



test_that("make_adaptive_composition", {
  meas_rr <- make_randomized_response_bool(prob = 0.75)

  meas_sc <- make_adaptive_composition(
    input_domain = meas_rr("input_domain"),
    input_metric = meas_rr("input_metric"),
    output_measure = meas_rr("output_measure"),
    d_mids = c(2, 2),
    d_in = 1L
  )

  sc_qbl <- meas_sc(arg = TRUE)
  expect_type(sc_qbl(query = meas_rr), "logical")
  expect_type(sc_qbl(query = meas_rr), "logical")
  expect_error(sc_qbl(query = meas_rr))
})


test_that("make_sequential_composition", {
  meas_rr <- make_randomized_response_bool(prob = 0.75)

  expect_warning({
    meas_sc <- make_sequential_composition(
      input_domain = meas_rr("input_domain"),
      input_metric = meas_rr("input_metric"),
      output_measure = meas_rr("output_measure"),
      d_mids = c(2, 2),
      d_in = 1L
    )
  })

  sc_qbl <- meas_sc(arg = TRUE)
  expect_type(sc_qbl(query = meas_rr), "logical")
  expect_type(sc_qbl(query = meas_rr), "logical")
  expect_error(sc_qbl(query = meas_rr))
})

test_that("test_sequential_odometer", {
  max_influence <- 1L
  space <- c(vector_domain(atom_domain(.T = "i32")), symmetric_distance())
  o_sc <- space |> then_fully_adaptive_composition(max_divergence())

  expect_equal(toString(o_sc), paste0(
    "Odometer(\n",
    "  input_domain=VectorDomain(AtomDomain(T=i32)),\n",
    "  input_metric=SymmetricDistance(),\n",
    "  output_measure=MaxDivergence\n",
    ")"
  ))

  oqbl_sc <- o_sc(arg = rep(1L, 200))
  expect_equal(oqbl_sc(d_in = max_influence), 0.0)

  m_sum <- space |> then_clamp(c(0L, 10L)) |> then_sum() |> then_laplace(100.)

  # evaluating
  expect_type(oqbl_sc(query = m_sum), "integer")
  expect_equal(oqbl_sc(d_in = max_influence), m_sum(d_in = max_influence))

  m_lap <- make_laplace(atom_domain(.T = "i32"), absolute_distance(.T = "i32"), 200.)
  t_sum <- space |> then_clamp(c(0L, 10L)) |> then_sum()
  expect_warning({
    m_sum_compositor <- t_sum |> then_sequential_composition(
      output_measure = max_divergence(),
      d_in = t_sum(d_in = max_influence),
      d_mids = c(0.2, 0.09)
    )
  })
  qbl_summed <- oqbl_sc(query = m_sum_compositor)
  # it's slightly larger, checking greater than will do
  expect_gt(oqbl_sc(d_in = max_influence), m_sum(d_in = max_influence) + 0.2 + 0.09)

  expect_type(qbl_summed(query = m_lap), "integer") # child release
  expect_type(qbl_summed(query = m_lap), "integer") # child release
  expect_type(oqbl_sc(query = m_sum), "integer") # root release

  # it's slightly larger, checking greater than will do
  expect_gt(oqbl_sc(d_in = max_influence), m_sum(d_in = max_influence) * 2 + 0.2 + 0.09)
})


test_that("test_odometer_supporting_elements", {
  sc_odo <- make_fully_adaptive_composition(
    vector_domain(atom_domain(.T = "i32")),
    symmetric_distance(),
    max_divergence()
  )

  expect_s3_class(sc_odo(arg = 1L), "odometer_queryable")
  expect_equal(toString(sc_odo("input_domain")), toString(vector_domain(atom_domain(.T = "i32"))))
  expect_equal(toString(sc_odo("input_metric")), toString(symmetric_distance()))
  expect_equal(toString(sc_odo("output_measure")), toString(max_divergence()))
})
