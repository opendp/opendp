# ordered random constructor

Make a Transformation that converts the unordered dataset metric
`SymmetricDistance` to the respective ordered dataset metric
`InsertDeleteDistance` by assigning a random permutation.

## Usage

``` r
make_ordered_random(input_domain, input_metric)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

## Value

Transformation

## Details

|                   |                      |
|-------------------|----------------------|
| `MI`              | `MI::OrderedMetric`  |
| SymmetricDistance | InsertDeleteDistance |
| ChangeOneDistance | HammingDistance      |

Required features: `contrib`

[make_ordered_random in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_ordered_random.html)

**Supporting Elements:**

- Input Domain: `D`

- Output Domain: `MI`

- Input Metric: `D`

- Output Metric: `MI::OrderedMetric`
