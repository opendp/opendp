# Privacy measure used to define \\(\epsilon(\alpha)\\)-Rényi differential privacy.

In the following proof definition, \\(d\\) corresponds to an RDP curve
when also quantified over all adjacent datasets. That is, an RDP curve
\\(\epsilon(\alpha)\\) is no smaller than \\(d(\alpha)\\) for any
possible choices of \\(\alpha\\), and over all pairs of adjacent
datasets \\(x, x'\\) where \\(Y \sim M(x)\\), \\(Y' \sim M(x')\\).
\\(M(\cdot)\\) is a measurement (commonly known as a mechanism). The
measurement's input metric defines the notion of adjacency, and the
measurement's input domain defines the set of possible datasets.

## Usage

``` r
renyi_divergence()
```

## Value

Measure

## Details

**Proof Definition:**

For any two distributions \\(Y, Y'\\) and any curve \\(d\\), \\(Y, Y'\\)
are \\(d\\)-close under the Rényi divergence measure if, for any given
\\(\alpha \in (1, \infty)\\),

\\(D\_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}\_{x \sim Y'}
\Big\[\ln \left( \dfrac{\Pr\[Y = x\]}{\Pr\[Y' = x\]} \right)^\alpha
\Big\] \leq d(\alpha)\\)

Note that this \\(\epsilon\\) and \\(\alpha\\) are not privacy
parameters \\(\epsilon\\) and \\(\alpha\\) until quantified over all
adjacent datasets, as is done in the definition of a measurement.
