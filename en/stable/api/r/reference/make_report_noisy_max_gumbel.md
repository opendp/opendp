# report noisy max gumbel constructor

Make a Measurement that takes a vector of scores and privately selects
the index of the highest score.

## Usage

``` r
make_report_noisy_max_gumbel(
  input_domain,
  input_metric,
  scale,
  optimize = "max"
)
```

## Arguments

- input_domain:

  Domain of the input vector. Must be a non-nullable `VectorDomain`

- input_metric:

  Metric on the input domain. Must be `LInfDistance`

- scale:

  Scale for the noise distribution

- optimize:

  Set to "min" to report noisy min

## Value

Measurement

## Details

Required features: `contrib`

[make_report_noisy_max_gumbel in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_report_noisy_max_gumbel.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Type: `LInfDistance<TIA>`

- Input Metric: `MaxDivergence`

- Output Measure: `usize`
