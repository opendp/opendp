# Convert a `make_` constructor into a `then_` constructor.

This mirrors the Python extras utility for constructors whose first two
arguments are `input_domain` and `input_metric`.

## Usage

``` r
to_then(constructor)
```

## Arguments

- constructor:

  A constructor whose first two arguments are `input_domain` and
  `input_metric`.

## Value

A partial constructor suitable for `|>` chaining.
