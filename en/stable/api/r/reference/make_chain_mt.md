# chain mt constructor

Construct the functional composition (`measurement1` ○
`transformation0`). Returns a Measurement that when invoked, computes
`measurement1(transformation0(x))`.

## Usage

``` r
make_chain_mt(measurement1, transformation0)
```

## Arguments

- measurement1:

  outer mechanism

- transformation0:

  inner transformation

## Value

Measurement

## Details

Required features: `contrib`

[make_chain_mt in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_chain_mt.html)
