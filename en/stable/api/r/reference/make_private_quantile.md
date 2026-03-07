# private quantile constructor

Makes a Measurement the computes the quantile of a dataset.

## Usage

``` r
make_private_quantile(
  input_domain,
  input_metric,
  output_measure,
  candidates,
  alpha,
  scale
)
```

## Arguments

- input_domain:

  Uses a tighter sensitivity when the size of vectors in the input
  domain is known.

- input_metric:

  Either SymmetricDistance or InsertDeleteDistance.

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

## Details

Required features: `contrib`

[make_private_quantile in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_private_quantile.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `T`
