library(opendp)
enable_features("contrib", "floating-point")

get_context_internal <- function(name) {
  if (exists(name, mode = "function", inherits = TRUE)) {
    return(get(name, mode = "function", inherits = TRUE))
  }
  get(name, envir = asNamespace("opendp"), inherits = FALSE)
}

expect_unlisted_equal <- function(object, expected, tolerance = testthat_tolerance()) {
  expect_equal(unlist(object), expected, tolerance = tolerance)
}

test_that("loss_of constructs privacy losses and warns on unusual values", {
  pure <- loss_of(epsilon = 3)
  approx <- loss_of(rho = 0.5, delta = 1e-7)

  expect_equal(toString(pure[[1]]), "MaxDivergence")
  expect_equal(pure[[2]], 3)
  expect_equal(toString(approx[[1]]), "Approximate(ZeroConcentratedDivergence)")
  expect_equal(approx[[2]], c(0.5, 1e-7))

  expect_warning(
    loss_of(epsilon = 100),
    "epsilon should be less than or equal to 5"
  )
  expect_message(
    expect_warning(
      loss_of(epsilon = 2, delta = 1e-5),
      "delta should be less than or equal to 1e-06"
    ),
    "epsilon is typically less than or equal to 1"
  )
})

test_that("unit_of constructs supported privacy units and validates arguments", {
  expect_equal(toString(unit_of(contributions = 3L)[[1]]), "SymmetricDistance()")
  expect_equal(unit_of(contributions = 3L)[[2]], 3L)

  expect_equal(toString(unit_of(changes = 3L, ordered = TRUE)[[1]]), "HammingDistance()")
  expect_equal(unit_of(changes = 3L, ordered = TRUE)[[2]], 3L)

  expect_equal(toString(unit_of(local = TRUE)[[1]]), "DiscreteDistance()")
  expect_equal(unit_of(local = TRUE)[[2]], 1L)

  expect_error(
    unit_of(local = TRUE, contributions = 1L),
    "Must specify exactly one distance"
  )
  expect_error(
    unit_of(l1 = 1, ordered = TRUE),
    '"ordered" is only valid with "changes" or "contributions"'
  )
})

test_that("domain_of metric_of and space_of build helper objects", {
  expect_equal(
    toString(domain_of(c(1.0, 2.0), infer = TRUE)),
    "VectorDomain(AtomDomain(T=f64))"
  )
  expect_equal(toString(metric_of("AbsoluteDistance<i32>")), "AbsoluteDistance(i32)")

  inferred_scalar_space <- space_of(1.0, infer = TRUE)
  expect_equal(toString(inferred_scalar_space[[1]]), "AtomDomain(T=f64)")
  expect_equal(toString(inferred_scalar_space[[2]]), "AbsoluteDistance(f64)")

  vector_space <- space_of("Vec<i32>")
  expect_equal(toString(vector_space[[1]]), "VectorDomain(AtomDomain(T=i32))")
  expect_equal(toString(vector_space[[2]]), "SymmetricDistance()")
})

test_that("context and query stringification include key fields", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 1L
  )
  q <- query(context)

  expect_match(toString(context), "^Context\\(")
  expect_match(toString(context), "accountant = Measurement")
  expect_match(toString(q), "^Query\\(")
  expect_match(toString(q), "output_measure = MaxDivergence")
  expect_match(toString(q), "context        = Context\\(")
})

test_that("context query resolves auto through then_laplace", {
  context <- Context$compositor(
    data = c(5.0, 6.0, 7.0),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 1L
  )

  q <- query(context) |>
    then_clamp(c(0.0, 10.0)) |>
    then_sum() |>
    then_laplace(auto())

  expect_type(param(q, .T = "float"), "double")
  expect_type(release(q), "double")
  expect_length(current_privacy_loss(context), 1L)
  expect_length(remaining_privacy_loss(context), 0L)
})

test_that("stand-alone Query resolves auto against explicit data", {
  q <- Query(
    chain = space_of("Vec<f64>"),
    output_measure = max_divergence(),
    d_in = 1L,
    d_out = 1.0
  ) |>
    then_clamp(c(0.0, 10.0)) |>
    then_sum() |>
    then_laplace(auto())

  expect_type(param(q, .T = "float"), "double")
  expect_type(release(q, data = c(1.0, 2.0, 3.0)), "double")
})

