# partial resize constructor

See documentation for
[`make_resize()`](https://docs.opendp.org/reference/make_resize.md) for
details.

## Usage

``` r
then_resize(lhs, size, constant, .MO = "SymmetricDistance")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- size:

  Number of records in output data.

- constant:

  Value to impute with.

- .MO:

  Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`

## Value

A vector of the same type `TA`, but with the provided `size`.
