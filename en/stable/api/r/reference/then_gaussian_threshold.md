# partial gaussian threshold constructor

See documentation for
[`make_gaussian_threshold()`](https://docs.opendp.org/reference/make_gaussian_threshold.md)
for details.

## Usage

``` r
then_gaussian_threshold(
  lhs,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<ZeroConcentratedDivergence>"
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Noise scale parameter for the laplace distribution. `scale` ==
  standard_deviation / sqrt(2).

- threshold:

  Exclude pairs with values whose distance from zero exceeds this value.

- k:

  The noise granularity in terms of 2^k.

- .MO:

  Output Measure.

## Value

Measurement
