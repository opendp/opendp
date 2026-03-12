# partial gaussian constructor

See documentation for
[`make_gaussian()`](https://docs.opendp.org/reference/make_gaussian.md)
for details.

## Usage

``` r
then_gaussian(lhs, scale, k = NULL, .MO = "ZeroConcentratedDivergence")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Noise scale parameter for the gaussian distribution. `scale` ==
  standard_deviation.

- k:

  The noise granularity in terms of 2^k.

- .MO:

  Output Measure. The only valid measure is
  `ZeroConcentratedDivergence`.

## Value

Measurement

## Examples

``` r
library(opendp)
enable_features("contrib")
gaussian <- make_gaussian(
  atom_domain(.T = f64, nan = FALSE),
  absolute_distance(.T = f64),
  scale = 1.0)
#> Error in .Call("domains__atom_domain", bounds, nan, .T, rt_parse(.T.bounds),     rt_parse(.T.nan), log_, PACKAGE = "opendp"): "domains__atom_domain" not available for .Call() for package "opendp"
gaussian(arg = 100.0)
#> Error in gaussian(arg = 100): unused argument (arg = 100)

# Or, more readably, define the space and then chain:
space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
#> Error in .Call("domains__atom_domain", bounds, nan, .T, rt_parse(.T.bounds),     rt_parse(.T.nan), log_, PACKAGE = "opendp"): "domains__atom_domain" not available for .Call() for package "opendp"
gaussian <- space |> then_gaussian(scale = 1.0)
#> Error: object 'space' not found
gaussian(arg = 100.0)
#> Error in gaussian(arg = 100): unused argument (arg = 100)

# Sensitivity of this measurement:
gaussian(d_in = 1)
#> Error in gaussian(d_in = 1): unused argument (d_in = 1)
gaussian(d_in = 2)
#> Error in gaussian(d_in = 2): unused argument (d_in = 2)
gaussian(d_in = 4)
#> Error in gaussian(d_in = 4): unused argument (d_in = 4)

# Typically will be used with vectors rather than individual numbers:
space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = i32))
#> Error in .Call("domains__atom_domain", bounds, nan, .T, rt_parse(.T.bounds),     rt_parse(.T.nan), log_, PACKAGE = "opendp"): "domains__atom_domain" not available for .Call() for package "opendp"
gaussian <- space |> then_gaussian(scale = 1.0)
#> Error: object 'space' not found
gaussian(arg = c(10L, 20L, 30L))
#> Error in gaussian(arg = c(10L, 20L, 30L)): unused argument (arg = c(10, 20, 30))
```
