# bounded int ordered sum constructor

Make a Transformation that computes the sum of bounded ints. You may
need to use `make_ordered_random` to impose an ordering on the data.

## Usage

``` r
make_bounded_int_ordered_sum(bounds, .T = NULL)
```

## Arguments

- bounds:

  Tuple of lower and upper bounds for data in the input domain.

- .T:

  Atomic Input Type and Output Type

## Value

Transformation

## Details

Required features: `contrib`

[make_bounded_int_ordered_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_bounded_int_ordered_sum.html)

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
