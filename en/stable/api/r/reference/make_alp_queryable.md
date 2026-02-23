# alp queryable constructor

Measurement to release a queryable containing a DP projection of bounded
sparse data.

## Usage

``` r
make_alp_queryable(
  input_domain,
  input_metric,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L
)
```

## Arguments

- input_domain:

  Domain of input data

- input_metric:

  Metric on input domain

- scale:

  Privacy loss parameter. This is equal to epsilon/sensitivity.

- total_limit:

  Either the true value or an upper bound estimate of the sum of all
  values in the input.

- value_limit:

  Upper bound on individual values (referred to as β). Entries above β
  are clamped.

- size_factor:

  Optional multiplier (default of 50) for setting the size of the
  projection.

- alpha:

  Optional parameter (default of 4) for scaling and determining p in
  randomized response step.

## Value

Measurement

## Details

The size of the projection is O(total \* size_factor \* scale / alpha).
The evaluation time of post-processing is O(beta \* scale / alpha).

`size_factor` is an optional multiplier (defaults to 50) for setting the
size of the projection. There is a memory/utility trade-off. The value
should be sufficiently large to limit hash collisions.

Required features: `contrib`

[make_alp_queryable in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measurements/fn.make_alp_queryable.html)

**Citations:**

- [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal
  Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4

**Supporting Elements:**

- Input Domain: `MapDomain<AtomDomain<K>, AtomDomain<CI>>`

- Output Type: `L01InfDistance<AbsoluteDistance<CI>>`

- Input Metric: `MaxDivergence`

- Output Measure: `Queryable<K, f64>`
