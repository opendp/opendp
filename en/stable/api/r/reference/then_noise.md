# partial noise constructor

See documentation for
[`make_noise()`](https://docs.opendp.org/reference/make_noise.md) for
details.

## Usage

``` r
then_noise(lhs, output_measure, scale, k = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  Privacy measure. Either `MaxDivergence` or
  `ZeroConcentratedDivergence`.

- scale:

  Noise scale parameter.

- k:

  The noise granularity in terms of 2^k.

## Value

Measurement
