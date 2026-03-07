# geometric constructor

Equivalent to `make_laplace` but restricted to an integer support. Can
specify `bounds` to run the algorithm in near constant-time.

## Usage

``` r
make_geometric(
  input_domain,
  input_metric,
  scale,
  bounds = NULL,
  .MO = "MaxDivergence"
)
```

## Arguments

- input_domain:

  Domain of the data type to be privatized.

- input_metric:

  Metric of the data type to be privatized.

- scale:

  Noise scale parameter for the distribution. `scale` ==
  standard_deviation / sqrt(2).

- bounds:

  Set bounds on the count to make the algorithm run in constant-time.

- .MO:

  Measure used to quantify privacy loss. Valid values are just
  `MaxDivergence`

## Value

Measurement

## Details

Required features: `contrib`

[make_geometric in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_geometric.html)

**Citations:**

- [GRS12 Universally Utility-Maximizing Privacy
  Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `DI::Carrier`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/measurements/noise/distribution/geometric/make_geometric.pdf)
