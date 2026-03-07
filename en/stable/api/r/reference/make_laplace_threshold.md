# laplace threshold constructor

Make a Measurement that uses propose-test-release to privatize a hashmap
of counts.

## Usage

``` r
make_laplace_threshold(
  input_domain,
  input_metric,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<MaxDivergence>"
)
```

## Arguments

- input_domain:

  Domain of the input.

- input_metric:

  Metric for the input domain.

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

## Details

This function takes a noise granularity in terms of 2^k. Larger
granularities are more computationally efficient, but have a looser
privacy map. If k is not set, k defaults to the smallest granularity.

Required features: `contrib`

[make_laplace_threshold in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_laplace_threshold.html)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `DI::Carrier`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/noise_threshold/distribution/laplace/make_laplace_threshold.pdf)
