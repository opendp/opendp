# sized bounded int ordered sum constructor

Make a Transformation that computes the sum of bounded ints with known
dataset size.

## Usage

``` r
make_sized_bounded_int_ordered_sum(size, bounds, .T = NULL)
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

This uses a restricted-sensitivity proof that takes advantage of known
dataset size for better utility. You may need to use
`make_ordered_random` to impose an ordering on the data.

Required features: `contrib`

[make_sized_bounded_int_ordered_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_sized_bounded_int_ordered_sum.html)

**Citations:**

- [CSVW22 Widespread Underestimation of
  Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)

- [DMNS06 Calibrating Noise to Sensitivity in Private Data
  Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Domain: `InsertDeleteDistance`

- Input Metric: `AtomDomain<T>`

- Output Metric: `AbsoluteDistance<T>`
