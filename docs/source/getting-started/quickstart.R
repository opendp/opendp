# init
library(opendp)
enable_features("contrib")

# /init

# demo
space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
base_laplace <- space |> then_base_laplace(1.)
dp_value <- base_laplace(arg = 123.0)

# /demo