# count distinct constructor

Make a Transformation that computes a count of the number of unique,
distinct records in data.

## Usage

``` r
make_count_distinct(input_domain, input_metric, .TO = "int")
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- .TO:

  Output Type. Must be numeric.

## Value

Transformation

## Details

Required features: `contrib`

[make_count_distinct in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_count_distinct.html)

**Citations:**

- [GRS12 Universally Utility-Maximizing Privacy
  Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `AtomDomain<TO>`

- Output Metric: `AbsoluteDistance<TO>`
