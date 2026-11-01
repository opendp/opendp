# Privacy measure used to define \\((\epsilon, \delta)\\)-approximate differential privacy.

In the following definition, \\(d\\) corresponds to \\((\epsilon,
\delta)\\) when also quantified over all adjacent datasets. That is,
\\((\epsilon, \delta)\\) is no smaller than \\(d\\) (by product
ordering), over all pairs of adjacent datasets \\(x, x'\\) where \\(Y
\sim M(x)\\), \\(Y' \sim M(x')\\). \\(M(\cdot)\\) is a measurement
(commonly known as a mechanism). The measurement's input metric defines
the notion of adjacency, and the measurement's input domain defines the
set of possible datasets.

## Usage

``` r
fixed_smoothed_max_divergence()
```

## Value

Measure

## Details

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and any 2-tuple \\(d\\) of
non-negative numbers \\(\epsilon\\) and \\(\delta\\), \\(Y, Y'\\) are
\\(d\\)-close under the fixed smoothed max divergence measure whenever

\\(D\_\infty^\delta(Y, Y') = \max\_{S \subseteq \textrm{Supp}(Y)}
\Big\[\ln \dfrac{\Pr\[Y \in S\] + \delta}{\Pr\[Y' \in S\]} \Big\] \leq
\epsilon\\).

Note that this \\(\epsilon\\) and \\(\delta\\) are not privacy
parameters \\(\epsilon\\) and \\(\delta\\) until quantified over all
adjacent datasets, as is done in the definition of a measurement.
