# Convert a desired `accuracy` (tolerance) into a discrete gaussian noise scale at a statistical significance level `alpha`.

[accuracy_to_discrete_gaussian_scale in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/accuracy/fn.accuracy_to_discrete_gaussian_scale.html)

## Usage

``` r
accuracy_to_discrete_gaussian_scale(accuracy, alpha, .T = NULL)
```

## Arguments

- accuracy:

  Desired accuracy. A tolerance for how far values may diverge from the
  input to the mechanism.

- alpha:

  Statistical significance, level-`alpha`, or (1. - `alpha`)100%
  confidence. Must be within (0, 1\].

- .T:

  Data type of `accuracy` and `alpha`

## Details

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/accuracy/accuracy_to_discrete_gaussian_scale.pdf)
