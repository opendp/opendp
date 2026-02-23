# fixed approxDP to approxDP constructor

Constructs a new output measurement where the output measure is casted
from `Approximate<MaxDivergence>` to `SmoothedMaxDivergence`.

## Usage

``` r
make_fixed_approxDP_to_approxDP(measurement)
```

## Arguments

- measurement:

  a measurement with a privacy measure to be casted

## Value

Measurement

## Details

Required features: `contrib`

[make_fixed_approxDP_to_approxDP in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_fixed_approxDP_to_approxDP.html)
