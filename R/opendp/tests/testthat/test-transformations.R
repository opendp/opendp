library(opendp)
enable_features("contrib")


INT_DATA <- c(1L, 2L, 3L, 4L, 5L, 6L, 7L, 8L, 9L)
FLOAT_DATA <- c(1, 2, 3, 4, 5, 6, 7, 8, 9)
STR_DATA <- c("1", "2", "3", "4", "5", "6", "7", "8", "9")

hashtab_eq <- function(x, y) {
  if (utils::numhash(x) != utils::numhash(y)) {
    return(FALSE)
  }
  equal <- TRUE
  utils::maphash(x, function(k, v) {
    equal <<- equal && identical(utils::gethash(y, k), v)
  })
  equal
}


test_that("choose_branching_factor", {
  expect_equal(choose_branching_factor(100L), 100L)
  expect_equal(choose_branching_factor(10000L), 22L)
})

test_that("make_clamp", {
  domain <- vector_domain(atom_domain(.T = "i32"))
  metric <- symmetric_distance()
  trans <- make_clamp(domain, metric, bounds = list(0L, 2L))

  expect_equal(trans(arg = c(1L, 2L, 3L)), c(1L, 2L, 2L))
  expect_equal(trans(d_in = 1L), 1L)
})

test_that("make_sum", {
  domain <- vector_domain(atom_domain(bounds = list(1L, 2L)))
  metric <- symmetric_distance()
  trans <- make_sum(domain, metric)

  expect_equal(trans(arg = c(1L, 2L)), 3L)
  expect_equal(trans(d_in = 1L), 2L)

  expect_equal(trans(arg = c(1L, 2L)), 3L)
  expect_equal(trans(d_in = 1L), 2L)
})


test_that("test_cast_impute", {
  input_space <- c(vector_domain(atom_domain(.T = f64)), symmetric_distance())
  caster <- input_space |>
    then_cast(.TOA = i32) |>
    then_impute_constant(-1L)
  expect_equal(caster(arg = c(1., 2., 3.)), c(1L, 2L, 3L))

  caster <- input_space |>
    then_cast(.TOA = i32) |>
    then_impute_constant(1L)
  expect_equal(caster(arg = c(NaN, 2.)), c(1L, 2L))
})

test_that("test_cast_drop_null", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  caster <- input_space |>
    then_cast(.TOA = i32) |>
    then_drop_null()
  expect_equal(caster(arg = c("A", "2", "3")), c(2L, 3L))

  caster <- input_space |>
    then_cast_inherent(.TOA = f64) |>
    then_drop_null()
  expect_equal(caster(arg = c("a", "2.")), 2L)

  input_space <- c(vector_domain(atom_domain(.T = f64)), symmetric_distance())
  caster <- input_space |>
    then_cast(.TOA = i32) |>
    then_drop_null()
  expect_equal(caster(arg = c(NaN, 2.)), 2L)
})

test_that("test_cast_inherent", {
  input_space <- c(vector_domain(atom_domain(.T = i32)), symmetric_distance())
  caster <- input_space |> then_cast_inherent(.TOA = f64)

  expect_equal(caster(arg = c(1L, 2L)), c(1., 2.))
})

test_that("test_impute_constant_inherent", {
  tester <- make_split_lines() |>
    then_cast_inherent(.TOA = f64) |>
    then_impute_constant(-1.)
  expect_equal(tester(arg = "nan\n1."), c(-1., 1.))
})

test_that("test_cast_default", {
  caster <- make_cast_default(
    vector_domain(atom_domain(.T = f64)),
    symmetric_distance(),
    .TOA = i32
  )
  expect_equal(caster(arg = c(NaN, 2.)), c(0L, 2L))
})

test_that("test_impute_uniform", {
  caster <- make_impute_uniform_float(
    vector_domain(atom_domain(.T = f64, nan = TRUE)),
    symmetric_distance(),
    bounds = c(-1., 2.)
  )
  expect_lte(-1., caster(arg = NaN))
  expect_lte(caster(arg = NaN), 2.)
})

test_that("test_is_equal", {
  input_domain <- vector_domain(atom_domain(.T = i32))
  input_metric <- symmetric_distance()
  tester <- make_is_equal(input_domain, input_metric, 3L)
  expect_equal(tester(arg = c(1L, 2L, 3L)), c(FALSE, FALSE, TRUE))
})

test_that("test_is_null", {
  tester <- make_split_lines() |>
    then_cast_inherent(.TOA = f64) |>
    then_is_null()
  expect_equal(tester(arg = "nan\n1.\ninf"), c(TRUE, FALSE, FALSE))

  tester <- make_split_lines() |>
    then_cast(.TOA = f64) |>
    then_is_null()
  expect_equal(tester(arg = "nan\n1.\ninf"), c(TRUE, FALSE, FALSE))
})

