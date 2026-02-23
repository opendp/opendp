# mean constructor

Make a Transformation that computes the mean of bounded data.

## Usage

``` r
make_mean(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

This uses a restricted-sensitivity proof that takes advantage of known
dataset size. Use `make_clamp` to bound data and `make_resize` to
establish dataset size.

Required features: `contrib`

[make_mean in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_mean.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<T>>`

- Output Domain: `MI`

- Input Metric: `AtomDomain<T>`

- Output Metric: `AbsoluteDistance<T>`
