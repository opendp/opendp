# is null constructor

Make a Transformation that checks if each element in a vector is null or
nan.

## Usage

``` r
make_is_null(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

Required features: `contrib`

[make_is_null in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_is_null.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<DIA>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<bool>>`

- Output Metric: `M`
