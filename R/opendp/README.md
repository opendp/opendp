# OpenDP R API

The OpenDP R API largely mirrors the Python API, with some key differences.
Most Python examples have not yet been translated to R:
These hints will help you apply the Python examples to R.

- After invoking `library(opendp)` all functions are available: There are no namespaces like those of the Python library.
- The chaining operator in R is `|>`, as opposed to `>>` in Python.
- Type arguments like `.T` are prefixed with a period in R to avoid name conflicts, while in Python it is plain `T`.
- All portions of the Python API that are derived from Rust have parallels in the R API, but packages like `context` that exist only in Python have not yet been implemented in R.

Next: [Reference](reference/index.html)

<!-- The line above works in the rendered docs, but not in github. This file is skipped by link checker. -->