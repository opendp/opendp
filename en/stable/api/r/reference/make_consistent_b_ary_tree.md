# consistent b ary tree constructor

Postprocessor that makes a noisy b-ary tree internally consistent, and
returns the leaf layer.

## Usage

``` r
make_consistent_b_ary_tree(branching_factor, .TIA = "int", .TOA = "float")
```

## Arguments

- branching_factor:

  the maximum number of children

- .TIA:

  Atomic type of the input data. Should be an integer type.

- .TOA:

  Atomic type of the output data. Should be a float type.

## Value

Function

## Details

The input argument of the function is a balanced `b`-ary tree implicitly
stored in breadth-first order Tree is assumed to be complete, as in, all
leaves on the last layer are on the left. Non-existent leaves are
assumed to be zero.

The output remains consistent even when leaf nodes are missing. This is
due to an adjustment to the original algorithm to apportion corrections
to children relative to their variance.

Required features: `contrib`

[make_consistent_b\_ary_tree in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_consistent_b_ary_tree.html)

**Citations:**

- [HRMS09 Boosting the Accuracy of Differentially Private Histograms
  Through Consistency, section 4.1](https://arxiv.org/pdf/0904.0942.pdf)

**Supporting Elements:**

- Input Type: `Vec<TIA>`

- Output Type: `Vec<TOA>`
