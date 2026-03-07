# create dataframe constructor

Make a Transformation that constructs a dataframe from a
`Vec<Vec<String>>` (a vector of records).

## Usage

``` r
make_create_dataframe(col_names, .K = NULL)
```

## Arguments

- col_names:

  Column names for each record entry.

- .K:

  categorical/hashable data type of column names

## Value

Transformation

## Details

Required features: `contrib`

[make_create_dataframe in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_create_dataframe.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<VectorDomain<AtomDomain<String>>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `DataFrameDomain<K>`

- Output Metric: `SymmetricDistance`
