# impute uniform float constructor

Make a Transformation that replaces NaN values in `Vec<TA>` with
uniformly distributed floats within `bounds`.

## Usage

``` r
make_impute_uniform_float(input_domain, input_metric, bounds)
```

## Arguments

- input_domain:

  Domain of the input.

- input_metric:

  Metric of the input.

- bounds:

  Tuple of inclusive lower and upper bounds.

## Value

Transformation

## Details

Required features: `contrib`

[make_impute_uniform_float in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_impute_uniform_float.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TA>>`

- Output Metric: `M`
