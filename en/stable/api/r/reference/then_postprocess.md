# Compose a measurement with a postprocessing function.

This provides Python-like ergonomics for chaining a pure function after
a measurement release.

## Usage

``` r
then_postprocess(lhs, f, .TO = "ExtrinsicObject")
```

## Arguments

- lhs:

  A measurement to postprocess.

- f:

  A pure postprocessing function applied to the release.

- .TO:

  The runtime type of the postprocessed output.

## Value

A composed measurement.
