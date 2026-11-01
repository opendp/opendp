# Infer a runtime type from a public example

Mirrors Python's public `RuntimeType.infer(...)` entrypoint.

## Usage

``` r
rt_infer(public_example, allow_extrinsic = FALSE)
```

## Arguments

- public_example:

  A value whose runtime type should be inferred.

- allow_extrinsic:

  Return `ExtrinsicObject` for unknown host objects when `TRUE`.

## Value

A normalized `runtime_type`.
