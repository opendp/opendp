# partial randomized response bool constructor

See documentation for
[`make_randomized_response_bool()`](https://docs.opendp.org/reference/make_randomized_response_bool.md)
for details.

## Usage

``` r
then_randomized_response_bool(lhs, prob, constant_time = FALSE)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- prob:

  Probability of returning the correct answer. Must be in `[0.5, 1]`

- constant_time:

  Set to true to enable constant time. Slower.

## Value

Measurement
