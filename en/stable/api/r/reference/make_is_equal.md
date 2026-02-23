# is equal constructor

Make a Transformation that checks if each element is equal to `value`.

## Usage

``` r
make_is_equal(input_domain, input_metric, value)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- value:

  value to check against

## Value

Transformation

## Details

Required features: `contrib`

[make_is_equal in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_is_equal.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<bool>>`

- Output Metric: `M`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/transformations/manipulation/make_is_equal.pdf)
