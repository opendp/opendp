# chain tt constructor

Construct the functional composition (`transformation1` ○
`transformation0`). Returns a Transformation that when invoked, computes
`transformation1(transformation0(x))`.

## Usage

``` r
make_chain_tt(transformation1, transformation0)
```

## Arguments

- transformation1:

  outer transformation

- transformation0:

  inner transformation

## Value

Transformation

## Details

Required features: `contrib`

[make_chain_tt in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_chain_tt.html)
