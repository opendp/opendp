# Find the closest passing value to the decision boundary of `predicate`

Missing bounds are inferred. Use `NULL` for either element of `bounds`
to conduct a one-sided search, or omit `bounds` to conduct an
exponential search.

## Usage

``` r
binary_search(predicate, bounds = NULL, .T = NULL, return_sign = FALSE)
```

## Arguments

- predicate:

  a monotonic unary function from a number to a boolean

- bounds:

  a two-element list of optional lower and upper bounds on the input of
  `predicate`; use `NULL` for either bound to infer it

- .T:

  type of argument to `predicate`, one of float or int; when set, takes
  precedence over the type inferred from `bounds`

- return_sign:

  if True, also return the direction away from the decision boundary

## Value

the discovered parameter within the bounds

## Examples

``` r
binary_search(\(x) x <= -5L, bounds = list(-10L, NULL))
#> Error: "internal___binary_search" not available for .Call() for package "opendp"
```
