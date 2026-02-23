# quantile score candidates constructor

Makes a Transformation that scores how similar each candidate is to the
given `alpha`-quantile on the input dataset.

## Usage

``` r
make_quantile_score_candidates(input_domain, input_metric, candidates, alpha)
```

## Arguments

- input_domain:

  Uses a smaller sensitivity when the size of vectors in the input
  domain is known.

- input_metric:

  Either SymmetricDistance or InsertDeleteDistance.

- candidates:

  Potential quantiles to score

- alpha:

  a value in \\(\[0, 1\]\\). Choose 0.5 for median

## Value

Transformation

## Details

Required features: `contrib`

[make_quantile_score_candidates in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_quantile_score_candidates.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `MI`

- Input Metric: `VectorDomain<AtomDomain<u64>>`

- Output Metric: `LInfDistance<u64>`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/transformations/quantile_score_candidates/make_quantile_score_candidates.pdf)
