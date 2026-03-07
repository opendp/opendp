# laplace constructor

Make a Measurement that adds noise from the Laplace(`scale`)
distribution to the input.

## Usage

``` r
make_laplace(
  input_domain,
  input_metric,
  scale,
  k = NULL,
  .MO = "MaxDivergence"
)
```

## Arguments

- input_domain:

  Domain of the data type to be privatized.

- input_metric:

  Metric of the data type to be privatized.

- scale:

  Noise scale parameter for the Laplace distribution. `scale` ==
  standard_deviation / sqrt(2).

- k:

  The noise granularity in terms of 2^k, only valid for domains over
  floats.

- .MO:

  Measure used to quantify privacy loss. Valid values are just
  `MaxDivergence`

## Value

Measurement

## Details

Valid inputs for `input_domain` and `input_metric` are:

|                                 |            |                        |
|---------------------------------|------------|------------------------|
| `input_domain`                  | input type | `input_metric`         |
| `atom_domain(T)` (default)      | `T`        | `absolute_distance(T)` |
| `vector_domain(atom_domain(T))` | `Vec<T>`   | `l1_distance(T)`       |

Internally, all sampling is done using the discrete Laplace
distribution.

Required features: `contrib`

[make_laplace in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_laplace.html)

**Citations:**

- [GRS12 Universally Utility-Maximizing Privacy
  Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

- [CKS20 The Discrete Gaussian for Differential
  Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `DI::Carrier`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/noise/distribution/laplace/make_laplace.pdf)
