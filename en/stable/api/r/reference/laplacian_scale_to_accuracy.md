# Convert a Laplacian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

[laplacian_scale_to_accuracy in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/accuracy/fn.laplacian_scale_to_accuracy.html)

## Usage

``` r
laplacian_scale_to_accuracy(scale, alpha, .T = NULL)
```

## Arguments

- scale:

  Laplacian noise scale.

- alpha:

  Statistical significance, level-`alpha`, or (1. - `alpha`)100%
  confidence. Must be within (0, 1\].

- .T:

  Data type of `scale` and `alpha`
