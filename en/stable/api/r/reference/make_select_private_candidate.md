# select private candidate constructor

Select a private candidate whose score is above a threshold.

## Usage

``` r
make_select_private_candidate(measurement, stop_probability, threshold)
```

## Arguments

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

## Details

Given `measurement` that satisfies ε-DP, returns new measurement M' that
satisfies 2ε-DP. M' releases the first invocation of `measurement` whose
score is above `threshold`.

Each time a score is below `threshold` the algorithm may terminate with
probability `stop_probability` and return nothing.

`measurement` should make releases in the form of (score, candidate). If
you are writing a custom scorer measurement in Python, specify the
output type as `TO=(float, "ExtrinsicObject")`. This ensures that the
float value is accessible to the algorithm. The candidate, left as
arbitrary Python data, is held behind the ExtrinsicObject.

Algorithm 1 in [Private selection from private
candidates](https://arxiv.org/pdf/1811.07971.pdf#page=7) (Liu and
Talwar, STOC 2019).

Required features: `contrib`

[make_select_private_candidate in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/combinators/fn.make_select_private_candidate.html)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MaxDivergence`

- Output Measure: `Option<(f64, TO)>`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.14.1/proofs/rust/src/combinators/select_private_candidate/make_select_private_candidate.pdf)
