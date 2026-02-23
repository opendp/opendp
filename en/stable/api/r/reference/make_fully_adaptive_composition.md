# fully adaptive composition constructor

Construct an odometer that can spawn a compositor queryable.

## Usage

``` r
make_fully_adaptive_composition(input_domain, input_metric, output_measure)
```

## Arguments

- input_domain:

  indicates the space of valid input datasets

- input_metric:

  how distances are measured between members of the input domain

- output_measure:

  how privacy is measured

## Value

Odometer

## Details

Required features: `contrib`

[make_fully_adaptive_composition in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_fully_adaptive_composition.html)

**Supporting Elements:**

- Input Domain `DI`

- Input Metric `MI`

- Output Measure `MO`

- Query `Measurement<DI, MI, MO, TO>`

- Answer `TO`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/combinators/sequential_composition/fully_adaptive/make_fully_adaptive_composition.pdf)
