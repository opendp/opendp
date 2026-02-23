# identity constructor

Make a Transformation representing the identity function.

## Usage

``` r
make_identity(domain, metric)
```

## Arguments

- domain:

  Domain of input data

- metric:

  Metric on input domain

## Value

Transformation

## Details

WARNING: In Python, this function does not ensure that the domain and
metric form a valid metric space. However, if the domain and metric do
not form a valid metric space, then the resulting Transformation won't
be chainable with any valid Transformation, so it cannot be used to
introduce an invalid metric space into a chain of valid Transformations.

Required features: `contrib`, `honest-but-curious`

[make_identity in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_identity.html)

**Why honest-but-curious?:**

For the result to be a valid transformation, the `input_domain` and
`input_metric` pairing must form a valid metric space. For instance, the
symmetric distance metric and atom domain do not form a valid metric
space, because the metric cannot be used to measure distances between
any two elements of an atom domain. Whereas, the symmetric distance
metric and vector domain, or absolute distance metric and atom domain on
a scalar type, both form valid metric spaces.

**Supporting Elements:**

- Input Domain: `D`

- Output Domain: `M`

- Input Metric: `D`

- Output Metric: `M`
