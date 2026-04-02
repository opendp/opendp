library(opendp)

enable_features("honest-but-curious", "contrib")


# make-base-constant
make_base_constant <- function(constant) {
  function_ <- function(.arg) constant
  privacy_map <- function(d_in) 0.

  make_user_measurement(
    input_domain = atom_domain(.T = i32),
    input_metric = absolute_distance(i32),
    output_measure = max_divergence(),
    function_ = function_,
    privacy_map = privacy_map,
    .TO = String
  )
}
# /make-base-constant


# use-measurement
meas <- (
  new_privacy_context(vector_domain(atom_domain((0L, 10L))), symmetric_distance()) >>
    then_sum() >>
    make_base_constant("denied")
)
meas(arg = c(2L, 3L, 4L))
meas(d_in = 1L)
# /use-measurement
