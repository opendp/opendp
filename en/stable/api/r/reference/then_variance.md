# partial variance constructor

See documentation for
[`make_variance()`](https://docs.opendp.org/reference/make_variance.md)
for details.

## Usage

``` r
then_variance(lhs, ddof = 1L, .S = "Pairwise<.T>")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- ddof:

  Delta degrees of freedom. Set to 0 if not a sample, 1 for sample
  estimate.

- .S:

  Summation algorithm to use on data type `T`. One of `Sequential<T>` or
  `Pairwise<T>`.

## Value

Transformation
