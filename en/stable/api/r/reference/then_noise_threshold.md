# partial noise threshold constructor

See documentation for
[`make_noise_threshold()`](https://docs.opendp.org/reference/make_noise_threshold.md)
for details.

## Usage

``` r
then_noise_threshold(lhs, output_measure, scale, threshold, k = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  Privacy measure. Either `MaxDivergence` or
  `ZeroConcentratedDivergence`.

- scale:

  Noise scale parameter for the laplace distribution. `scale` ==
  standard_deviation / sqrt(2).

- threshold:

  Exclude counts that are less than this minimum value.

- k:

  The noise granularity in terms of 2^k.

## Value

Measurement
