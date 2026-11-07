# Resolved Polars plan splitting

`crate::polars::split::split_plan` accepts a resolved private plan, not an
analyst's unresolved DP expressions. It is not a privacy validator.

The first fragment is a Select/GroupBy over the unchanged shared `base`.
The resolved base is opaque: the splitter neither traverses it nor checks its
determinism or supported operations.
Compatible scalar aggregate inputs share that fragment. The continuation
projects the original randomized expressions over an IPC scan, retaining key
columns by name. Public in-memory key frames become additional independent
fragments; their existing Distinct, keyed Join, and noisy imputation remain in
the continuation. No fragment consumes another fragment.

At the split boundary and in the continuation, unsupported plugins (including
quantile scoring), vector-valued private inputs, multiple private boundaries,
and reshaping of noisy aggregates are rejected.
Native Rust anonymous UDFs retain their original Polars serialization limitations;
use resolved FFI-plugin plans for execution in Python.

The splitter performs no I/O or schema resolution. The executor owns protected
IPC materialization, consistent snapshots, one-shot execution, final shuffling,
and cleanup. Neither plans containing private data nor intermediate IPC files
should be exposed to analysts.

## Tests

From `rust/`, with Python Polars **1.36.1** installed (`PYTHON` can override the
Python executable):

```sh
cargo test --no-default-features --features polars,untrusted polars::split::test
```

The tests materialize fragments using the already-enabled Rust engine **only as
a test oracle**, and compare zero-noise results against ordinary OpenDP plans.
The missing-file test serializes a Rust IPC scan before materialization and then
executes it in Python. The ignored FFI test additionally executes an actual noise
plugin, division, and threshold filter from a serialized continuation:

```sh
OPENDP_POLARS_LIB_PATH=/absolute/path/to/libopendp.dylib \
  cargo test --no-default-features --features polars-ffi,untrusted \
  polars::split::test -- --include-ignored
```

Use a matching OpenDP Polars-plugin binary (`.so`/`.dll` on other platforms).
