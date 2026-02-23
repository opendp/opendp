# metric unbounded constructor

Make a Transformation that converts the bounded dataset metric `MI` to
the respective unbounded dataset metric with a no-op.

## Usage

``` r
make_metric_unbounded(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

|                   |                       |
|-------------------|-----------------------|
| `MI`              | `MI::UnboundedMetric` |
| ChangeOneDistance | SymmetricDistance     |
| HammingDistance   | InsertDeleteDistance  |

Required features: `contrib`

[make_metric_unbounded in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_metric_unbounded.html)

**Supporting Elements:**

- Input Domain: `D`

- Output Domain: `MI`

- Input Metric: `D`

- Output Metric: `MI::UnboundedMetric`
