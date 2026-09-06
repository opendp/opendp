# Parse a runtime type descriptor into a runtime_type object

Mirrors Python's public `RuntimeType.parse(...)` entrypoint.

## Usage

``` r
rt_parse(type_name, generics = list())
```

## Arguments

- type_name:

  A type descriptor to normalize.

- generics:

  Generic type names to preserve during parsing.

## Value

A normalized `runtime_type`.
