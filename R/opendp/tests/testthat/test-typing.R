library(opendp)

test_that("rt_parse-vec", {
  rtype <- rt_parse("Vec<i32>")
  expect_equal(rtype$origin, "Vec")
  expect_equal(rtype$args[[1]], i32)
})

test_that("rt_parse-tuple", {
  rtype <- rt_parse("(f64, i32)")
  expect_equal(rtype$origin, "Tuple")
  expect_equal(rtype$args[[1]], f64)
  expect_equal(rtype$args[[2]], i32)
})
