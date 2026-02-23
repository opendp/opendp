# noisy top k constructor

Make a Measurement that takes a vector of scores and privately selects
the index of the highest score.

## Usage

``` r
make_noisy_top_k(
  input_domain,
  input_metric,
  output_measure,
  k,
  scale,
  negate = FALSE
)
```

## Arguments

- input_domain:

  Domain of the input vector. Must be a non-nullable VectorDomain.

- input_metric:

  Metric on the input domain. Must be LInfDistance

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

## Details

Required features: `contrib`

[make_noisy_top_k in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_noisy_top_k.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Type: `LInfDistance<TIA>`

- Input Metric: `MO`

- Output Measure: `Vec<usize>`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/noisy_top_k/make_noisy_top_k.pdf)
