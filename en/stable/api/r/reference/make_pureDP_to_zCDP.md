# pureDP to zCDP constructor

Constructs a new output measurement where the output measure is casted
from `MaxDivergence` to `ZeroConcentratedDivergence`.

## Usage

``` r
make_pureDP_to_zCDP(measurement)
```

## Arguments

- measurement:

  a measurement with a privacy measure to be casted

## Value

Measurement

## Details

Required features: `contrib`

[make_pureDP_to_zCDP in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_pureDP_to_zCDP.html)

**Citations:**

- [BS16 Concentrated Differential Privacy: Simplifications, Extensions,
  and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
