# init
library(opendp)
enable_features("contrib")
# /init

# demo
space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
laplace_mechanism <- space |> then_laplace(1.)
dp_value <- laplace_mechanism(arg = 123.0)
# /demo
