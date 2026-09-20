# partial geometric constructor

See documentation for
[`make_geometric()`](https://docs.opendp.org/reference/make_geometric.md)
for details.

## Usage

``` r
then_geometric(lhs, scale, bounds = NULL, .MO = "MaxDivergence")
```

## Arguments

- lhs:

  The prior transformation or metric space.

- scale:

  Noise scale parameter for the distribution. `scale` ==
  standard_deviation / sqrt(2).

- bounds:

  Set bounds on the count to make the algorithm run in constant-time.

- .MO:

  Measure used to quantify privacy loss. Valid values are just
  `MaxDivergence`

## Value

Measurement
