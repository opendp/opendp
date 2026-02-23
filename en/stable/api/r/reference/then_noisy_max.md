# partial noisy max constructor

See documentation for
[`make_noisy_max()`](https://docs.opendp.org/reference/make_noisy_max.md)
for details.

## Usage

``` r
then_noisy_max(lhs, output_measure, scale, negate = FALSE)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  One of `MaxDivergence`, `ZeroConcentratedDivergence`

- scale:

  Scale for the noise distribution

- negate:

  Set to true to return min

## Value

Measurement
