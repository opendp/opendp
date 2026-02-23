# zCDP to approxDP constructor

Constructs a new output measurement where the output measure is casted
from `ZeroConcentratedDivergence` to `SmoothedMaxDivergence`.

## Usage

``` r
make_zCDP_to_approxDP(measurement)
```

## Arguments

- measurement:

  a measurement with a privacy measure to be casted

## Value

Measurement

## Details

Required features: `contrib`

[make_zCDP_to_approxDP in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_zCDP_to_approxDP.html)
