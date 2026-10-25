# Construct a new UserDistance. Any two instances of an UserDistance are equal if their string descriptors are equal.

Required features: `honest-but-curious`

## Usage

``` r
user_distance(identifier, descriptor = NULL)
```

## Arguments

- identifier:

  A string description of the metric.

- descriptor:

  Additional constraints on the domain.

## Value

Metric

## Details

**Why honest-but-curious?:**

Your definition of `d` must satisfy the requirements of a pseudo-metric:

1.  for any \\(x\\), \\(d(x, x) = 0\\)

2.  for any \\(x, y\\), \\(d(x, y) \ge 0\\) (non-negativity)

3.  for any \\(x, y\\), \\(d(x, y) = d(y, x)\\) (symmetry)

4.  for any \\(x, y, z\\), \\(d(x, z) \le d(x, y) + d(y, z)\\) (triangle
    inequality)
