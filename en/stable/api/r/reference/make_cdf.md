# cdf constructor

Postprocess a noisy array of float summary counts into a cumulative
distribution.

## Usage

``` r
make_cdf(.TA = "float")
```

## Arguments

- .TA:

  Atomic Type. One of `f32` or `f64`

## Value

Function

## Details

Required features: `contrib`

[make_cdf in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_cdf.html)

**Supporting Elements:**

- Input Type: `Vec<TA>`

- Output Type: `Vec<TA>`
