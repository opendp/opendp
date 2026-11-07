# Resolved Polars plan splitting

The internal `split_plan` returns two native Polars plans:

- A `DslPlan::SinkMultiple` of deterministic IPC sinks, even for one head.
- A continuation with IPC scans, randomized operations and final results.

The already-analyzed input preceding the private Select/GroupBy stays opaque and
unchanged. Splitting performs no execution, I/O or schema resolution. One allocator
reserves names across outputs, keys and public schemas before naming intermediates.

## Logical histogram boundary

Trusted `NoisePlugin` and `NoisyMaxPlugin` occurrences identify randomized
boundaries. Compatible scalar inputs share a deterministic head. Each resolved
`DiscreteQuantileScorePlugin` gets an independent logical sparse-histogram head;
there is no statistic-specific planner, validity pass or deduplication.

`DiscreteQuantileScorePlugin` retains its original fixed-size score-array contract.
The separate internal `DiscreteQuantileHistogramPlugin` carries its complete
resolved parameters and unchanged input expression, including casts, replacements,
and aggregation-local filter/drop operations. It returns sparse UInt64 bucket/count
structs. For k candidates, 2k+1 buckets distinguish equality from the adjacent open
intervals. Native reference execution counts directly with exact typed comparisons;
it does not construct a list of per-observation bucket IDs.

The native logical head aggregates in the original Select/GroupBy context and
flattens only its O(k) sparse struct output. This is an aggregate relation contract,
not SQL lowering. The splitter contains no candidate comparison tree or binning
implementation. `polars-to-ibis` owns the lowering and chooses its SQL binning:
contribution rows are counted directly with GROUP BY original keys plus bucket ID.
SQL must not materialize lists, ARRAY_AGG/UNNEST, or regroup exploded observations.
The translator recognizes the whole logical aggregate/flatten relation rather than
erasing the FFI function or interpreting sparse flattening as SQL list operations.

The FFI symbol is `discrete_quantile_histogram_plugin`, with kwargs:

```text
{scorer: {candidates: Polars Series IPC bytes, alpha: [u64,u64], size_limit: u64},
 bucket_name: string, count_name: string}
```

Observed groups whose input contributes no values retain one zero-count histogram
row. The SQL lowering preserves the original private observed-key domain with a
null-safe alignment where needed; this is not public-key sanitization. Grouped empty
base input has no observed groups. A global aggregation still produces one result,
including on empty input. Literal inputs contribute once per group, or once globally
even on an empty base, not once per source row. Retaining these semantics ensures
black-box head joins never discard valid results from other heads.

## Local continuation and shared key release

After IPC, the continuation groups by original keys only. The existing
`HistogramQuantileScorePlugin` fills omitted bins with zero and reconstructs the
original fixed-size UInt64 scores. It shares exact score arithmetic with the
ordinary scorer: total/prefix counts stay u64, only strict tails are clipped, and
alpha constants, size limit and overflow behavior are unchanged. Original noisy-max
and candidate-lookup plugins then run in Polars.

Maximal single-head expression subtrees run locally; cross-head expressions wait
above null-equal keyed joins (or one-row global cross joins). Scalar literal noise
retains its one-draw broadcasting semantics. No privacy accounting changes.

The original shared threshold Filter, public-key Join/Distinct and missing-group
fill remain above the combined results. Public post-join imputation handles truly
absent private groups exactly once; it is not independently applied to each head.
Public in-memory frames become independent IPC heads. No head reads another head.

The continuation has no populated DataFrame scans, but sink serialization may embed
base/public DataFrames. Both plans and intermediates remain executor-only. Trusted
OnceFrame integration must protect files, maintain snapshots, enforce one-shot use,
shuffle final results and clean up; this change does not implement that executor.
Multiple private Select/GroupBy boundaries remain unsupported. Anonymous Rust UDFs
retain serialization limitations; cross-process use needs FFI and a matching library.

## Paired translator and tests

The paired `polars-to-ibis` implementation lives in the sibling checkout on
`feat/opendp-sparse-histogram` (tested commit `c2ac27cc126b4fe7ad2c3547d415168a138c2f6e`).
In addition to the logical histogram relation, it
must preserve typed literals, preprocessing masks, null-key equality, literal/global
cardinality and UInt64 IPC values. The old Python `run_on_database` scalar bridge is
not an executor for this protocol.

From `rust/`, with Python Polars **1.36.1**:

```sh
cargo fmt --check
cargo test --no-default-features --features polars,untrusted polars::split::test
cargo test --no-default-features --features polars,untrusted quantile
cargo build --release --no-default-features --features polars-ffi,untrusted
```

For actual translator execution, install the sibling converter plus Ibis/DuckDB in
an isolated Python environment. The ignored integration test serializes the actual
Rust-generated head relations, calls public `convert_polars_to_ibis`, executes their
SQL, checks exact sparse counts and absence of SQL list materialization, writes IPC,
and executes the native continuation against unsplit zero-noise results. It covers
mixed/local/cross-head expressions, filtering/drops and zero-contribution groups,
null keys, null/NaN replacement, literals, large integers, empty inputs, public-key
imputation/deduplication, thresholding and naming collisions. It also exercises
resolved non-strict overflow casts and Float32 dynamic fills/intermediate arithmetic,
so a final output cast cannot hide changed bucket comparisons.

```sh
PYTHON=/path/to/venv/bin/python \
OPENDP_POLARS_LIB_PATH="$PWD/target/release/libopendp.dylib" \
  cargo test --no-default-features --features polars-ffi,untrusted \
  polars::split::test -- --include-ignored
```

Use `.so`/`.dll` on other platforms. Native exact-score tests additionally cover
candidate ties/gaps, alpha extremes, clipping, integer/float types and preprocessing;
near-u64 counts use an independent u128 oracle. Zero-scale noisy max still breaks
ties randomly, so empty/missing-group result comparisons use singleton candidates.
