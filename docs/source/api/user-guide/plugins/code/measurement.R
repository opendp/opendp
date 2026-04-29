library(opendp)

enable_features("honest-but-curious", "contrib")


# make-base-constant
make_base_constant <- function(input_domain, input_metric, constant) {
  function_ <- function(.arg) constant
  privacy_map <- function(d_in) 0.

  make_user_measurement(
    input_domain = input_domain,
    input_metric = input_metric,
    output_measure = pure_dp(),
    function_ = function_,
    privacy_map = privacy_map,
    .TO = String
  )
}
then_base_constant <- to_then(make_base_constant)
# /make-base-constant


# use-measurement
space <- c(
  vector_domain(atom_domain(bounds = c(0L, 10L))),
  symmetric_distance()
)
meas <- space |>
  then_sum() |>
  then_base_constant("denied")
meas(arg = c(2L, 3L, 4L))
meas(d_in = 1L)
# /use-measurement
