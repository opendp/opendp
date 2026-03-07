# split records constructor

Make a Transformation that splits each record in a `Vec<String>` into a
`Vec<Vec<String>>`.

## Usage

``` r
make_split_records(separator)
```

## Arguments

- separator:

  The token(s) that separate entries in each record.

## Value

Transformation

## Details

Required features: `contrib`

[make_split_records in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_split_records.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<String>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `VectorDomain<VectorDomain<AtomDomain<String>>>`

- Output Metric: `SymmetricDistance`
