# clamp constructor

Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.

## Usage

``` r
make_clamp(input_domain, input_metric, bounds)
```

## Arguments

- input_domain:

  Domain of input data.

- input_metric:

  Metric on input domain.

- bounds:

  Tuple of inclusive lower and upper bounds.

## Value

Transformation

## Details

If datum is less than lower, let datum be lower. If datum is greater
than upper, let datum be upper.

Required features: `contrib`

[make_clamp in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_clamp.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TA>>`

- Output Metric: `M`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/transformations/clamp/make_clamp.pdf)
