# split lines constructor

Make a Transformation that takes a string and splits it into a
`Vec<String>` of its lines.

## Usage

``` r
make_split_lines()
```

## Value

Transformation

## Details

Required features: `contrib`

[make_split_lines in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_split_lines.html)

**Supporting Elements:**

- Input Domain: `AtomDomain<String>`

- Output Domain: `SymmetricDistance`

- Input Metric: `VectorDomain<AtomDomain<String>>`

- Output Metric: `SymmetricDistance`
