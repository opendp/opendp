# variance constructor

Make a Transformation that computes the variance of bounded data.

## Usage

``` r
make_variance(input_domain, input_metric, ddof = 1L, .S = "Pairwise<.T>")
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- ddof:

  Delta degrees of freedom. Set to 0 if not a sample, 1 for sample
  estimate.

- .S:

  Summation algorithm to use on data type `T`. One of `Sequential<T>` or
  `Pairwise<T>`.

## Value

Transformation

## Details

This uses a restricted-sensitivity proof that takes advantage of known
dataset size. Use `make_clamp` to bound data and `make_resize` to
establish dataset size.

Required features: `contrib`

[make_variance in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_variance.html)

**Citations:**

- [DHK15 Differential Privacy for Social Science
  Inference](http://hona.kr/papers/files/DOrazioHonakerKingPrivacy.pdf)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<S::Item>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `AtomDomain<S::Item>`

- Output Metric: `AbsoluteDistance<S::Item>`
