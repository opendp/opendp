# Parse a runtime type or infer it from an example

Mirrors Python's public `RuntimeType.parse_or_infer(...)` entrypoint.

## Usage

``` r
parse_or_infer(
  type_name,
  public_example,
  generics = list(),
  allow_extrinsic = FALSE
)
```

## Arguments

- type_name:

  A type descriptor to normalize, or `NULL`.

- public_example:

  A value to infer from when `type_name` is `NULL`.

- generics:

  Generic type names to preserve during parsing.

- allow_extrinsic:

  Return `ExtrinsicObject` for unknown host objects when `TRUE`.

## Value

A normalized `runtime_type`.
