# count by categories constructor

Make a Transformation that computes the number of times each category
appears in the data. This assumes that the category set is known.

## Usage

``` r
make_count_by_categories(
  input_domain,
  input_metric,
  categories,
  null_category = TRUE,
  .MO = "L1Distance<int>",
  .TOA = "int"
)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- categories:

  The set of categories to compute counts for.

- null_category:

  Include a count of the number of elements that were not in the
  category set at the end of the vector.

- .MO:

  Output Metric.

- .TOA:

  Atomic Output Type that is numeric.

## Value

The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).

## Details

Required features: `contrib`

[make_count_by_categories in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_count_by_categories.html)

**Citations:**

- [GRS12 Universally Utility-Maximizing Privacy
  Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)

- [BV17 Differential Privacy on Finite
  Computers](https://arxiv.org/abs/1709.05396)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `VectorDomain<AtomDomain<TOA>>`

- Output Metric: `MO`
