# quantiles from counts constructor

Postprocess a noisy array of summary counts into quantiles.

## Usage

``` r
make_quantiles_from_counts(
  bin_edges,
  alphas,
  interpolation = "linear",
  .TA = NULL,
  .F = "float"
)
```

## Arguments

- bin_edges:

  The edges that the input data was binned into before counting.

- alphas:

  Return all specified `alpha`-quantiles.

- interpolation:

  Must be one of `linear` or `nearest`

- .TA:

  Atomic Type of the bin edges and data.

- .F:

  Float type of the alpha argument. One of `f32` or `f64`

## Value

Function

## Details

Required features: `contrib`

[make_quantiles_from_counts in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_quantiles_from_counts.html)

**Supporting Elements:**

- Input Type: `Vec<TA>`

- Output Type: `Vec<TA>`
