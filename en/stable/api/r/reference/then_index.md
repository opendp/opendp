# partial index constructor

See documentation for
[`make_index()`](https://docs.opendp.org/reference/make_index.md) for
details.

## Usage

``` r
then_index(lhs, categories, null, .TOA = NULL)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- categories:

  The set of categories to index into.

- null:

  Category to return if the index is out-of-range of the category set.

- .TOA:

  Atomic Output Type. Output data will be `Vec<TOA>`.

## Value

Transformation
