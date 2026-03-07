# bounded int split sum constructor

Make a Transformation that computes the sum of bounded ints. Adds the
saturating sum of the positives to the saturating sum of the negatives.

## Usage

``` r
make_bounded_int_split_sum(bounds, .T = NULL)
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

[make_bounded_int_split_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_bounded_int_split_sum.html)

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
