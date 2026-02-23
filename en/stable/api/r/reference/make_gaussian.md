# gaussian constructor

Make a Measurement that adds noise from the Gaussian(`scale`)
distribution to the input.

## Usage

``` r
make_gaussian(
  input_domain,
  input_metric,
  scale,
  k = NULL,
  .MO = "ZeroConcentratedDivergence"
)
```

## Arguments

- input_domain:

  Domain of the data type to be privatized.

- input_metric:

  Metric of the data type to be privatized.

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

## Details

Valid inputs for `input_domain` and `input_metric` are:

|                                 |            |                         |
|---------------------------------|------------|-------------------------|
| `input_domain`                  | input type | `input_metric`          |
| `atom_domain(T)`                | `T`        | `absolute_distance(QI)` |
| `vector_domain(atom_domain(T))` | `Vec<T>`   | `l2_distance(QI)`       |

Required features: `contrib`

[make_gaussian in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_gaussian.html)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `DI::Carrier`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/noise/distribution/gaussian/make_gaussian.pdf)

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
