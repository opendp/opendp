# chain pm constructor

Construct the functional composition (`postprocess1` ○ `measurement0`).
Returns a Measurement that when invoked, computes
`postprocess1(measurement0(x))`. Used to represent non-interactive
postprocessing.

## Usage

``` r
make_chain_pm(postprocess1, measurement0)
```

## Arguments

- postprocess1:

  outer postprocessor

- measurement0:

  inner measurement/mechanism

## Value

Measurement

## Details

Required features: `contrib`

[make_chain_pm in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_chain_pm.html)
