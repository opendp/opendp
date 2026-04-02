library(opendp)

enable_features("honest-but-curious")


# make-repeat
make_repeat <- function(multiplicity) {
  function_ <- function(arg) rep(arg, multiplicity)
  stability_map <- function(d_in) d_in * multiplicity

  make_user_transformation(
    input_domain = vector_domain(atom_domain(.T = i32)),
    input_metric = symmetric_distance(),
    output_domain = vector_domain(atom_domain(.T = i32)),
    output_metric = symmetric_distance(),
    function_ = function_,
    stability_map = stability_map
  )
}
# /make-repeat


# use-transformation
trans <- (
  new_privacy_context(vector_domain(atom_domain(.T = String)), symmetric_distance()) >>
    then_cast_default(.TOA = i32) >>
    make_repeat(2L) >>
    then_clamp(c(1L, 2L)) >>
    then_sum() >>
    then_laplace(1.)
)
release <- trans(arg = c("0", "1", "2", "3"))
trans(d_in = 1L)
# /use-transformation
