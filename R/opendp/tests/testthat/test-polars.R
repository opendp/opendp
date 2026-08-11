skip_if_incompatible_polars <- function() {
  skip_if_not_installed("polars", minimum_version = "1.7.0")
  skip_if(
    utils::packageVersion("polars") != package_version("1.7.0"),
    "binary DSL test requires exactly r-polars 1.7.0"
  )
}

test_that("wild_expr_domain constructs row-by-row and aggregation domains", {
  columns <- list(
    series_domain("A", atom_domain(.T = "f64", nan = FALSE)),
    series_domain("B", atom_domain(.T = "i32"))
  )
  # Row-by-row (no margin), and several aggregation margins.
  expect_no_error(wild_expr_domain(columns))
  expect_no_error(wild_expr_domain(columns, by = character(0)))
  expect_no_error(wild_expr_domain(columns, by = "B", max_groups = 10))
  expect_no_error(wild_expr_domain(columns, by = "B", invariant = "keys"))
  expect_no_error(
    wild_expr_domain(columns, by = "B", max_length = 50, invariant = "lengths")
  )
})

test_that("wild_expr_domain validates its arguments", {
  columns <- list(series_domain("A", atom_domain(.T = "f64", nan = FALSE)))
  expect_error(wild_expr_domain("not a list"), "must be a list")
  expect_error(wild_expr_domain(columns, by = NA_character_), "missing values")
  expect_error(
    wild_expr_domain(columns, by = "A", invariant = "nonsense"),
    "keys"
  )
  expect_error(
    wild_expr_domain(columns, max_length = 10),
    "require a non-NULL"
  )
})

test_that("r-polars expressions round-trip through OpenDP", {
  skip_if_incompatible_polars()
  pl <- polars::pl

  expressions <- list(
    pl$col("x"),
    pl$col("x")$sum(),
    (pl$col("x") + 1)$alias("y"),
    pl$col("x")$cast(pl$Float64),
    pl$when(pl$col("x") > 0)$then(1)$otherwise(0)
  )

  for (expr in expressions) {
    returned <- opendp_roundtrip_expr(expr$meta$serialize())
    expr2 <- pl$deserialize_expr(returned)
    expect_no_error(pl$LazyFrame(x = 1:5)$select(expr2)$collect())
  }
})

test_that("r-polars lazy frames round-trip through OpenDP", {
  skip_if_incompatible_polars()
  pl <- polars::pl

  lf <- pl$LazyFrame(
    group = c("a", "a", "b"),
    value = c(1, 2, 3)
  )$
    group_by("group")$
    agg(pl$col("value")$sum())

  lf2 <- pl$deserialize_lf(opendp_roundtrip_lazyframe(lf$serialize()))
  expect_equal(
    as.data.frame(lf2$collect()$sort("group")),
    as.data.frame(lf$collect()$sort("group"))
  )
})

test_that("Polars round trips reject malformed input", {
  skip_if_incompatible_polars()
  expect_error(opendp_roundtrip_expr(as.raw(0:3)), "deserializing r-polars 'Expr'")
  expect_error(opendp_roundtrip_lazyframe(as.raw(0:3)), "deserializing 'DslPlan'")
})

test_that("typed OpenDP plugin expressions survive r-polars plans", {
  skip_if_incompatible_polars()
  pl <- polars::pl

  private_len <- dp_len()
  private_sum <- dp_sum(pl$col("value"), bounds = c(0, 100))

  expect_no_error(
    pl$deserialize_expr(opendp_roundtrip_expr(
      private_sum$meta$serialize(format = "binary")
    ))
  )

  lf <- pl$LazyFrame(
    group = c("a", "a", "b"),
    value = c(1, 2, 3)
  )$
    group_by("group")$
    agg(n = private_len, total = private_sum)

  explanation <- lf$explain(optimized = FALSE)
  expect_match(explanation, "dp_frame_len", fixed = TRUE)
  expect_match(explanation, "dp_sum", fixed = TRUE)
  expect_named(lf$collect_schema(), c("group", "n", "total"))

  lf2 <- pl$deserialize_lf(opendp_roundtrip_lazyframe(lf$serialize()))
  expect_match(lf2$explain(optimized = FALSE), "dp_sum", fixed = TRUE)
  expect_error(
    lf2$collect(),
    "must be passed through make_private_lazyframe before execution",
    fixed = TRUE
  )
})

