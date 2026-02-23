# subset by constructor

Make a Transformation that subsets a dataframe by a boolean column.

## Usage

``` r
make_subset_by(indicator_column, keep_columns, .TK = NULL)
```

## Arguments

- indicator_column:

  name of the boolean column that indicates inclusion in the subset

- keep_columns:

  list of column names to apply subset to

- .TK:

  Type of the column name

## Value

Transformation

## Details

Required features: `contrib`

[make_subset_by in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_subset_by.html)

**Supporting Elements:**

- Input Domain: `DataFrameDomain<TK>`

- Output Domain: `SymmetricDistance`

- Input Metric: `DataFrameDomain<TK>`

- Output Metric: `SymmetricDistance`
