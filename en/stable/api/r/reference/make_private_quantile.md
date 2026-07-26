# private quantile constructor

Make a Measurement that computes the quantile of a dataset.

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
documentation.](https://docs.rs/opendp/0.15.1/opendp/measurements/fn.make_private_quantile.html)

**Citations:**

- [Smith11 Privacy-Preserving Statistical Estimation with Optimal
  Convergence Rates](https://doi.org/10.1145/1993636.1993743)

- [MS20 Permute-and-Flip: A New Mechanism for Differentially Private
  Selection](https://arxiv.org/abs/2010.12603)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `T`
