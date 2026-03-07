# resize constructor

Make a Transformation that either truncates or imputes records with
`constant` to match a provided `size`.

## Usage

``` r
make_resize(
  input_domain,
  input_metric,
  size,
  constant,
  .MO = "SymmetricDistance"
)
```

## Arguments

- input_domain:

  Domain of input data.

- input_metric:

  Metric of input data.

- size:

  Number of records in output data.

- constant:

  Value to impute with.

- .MO:

  Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`

## Value

A vector of the same type `TA`, but with the provided `size`.

## Details

Required features: `contrib`

[make_resize in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_resize.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TA>>`

- Output Domain: `MI`

- Input Metric: `VectorDomain<AtomDomain<TA>>`

- Output Metric: `MO`
