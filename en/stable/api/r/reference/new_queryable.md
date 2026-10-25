# Construct a queryable from a user-defined transition function.

Required features: `contrib`

## Usage

``` r
new_queryable(transition, .Q = "ExtrinsicObject", .A = "ExtrinsicObject")
```

## Arguments

- transition:

  A transition function taking a reference to self, a query, and an
  internal/external indicator

- .Q:

  Query Type

- .A:

  Output Type
