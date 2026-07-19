# partial laplace threshold constructor

See documentation for
[`make_laplace_threshold()`](https://docs.opendp.org/reference/make_laplace_threshold.md)
for details.

## Usage

``` r
then_laplace_threshold(
  lhs,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<MaxDivergence>"
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Noise scale parameter for the laplace distribution. `scale` ==
  standard_deviation / sqrt(2).

- threshold:

  Exclude counts that are less than this minimum value.

- k:

  The noise granularity in terms of 2^k.

- .MO:

  Output Measure.

## Value

Measurement