test_that("stand-alone query without a concrete measurement cannot be released", {
  q <- Query(
    chain = space_of("Vec<f64>"),
    output_measure = max_divergence(),
    d_in = 1L,
    d_out = 1.0
  ) |>
    then_impute_constant(0.0)

  expect_error(release(q, data = c(1.0, 2.0, 3.0)), "Query is not yet a measurement or odometer")
  expect_error(resolve(q), "Query is not yet a measurement or odometer")
  expect_error(param(q), "Query is not yet a measurement or odometer")
})

test_that("stand-alone query auto requires both d_in and d_out", {
  q <- Query(
    chain = space_of("Vec<i32>"),
    output_measure = max_divergence(),
    d_in = 1L
  ) |>
    then_count() |>
    then_laplace(auto())

  expect_error(
    param(q, .T = "float"),
    "Cannot resolve auto\\(\\) without both d_in and d_out"
  )
})

test_that("multiple auto sentinels are rejected", {
  q <- Query(
    chain = space_of("Vec<f64>"),
    output_measure = max_divergence(),
    d_in = 1L,
    d_out = 1.0
  )

  expect_error(
    q |> then_resize(size = auto(), constant = auto()),
    "At most one auto\\(\\) may be unresolved at a time"
  )
})

test_that("nested compositor returns a child context", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 2L
  )

  subcontext <- query(context) |>
    then_clamp(c(0L, 10L)) |>
    then_sum() |>
    compositor(split_evenly_over = 1L) |>
    release()

  expect_s3_class(subcontext, "opendp_context")

  answer <- query(subcontext) |>
    then_laplace(auto()) |>
    release()

  expect_type(answer, "integer")
})

test_that("nested compositor updates the child query space after transformations", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 2L,
    domain = vector_domain(atom_domain(.T = "i32"))
  )

  subcontext <- query(context) |>
    then_clamp(c(0L, 1L)) |>
    then_sum() |>
    compositor(split_evenly_over = 1L) |>
    release()

  subquery <- query(subcontext)
  expect_equal(toString(subcontext$accountant("input_domain")), "VectorDomain(AtomDomain(T=i32))")
  expect_equal(toString(subquery$chain[[1]]), "AtomDomain(T=i32)")
  expect_equal(toString(subquery$chain[[2]]), "AbsoluteDistance(i32)")
})

test_that("static split accounting consumes d_mids and exhausts the allowance", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_by_weights = c(1, 2)
  )

  expect_length(current_privacy_loss(context), 0L)
  expect_unlisted_equal(remaining_privacy_loss(context), c(1 / 3, 2 / 3), tolerance = 1e-6)
  expect_error(
    query(context, epsilon = 0.5),
    "Expected no privacy arguments for this context query"
  )

  q1 <- query(context) |>
    then_count() |>
    then_laplace(auto())
  q2 <- query(context) |>
    then_count() |>
    then_laplace(auto())

  expect_equal(param(q1, .T = "float"), 3)
  expect_equal(param(q2, .T = "float"), 3)
  expect_type(release(q1), "integer")
  expect_unlisted_equal(current_privacy_loss(context), 1 / 3, tolerance = 1e-6)
  expect_unlisted_equal(remaining_privacy_loss(context), 2 / 3, tolerance = 1e-6)

  expect_type(release(q2), "integer")
  expect_unlisted_equal(current_privacy_loss(context), c(1 / 3, 2 / 3), tolerance = 1e-6)
  expect_length(remaining_privacy_loss(context), 0L)
  expect_error(query(context), "Privacy allowance has been exhausted")
})

test_that("fully adaptive pure DP filter tracks consumed and remaining budget", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0)
  )

  expect_equal(current_privacy_loss(context), 0)
  expect_equal(remaining_privacy_loss(context), 1.0)

  q1 <- query(context, epsilon = 0.5) |>
    then_count() |>
    then_laplace(auto())

  expect_equal(param(q1, .T = "float"), 2)
  expect_type(release(q1), "integer")
  expect_equal(current_privacy_loss(context), 0.5)
  expect_equal(remaining_privacy_loss(context), 0.5)

  q2 <- query(context) |>
    then_count() |>
    then_laplace(2.0)

  expect_type(release(q2), "integer")
  expect_equal(current_privacy_loss(context), 1)
  expect_equal(remaining_privacy_loss(context), 0)
  expect_error(
    release(query(context, epsilon = 0.5) |> then_count() |> then_laplace(auto())),
    "filter is now exhausted"
  )
})

