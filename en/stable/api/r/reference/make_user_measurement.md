# user measurement constructor

Construct a Measurement from user-defined callbacks.

## Usage

``` r
make_user_measurement(
  input_domain,
  input_metric,
  output_measure,
  function_,
  privacy_map,
  .TO = "ExtrinsicObject"
)
```

## Arguments

- input_domain:

  A domain describing the set of valid inputs for the function.

- input_metric:

  The metric from which distances between adjacent inputs are measured.

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

## Details

Required features: `contrib`, `honest-but-curious`

**Why honest-but-curious?:**

This constructor only returns a valid measurement if for every pair of
elements \\(x, x'\\) in `input_domain`, and for every pair
`(d_in, d_out)`, where `d_in` has the associated type for `input_metric`
and `d_out` has the associated type for `output_measure`, if \\(x, x'\\)
are `d_in`-close under `input_metric`, `privacy_map(d_in)` does not
raise an exception, and `privacy_map(d_in) <= d_out`, then
`function(x), function(x')` are d_out-close under `output_measure`.

In addition, `function` must not have side-effects, and `privacy_map`
must be a pure function.

**Supporting Elements:**

- Input Domain: `AnyDomain`

- Output Type: `AnyMetric`

- Input Metric: `AnyMeasure`

- Output Measure: `AnyObject`
