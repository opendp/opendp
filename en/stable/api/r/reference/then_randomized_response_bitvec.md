# partial randomized response bitvec constructor

See documentation for
[`make_randomized_response_bitvec()`](https://docs.opendp.org/reference/make_randomized_response_bitvec.md)
for details.

## Usage

``` r
then_randomized_response_bitvec(lhs, f, constant_time = FALSE)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- f:

  Per-bit flipping probability. Must be in \\((0, 1\]\\).

- constant_time:

  Whether to run the Bernoulli samplers in constant time, this is likely
  to be extremely slow.

## Value

Measurement
