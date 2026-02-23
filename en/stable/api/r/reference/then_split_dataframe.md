# partial split dataframe constructor

See documentation for
[`make_split_dataframe()`](https://docs.opendp.org/reference/make_split_dataframe.md)
for details.

## Usage

``` r
then_split_dataframe(lhs, separator, col_names, .K = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- separator:

  The token(s) that separate entries in each record.

- col_names:

  Column names for each record entry.

- .K:

  categorical/hashable data type of column names

## Value

Transformation
