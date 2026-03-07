# cast default constructor

Make a Transformation that casts a vector of data from type `TIA` to
type `TOA`. Any element that fails to cast is filled with default.

## Usage

``` r
make_cast_default(input_domain, input_metric, .TOA)
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

|        |                  |
|--------|------------------|
| `TIA`  | `TIA::default()` |
| float  | `0.`             |
| int    | `0`              |
| string | `""`             |
| bool   | `false`          |

Required features: `contrib`

[make_cast_default in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_cast_default.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TOA>>`

- Output Metric: `M`
