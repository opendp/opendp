library(opendp)
enable_features("contrib")
gaussian <- make_gaussian(
  atom_domain(.T = f64),
  absolute_distance(.T = f64),
  scale = 1.0)
gaussian(arg = 100.0)

# Or, more readably, define the space and then chain:
space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
gaussian <- space |> then_gaussian(scale = 1.0)
gaussian(arg = 100.0)

# Sensitivity of this measurement:
gaussian(d_in = 1)
gaussian(d_in = 2)
gaussian(d_in = 4)

# Typically will be used with vectors rather than individual numbers:
space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = i32))
gaussian <- space |> then_gaussian(scale = 1.0)
gaussian(arg = c(10L, 20L, 30L))