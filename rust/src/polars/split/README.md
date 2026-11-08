# Resolved Polars plan splitting

The internal `split_plan` returns two native Polars plans:

- A `DslPlan::SinkMultiple` of deterministic IPC sinks, even for one head.
- A continuation with IPC scans, randomized operations and final results.

The already-analyzed input preceding the private Select/GroupBy stays opaque and
unchanged, including unknown plugins. Splitting performs no execution, I/O or
schema resolution. One allocator reserves names across outputs, keys and public
schemas before naming intermediates.

## Ordinary histogram boundary

Trusted `NoisePlugin` and `NoisyMaxPlugin` occurrences identify randomized
boundaries. Compatible scalar inputs share a deterministic head. Each resolved
`DiscreteQuantileScorePlugin` gets an independent ordinary sparse-histogram head;
there is no statistic-specific translator handler or deduplication.

For k candidates, 2k+1 buckets distinguish equality from adjacent open intervals.
A native HStack projects the scorer input to a fresh column, explicitly cast to
the resolved candidates' dtype. A second HStack assigns UInt64 bucket IDs with a
balanced typed when/then/otherwise comparison tree. Candidate scalars retain their
original numeric types, including Float32 and extreme integers. No conversion
through f64 or contextual type-inference pipeline is involved.

The head groups directly by the original keys plus bucket, aggregating
`len().cast(UInt64)` as count. It contains no list aggregation, explode or unnest.
The ordinary translator registries can lower it to one GROUP BY with COUNT(*).
No new source plugin or specialized serialization protocol is needed.

Only the scorer input is inspected: literal-only inputs and cardinality-changing
filter/drop_nulls/drop_nans expressions are explicitly rejected during extraction.
Base filters/drops/plugins remain opaque and allowed. Scalar literal **noise**
retains its original one-draw broadcasting semantics.

The analyzed accepted scorer inputs are non-null/non-NaN and row-preserving, so
observed groups contribute at least one bucket. Empty histograms have zero rows,
without sentinel/scaffold rows or observed-key alignment. Global reconstruction
still produces one all-zero score array; grouped empty input produces no groups.

## Local continuation and shared key release

After IPC, the continuation groups by original keys only (or selects globally).
The existing `HistogramQuantileScorePlugin` fills omitted bins with zero and
reconstructs the original fixed-size UInt64 scores. It shares exact score arithmetic
with the ordinary scorer: total/prefix counts stay u64, only strict tails are
clipped, and alpha constants, size limit and overflow behavior are unchanged.
Original noisy-max and candidate-lookup plugins then run in Polars.

Maximal single-head expression subtrees run locally; cross-head expressions wait
above null-equal keyed joins (or one-row global cross joins). No privacy accounting
changes. The original shared threshold Filter, public-key Join/Distinct and
missing-group fill remain above the combined results. Public post-join imputation
handles truly absent private groups exactly once. Public in-memory frames become
independent IPC heads. No head reads another head.

The continuation has no populated DataFrame scans, but sink serialization may embed
base/public DataFrames. Both plans and intermediates remain executor-only. Trusted
OnceFrame integration must protect files, maintain snapshots, enforce one-shot use,
shuffle final results and clean up; this change does not implement that executor.
Multiple private Select/GroupBy boundaries remain unsupported. Anonymous Rust UDFs
retain serialization limitations; cross-process use needs FFI and a matching library.

## Tests

From `rust/`, with Python Polars **1.36.1**:

```sh
cargo fmt --check
cargo test --no-default-features --features polars,untrusted polars::split::test
cargo test --no-default-features --features polars,untrusted quantile
cargo build --release --no-default-features --features polars-ffi,untrusted
```

For actual translator execution, install the sibling `polars-to-ibis` converter plus
Ibis/DuckDB in an isolated Python environment. The ignored integration test
serializes actual Rust-generated ordinary heads, calls public
`convert_polars_to_ibis`, executes SQL, checks exact sparse counts and absence of
SQL list materialization, writes IPC, and executes the native continuation against
unsplit zero-noise results. It covers mixed/local/cross-head expressions, null keys,
null/NaN replacement, large integers, empty inputs, public-key imputation/deduplication,
thresholding and naming collisions. It also exercises resolved non-strict overflow
casts, Float32 root fills and explicitly typed Float32 preprocessing. Generalized
weak intermediate arithmetic coercion is outside this change's scope.

```sh
PYTHON=/path/to/venv/bin/python \
OPENDP_POLARS_LIB_PATH="$PWD/target/release/libopendp.dylib" \
OPENDP_SPLIT_WIRE_FIXTURE=/tmp/opendp-ordinary-histogram-wire.json \
  cargo test --no-default-features --features polars-ffi,untrusted \
  polars::split::test -- --include-ignored
```

Use `.so`/`.dll` on other platforms. The optional fixture is the actual serialized
first head. Native tests additionally cover exact bucket IDs and balanced depth,
exact score arrays across candidate ties/gaps, alpha extremes, clipping, numeric
types and preprocessing, all-zero multi-candidate empty scores, and opaque base
pointer preservation. Near-u64 counts use an independent u128 oracle. Zero-scale
noisy max still breaks ties randomly, so empty/missing-group result comparisons use
singleton candidates.
