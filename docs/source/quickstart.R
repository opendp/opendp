# 1-library
library(opendp)
enable_features("contrib")
# 2-use
space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
laplace_mechanism <- space |> then_laplace(scale = 1.)
dp_agg <- laplace_mechanism(arg = 23.4)