# partial user transformation constructor

See documentation for
[`make_user_transformation()`](https://docs.opendp.org/reference/make_user_transformation.md)
for details.

## Usage

``` r
then_user_transformation(
  lhs,
  input_domain,
  input_metric,
  output_domain,
  output_metric,
  function_,
  stability_map
)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- input_domain:

  A domain describing the set of valid inputs for the function.

- input_metric:

  The metric from which distances between adjacent inputs are measured.

- output_domain:

  A domain describing the set of valid outputs of the function.

- output_metric:

  The metric from which distances between outputs of adjacent inputs are
  measured.

- function\_:

  A function mapping data from `input_domain` to `output_domain`.

- stability_map:

  A function mapping distances from `input_metric` to `output_metric`.

## Value

Transformation
