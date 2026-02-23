# split dataframe constructor

Make a Transformation that splits each record in a String into a
`Vec<Vec<String>>`, and loads the resulting table into a dataframe keyed
by `col_names`.

## Usage

``` r
make_split_dataframe(separator, col_names, .K = NULL)
```

## Arguments

- separator:

  The token(s) that separate entries in each record.

- col_names:

  Column names for each record entry.

- .K:

  categorical/hashable data type of column names

## Value

Transformation

## Details

Required features: `contrib`

[make_split_dataframe in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_split_dataframe.html)

**Supporting Elements:**

- Input Domain: `AtomDomain<String>`

- Output Domain: `SymmetricDistance`

- Input Metric: `DataFrameDomain<K>`

- Output Metric: `SymmetricDistance`
