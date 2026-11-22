# Invocation-local entropy amortization experiment

Baseline optimized samplers were committed and pushed as
[`4a51edde7`](https://github.com/opendp/opendp/commit/4a51edde7) on
`origin/perf/continuous-samplers`. This follow-up is published on
[`origin/perf/continuous-sampler-entropy`](https://github.com/opendp/opendp/tree/perf/continuous-sampler-entropy),
which includes the baseline optimizations.

## Results

Same M4 Pro / Rust 1.96 release / OpenSSL private-randomness setup as the
[baseline report](README.md). The new vector benchmark measures per-element
latency for repeated vector calls. Each number is the median of five batches
of 16,384 total outputs, with three warm-up calls per configuration. Includes
output allocation and exact finalization; excludes sampler construction. These
are sampler timings, not whole-measurement or FFI timings. No CPU affinity was
set; raw min/max batch times are retained, including startup outliers.

Location 0, scale 1; microseconds per output:

| Vector length | Sampler | Pushed baseline | Buffered | Speedup |
| ---: | :--- | ---: | ---: | ---: |
| 1 | Laplace | 1.872 | 1.354 | 1.38× |
| 1 | Gaussian | 3.278 | 1.674 | 1.96× |
| 16 | Laplace | 1.838 | 1.175 | 1.56× |
| 16 | Gaussian | 3.227 | 1.546 | 2.09× |
| 256 | Laplace | 1.820 | 1.151 | 1.58× |
| 256 | Gaussian | 3.244 | 1.527 | 2.12× |
| 4096 | Laplace | 1.815 | 1.144 | 1.59× |
| 4096 | Gaussian | 3.265 | 1.520 | 2.15× |

The existing singleton benchmark independently measured about 1.35 µs for
Laplace and 1.66 µs for Gaussian. Against the main discrete baseline from the
original report (11.48 / 30.21 µs at 2^-1074 spacing), these are approximately
**8.5× / 18×** faster. The rounded continuous and rounded lattice distributions
are still distinct laws.

The vector benchmark also covers location 1e6 at scale 1, and location 0 at scale
1e-6. Full data: [`results/amortization/`](results/amortization/). `before.csv`
is the pushed baseline, `final.csv` is the chosen configuration, and
`scalar-final.csv` is the original singleton benchmark after buffering.

### Buffer-size choice

Exploratory unit-scale times (µs/output):

| Capacity | Scalar Laplace | Scalar Gaussian | Length-4096 Laplace | Length-4096 Gaussian |
| ---: | ---: | ---: | ---: | ---: |
| 64 bytes | 1.32 | 1.80 | 1.22 | 1.72 |
| 256 bytes | 1.32 | 1.65 | 1.15 | 1.53 |
| 1024 bytes | 1.42 | 1.71 | 1.13 | 1.47 |

**256 bytes** is a reasonable fixed compromise: 1024 has a small large-vector
benefit but costs more for singleton calls and retains more unused randomness.
These exploratory runs precede final cleanup; `final.csv` is the final-code run.
Unused bytes are discarded when the invocation ends. The implementation does not
choose capacity based on private input values.

### Non-OpenSSL fallback

A second buffer over the existing `rand` backend regressed singleton timings
(unit-scale Laplace 1.21 → 1.47 µs, Gaussian 1.64 → 1.83 µs) and barely helped
large vectors. Therefore the experiment **only enables buffering with
`use-openssl`**; the fallback uses direct `SystemEntropy`. Large-vector fallback
latencies remain essentially unchanged. The fallback's existing security caveat
is unchanged; this is not a recommendation to substitute it for OpenSSL.

Raw fallback runs are `rand-before.csv`, `rand-256.csv` (rejected extra-buffer
variant), and `rand-final.csv` (direct fallback). The final run has a startup
outlier in the first Laplace singleton configuration; its min/max are preserved.

## Implementation and scope

- New private `EntropySource` requires only fallible byte filling. Word, byte
  coin, and unbiased bounded-draw helpers share the existing sampling algorithms.
- `BufferedEntropy<256>` is created once per vector invocation, reused across
  elements, rejection tests, signs and final refinements. Public exact scalar
  Gaussian wrappers get one invocation-local buffer too.
- Refills use the **same configured backend**. There is no new PRNG, global/TLS
  cache, cloneable entropy state, cross-call cache, retry/precision cap, or change
  to the native Gaussian specialization.
- Empty reads are lazy. Refills publish bytes only after a successful backend
  call. Errors propagate immediately, not as rejection/resampling events; callers
  must discard partially written output destinations on error.
- PSRN refinement now draws before changing its prefix. This also fixes an
  existing error-state bug where a failed draw left a shifted numerator with an
  unchanged denominator.
- Public uniform and ordinary PSRN sampling stay system-backed. Private
  source-taking cores retain native-endian native words, big-endian UBig values,
  and the existing unbiased rejection thresholds. Positive upper bounds remain
  their precondition; pre-existing zero-bound handling is outside this change.

## Security/lifetime limitations

The source is local to an invocation and cannot be cloned. It does not survive
into subsequent calls, including calls after an ordinary between-call fork. It
is **not** fork/snapshot protection for an already-active sampling stack: such a
snapshot would also copy unread buffer bytes. Buffering retains unused future
randomness in memory, and stack-buffer drop is **not guaranteed erasure**.
No constant-time, memory-compromise, or stronger RNG security claim is added.

## Validation

- **91 sampler tests pass** with both OpenSSL and the direct fallback.
- Full library suite: **270 passed, 3 failed, 2 ignored**. The same three
  pre-existing failures are listed in the baseline report.
- New tests check byte ordering/nonreuse across unaligned and multiple refills,
  empty reads, rejected zero capacity, partial refill failure and recovery,
  mixed coin/word draws, exhaustive unbiased u8 bounded sampling, native and
  beyond-native rejection/error paths, and preservation of a nonzero PSRN prefix
  on a failed refinement.
- Scripted byte-stream tests compare complete Laplace and Gaussian output bits
  **and entropy consumption** over 1000 outputs with direct and 7-/256-byte
  sources, including ordinary/extreme locations and tiny/large scales. These
  exercise rejection and final refinement without changing the sampling trace.
- Rust formatting and whitespace checks pass. Independent read-only review found
  no correctness blockers or accidental system draws in injected exact paths.
  Suggested optional hardening is fault injection at every later fill in full
  sampler traces; current full-sampler error tests inject the initial failure.
  This experiment still needs OpenDP's formal review before production use.

## Reproduce

From `rust/` in this worktree:

```sh
CARGO_TARGET_DIR=/tmp/entropy-buffer-target cargo run --release \
  --no-default-features --features contrib,use-openssl \
  --example continuous_sampler_vector_bench -- 16384 5

CARGO_TARGET_DIR=/tmp/entropy-buffer-target cargo test --release \
  --no-default-features --features contrib,use-openssl --lib traits::samplers
```

For the pushed baseline, create a worktree at `4a51edde7`, copy only
`rust/examples/continuous_sampler_vector_bench.rs` there, and use its original
sampler source. Copy the ignored Cargo.lock to keep dependency versions fixed.
Use a **separate target directory**, as in the baseline report. Omit
`use-openssl` from features to reproduce the fallback comparison. To repeat the
capacity experiment, change the private OpenSSL `SamplerEntropy` alias in
`rust/src/traits/samplers/entropy.rs`, rebuild and rerun.
