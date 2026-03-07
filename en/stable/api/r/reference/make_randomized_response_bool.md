# randomized response bool constructor

Make a Measurement that implements randomized response on a boolean
value.

## Usage

``` r
make_randomized_response_bool(prob, constant_time = FALSE)
```

## Arguments

- prob:

  Probability of returning the correct answer. Must be in `[0.5, 1]`

- constant_time:

  Set to true to enable constant time. Slower.

## Value

Measurement

## Details

Required features: `contrib`

[make_randomized_response_bool in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_randomized_response_bool.html)

**Supporting Elements:**

- Input Domain: `AtomDomain<bool>`

- Output Type: `DiscreteDistance`

- Input Metric: `MaxDivergence`

- Output Measure: `bool`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/randomized_response/make_randomized_response_bool.pdf)
