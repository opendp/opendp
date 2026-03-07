# Convert a discrete Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

\\(\alpha = P\[Y \ge accuracy\]\\), where \\(Y = \| X - z \|\\), and
\\(X \sim \mathcal{L}\_{Z}(0, scale)\\). That is, \\(X\\) is a discrete
Laplace random variable and \\(Y\\) is the distribution of the errors.

## Usage

``` r
discrete_laplacian_scale_to_accuracy(scale, alpha, .T = NULL)
```

## Arguments

- scale:

  Discrete Laplacian noise scale.

- alpha:

  Statistical significance, level-`alpha`, or (1. - `alpha`)100%
  confidence. Must be within (0, 1\].

- .T:

  Data type of `scale` and `alpha`

## Details

This function returns a float accuracy. You can take the floor without
affecting the coverage probability.

[discrete_laplacian_scale_to_accuracy in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/accuracy/fn.discrete_laplacian_scale_to_accuracy.html)

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/accuracy/discrete_laplacian_scale_to_accuracy.pdf)
