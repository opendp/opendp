# Privacy measure used to define \\(\epsilon\\)-pure differential privacy.

In the following proof definition, \\(d\\) corresponds to \\(\epsilon\\)
when also quantified over all adjacent datasets. That is, \\(\epsilon\\)
is the greatest possible \\(d\\) over all pairs of adjacent datasets
\\(x, x'\\) where \\(Y \sim M(x)\\), \\(Y' \sim M(x')\\). \\(M(\cdot)\\)
is a measurement (commonly known as a mechanism). The measurement's
input metric defines the notion of adjacency, and the measurement's
input domain defines the set of possible datasets.

## Usage

``` r
max_divergence()
```

## Value

Measure

## Details

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and any non-negative \\(d\\),
\\(Y, Y'\\) are \\(d\\)-close under the max divergence measure whenever

\\(D\_\infty(Y, Y') = \max\_{S \subseteq \textrm{Supp}(Y)} \Big\[\ln
\dfrac{\Pr\[Y \in S\]}{\Pr\[Y' \in S\]} \Big\] \leq d\\).
