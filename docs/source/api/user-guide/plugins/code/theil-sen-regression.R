library(opendp)

enable_features("contrib", "honest-but-curious")


# pairwise-predict
pairwise_predict <- function(data, x_cuts) {
  data <- as.matrix(data)
  data <- data[seq_len(nrow(data) %/% 2L * 2L), , drop = FALSE]
  data <- data[sample(nrow(data)), , drop = FALSE]

  midpoint <- nrow(data) %/% 2L
  p1 <- data[seq_len(midpoint), , drop = FALSE]
  p2 <- data[midpoint + seq_len(midpoint), , drop = FALSE]

  dx <- p2[, 1] - p1[, 1]
  dy <- p2[, 2] - p1[, 2]
  x_bar <- (p1[, 1] + p2[, 1]) / 2
  y_bar <- (p1[, 2] + p2[, 2]) / 2

  points <- sapply(x_cuts, function(x_cut) dy / dx * (x_cut - x_bar) + y_bar)
  points[dx != 0, , drop = FALSE]
}

make_pairwise_predict <- function(x_cuts, runs = 1L) {
  make_user_transformation(
    input_domain = user_domain(
      identifier = "Matrix2Domain",
      member = function(x) is.matrix(x) && ncol(x) == 2L
    ),
    input_metric = symmetric_distance(),
    output_domain = user_domain(
      identifier = "Matrix2Domain",
      member = function(x) is.matrix(x) && ncol(x) == 2L
    ),
    output_metric = symmetric_distance(),
    function_ = function(x) do.call(rbind, replicate(runs, pairwise_predict(x, x_cuts), simplify = FALSE)),
    stability_map = function(d_in) d_in * runs
  )
}
# /pairwise-predict


# private-medians
make_select_column <- function(j) {
  make_user_transformation(
    input_domain = user_domain(
      identifier = "Matrix2Domain",
      member = function(x) is.matrix(x) && ncol(x) == 2L
    ),
    input_metric = symmetric_distance(),
    output_domain = vector_domain(atom_domain(.T = f64)),
    output_metric = symmetric_distance(),
    function_ = function(x) x[, j],
    stability_map = function(d_in) d_in
  )
}

make_private_percentile_medians <- function(output_measure, y_bounds, scale) {
  m_median <- then_private_quantile(
    output_measure = output_measure,
    candidates = seq(y_bounds[[1]], y_bounds[[2]], length.out = 100L),
    alpha = 0.5,
    scale = scale
  )

  make_composition(c(
    make_select_column(1L) >> m_median,
    make_select_column(2L) >> m_median
  ))
}
# /private-medians


# mechanism
make_private_theil_sen <- function(output_measure, x_bounds, y_bounds, scale, runs = 1L) {
  x_cuts <- x_bounds[[1]] + (x_bounds[[2]] - x_bounds[[1]]) * c(0.25, 0.75)
  p_inv <- solve(cbind(x_cuts, 1))
  postprocess <- new_function(
    function_ = function(ys) as.vector(p_inv %*% ys),
    .TO = "Vec<f64>"
  )

  make_pairwise_predict(x_cuts, runs) >>
    make_private_percentile_medians(output_measure, y_bounds, scale) >>
    postprocess
}

x_bounds <- c(-3, 3)
y_bounds <- c(-10, 10)
meas <- make_private_theil_sen(max_divergence(), x_bounds, y_bounds, scale = 1.)
meas(d_in = 1L)
# /mechanism


# release
f <- function(x) x * 2 + 1

x <- rnorm(100)
y <- f(x) + rnorm(100, sd = 0.5)
private_data <- cbind(x, y)
meas(arg = private_data)
# /release
