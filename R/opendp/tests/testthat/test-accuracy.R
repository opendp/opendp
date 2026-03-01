test_that("test_laplacian_scale_to_accuracy", {
  expect_lt(laplacian_scale_to_accuracy(scale = 1., alpha = 0.05), 3)
})

test_that("test_accuracy_to_laplacian_scale", {
  expect_lt(accuracy_to_laplacian_scale(accuracy = 1., alpha = 0.05), 0.334)
})


test_that("test_gaussian_scale_to_accuracy", {
  expect_lt(gaussian_scale_to_accuracy(scale = 1., alpha = 0.05), 1.96)
})


test_that("test_accuracy_to_gaussian_scale", {
  expect_lt(accuracy_to_gaussian_scale(accuracy = 1., alpha = 0.05), 0.511)
})
