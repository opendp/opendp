# randomized response bitvec constructor

Make a Measurement that implements randomized response on a bit vector.

## Usage

``` r
make_randomized_response_bitvec(
  input_domain,
  input_metric,
  f,
  constant_time = FALSE
)
```

## Arguments

- input_domain:

  BitVectorDomain with max_weight

- input_metric:

  DiscreteDistance

- f:

  Per-bit flipping probability. Must be in \\((0, 1\]\\).

- constant_time:

  Whether to run the Bernoulli samplers in constant time, this is likely
  to be extremely slow.

## Value

Measurement

## Details

This primitive can be useful for implementing RAPPOR.

Required features: `contrib`

[make_randomized_response_bitvec in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_randomized_response_bitvec.html)

**Citations:**

- [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal
  Response](https://arxiv.org/abs/1407.6981)

**Supporting Elements:**

- Input Domain: `BitVectorDomain`

- Output Type: `DiscreteDistance`

- Input Metric: `MaxDivergence`

- Output Measure: `BitVector`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/randomized_response_bitvec/make_randomized_response_bitvec.pdf)
