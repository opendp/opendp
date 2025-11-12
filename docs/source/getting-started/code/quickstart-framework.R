# init
library(opendp)
enable_features("contrib")
# /init

# demo
space <- c(atom_domain(.T = "i32"), absolute_distance(.T = "i32"))
laplace_mechanism <- space |> then_laplace(1.)
dp_value <- laplace_mechanism(arg = 123L)
# /demo
