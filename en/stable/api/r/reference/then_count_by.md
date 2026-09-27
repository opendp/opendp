# partial count by constructor

See documentation for
[`make_count_by()`](https://docs.opendp.org/reference/make_count_by.md)
for details.

## Usage

``` r
then_count_by(lhs, .TV = "int")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- .TV:

  Type of Value. Express counts in terms of this integral type.

## Value

The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for
each unique data input (`TK`).
