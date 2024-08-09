library(opendp)

test_that("rt_parse-vec", {
  rt <- rt_parse("Vec<i32>")
  expect_equal(rt$origin, "Vec")
  expect_equal(rt$args[[1]], i32)
})

test_that("rt_parse-tuple", {
  rt <- rt_parse("(f64, i32)")
  expect_equal(rt$origin, "Tuple")
  expect_equal(rt$args[[1]], f64)
  expect_equal(rt$args[[2]], i32)
})
