# partial sized bounded float ordered sum constructor

See documentation for
[`make_sized_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/make_sized_bounded_float_ordered_sum.md)
for details.

## Usage

``` r
then_sized_bounded_float_ordered_sum(lhs, size, bounds, .S = "Pairwise<.T>")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- size:

  Number of records in input data.

- bounds:

  Tuple of lower and upper bounds for data in the input domain.

- .S:

  Summation algorithm to use over some data type `T` (`T` is shorthand
  for `S::Item`)

## Value

Transformation