test_that("typed plugin constructors validate numeric parameters", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  expect_error(dp_len(scale = -1), "non-negative")
  expect_error(dp_sum(pl$col("x"), c(2, 1)), "lower bound")
  expect_error(dp_sum(pl$col("x"), c(0, Inf)), "finite")
  expect_error(dp_mean(pl$col("x"), c(2, 1)), "lower bound")
  expect_error(dp_mean(pl$col("x"), c(0, Inf)), "finite")
  expect_error(dp_median(pl$col("x"), numeric()), "non-empty")
  expect_error(dp_median(pl$col("x"), c(1, 2), scale = -1), "non-negative")
  expect_error(dp_quantile(pl$col("x"), 1.5, c(1, 2)), "between 0 and 1")
  expect_error(dp_quantile(pl$col("x"), 0.5, numeric()), "non-empty")
  expect_error(dp_len(signed = NA), "TRUE or FALSE")
  expect_error(dp_len(signed = "yes"), "TRUE or FALSE")
  expect_error(dp_noise(pl$col("x"), scale = -1), "non-negative")
  expect_error(dp_count(pl$col("x"), scale = Inf), "finite")
  expect_error(dp_n_unique(pl$col("x"), scale = "big"), "non-negative")
})

test_that("aggregation plugin expressions survive r-polars plans", {
  skip_if_incompatible_polars()
  pl <- polars::pl

  candidates <- c(0, 25, 50, 75, 100)
  # Map each expected plugin symbol (as it appears in the plan) to a
  # constructor applied to `value`. Note the noise plugin serializes as
  # "noise", not "dp_noise".
  cases <- list(
    dp_mean = dp_mean(pl$col("value"), bounds = c(0, 100)),
    dp_median = dp_median(pl$col("value"), candidates = candidates),
    dp_quantile = dp_quantile(pl$col("value"), 0.25, candidates = candidates),
    noise = dp_noise(pl$col("value")$sum()),
    dp_count = dp_count(pl$col("value")),
    dp_null_count = dp_null_count(pl$col("value")),
    dp_n_unique = dp_n_unique(pl$col("value"))
  )

  for (symbol in names(cases)) {
    round_tripped <- pl$deserialize_expr(opendp_roundtrip_expr(
      cases[[symbol]]$meta$serialize(format = "binary")
    ))
    lf <- pl$LazyFrame(
      group = c("a", "a", "b"),
      value = c(1, 2, 3)
    )$
      group_by("group")$
      agg(out = round_tripped)
    expect_match(lf$explain(optimized = FALSE), symbol, fixed = TRUE)
  }
})

test_that("private r-polars lazy frames release a DataFrame exactly once", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  input_domain <- lazyframe_domain(list(
    series_domain("group", atom_domain(.T = "String")),
    series_domain("value", atom_domain(.T = "f64", nan = FALSE))
  ))
  input_domain <- with_margin(
    input_domain,
    by = "group",
    max_length = 10,
    max_groups = 10
  )

  analysis <- pl$DataFrame(
    group = character(),
    value = double()
  )$
    lazy()$
    group_by("group")$
    agg(
      n = dp_len(),
      total = dp_sum(pl$col("value"), bounds = c(0, 100))
    )

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = approximate(max_divergence()),
    lazyframe = analysis,
    global_scale = 1,
    threshold = 3
  )
  private_data <- pl$DataFrame(
    group = c("a", "a", "b", "b", "b"),
    value = c(1, 2, 3, 4, 5)
  )$lazy()

  release <- measurement(arg = private_data)
  result <- release$collect()
  expect_s3_class(result, "polars_data_frame")
  expect_named(result$schema, c("group", "n", "total"))
  expect_error(release$collect(), "OnceFrame has been exhausted", fixed = TRUE)
})

test_that("pure-DP grouped release is exact at zero scale", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  # Public keys ("keys" invariant) allow pure-DP release without a threshold.
  input_domain <- lazyframe_domain(list(
    series_domain("group", atom_domain(.T = "String")),
    series_domain("value", atom_domain(.T = "f64", nan = FALSE))
  ))
  input_domain <- with_margin(
    input_domain,
    by = "group",
    max_length = 50,
    max_groups = 10,
    invariant = "keys"
  )

  analysis <- pl$DataFrame(
    group = character(),
    value = double()
  )$
    lazy()$
    group_by("group")$
    agg(total = dp_sum(pl$col("value"), bounds = c(0, 100)))

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = max_divergence(),
    lazyframe = analysis,
    global_scale = 0
  )
  private_data <- pl$DataFrame(
    group = c("a", "a", "b"),
    value = c(1, 2, 3)
  )$lazy()

  result <- as.data.frame(measurement(arg = private_data)$collect()$sort("group"))
  expect_equal(result$group, c("a", "b"))
  expect_equal(as.numeric(result$total), c(3, 3))
})

