# cast constructor

Make a Transformation that casts a vector of data from type `TIA` to
type `TOA`. For each element, failure to parse results in `None`, else
`Some(out)`.

## Usage

``` r
make_cast(input_domain, input_metric, .TOA)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- .TOA:

  Atomic Output Type to cast into

## Value

Transformation

## Details

Can be chained with `make_impute_constant` or `make_drop_null` to handle
nullity.

Required features: `contrib`

[make_cast in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_cast.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<OptionDomain<AtomDomain<TOA>>>`

- Output Metric: `M`
