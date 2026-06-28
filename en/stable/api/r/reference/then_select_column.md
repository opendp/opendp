# partial select column constructor

See documentation for
[`make_select_column()`](https://docs.opendp.org/reference/make_select_column.md)
for details.

## Usage

``` r
then_select_column(lhs, key, .TOA, .K = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- key:

  categorical/hashable data type of the key/column name

- .TOA:

  Atomic Output Type to downcast vector to

- .K:

  data type of key

## Value

Transformation
