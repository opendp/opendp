# Privacy measure used to define \\(\epsilon(\delta)\\)-approximate differential privacy.

In the following proof definition, \\(d\\) corresponds to a privacy
profile when also quantified over all adjacent datasets. That is, a
privacy profile \\(\epsilon(\delta)\\) is no smaller than
\\(d(\delta)\\) for all possible choices of \\(\delta\\), and over all
pairs of adjacent datasets \\(x, x'\\) where \\(Y \sim M(x)\\), \\(Y'
\sim M(x')\\). \\(M(\cdot)\\) is a measurement (commonly known as a
mechanism). The measurement's input metric defines the notion of
adjacency, and the measurement's input domain defines the set of
possible datasets.

## Usage

``` r
smoothed_max_divergence()
```

## Value

Measure

## Details

The distance \\(d\\) is of type PrivacyProfile, so it can be invoked
with an \\(\epsilon\\) to retrieve the corresponding \\(\delta\\).

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and any curve \\(d(\cdot)\\),
\\(Y, Y'\\) are \\(d\\)-close under the smoothed max divergence measure
whenever, for any choice of non-negative \\(\epsilon\\), and \\(\delta =
d(\epsilon)\\),

\\(D\_\infty^\delta(Y, Y') = \max\_{S \subseteq \textrm{Supp}(Y)}
\Big\[\ln \dfrac{\Pr\[Y \in S\] + \delta}{\Pr\[Y' \in S\]} \Big\] \leq
\epsilon\\).

Note that \\(\epsilon\\) and \\(\delta\\) are not privacy parameters
\\(\epsilon\\) and \\(\delta\\) until quantified over all adjacent
datasets, as is done in the definition of a measurement.
