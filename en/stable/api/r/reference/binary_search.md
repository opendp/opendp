# Find the closest passing value to the decision boundary of `predicate`

If bounds are not passed, conducts an exponential search.

## Usage

``` r
binary_search(predicate, bounds = NULL, .T = NULL, return_sign = FALSE)
```

## Arguments

- predicate:

  a monotonic unary function from a number to a boolean

- bounds:

  a 2-tuple of the lower and upper bounds on the input of `make_chain`

- .T:

  type of argument to `predicate`, one of float or int

- return_sign:

  if True, also return the direction away from the decision boundary

## Value

the discovered parameter within the bounds
