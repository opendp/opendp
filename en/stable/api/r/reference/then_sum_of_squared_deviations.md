# partial sum of squared deviations constructor

See documentation for
[`make_sum_of_squared_deviations()`](https://docs.opendp.org/reference/make_sum_of_squared_deviations.md)
for details.

## Usage

``` r
then_sum_of_squared_deviations(lhs, .S = "Pairwise<.T>")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- .S:

  Summation algorithm to use on data type `T`. One of `Sequential<T>` or
  `Pairwise<T>`.

## Value

Transformation
