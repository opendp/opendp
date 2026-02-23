# lipschitz float mul constructor

Make a transformation that multiplies an aggregate by a constant.

## Usage

``` r
make_lipschitz_float_mul(input_domain, input_metric, constant, bounds)
```

## Arguments

- input_domain:

  The domain of the input.

- input_metric:

  The metric of the input.

- constant:

  The constant to multiply aggregates by.

- bounds:

  Tuple of inclusive lower and upper bounds.

## Value

Transformation

## Details

The bounds clamp the input, in order to bound the increase in
sensitivity from float rounding.

Required features: `contrib`

[make_lipschitz_float_mul in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_lipschitz_float_mul.html)

**Supporting Elements:**

- Input Domain: `AtomDomain<TA>`

- Output Domain: `AbsoluteDistance<TA>`

- Input Metric: `AtomDomain<TA>`

- Output Metric: `AbsoluteDistance<TA>`
