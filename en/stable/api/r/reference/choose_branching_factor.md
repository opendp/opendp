# Returns an approximation to the ideal `branching_factor` for a dataset of a given size, that minimizes error in cdf and quantile estimates based on b-ary trees.

Required features: `contrib`

## Usage

``` r
choose_branching_factor(size_guess)
```

## Arguments

- size_guess:

  A guess at the size of your dataset.

## Value

int

## Details

[choose_branching_factor in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.choose_branching_factor.html)

**Citations:**

- [QYL13 Understanding Hierarchical Methods for Differentially Private
  Histograms](http://www.vldb.org/pvldb/vol6/p1954-qardaji.pdf)
