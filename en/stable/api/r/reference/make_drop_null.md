# drop null constructor

Make a Transformation that drops null values.

## Usage

``` r
make_drop_null(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

|                                                 |
|-------------------------------------------------|
| input_domain                                    |
| `vector_domain(option_domain(atom_domain(TA)))` |
| `vector_domain(atom_domain(TA))`                |
|                                                 |

Required features: `contrib`

[make_drop_null in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_drop_null.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<DIA>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<DIA::Imputed>>`

- Output Metric: `M`
