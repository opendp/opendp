# privacy filter constructor

Combinator that limits the privacy loss of an odometer.

## Usage

``` r
make_privacy_filter(odometer, d_in, d_out)
```

## Arguments

- odometer:

  A privacy odometer

- d_in:

  Upper bound on the distance between adjacent datasets

- d_out:

  Upper bound on the privacy loss

## Value

Measurement

## Details

Adjusts the queryable returned by the odometer to reject any query that
would increase the total privacy loss above the privacy guarantee of the
mechanism.

Required features: `contrib`

[make_privacy_filter in Rust
documentation.](https://docs.rs/opendp/0.15.1/opendp/combinators/fn.make_privacy_filter.html)

**Citations:**

- [RRUV21 Privacy Odometers and Filters: Pay-as-you-Go
  Composition](https://arxiv.org/abs/1605.08294v1)

- [HSTVVXZ23 Concurrent Composition for Interactive Differential Privacy
  with Adaptive Privacy-Loss
  Parameters](https://arxiv.org/abs/2309.05901)

**Supporting Elements:**

- Input Domain: `DI`

- Output Type: `MI`

- Input Metric: `MO`

- Output Measure: `OdometerQueryable<Q, A, MI::Distance, MO::Distance>`

**Proof Definition:**

[(Proof
Document)](https://docs.opendp.org/en/v0.15.1/proofs/rust/src/combinators/privacy_filter/make_privacy_filter.pdf)
