# Privacy measure used to define \\(\delta\\)-approximate PM-differential privacy.

In the following definition, \\(d\\) corresponds to privacy parameters
\\((d', \delta)\\) when also quantified over all adjacent datasets
(\\(d'\\) is the privacy parameter corresponding to privacy measure PM).
That is, \\((d', \delta)\\) is no smaller than \\(d\\) (by product
ordering), over all pairs of adjacent datasets \\(x, x'\\) where \\(Y
\sim M(x)\\), \\(Y' \sim M(x')\\). \\(M(\cdot)\\) is a measurement
(commonly known as a mechanism). The measurement's input metric defines
the notion of adjacency, and the measurement's input domain defines the
set of possible datasets.

## Usage

``` r
approximate(measure)
```

## Arguments

- measure:

  inner privacy measure

## Value

ApproximateDivergence

## Details

[approximate in Rust
documentation.](https://docs.rs/opendp/0.14.1/opendp/measures/struct.Approximate.html)

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and 2-tuple \\(d = (d',
\delta)\\), where \\(d'\\) is the distance with respect to privacy
measure PM, \\(Y, Y'\\) are \\(d\\)-close under the approximate PM
measure whenever, for any choice of \\(\delta \in \[0, 1\]\\), there
exist events \\(E\\) (depending on \\(Y\\)) and \\(E'\\) (depending on
\\(Y'\\)) such that \\(\Pr\[E\] \ge 1 - \delta\\), \\(\Pr\[E'\] \ge 1 -
\delta\\), and

\\(D\_{\mathrm{PM}}^\delta(Y\|\_E, Y'\|\_{E'}) =
D\_{\mathrm{PM}}(Y\|\_E, Y'\|\_{E'})\\)

where \\(Y\|\_E\\) denotes the distribution of \\(Y\\) conditioned on
the event \\(E\\).

Note that this \\(\delta\\) is not privacy parameter \\(\delta\\) until
quantified over all adjacent datasets, as is done in the definition of a
measurement.
