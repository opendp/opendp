# b ary tree constructor

Expand a vector of counts into a b-ary tree of counts, where each branch
is the sum of its `b` immediate children.

## Usage

``` r
make_b_ary_tree(input_domain, input_metric, leaf_count, branching_factor)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- leaf_count:

  The number of leaf nodes in the b-ary tree.

- branching_factor:

  The number of children on each branch of the resulting tree. Larger
  branching factors result in shallower trees.

## Value

Transformation

## Details

Required features: `contrib`

[make_b\_ary_tree in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_b_ary_tree.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<TA>>`

- Output Metric: `M`