test_that("test_split_lines__cast__impute", {
  expect_equal(make_split_lines()(arg = "1\n2\n3"), c("1", "2", "3"))
  query <- make_split_lines() |>
    then_cast(.TOA = i32) |>
    then_impute_constant(constant = 2L)

  expect_equal(query(arg = "1\n2\n3"), c(1L, 2L, 3L))
  expect_equal(query(d_in = 1L), 1L)
})

test_that("test_inherent_cast__impute", {
  casted <- make_split_lines() |>
    then_cast_inherent(.TOA = f64) |>
    then_impute_constant(constant = 9.)

  expect_equal(casted(arg = "a\n23.23\n12"), c(9.0, 23.23, 12.0))
  expect_equal(casted(d_in = 1L), 1L)
})

test_that("test_inherent_cast__impute_uniform", {
  casted <- make_split_lines() |>
    then_cast_inherent(.TOA = f64) |>
    then_impute_uniform_float(bounds = c(23., 32.5))

  res <- casted(arg = "a\n23.23\n12")
  expect_equal(tail(res, -1), c(23.23, 12.0))
  expect_lte(23.0, res[[1]])
  expect_lte(res[[1]], 32.5)

  expect_equal(casted(d_in = 1L), 1L)
})

test_that("test_clamp", {
  input_domain <- vector_domain(atom_domain(.T = i32))
  input_metric <- symmetric_distance()
  query <- c(input_domain, input_metric) |> then_clamp(bounds = c(-1L, 1L))
  expect_equal(query(arg = c(-10L, 0L, 10L)), c(-1L, 0L, 1L))
  expect_equal(query(d_in = 1L), 1L)

  query2 <- make_clamp(input_domain, input_metric, bounds = c(-1L, 1L))
  expect_equal(query2(arg = c(-10L, 0L, 10L)), c(-1L, 0L, 1L))
  expect_equal(query2(d_in = 1L), 1L)
})

test_that("test_bounded_mean", {
  query <- make_mean(vector_domain(atom_domain(bounds = c(0.0, 10.0)), size = 9L), symmetric_distance())
  expect_equal(query(arg = FLOAT_DATA), 5.)
  expect_lt(query(d_in = 2L), 10. / 9. + 1e-6)
})

test_that("test_bounded_sum", {
  query <- make_sum(vector_domain(atom_domain(bounds = c(0., 10.))), symmetric_distance())
  expect_equal(query(arg = FLOAT_DATA), 45.)
  expect_lt(query(d_in = 1L), 20.)

  query <- make_sum(vector_domain(atom_domain(bounds = c(0L, 10L))), symmetric_distance())
  expect_equal(query(arg = INT_DATA), 45L)
  expect_equal(query(d_in = 1L), 10L)

  expect_error(query(arg = FLOAT_DATA))
})

test_that("test_sized_bounded_sum", {
  domain <- vector_domain(atom_domain(bounds = c(0., 10.)), size = 9L)
  metric <- symmetric_distance()
  query <- c(domain, metric) |> then_sum()
  expect_equal(query(arg = FLOAT_DATA), 45.)
  expect_lt(query(d_in = 1L), 10. + 1e-12)

  domain <- vector_domain(atom_domain(bounds = c(0., 10.)), size = 10000L)
  query <- c(domain, metric) |> then_sum()
  expect_lt(query(d_in = 1L), 10. + 1e-9)

  domain <- vector_domain(atom_domain(bounds = c(0., 10.)), size = 100000L)
  query <- c(domain, metric) |> then_sum()
  expect_lt(query(d_in = 1L), 10. + 1e-8)

  domain <- vector_domain(atom_domain(bounds = c(0., 10.)), size = 1000000L)
  query <- c(domain, metric) |> then_sum()
  expect_lt(query(d_in = 1L), 10. + 1e-7)

  domain <- vector_domain(atom_domain(bounds = c(0., 10.)), size = 10000000L)
  query <- c(domain, metric) |> then_sum()
  expect_lt(query(d_in = 1L), 10. + 1e-5)
})

test_that("test_bounded_variance", {
  query <- make_variance(
    vector_domain(atom_domain(bounds = c(0., 10.)), size = 9L),
    symmetric_distance()
  )
  expect_equal(query(arg = FLOAT_DATA), 7.5)
  expect_lt(query(d_in = 2L), 11.111111 + 1e-6)
})

test_that("test_sum_of_squared_deviances", {
  query <- make_sum_of_squared_deviations(
    vector_domain(atom_domain(bounds = c(0., 10.)), size = 9L),
    symmetric_distance()
  )
  expect_equal(query(arg = FLOAT_DATA), 60.0)
  expect_lt(query(d_in = 2L), 88.888888 + 1e-4)
})

test_that("test_count", {
  transformation <- make_count(vector_domain(atom_domain(.T = i32)), symmetric_distance())
  expect_equal(transformation(arg = c(1L, 2L, 3L)), 3L)
  expect_equal(transformation(d_in = 1L), 1L)
})

