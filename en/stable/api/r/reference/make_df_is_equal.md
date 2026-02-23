# df is equal constructor

Make a Transformation that checks if each element in a column in a
dataframe is equivalent to `value`.

## Usage

``` r
make_df_is_equal(input_domain, input_metric, column_name, value, .TIA = NULL)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- column_name:

  Column name to be transformed

- value:

  Value to check for equality

- .TIA:

  Atomic Input Type to cast from

## Value

Transformation

## Details

Required features: `contrib`

[make_df_is_equal in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_df_is_equal.html)

**Supporting Elements:**

- Input Domain: `DataFrameDomain<TK>`

- Output Domain: `M`

- Input Metric: `DataFrameDomain<TK>`

- Output Metric: `M`
