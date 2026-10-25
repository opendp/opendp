# partial laplace constructor

See documentation for
[`make_laplace()`](https://docs.opendp.org/reference/make_laplace.md)
for details.

## Usage

``` r
then_laplace(lhs, scale, k = NULL, .MO = "MaxDivergence")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Noise scale parameter for the Laplace distribution. `scale` ==
  standard_deviation / sqrt(2).

- k:

  The noise granularity in terms of 2^k, only valid for domains over
  floats.

- .MO:

  Measure used to quantify privacy loss. Valid values are just
  `MaxDivergence`

## Value

Measurement
