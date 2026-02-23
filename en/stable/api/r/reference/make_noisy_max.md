# noisy max constructor

Make a Measurement that takes a vector of scores and privately selects
the index of the highest score.

## Usage

``` r
make_noisy_max(
  input_domain,
  input_metric,
  output_measure,
  scale,
  negate = FALSE
)
```

## Arguments

- input_domain:

  Domain of the input vector. Must be a non-nullable `VectorDomain`

- input_metric:

  Metric on the input domain. Must be `LInfDistance`

- output_measure:

  One of `MaxDivergence`, `ZeroConcentratedDivergence`

- scale:

  Scale for the noise distribution

- negate:

  Set to true to return min

## Value

Measurement

## Details

Required features: `contrib`

[make_noisy_max in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_noisy_max.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Type: `LInfDistance<TIA>`

- Input Metric: `MO`

- Output Measure: `usize`
