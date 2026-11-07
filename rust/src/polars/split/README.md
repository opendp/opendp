# Resolved Polars plan splitting

The internal `split_plan` accepts a resolved OpenDP plan and returns two native
Polars plans:

- A `DslPlan::SinkMultiple` of deterministic IPC file sinks, even for one head.
- One continuation containing IPC scans, randomized operations and final results.

The input subtree preceding the private Select/GroupBy has already passed
stability analysis. Every private deterministic head references that unchanged,
opaque base. A split-wide allocator reserves expression names and declared public
schemas before generating intermediate names. Output names and join keys stay
unchanged. Splitting performs no execution, I/O, or schema resolution.

## Boundaries and local continuations

Trusted `NoisePlugin` and `NoisyMaxPlugin` occurrences identify randomized
boundaries. The splitter trusts existing privacy/stability analysis; it does not
validate deterministic operators or postprocessing, or change privacy accounting.
Compatible scalar aggregate inputs share a head, including deterministic operands
already established to be public. There is no statistic-specific planner or
cross-statistic deduplication.

Each resolved `DiscreteQuantileScorePlugin` occurrence gets an independent
histogram head. A balanced comparison tree assigns buckets directly to the
original scorer input in its original Select/GroupBy aggregation context. This
preserves expression-local filter/drop_nulls/drop_nans semantics, not just
row-preserving preprocessing. For k sorted candidates, the 2k+1 typed buckets
distinguish each equality from adjacent open intervals; CASE depth is O(log k).

Native implode collects one bucket list per original group. Explode retains an
empty-list null sentinel, so an observed group with no remaining observations
is not lost. A second GroupBy uses original output key columns plus the bucket
(with null mapped to typed zero), and counts the original non-null bucket values.
The sentinel therefore contributes bucket zero/count zero. Resulting counts are
cast to UInt64; SQL never computes the scores. This same structural construction
handles literals and empty global inputs without operator/cardinality classifiers.

The histogram's local continuation reconstructs zero-count bins and the original
fixed-size UInt64 score array. Its aggregation UDF and the ordinary scorer share
exact score arithmetic: prefixes and total counts remain u64, and only the strict
less/greater tails are clipped by the original size limit before multiplication
and abs_diff. Existing alpha constants and overflow behavior are retained. The
original noisy-max and candidate-lookup plugins then execute in Polars.

Postprocessing placement follows expression dependencies. Maximal expressions
belonging to one head run in its local continuation; cross-head expressions run
above the joins. Each completed head is a one-row-per-observed-group black box.
Results combine on the original keys with null-key equality, or by a one-row
cross join for ungrouped queries. Empty ungrouped inputs still produce one row;
grouped reconstruction retains observed groups even when buckets are absent.

One narrow structural distinction preserves Polars' literal semantics. Trees
whose terminal nodes are all scalar literals stay scalar around noise, rather
than becoming columns with independent draws per group. A literal input to the
scorer represents one observation per observed group, or one global observation
even for an empty input; its original aggregation context is retained by the
generic implode/explode construction above, not a literal-specific histogram
branch. Nullary context inputs such as len() are not literals. This is not an
operator whitelist or a second validity analysis.

## Shared key release and trust

The original threshold Filter, public-key Join/Distinct and missing-group fill
remain above the combined private result. They are not duplicated per head.
Randomized fill expressions intentionally execute after the public-key join.
Public in-memory frames become additional independent IPC-writing heads. No
head reads another head's output.

The continuation contains no populated DataFrame scans, but serializing a sink
plan can still embed base/public DataFrames. Both plans and private intermediates
must remain executor-only. Trusted OnceFrame integration must enforce protected
files, consistent snapshots, one-shot execution, final shuffling and cleanup;
this change does not implement that executor.

Independent quantiles are supported, not multi-quantiles or shared histogram
optimization. Multiple private Select/GroupBy boundaries remain unsupported.
Anonymous Rust UDFs retain their serialization limitations; cross-process
continuations need FFI-backed plugins and a matching rebuilt OpenDP library.
Quantile/median desugaring preserves the caller's Rust/FFI representation.

## Ibis conversion requirements

No SQL compatibility checks or Ibis executor are added. The external
`polars_to_ibis` converter is not present in this checkout/environment, so its
current dispatch coverage has not been verified. The generated deterministic
plans require these conversions:

- Original Select/GroupBy aggregation context, including expression-level
  filtering/dropping, null/NaN replacement and casts. Native bucket implode/list
  collection must preserve one list per observed group, including empty lists.
- Typed candidate scalar comparisons and nested ternary/CASE expressions without
  coercing large integers through floating point.
- Native list Explode/SQL UNNEST with empty-list/null-sentinel retention. An inner
  unnest that discards empty lists is not equivalent.
- GroupBy on original output keys plus the null-filled bucket, with COUNT(nonnull
  original bucket), **not COUNT(*)**; exact UInt64 values at the IPC boundary.
- Literal-only global input represents one observation even when the base is
  empty. Preserve native constant projection/aggregation semantics and file sinks.

The grouped collection and sentinel-preserving explode/unnest surface needs
converter support; no equivalent SQL lowering was verified here. This is an
explicit converter requirement, not SQL validation inside the splitter.

Fixed-size arrays, exact score arithmetic, noisy max, candidate lookup and all
other randomized operations stay in the Polars continuation, not SQL. The current
Python `run_on_database` scalar bridge is not an executor for this protocol.

## Tests

From `rust/`, with Python Polars **1.36.1** (`PYTHON` overrides the executable):

```sh
cargo fmt --check
cargo test --no-default-features --features polars,untrusted polars::split::test
cargo test --no-default-features --features polars,untrusted quantile
```

Native IPC sinks execute only in tests using the already-enabled Rust engine.
Tests compare complete zero-noise results with unsplit OpenDP plans and compare
histogram-backed score arrays exactly with the ordinary scorer, including ties,
alpha extremes, preprocessing, integer/float types, empties and clipping. Actual
resolved filter/drop_nulls/drop_nans tests cover partial/all removal, retained
zero-count groups, mixed heads and shared public-key imputation. Near-u64
count tests use an independent u128 arithmetic oracle. Zero-scale noisy max can
still randomly break ties; deterministic empty/missing-group comparisons use a
singleton candidate set.

Serialization tests serialize both stages before IPC exists, materialize the
restored sinks, and execute the continuation in Python. Rebuild the matching
plugin for the histogram scorer before enabling ignored FFI tests:

```sh
cargo build --release --no-default-features --features polars-ffi,untrusted
OPENDP_POLARS_LIB_PATH="$PWD/target/release/libopendp.dylib" \
  cargo test --no-default-features --features polars-ffi,untrusted \
  polars::split::test -- --include-ignored
```

Use the platform-specific `.so` or `.dll` library path where appropriate.
