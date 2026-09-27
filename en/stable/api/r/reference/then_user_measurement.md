# partial user measurement constructor

See documentation for
[`make_user_measurement()`](https://docs.opendp.org/reference/make_user_measurement.md)
for details.

## Usage

``` r
then_user_measurement(
  lhs,
  output_measure,
  function_,
  privacy_map,
  .TO = "ExtrinsicObject"
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- output_measure:

  The measure from which distances between adjacent output distributions
  are measured.

- function\_:

  A function mapping data from `input_domain` to a release of type `TO`.

- privacy_map:

  A function mapping distances from `input_metric` to `output_measure`.

- .TO:

  The data type of outputs from the function.

## Value

Measurement
