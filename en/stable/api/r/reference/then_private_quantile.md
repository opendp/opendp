# partial private quantile constructor

See documentation for
[`make_private_quantile()`](https://docs.opendp.org/reference/make_private_quantile.md)
for details.

## Usage

``` r
then_private_quantile(lhs, output_measure, candidates, alpha, scale)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  Either MaxDivergence or ZeroConcentratedDivergence.

- candidates:

  Potential quantiles to score

- alpha:

  a value in \\(\[0, 1\]\\). Choose 0.5 for median

- scale:

  the scale of the noise added

## Value

Measurement
