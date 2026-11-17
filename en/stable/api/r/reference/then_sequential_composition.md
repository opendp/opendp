# partial sequential composition constructor

See documentation for
[`make_sequential_composition()`](https://docs.opendp.org/reference/make_sequential_composition.md)
for details.

## Usage

``` r
then_sequential_composition(lhs, output_measure, d_in, d_mids)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  how privacy is measured

- d_in:

  maximum distance between adjacent input datasets

- d_mids:

  maximum privacy expenditure of each query

## Value

Measurement
