# Resolved Polars plan splitting

The internal `split_plan` accepts a resolved OpenDP plan and returns two native
Polars plans:

- A `DslPlan::SinkMultiple` of IPC file sinks, even for a single head.
- A randomized continuation containing corresponding native IPC scans.

Each aggregate sink retains the unchanged input subtree preceding the private
Select/GroupBy. That base has already passed stability analysis and is opaque
to the splitter. Compatible scalar aggregate inputs share a head. One internal
naming allocator reserves relevant expression names and declared public schemas
across the split before generating intermediate columns. User-facing output
names and explicit join keys are preserved.

Public in-memory key frames become additional independent IPC-writing heads.
Their existing Distinct, keyed Join, and noisy missing-group imputation remain
in the continuation. No head reads another head's output. Externalization keeps
populated DataFrame scans out of the continuation, **not** the deterministic
plan: serializing the sink plan can still embed base or public-key DataFrames.
Both plans and their private intermediates must remain executor-only.

The splitter trusts OpenDP's stability and privacy analysis. Trusted NoisePlugin
occurrences identify the randomized boundaries; deterministic input subtrees
are extracted intact, without classifying their operators. Postprocessing is
rewritten structurally, not validated. Public deterministic operands (such as
an exact length already proven public) also retain their original context.
SQL support belongs to the external translator.

One structural distinction remains: an expression whose terminal nodes are all
scalar literals stays in the continuation. OpenDP's validity analysis does not
change Polars' literal broadcasting state. Replacing that expression with an
IPC column would turn one shared noise draw into independent draws per group.
Nullary context inputs such as len() are not literal leaves and are extracted.
This is not an operator whitelist or a second cardinality analysis.

Only trusted NoisePlugin boundaries are split; noisy-max/DP-quantile mechanisms
and multiple private boundaries remain unsupported. Rust anonymous UDFs retain
their original serialization limitations; use FFI-backed plugins for Python
execution.

Splitting performs no execution, I/O, or schema resolution. The eventual trusted
OnceFrame integration must enforce protected IPC files, consistent snapshots,
one-shot execution, final shuffling, and cleanup. There is no Ibis executor yet.

## Tests

From `rust/`, with Python Polars **1.36.1** installed (`PYTHON` can override the
Python executable):

```sh
cargo test --no-default-features --features polars,untrusted polars::split::test
```

The tests execute native IPC sinks using the already-enabled Rust engine only
as a test oracle. They compare zero-noise results against ordinary OpenDP plans,
check scalar broadcasting and name collisions, and serialize/deserialize
SinkMultiple and IPC scans before their destination files exist. Python then
executes the materialized scan. Additional FFI continuation coverage requires
a matching OpenDP Polars-plugin binary (`.so`/`.dll` on other platforms):

```sh
OPENDP_POLARS_LIB_PATH=/absolute/path/to/libopendp.dylib \
  cargo test --no-default-features --features polars-ffi,untrusted \
  polars::split::test -- --include-ignored
```
