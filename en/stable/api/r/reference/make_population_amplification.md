# population amplification constructor

Construct an amplified measurement from a `measurement` with privacy
amplification by subsampling. This measurement does not perform any
sampling. It is useful when you have a dataset on-hand that is a simple
random sample from a larger population.

## Usage

``` r
make_population_amplification(measurement, population_size)
```

## Arguments

- measurement:

  the computation to amplify

- population_size:

  the size of the population from which the input dataset is a simple
  sample

## Value

Measurement

## Details

The DIA, DO, MI and MO between the input measurement and amplified
output measurement all match.

Required features: `contrib`, `honest-but-curious`

[make_population_amplification in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_population_amplification.html)

**Why honest-but-curious?:**

The privacy guarantees are only valid if the input dataset is a simple
sample from a population with `population_size` records.
