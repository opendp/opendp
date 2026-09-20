# composition constructor

Construct the DP composition \[`measurement0`, `measurement1`, ...\].
Returns a Measurement that when invoked, computes
`[measurement0(x), measurement1(x), ...]`

## Usage

``` r
make_composition(measurements)
```

## Arguments

- measurements:

  A vector of Measurements to compose.

## Value

Measurement

## Details

All metrics and domains must be equivalent.

**Composition Properties**

- sequential: all measurements are applied to the same dataset

- basic: the composition is the linear sum of the privacy usage of each
  query

- noninteractive: all mechanisms specified up-front (but each can be
  interactive)

- compositor: all privacy parameters specified up-front (via the map)

Required features: `contrib`
