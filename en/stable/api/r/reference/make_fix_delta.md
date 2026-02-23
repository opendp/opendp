# fix delta constructor

Fix the delta parameter in the privacy map of a `measurement` with a
SmoothedMaxDivergence output measure.

## Usage

``` r
make_fix_delta(measurement, delta)
```

## Arguments

- measurement:

  a measurement with a privacy curve to be fixed

- delta:

  parameter to fix the privacy curve with

## Value

Measurement

## Details

Required features: `contrib`

[make_fix_delta in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_fix_delta.html)
