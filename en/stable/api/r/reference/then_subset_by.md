# partial subset by constructor

See documentation for
[`make_subset_by()`](https://docs.opendp.org/reference/make_subset_by.md)
for details.

## Usage

``` r
then_subset_by(lhs, indicator_column, keep_columns, .TK = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- indicator_column:

  name of the boolean column that indicates inclusion in the subset

- keep_columns:

  list of column names to apply subset to

- .TK:

  Type of the column name

## Value

Transformation
