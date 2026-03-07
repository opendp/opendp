# noise constructor

Make a Measurement that adds noise from the appropriate distribution to
the input.

## Usage

``` r
make_noise(input_domain, input_metric, output_measure, scale, k = NULL)
```

## Arguments

- input_domain:

  Domain of the data type to be privatized.

- input_metric:

  Metric of the data type to be privatized.

- output_measure:

  Privacy measure. Either `MaxDivergence` or
  `ZeroConcentratedDivergence`.

- scale:

  Noise scale parameter.

- k:

  The noise granularity in terms of 2^k.

## Value

Measurement

## Details

Valid inputs for `input_domain` and `input_metric` are:

|                                 |            |                         |
|---------------------------------|------------|-------------------------|
| `input_domain`                  | input type | `input_metric`          |
| `atom_domain(T)`                | `T`        | `absolute_distance(QI)` |
| `vector_domain(atom_domain(T))` | `Vec<T>`   | `l2_distance(QI)`       |

Required features: `contrib`

[make_noise in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_noise.html)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `DI::Carrier`
