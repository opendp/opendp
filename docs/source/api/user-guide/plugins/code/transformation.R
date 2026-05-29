library(opendp)

enable_features("honest-but-curious", "contrib")


# make-repeat
make_repeat <- function(input_domain, input_metric, multiplicity) {
  function_ <- function(arg) rep(arg, multiplicity)
  stability_map <- function(d_in) d_in * multiplicity

  make_user_transformation(
    input_domain = input_domain,
    input_metric = input_metric,
    output_domain = input_domain,
    output_metric = input_metric,
    function_ = function_,
    stability_map = stability_map
  )
}
then_repeat <- to_then(make_repeat)
# /make-repeat


# use-transformation
space <- c(
  vector_domain(atom_domain(.T = String)),
  symmetric_distance()
)
trans <- space |>
  then_cast_default(.TOA = i32) |>
  then_repeat(2L) |>
  then_clamp(c(1L, 2L)) |>
  then_sum() |>
  then_laplace(1.)
release <- trans(arg = c("0", "1", "2", "3"))
trans(d_in = 1L)
# /use-transformation
