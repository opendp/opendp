# metric bounded constructor

Make a Transformation that converts the unbounded dataset metric `MI` to
the respective bounded dataset metric with a no-op.

## Usage

``` r
make_metric_bounded(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

The constructor enforces that the input domain has known size, because
it must have known size to be valid under a bounded dataset metric.

|                      |                     |
|----------------------|---------------------|
| `MI`                 | `MI::BoundedMetric` |
| SymmetricDistance    | ChangeOneDistance   |
| InsertDeleteDistance | HammingDistance     |

Required features: `contrib`

[make_metric_bounded in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_metric_bounded.html)

**Supporting Elements:**

- Input Domain: `D`

- Output Domain: `MI`

- Input Metric: `D`

- Output Metric: `MI::BoundedMetric`
