# sum constructor

Make a Transformation that computes the sum of bounded data. Use
`make_clamp` to bound data.

## Usage

``` r
make_sum(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of the input data.

- input_metric:

  One of `SymmetricDistance` or `InsertDeleteDistance`.

## Value

Transformation

## Details

If dataset size is known, uses a restricted-sensitivity proof that takes
advantage of known dataset size for better utility.

Required features: `contrib`

[make_sum in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_sum.html)

**Citations:**

- [CSVW22 Widespread Underestimation of
  Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)

- [DMNS06 Calibrating Noise to Sensitivity in Private Data
  Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Domain: `MI`

- Input Metric: `AtomDomain<T>`

- Output Metric: `AbsoluteDistance<T>`
