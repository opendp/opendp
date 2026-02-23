# partial noisy top k constructor

See documentation for
[`make_noisy_top_k()`](https://docs.opendp.org/reference/make_noisy_top_k.md)
for details.

## Usage

``` r
then_noisy_top_k(lhs, output_measure, k, scale, negate = FALSE)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  One of `MaxDivergence` or `ZeroConcentratedDivergence`

- k:

  Number of indices to select.

- scale:

  Scale for the noise distribution.

- negate:

  Set to true to return bottom k

## Value

Measurement
