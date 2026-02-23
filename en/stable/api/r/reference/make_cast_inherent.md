# cast inherent constructor

Make a Transformation that casts a vector of data from type `TIA` to a
type that can represent nullity `TOA`. If cast fails, fill with `TOA`'s
null value.

## Usage

``` r
make_cast_inherent(input_domain, input_metric, .TOA)
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

|       |                  |
|-------|------------------|
| `TIA` | `TIA::default()` |
| float | NaN              |

Required features: `contrib`

[make_cast_inherent in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_cast_inherent.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TOA>>`

- Output Metric: `M`
