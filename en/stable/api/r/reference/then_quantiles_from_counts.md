# partial quantiles from counts constructor

See documentation for
[`make_quantiles_from_counts()`](https://docs.opendp.org/reference/make_quantiles_from_counts.md)
for details.

## Usage

``` r
then_quantiles_from_counts(
  lhs,
  bin_edges,
  alphas,
  interpolation = "linear",
  .TA = NULL,
  .F = "float"
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- bin_edges:

  The edges that the input data was binned into before counting.

- alphas:

  Return all specified `alpha`-quantiles.

- interpolation:

  Must be one of `linear` or `nearest`

- .TA:

  Atomic Type of the bin edges and data.

- .F:

  Float type of the alpha argument. One of `f32` or `f64`

## Value

Function
