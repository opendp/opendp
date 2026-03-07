# index constructor

Make a transformation that treats each element as an index into a vector
of categories.

## Usage

``` r
make_index(input_domain, input_metric, categories, null, .TOA = NULL)
```

## Arguments

- input_domain:

  The domain of the input vector.

- input_metric:

  The metric of the input vector.

- categories:

  The set of categories to index into.

- null:

  Category to return if the index is out-of-range of the category set.

- .TOA:

  Atomic Output Type. Output data will be `Vec<TOA>`.

## Value

Transformation

## Details

Required features: `contrib`

[make_index in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_index.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<usize>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TOA>>`

- Output Metric: `M`
