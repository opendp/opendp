# OpenDP R API

The OpenDP R API largely mirrors the Python API, with some key differences.
Most Python examples have not yet been translated to R:
These hints will help you apply the Python examples to R.

- After invoking `library(opendp)` all functions are available: There are no namespaces like those of the Python library.
- The chaining operator in R is `|>`, as opposed to `>>` in Python.
- Type arguments like `.T` are prefixed with a period in R to avoid name conflicts, while in Python it is plain `T`.
- The Context API uses the existing `then_*` functions in a pipe instead of dynamically adding methods to query objects.

## Context API

The Context API coordinates privacy accounting and query execution. Declare the
privacy unit, total privacy loss, and number of planned releases up front:

```r
library(opendp)
enable_features("contrib", "idealized-numerics")

context <- Context$compositor(
  data = c(5L, 6L, 7L),
  privacy_unit = unit_of(contributions = 1L),
  privacy_loss = loss_of(epsilon = 1),
  split_evenly_over = 2L
)

count_query <- query(context) |>
  then_count() |>
  then_laplace(auto())

# Inspecting the inferred scale does not spend privacy budget.
param(count_query, .T = "float")

dp_count <- release(count_query)
current_privacy_loss(context)
remaining_privacy_loss(context)
```

`auto()` asks the Context API to search for one numeric constructor argument
that makes the query satisfy its assigned privacy loss. At most one `auto()`
may be unresolved in a query chain.

Providing `split_evenly_over` or `split_by_weights` creates a static sequence of
query allowances. Without a split, the context uses a privacy filter, and each
query declares its own loss:

```r
context <- Context$compositor(
  data = c(5L, 6L, 7L),
  privacy_unit = unit_of(contributions = 1L),
  privacy_loss = loss_of(epsilon = 1)
)

release(
  query(context, epsilon = 0.25) |>
    then_count() |>
    then_laplace(auto()),
  .T = "float"
)
```

Queries are immutable builders: piping another `then_*` constructor returns a
new query. Privacy budget is consumed only by a successful `release()`, not by
building a query, calling `resolve()`, or inspecting `param()`.

Next: [Reference](reference/index.html)

<!-- The line above works in the rendered docs, but not in github. This file is skipped by link checker. -->
