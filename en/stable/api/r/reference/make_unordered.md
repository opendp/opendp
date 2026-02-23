# unordered constructor

Make a Transformation that converts the ordered dataset metric `MI` to
the respective ordered dataset metric with a no-op.

## Usage

``` r
make_unordered(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

|                      |                       |
|----------------------|-----------------------|
| `MI`                 | `MI::UnorderedMetric` |
| InsertDeleteDistance | SymmetricDistance     |
| HammingDistance      | ChangeOneDistance     |

Required features: `contrib`

[make_unordered in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_unordered.html)

**Supporting Elements:**

- Input Domain: `D`

- Output Domain: `MI`

- Input Metric: `D`

- Output Metric: `MI::UnorderedMetric`