test_that("fully adaptive zCDP filter tracks scalar rho budget", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(rho = 0.5)
  )

  expect_equal(current_privacy_loss(context), 0)
  expect_equal(remaining_privacy_loss(context), 0.5)

  q <- query(context, rho = 0.25) |>
    then_count() |>
    then_gaussian(auto())

  expect_equal(param(q, .T = "float"), sqrt(2), tolerance = 1e-6)
  expect_type(release(q), "integer")
  expect_equal(current_privacy_loss(context), 0.25)
  expect_equal(remaining_privacy_loss(context), 0.25)
})

test_that("approximate DP query requires explicit delta in query args", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0, delta = 1e-8)
  )

  expect_error(
    query(context, epsilon = 0.5),
    "Consider setting delta = 0.0 in your query"
  )

  q <- query(context, epsilon = 0.5, delta = 0.0) |>
    then_count() |>
    then_laplace(auto())

  expect_equal(param(q, .T = "float"), 2)
  expect_type(release(q), "integer")
  expect_unlisted_equal(current_privacy_loss(context), c(0.5, 0.0))
})

test_that("query release rejects explicit data when attached to a context", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 1L
  )

  q <- query(context) |>
    then_count() |>
    then_laplace(auto())

  expect_error(
    release(q, data = c(1L, 2L, 3L)),
    "Cannot specify data when the query is part of a context"
  )
})

test_that("auto is rejected outside the Context API", {
  expect_error(
    space_of("Vec<f64>") |>
      then_clamp(c(0.0, 1.0)) |>
      then_sum() |>
      then_laplace(auto()),
    "auto\\(\\) may only be used inside the Context API"
  )
})

test_that("translate_measure_distance supports exported measure conversions", {
  translate_measure_distance_ <- get_context_internal("translate_measure_distance")

  expect_equal(
    translate_measure_distance_(
      1.0,
      max_divergence(),
      approximate(max_divergence())
    ),
    c(1.0, 0.0)
  )

  approx_zcdp <- translate_measure_distance_(
    c(1.0, 1e-6),
    approximate(max_divergence()),
    approximate(zero_concentrated_divergence()),
    alpha = 0.3
  )
  expect_length(approx_zcdp, 2L)
  expect_gt(approx_zcdp[[1]], 0)
  expect_equal(approx_zcdp[[2]], 3e-7, tolerance = 1e-12)
})

test_that("cast_measure supports Approximate zCDP to Approximate DP", {
  cast_measure_ <- get_context_internal("cast_measure")
  space <- list(
    atom_domain(.T = "f64", nan = FALSE),
    absolute_distance(.T = "f64")
  )

  approx_zcdp <- make_approximate(space |> then_gaussian(2.0))
  approx_dp <- cast_measure_(
    approx_zcdp,
    approximate(max_divergence()),
    c(1.0, 1e-6)
  )

  expect_equal(toString(approx_dp("output_measure")), "Approximate(MaxDivergence)")
})

test_that("transformation-chained unbounded compositors are supported", {
  context <- Context$compositor(
    data = c(1L, 2L, 3L),
    privacy_unit = unit_of(contributions = 1L),
    privacy_loss = loss_of(epsilon = 1.0),
    split_evenly_over = 1L
  )

  subcontext <- query(context) |>
    then_clamp(c(0L, 10L)) |>
    then_sum() |>
    compositor(d_out = Inf) |>
    release()

  expect_s3_class(subcontext, "opendp_context")
  expect_equal(remaining_privacy_loss(subcontext), 1.0)

  answer <- query(subcontext, epsilon = 0.5) |>
    then_laplace(auto()) |>
    release()

  expect_type(answer, "integer")
  expect_equal(current_privacy_loss(subcontext), 0.5)
  expect_equal(remaining_privacy_loss(subcontext), 0.5)
})