test_that("test_count_distinct", {
  transformation <- make_count_distinct(vector_domain(atom_domain(.T = String)), symmetric_distance())
  arg <- c("1", "2", "3", "2", "7", "3", "4")
  expect_equal(transformation(arg = c("1", "2", "3", "2", "7", "3", "4")), 5)
  expect_equal(transformation(d_in = 1L), 1L)
})

test_that("test_count_by", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  query <- input_space |> then_count_by(.MO = "L1Distance<f64>", .TV = f64)
  expect_true(hashtab_eq(query(arg = STR_DATA), new_hashtab(STR_DATA, rep(1, each = 9))))
  expect_equal(query(d_in = 1L), 1.)
})

test_that("test_count_by_categories", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  query <- input_space |> then_count_by_categories(categories = c("1", "3", "4"), .MO = "L1Distance<i32>")
  expect_equal(query(arg = STR_DATA), c(1L, 1L, 1L, 6L))
  expect_equal(query(d_in = 1L), 1L)
})

test_that("test_resize", {
  input_space <- c(vector_domain(atom_domain(bounds = c(0L, 10L))), symmetric_distance())
  query <- input_space |> then_resize(size = 4L, constant = 0L)
  expect_equal(sort(query(arg = c(-1L, 2L, 5L))), c(-1L, 0L, 2L, 5L))
  expect_equal(query(d_in = 1L), 2L)

  input_space <- c(vector_domain(atom_domain(.T = i32)), symmetric_distance())
  query <- input_space |> then_resize(size = 4L, constant = 0L)
  expect_equal(sort(query(arg = c(-1L, 2L, 5L))), c(-1L, 0L, 2L, 5L))
  expect_equal(query(d_in = 1L), 2L)
})

test_that("test_indexing", {
  input_space <- c(vector_domain(atom_domain(.T = String)), symmetric_distance())
  find <- input_space |>
    then_find(categories = c("1", "3", "4")) |>
    then_impute_constant(3L)
  expect_equal(find(arg = STR_DATA), c(0L, 3L, 1L, 2L, 3L, 3L, 3L, 3L, 3L))
  expect_equal(find(d_in = 1L), 1L)

  input_space <- c(vector_domain(atom_domain(.T = i32)), symmetric_distance())
  binner <- input_space |> then_find_bin(edges = c(2L, 3L, 5L))
  expect_equal(binner(arg = INT_DATA), c(0L, 1L, 2L, 2L, 3L, 3L, 3L, 3L, 3L))

  indexer <- c(find("output_domain"), find("output_metric")) |> then_index(categories = c("A", "B", "C"), null = "NA")
  expect_equal(indexer(arg = c(0L, 1L, 3L, 1L, 5L)), c("A", "B", "NA", "B", "NA"))

  indexed <- find |> then_index(categories = c("A", "B", "C"), null = "NA")
  expect_equal(indexed(arg = STR_DATA), c("A", "NA", "B", "C", "NA", "NA", "NA", "NA", "NA"))

  indexed <- binner |> then_index(categories = c("A", "B", "C"), null = "NA")
  expect_equal(indexed(arg = INT_DATA), c("A", "B", "C", "C", "NA", "NA", "NA", "NA", "NA"))
})

test_that("test_lipschitz_mul_float", {
  trans <- make_sized_bounded_float_ordered_sum(10L, c(0., 10.)) |> then_lipschitz_float_mul(1 / 10, c(-3., 4.))

  expect_equal(trans(arg = rep(3., each = 10)), 0.4)
  expect_equal(trans(d_in = 2L), 1)
})

test_that("test_lipschitz_b_ary_tree", {
  leaf_count <- 7L
  branching_factor <- 2L
  input_space <- c(vector_domain(atom_domain(.T = i32)), l1_distance(.T = i32))
  tree_builder <- input_space |> then_b_ary_tree(leaf_count, branching_factor)
  expect_equal(
    tree_builder(arg = rep(1L, each = leaf_count)),
    c(7L, 4L, 3L, 2L, 2L, 2L, 1L, 1L, 1L, 1L, 1L, 1L, 1L, 1L)
  )

  suggested_factor <- choose_branching_factor(size_guess = 10000L)

  # the categories are bin names!
  meas_base <-
    c(vector_domain(atom_domain(.T = String)), symmetric_distance()) |>
    then_count_by_categories(categories = c("A", "B", "C", "D", "E", "F")) |>
    then_b_ary_tree(leaf_count, branching_factor) |>
    then_geometric(1.) |>
    then_consistent_b_ary_tree(branching_factor)

  meas_cdf <- meas_base |> then_cdf()
  meas_quantiles <- meas_base |> then_quantiles_from_counts(
    bin_edges = c(0., 10., 13., 17., 26., 70., 84., 100.),
    alphas = c(0., 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.)
  )

  data <- c(rep("A", each = 34), rep("B", each = 23), rep("C", each = 12), rep("D", each = 84), rep("E", each = 34), rep("F", each = 85), rep("G", each = 75))
  meas_cdf(arg = data)
  meas_quantiles(arg = data)

  expect_equal(meas_cdf(d_in = 1L), 4.)
})
