# Convert a gaussian scale into an accuracy estimate (tolerance) at a statistical significance level `alpha`.

[gaussian_scale_to_accuracy in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/accuracy/fn.gaussian_scale_to_accuracy.html)

## Usage

``` r
gaussian_scale_to_accuracy(scale, alpha, .T = NULL)
```

## Arguments

- scale:

  Gaussian noise scale.

- alpha:

  Statistical significance, level-`alpha`, or (1. - `alpha`)100%
  confidence. Must be within (0, 1\].

- .T:

  Data type of `scale` and `alpha`
