# count constructor

Make a Transformation that computes a count of the number of records in
data.

## Usage

``` r
make_count(input_domain, input_metric, .TO = "int")
```

## Arguments

- input_domain:

  Domain of the data type to be released.

- input_metric:

  Metric of the data type to be released.

- .TO:

  Output Type. Must be numeric.

## Value

Transformation

## Details

Required features: `contrib`

[make_count in Rust
documentation.](https://docs.rs/opendp/0.15.1/opendp/transformations/fn.make_count.html)

**Citations:**

- [GRS12 Universally Utility-Maximizing Privacy
  Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `AtomDomain<TO>`

- Output Metric: `AbsoluteDistance<TO>`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.15.1/proofs/rust/src/transformations/count/make_count.pdf)
