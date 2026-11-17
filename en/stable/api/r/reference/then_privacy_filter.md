# partial privacy filter constructor

See documentation for
[`make_privacy_filter()`](https://docs.opendp.org/reference/make_privacy_filter.md)
for details.

## Usage

``` r
then_privacy_filter(lhs, odometer, d_in, d_out)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- odometer:

  A privacy odometer

- d_in:

  Upper bound on the distance between adjacent datasets

- d_out:

  Upper bound on the privacy loss

## Value

Measurement
