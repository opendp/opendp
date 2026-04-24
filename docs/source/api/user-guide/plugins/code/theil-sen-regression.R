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

  dx <- p2[, 1L] - p1[, 1L]
  dy <- p2[, 2L] - p1[, 2L]
  x_bar <- (p1[, 1L] + p2[, 1L]) / 2L
  y_bar <- (p1[, 2L] + p2[, 2L]) / 2L

  predicted_points <- sapply(
    x_cuts,
    function(x_cut) dy / dx * (x_cut - x_bar) + y_bar
  )
  predicted_points[dx != 0L, , drop = FALSE]
}

make_pairwise_predict <- function(
  input_domain,
  input_metric,
  x_cuts,
  runs = 1L
) {
  make_user_transformation(
    input_domain = input_domain,
    input_metric = input_metric,
    output_domain = input_domain,
    output_metric = input_metric,
    function_ = function(x) {
      as.data.frame(
        do.call(
          rbind,
          replicate(runs, pairwise_predict(x, x_cuts), simplify = FALSE)
        )
      )
    },
    stability_map = function(d_in) d_in * runs
  )
}
then_pairwise_predict <- to_then(make_pairwise_predict)
# /pairwise-predict


# private-medians
make_select_column <- function(input_domain, input_metric, j) {
  make_user_transformation(
    input_domain = input_domain,
    input_metric = input_metric,
    output_domain = vector_domain(atom_domain(.T = f64)),
    output_metric = symmetric_distance(),
    function_ = function(x) x[, j],
    stability_map = function(d_in) d_in
  )
}
then_select_column <- to_then(make_select_column)

make_private_medians <- function(
  input_domain,
  input_metric,
  output_measure,
  y_bounds,
  scale
) {
  candidates <- seq(y_bounds[[1L]], y_bounds[[2L]], length.out = 100L)
  column_space <- c(input_domain, input_metric)
  first_median <- column_space |>
    then_select_column(1L) |>
    then_drop_null() |>
    then_private_quantile(
      output_measure = output_measure,
      candidates = candidates,
      alpha = 0.5,
      scale = scale
    )
  second_median <- column_space |>
    then_select_column(2L) |>
    then_drop_null() |>
    then_private_quantile(
      output_measure = output_measure,
      candidates = candidates,
      alpha = 0.5,
      scale = scale
    )

  make_composition(c(first_median, second_median))
}
then_private_medians <- to_then(make_private_medians)
# /private-medians


# mechanism
make_private_theil_sen <- function(
  input_domain,
  input_metric,
  output_measure,
  x_bounds,
  y_bounds,
  scale,
  runs = 1L
) {
  x_cuts <- x_bounds[[1L]] + (x_bounds[[2L]] - x_bounds[[1L]]) * c(0.25, 0.75)
  p_inv <- solve(cbind(x_cuts, 1L))

  c(input_domain, input_metric) |>
    then_pairwise_predict(x_cuts, runs) |>
    then_private_medians(output_measure, y_bounds, scale) |>
    then_postprocess(
      function(ys) as.vector(p_inv %*% as.numeric(unlist(ys))),
      .TO = "Vec<f64>"
    )
}
then_private_theil_sen <- to_then(make_private_theil_sen)

x_bounds <- c(-3.0, 3.0)
y_bounds <- c(-10.0, 10.0)
input_space <- c(
  user_domain(
    identifier = "DataFrame2Domain",
    member = function(x) is.data.frame(x) && ncol(x) == 2L
  ),
  symmetric_distance()
)
meas <- input_space |>
  then_private_theil_sen(max_divergence(), x_bounds, y_bounds, scale = 1.)
meas(d_in = 1L)
# /mechanism


# release
f <- function(x) x * 2L + 1L

x <- rnorm(100L)
y <- f(x) + rnorm(100L, sd = 0.5)
private_data <- data.frame(x = x, y = y)
meas(arg = private_data)
# /release
