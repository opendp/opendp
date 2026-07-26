# partial select private candidate constructor

See documentation for
[`make_select_private_candidate()`](https://docs.opendp.org/reference/make_select_private_candidate.md)
for details.

## Usage

``` r
then_select_private_candidate(lhs, measurement, stop_probability, threshold)
```

## Arguments

- lhs:

  The prior transformation or metric space.

- measurement:

  A measurement that releases a 2-tuple of (score, candidate)

- stop_probability:

  The probability of stopping early at any iteration.

- threshold:

  The threshold score. Return immediately if the score is above this
  threshold.

## Value

A measurement that returns a release from `measurement` whose score is
greater than `threshold`, or none.
