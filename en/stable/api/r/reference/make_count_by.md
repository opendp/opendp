# count by constructor

Make a Transformation that computes the count of each unique value in
data. This assumes that the category set is unknown.

## Usage

``` r
make_count_by(input_domain, input_metric, .TV = "int")
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- .TV:

  Type of Value. Express counts in terms of this integral type.

## Value

The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for
each unique data input (`TK`).

## Details

Required features: `contrib`

[make_count_by in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_count_by.html)

**Citations:**

- [BV17 Differential Privacy on Finite
  Computers](https://arxiv.org/abs/1709.05396)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TK>>`

- Output Domain: `SymmetricDistance`

- Input Metric: `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`

- Output Metric: `L01InfDistance<AbsoluteDistance<TV>>`
