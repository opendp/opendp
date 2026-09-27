# Solve for the ideal constructor argument to `make_chain`

Searches for the numeric parameter to `make_chain` that results in a
computation that most tightly satisfies `d_out` when datasets differ by
at most `d_in`.

## Usage

``` r
binary_search_param(make_chain, d_in, d_out, bounds = NULL, .T = NULL)
```

## Arguments

- make_chain:

  a function that takes a number and returns a Transformation or
  Measurement

- d_in:

  how far apart input datasets can be

- d_out:

  how far apart output datasets or distributions can be

- bounds:

  a 2-tuple of the lower and upper bounds on the input of `make_chain`

- .T:

  type of argument to `make_chain`, either "float" or "int"

## Value

the parameter to `make_chain` that results in a (`d_in`, `d_out`)-close
Transformation or Measurement
