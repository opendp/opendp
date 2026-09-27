# adaptive composition constructor

Construct a Measurement that when invoked, returns a queryable that
interactively composes measurements.

## Usage

``` r
make_adaptive_composition(
  input_domain,
  input_metric,
  output_measure,
  d_in,
  d_mids
)
```

## Arguments

- input_domain:

  indicates the space of valid input datasets

- input_metric:

  how distances are measured between members of the input domain

- output_measure:

  how privacy is measured

- d_in:

  maximum distance between adjacent input datasets

- d_mids:

  maximum privacy expenditure of each query

## Value

Measurement

## Details

Required features: `contrib`

[make_adaptive_composition in Rust
documentation.](https://docs.rs/opendp/0.15.1/opendp/combinators/fn.make_adaptive_composition.html)

**Citations:**

- [LW22 Composition Theorems for Interactive Differential
  Privacy](https://arxiv.org/abs/2207.09397)

- [VW21 Concurrent Composition of Differential
  Privacy](https://arxiv.org/abs/2105.14427)

- [HSTVVXZ23 Concurrent Composition for Interactive Differential Privacy
  with Adaptive Privacy-Loss
  Parameters](https://arxiv.org/abs/2309.05901)

**Composition Properties**

- sequential: all measurements are applied to the same dataset

- basic: the composition is the linear sum of the privacy usage of each
  query

- interactive: mechanisms can be specified based on answers to previous
  queries

- compositor: all privacy parameters specified up-front

If the privacy measure supports concurrency, this compositor allows you
to spawn multiple interactive mechanisms and interleave your queries
amongst them.

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `Queryable<Measurement<DI, MI, MO, TO>, TO>`
