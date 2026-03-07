# find constructor

Find the index of a data value in a set of categories.

## Usage

``` r
make_find(input_domain, input_metric, categories)
```

## Arguments

- input_domain:

  The domain of the input vector.

- input_metric:

  The metric of the input vector.

- categories:

  The set of categories to find indexes from.

## Value

Transformation

## Details

For each value in the input vector, finds the index of the value in
`categories`. If an index is found, returns `Some(index)`, else `None`.
Chain with `make_impute_constant` or `make_drop_null` to handle nullity.

Required features: `contrib`

[make_find in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_find.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<OptionDomain<AtomDomain<usize>>>`

- Output Metric: `M`
