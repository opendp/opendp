# impute constant constructor

Make a Transformation that replaces null/None data with `constant`.

## Usage

``` r
make_impute_constant(input_domain, input_metric, constant)
```

## Arguments

- input_domain:

  Domain of the input data. See table above.

- input_metric:

  Metric of the input data. A dataset metric.

- constant:

  Value to replace nulls with.

## Value

Transformation

## Details

If chaining after a `make_cast`, the input type is `Option<Vec<TA>>`. If
chaining after a `make_cast_inherent`, the input type is `Vec<TA>`,
where `TA` may take on float NaNs.

|                                                 |                   |
|-------------------------------------------------|-------------------|
| input_domain                                    | Input Data Type   |
| `vector_domain(option_domain(atom_domain(TA)))` | `Vec<Option<TA>>` |
| `vector_domain(atom_domain(TA))`                | `Vec<TA>`         |

Required features: `contrib`

[make_impute_constant in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_impute_constant.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<DIA>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<DIA::Imputed>>`

- Output Metric: `M`
