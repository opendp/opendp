# partial alp queryable constructor

See documentation for
[`make_alp_queryable()`](https://docs.opendp.org/reference/make_alp_queryable.md)
for details.

## Usage

``` r
then_alp_queryable(
  lhs,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Privacy loss parameter. This is equal to epsilon/sensitivity.

- total_limit:

  Either the true value or an upper bound estimate of the sum of all
  values in the input.

- value_limit:

  Upper bound on individual values (referred to as β). Entries above β
  are clamped.

- size_factor:

  Optional multiplier (default of 50) for setting the size of the
  projection.

- alpha:

  Optional parameter (default of 4) for scaling and determining p in
  randomized response step.

## Value

Measurement
