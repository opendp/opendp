# sequential composition constructor

Construct a Measurement that when invoked, returns a queryable that
interactively composes measurements.

## Usage

``` r
make_sequential_composition(
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

Required features: `contrib`

[make_sequential_composition in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_sequential_composition.html)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `Queryable<Measurement<DI, MI, MO, TO>, TO>`
