# sized bounded int checked sum constructor

Make a Transformation that computes the sum of bounded ints. The
effective range is reduced, as (bounds \* size) must not overflow.

## Usage

``` r
make_sized_bounded_int_checked_sum(size, bounds, .T = NULL)
```

## Arguments

- size:

  Number of records in input data.

- bounds:

  Tuple of lower and upper bounds for data in the input domain.

- .T:

  Atomic Input Type and Output Type

## Value

Transformation

## Details

Required features: `contrib`

[make_sized_bounded_int_checked_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_sized_bounded_int_checked_sum.html)

**Citations:**

- [CSVW22 Widespread Underestimation of
  Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)

- [DMNS06 Calibrating Noise to Sensitivity in Private Data
  Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `AtomDomain<T>`

- Output Metric: `AbsoluteDistance<T>`
