# sum of squared deviations constructor

Make a Transformation that computes the sum of squared deviations of
bounded data.

## Usage

``` r
make_sum_of_squared_deviations(input_domain, input_metric, .S = "Pairwise<.T>")
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- .S:

  Summation algorithm to use on data type `T`. One of `Sequential<T>` or
  `Pairwise<T>`.

## Value

Transformation

## Details

This uses a restricted-sensitivity proof that takes advantage of known
dataset size. Use `make_clamp` to bound data and `make_resize` to
establish dataset size.

|                         |                |
|-------------------------|----------------|
| S (summation algorithm) | input type     |
| `Sequential<S::Item>`   | `Vec<S::Item>` |
| `Pairwise<S::Item>`     | `Vec<S::Item>` |

`S::Item` is the type of all of the following: each bound, each element
in the input data, the output data, and the output sensitivity.

For example, to construct a transformation that computes the SSD of
`f32` half-precision floats, set `S` to `Pairwise<f32>`.

Required features: `contrib`

[make_sum_of_squared_deviations in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_sum_of_squared_deviations.html)

**Citations:**

- [CSVW22 Widespread Underestimation of
  Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)

- [DMNS06 Calibrating Noise to Sensitivity in Private Data
  Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<S::Item>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `AtomDomain<S::Item>`

- Output Metric: `AbsoluteDistance<S::Item>`
