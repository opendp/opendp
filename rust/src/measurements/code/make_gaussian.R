library(opendp)
enable_features("contrib")
gaussian <- make_gaussian(
                          atom_domain(.T = f64),
                          absolute_distance(.T = f64),
                          scale = 1.0)
gaussian(arg = 100.0)
