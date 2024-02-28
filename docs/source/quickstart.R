# 1-library
library(opendp)
enable_features("contrib")
# 2-use
space <- c(atom_domain(.T = "f64"), absolute_distance(.T = "f64"))
m_laplace <- space |> then_laplace(1.)
dp_agg <- m_laplace(arg = 23.4)