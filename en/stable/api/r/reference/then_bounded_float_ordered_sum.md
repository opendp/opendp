# partial bounded float ordered sum constructor

See documentation for
[`make_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/make_bounded_float_ordered_sum.md)
for details.

## Usage

``` r
then_bounded_float_ordered_sum(lhs, size_limit, bounds, .S = "Pairwise<.T>")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- size_limit:

  Upper bound on the number of records in input data. Used to bound
  sensitivity.

- bounds:

  Tuple of lower and upper bounds for data in the input domain.

- .S:

  Summation algorithm to use over some data type `T` (`T` is shorthand
  for `S::Item`)

## Value

Transformation