test_that("pure-DP global count queries are exact at zero scale", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  # A global select needs no margin (single implicit group), so pure-DP applies.
  # Use f64: R integers map to Polars Int32, not the i64 a naive domain implies.
  input_domain <- lazyframe_domain(list(
    series_domain("data", atom_domain(.T = "f64", nan = FALSE))
  ))

  analysis <- pl$DataFrame(data = double())$
    lazy()$
    select(
      count = dp_count(pl$col("data")),
      n_unique = dp_n_unique(pl$col("data")),
      null_count = dp_null_count(pl$col("data"))
    )

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = max_divergence(),
    lazyframe = analysis,
    global_scale = 0
  )
  private_data <- pl$LazyFrame(data = c(1, 1, 1, NA_real_))

  result <- as.data.frame(measurement(arg = private_data)$collect())
  expect_equal(as.numeric(result$count), 3)
  expect_equal(as.numeric(result$n_unique), 2)
  expect_equal(as.numeric(result$null_count), 1)
})

test_that("pure-DP frame length honors the signed flag at zero scale", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  # Empty grouping with public keys mirrors Python's test_filter setup.
  input_domain <- lazyframe_domain(list(
    series_domain("value", atom_domain(.T = "f64", nan = FALSE))
  ))
  input_domain <- with_margin(
    input_domain,
    by = character(0),
    max_length = 50,
    invariant = "keys"
  )

  analysis <- pl$DataFrame(value = double())$
    lazy()$
    select(
      unsigned_len = dp_len(),
      signed_len = dp_len(signed = TRUE)
    )

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = max_divergence(),
    lazyframe = analysis,
    global_scale = 0
  )
  private_data <- pl$DataFrame(value = c(1, 2, 3, 4, 5))$lazy()

  # signed changes only the output integer type, so at zero scale both equal 5.
  result <- as.data.frame(measurement(arg = private_data)$collect())
  expect_equal(as.numeric(result$unsigned_len), 5)
  expect_equal(as.numeric(result$signed_len), 5)
})

test_that("pure-DP mean is exact at zero scale", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  # dp_mean requires a public group length, so the margin uses the "lengths"
  # invariant (mirrors Python's test_private_lazyframe_mean).
  input_domain <- lazyframe_domain(list(
    series_domain("group", atom_domain(.T = "String")),
    series_domain("value", atom_domain(.T = "f64", nan = FALSE))
  ))
  input_domain <- with_margin(
    input_domain,
    by = "group",
    max_length = 50,
    max_groups = 10,
    invariant = "lengths"
  )

  analysis <- pl$DataFrame(
    group = character(),
    value = double()
  )$
    lazy()$
    group_by("group")$
    agg(avg = dp_mean(pl$col("value"), bounds = c(0, 10)))

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = max_divergence(),
    lazyframe = analysis,
    global_scale = 0
  )
  # group a: mean(1, 3) = 2; group b: mean(2, 4) = 3
  private_data <- pl$DataFrame(
    group = c("a", "a", "b", "b"),
    value = c(1, 3, 2, 4)
  )$lazy()

  result <- as.data.frame(measurement(arg = private_data)$collect()$sort("group"))
  expect_equal(result$group, c("a", "b"))
  expect_equal(as.numeric(result$avg), c(2, 3))
})

test_that("pure-DP median and quantile select the exact candidate at zero scale", {
  skip_if_incompatible_polars()
  pl <- polars::pl
  old_features <- getOption("opendp_features")
  on.exit(options(opendp_features = old_features), add = TRUE)
  enable_features("contrib", "honest-but-curious")

  # The median / quantile selection mechanisms need only public group keys
  # (mirrors Python's test_private_lazyframe_median). dp_median is dp_quantile
  # at alpha = 0.5, so both select candidate 3 from values 1..5.
  input_domain <- lazyframe_domain(list(
    series_domain("group", atom_domain(.T = "String")),
    series_domain("value", atom_domain(.T = "f64", nan = FALSE))
  ))
  input_domain <- with_margin(
    input_domain,
    by = "group",
    max_length = 50,
    invariant = "keys"
  )

  candidates <- c(1, 2, 3, 4, 5)
  analysis <- pl$DataFrame(
    group = character(),
    value = double()
  )$
    lazy()$
    group_by("group")$
    agg(
      med = dp_median(pl$col("value"), candidates = candidates),
      q = dp_quantile(pl$col("value"), 0.5, candidates = candidates)
    )

  measurement <- make_private_lazyframe(
    input_domain = input_domain,
    input_metric = symmetric_distance(),
    output_measure = max_divergence(),
    lazyframe = analysis,
    global_scale = 0
  )
  private_data <- pl$DataFrame(
    group = rep("a", 5),
    value = c(1, 2, 3, 4, 5)
  )$lazy()

  result <- as.data.frame(measurement(arg = private_data)$collect())
  expect_equal(result$group, "a")
  expect_equal(as.numeric(result$med), 3)
  expect_equal(as.numeric(result$q), 3)
})
