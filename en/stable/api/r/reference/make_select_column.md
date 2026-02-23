# select column constructor

Make a Transformation that retrieves the column `key` from a dataframe
as `Vec<TOA>`.

## Usage

``` r
make_select_column(key, .TOA, .K = NULL)
```

## Arguments

- key:

  categorical/hashable data type of the key/column name

- .TOA:

  Atomic Output Type to downcast vector to

- .K:

  data type of key

## Value

Transformation

## Details

Required features: `contrib`

[make_select_column in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_select_column.html)

**Supporting Elements:**

- Input Domain: `DataFrameDomain<K>`

- Output Domain: `SymmetricDistance`

- Input Metric: `VectorDomain<AtomDomain<TOA>>`

- Output Metric: `SymmetricDistance`
