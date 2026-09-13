# partial count by categories constructor

See documentation for
[`make_count_by_categories()`](https://docs.opendp.org/reference/make_count_by_categories.md)
for details.

## Usage

``` r
then_count_by_categories(
  lhs,
  categories,
  null_category = TRUE,
  .MO = "L1Distance<int>",
  .TOA = "int"
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- categories:

  The set of categories to compute counts for.

- null_category:

  Include a count of the number of elements that were not in the
  category set at the end of the vector.

- .MO:

  Output Metric.

- .TOA:

  Atomic Output Type that is numeric.

## Value

The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).
