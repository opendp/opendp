# sized bounded float checked sum constructor

Make a Transformation that computes the sum of bounded floats with known
dataset size.

## Usage

``` r
make_sized_bounded_float_checked_sum(size, bounds, .S = "Pairwise<.T>")
```

## Arguments

- size:

  Number of records in input data.

- bounds:

  Tuple of lower and upper bounds for data in the input domain.

- .S:

  Summation algorithm to use over some data type `T` (`T` is shorthand
  for `S::Item`)

## Value

Transformation

## Details

This uses a restricted-sensitivity proof that takes advantage of known
dataset size for better utility.

|                         |                |
|-------------------------|----------------|
| S (summation algorithm) | input type     |
| `Sequential<S::Item>`   | `Vec<S::Item>` |
| `Pairwise<S::Item>`     | `Vec<S::Item>` |

`S::Item` is the type of all of the following: each bound, each element
in the input data, the output data, and the output sensitivity.

For example, to construct a transformation that pairwise-sums `f32`
half-precision floats, set `S` to `Pairwise<f32>`.

Required features: `contrib`

[make_sized_bounded_float_checked_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_sized_bounded_float_checked_sum.html)

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
