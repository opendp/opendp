# canonical noise constructor

Make a Measurement that adds noise from a canonical noise distribution.
The implementation is tailored towards approximate-DP, resulting in
noise sampled from the Tulap distribution.

## Usage

``` r
make_canonical_noise(input_domain, input_metric, d_in, d_out)
```

## Arguments

- input_domain:

  Domain of the input.

- input_metric:

  Metric of the input.

- d_in:

  Sensitivity

- d_out:

  Privacy parameters (ε, δ)

## Value

Measurement

## Details

Required features: `contrib`

[make_canonical_noise in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_canonical_noise.html)

**Citations:**

- [AV23 Canonical Noise Distributions and Private Hypothesis
  Tests](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Canonical-noise-distributions-and-private-hypothesis-tests/10.1214/23-AOS2259.short)

**Supporting Elements:**

- Input Domain: `AtomDomain<f64>`

- Output Type: `AbsoluteDistance<f64>`

- Input Metric: `Approximate<MaxDivergence>`

- Output Measure: `f64`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/canonical_noise/make_canonical_noise.pdf)
