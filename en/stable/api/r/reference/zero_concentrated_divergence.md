# Privacy measure used to define \\(\rho\\)-zero concentrated differential privacy.

In the following proof definition, \\(d\\) corresponds to \\(\rho\\)
when also quantified over all adjacent datasets. That is, \\(\rho\\) is
the greatest possible \\(d\\) over all pairs of adjacent datasets \\(x,
x'\\) where \\(Y \sim M(x)\\), \\(Y' \sim M(x')\\). \\(M(\cdot)\\) is a
measurement (commonly known as a mechanism). The measurement's input
metric defines the notion of adjacency, and the measurement's input
domain defines the set of possible datasets.

## Usage

``` r
zero_concentrated_divergence()
```

## Value

Measure

## Details

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and any non-negative \\(d\\),
\\(Y, Y'\\) are \\(d\\)-close under the zero-concentrated divergence
measure if, for every possible choice of \\(\alpha \in (1, \infty)\\),

\\(D\_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}\_{x \sim Y'}
\Big\[\ln \left( \dfrac{\Pr\[Y = x\]}{\Pr\[Y' = x\]} \right)^\alpha
\Big\] \leq d \cdot \alpha\\).
