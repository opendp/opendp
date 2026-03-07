# find bin constructor

Make a transformation that finds the bin index in a monotonically
increasing vector of edges.

## Usage

``` r
make_find_bin(input_domain, input_metric, edges)
```

## Arguments

- input_domain:

  The domain of the input vector.

- input_metric:

  The metric of the input vector.

- edges:

  The set of edges to split bins by.

## Value

Transformation

## Details

For each value in the input vector, finds the index of the bin the value
falls into. `edges` splits the entire range of `TIA` into bins. The
first bin at index zero ranges from negative infinity to the first edge,
non-inclusive. The last bin at index `edges.len()` ranges from the last
bin, inclusive, to positive infinity.

To be valid, `edges` must be unique and ordered. `edges` are left
inclusive, right exclusive.

Required features: `contrib`

[make_find_bin in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/transformations/fn.make_find_bin.html)

**Supporting Elements:**

- Input Domain: `VectorDomain<AtomDomain<TIA>>`

- Output Domain: `M`

- Input Metric: `VectorDomain<AtomDomain<usize>>`

- Output Metric: `M`
