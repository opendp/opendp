# user transformation constructor

Construct a Transformation from user-defined callbacks.

## Usage

``` r
make_user_transformation(
  input_domain,
  input_metric,
  output_domain,
  output_metric,
  function_,
  stability_map
)
```

## Arguments

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

## Details

Required features: `contrib`, `honest-but-curious`

**Why honest-but-curious?:**

This constructor only returns a valid transformation if for every pair
of elements \\(x, x'\\) in `input_domain`, and for every pair
`(d_in, d_out)`, where `d_in` has the associated type for `input_metric`
and `d_out` has the associated type for `output_metric`, if \\(x, x'\\)
are `d_in`-close under `input_metric`, `stability_map(d_in)` does not
raise an exception, and `stability_map(d_in) <= d_out`, then
`function(x), function(x')` are d_out-close under `output_metric`.

In addition, for every element \\(x\\) in `input_domain`, `function(x)`
is a member of `output_domain` or raises a data-independent runtime
exception.

In addition, `function` must not have side-effects, and `stability_map`
must be a pure function.
