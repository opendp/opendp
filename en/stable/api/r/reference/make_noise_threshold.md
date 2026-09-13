# noise threshold constructor

Make a Measurement that uses propose-test-release to release a hashmap
of counts.

## Usage

``` r
make_noise_threshold(
  input_domain,
  input_metric,
  output_measure,
  scale,
  threshold,
  k = NULL
)
```

## Arguments

- input_domain:

  Domain of the input.

- input_metric:

  Metric for the input domain.

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

## Details

This function takes a noise granularity in terms of 2^k. Larger
granularities are more computationally efficient, but have a looser
privacy map. If k is not set, k defaults to the smallest granularity.

Required features: `contrib`

[make_noise_threshold in Rust
documentation.](https://docs.rs/opendp/0.15.1/opendp/measurements/fn.make_noise_threshold.html)

**Citations:**

- [Rogers23 A Unifying Privacy Analysis Framework for Unknown Domain
  Algorithms in Differential Privacy](https://arxiv.org/abs/2309.09170)

- [CKS20 The Discrete Gaussian for Differential
  Privacy](https://arxiv.org/abs/2004.00010)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `Approximate<MO>`

- Output Measure: `DI::Carrier`
