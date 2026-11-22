# Exact discrete Laplace and Gaussian optimization

Worktree: `../openDP-discrete-samplers`, branch `perf/discrete-samplers`.
Base: freshly checked `origin/main` at `ef1a238634d6196b01a1a1e77d3d5ad2340e3094`.
Published on [`origin/perf/discrete-samplers`](https://github.com/opendp/opendp/tree/perf/discrete-samplers).
The separate continuous-sampler branches remain unchanged.

## Matched benchmark results

Apple M4 Pro, macOS 26.5.2, Rust 1.96.0, release mode,
`--no-default-features --features contrib,use-openssl`. OpenSSL private
cryptographic randomness is unchanged. Both builds use the same resolved
Cargo.lock (Dashu 0.6.2, vendored OpenSSL 3.3.1). No dependency upgrades.
Separate Cargo target directories and sequential runs; 100 warm-up samples,
then median of five batches of 10,000 samples, no CPU affinity. The same
`rust/examples/discrete_sampler_bench.rs` builds on both revisions.

**Microseconds per sample**, including scale clone and scalar sampler setup:

| Scale in integer units | Output | Law | Main | Optimized | Speedup |
| ---: | :--- | :--- | ---: | ---: | ---: |
| 0.1 | integer | Laplace | 3.322 | 0.518 | 6.41× |
| 0.1 | integer | Gaussian | 7.366 | 1.123 | 6.56× |
| 1 | integer | Laplace | 2.454 | 0.413 | 5.95× |
| 1 | integer | Gaussian | 5.853 | 0.932 | 6.28× |
| 3/2 | integer | Laplace | 2.619 | 0.509 | 5.14× |
| 3/2 | integer | Gaussian | 4.485 | 0.768 | 5.84× |
| 10/3 | integer | Laplace | 2.610 | 0.495 | 5.28× |
| 10/3 | integer | Gaussian | 4.167 | 0.721 | 5.78× |
| 10 | integer | Laplace | 2.328 | 0.469 | 4.96× |
| 10 | integer | Gaussian | 4.070 | 0.729 | 5.58× |
| 1e6 | integer | Laplace | 2.377 | 0.454 | 5.24× |
| 1e6 | integer | Gaussian | 4.245 | 0.819 | 5.18× |
| 2^53 | integer | Laplace | 2.424 | 0.451 | 5.37× |
| 2^53 | integer | Gaussian | 4.814 | 0.958 | 5.03× |
| 2^1074 | integer | Laplace | 8.468 | 0.956 | 8.86× |
| 2^1074 | integer | Gaussian | 27.025 | 3.801 | 7.11× |
| (2^1074+1)/3 | integer | Laplace | 8.847 | 1.320 | 6.70× |
| (2^1074+1)/3 | integer | Gaussian | 27.459 | 3.745 | 7.33× |
| 2^1074 | f64, grid 2^-1074 | Laplace | 11.528 | 4.510 | 2.56× |
| 2^1074 | f64, grid 2^-1074 | Gaussian | 30.003 | 7.150 | 4.20× |

Raw integer results time generation of the actual arbitrary-precision lattice
sample. `f64_grid` additionally computes `RBig::from(integer) * 2^-1074` and rounds
that exact rational to f64. Physical scale is one in those rows. There is no
location shift or vector allocation in this harness. These are not FFI or
whole-measurement timings, and vector-level parameter caching is not implemented.

The zero-scale rows are deterministic nanosecond-scale early-return checks, not
meaningful speedup estimates (about 13–19 ns in these runs). Raw min/max batch
times, including startup outliers, are retained in `results/final*.csv`.

### Consequence for the continuous comparison

The previously reported roughly 18–20× Gaussian improvement compared the
optimized continuous sampler to **unoptimized main** discrete Gaussian.
After optimizing discrete too, that is no longer the relevant gap: the earlier
continuous singleton result (~1.66 µs) versus this discrete f64-grid result
(~7.15 µs) suggests **about 4.3×**, and Laplace ~1.35 versus ~4.51 µs suggests
**about 3.3×**. Those cross-branch numbers are indicative, not a new matched
end-to-end benchmark: this discrete harness lacks the continuous harness's
one-element output allocation, and runs were separate. The continuous samplers
also target a different mathematical distribution (rounded continuous rather
than rounded lattice).

Exact rational conversion now costs more than raw fine-grid Laplace sampling
itself. Native direct integer-to-grid-float finalization is a plausible next
optimization, but is **not implemented** here; naively rounding and scaling in
f64 could introduce double rounding and is not an acceptable substitute.

### Incremental experiments

Exploratory runs used five batches of 5,000 samples and fewer scale cases; they
are preserved separately from the final matched 10,000-sample runs.

| Stage | Unit Laplace | Unit Gaussian | 2^1074 raw Laplace | 2^1074 raw Gaussian |
| :--- | ---: | ---: | ---: | ---: |
| Initial baseline | 2.50 | 6.14 | 8.56 | 26.88 |
| Buffer + Gaussian parameter hoisting | 1.16 | 2.77 | 7.72 | 25.67 |
| Masked wide-uniform rejection | 0.83 | 2.11 | 6.64 | 24.57 |
| Lazy wide rational Bernoulli | 0.69 | 1.85 | 6.32 | 22.69 |
| Unreduced exponential continuation ratios | 0.46 | 1.28 | 5.89 | 22.06 |
| Also avoid normalizing geometric u/d | 0.44 | 1.23 | 1.00 | 15.56 |
| Integer Gaussian envelope + unit-ratio shortcut | 0.80* | 0.93 | 0.95 | 3.87 |

*The last exploratory run's first configurations had timing outliers; the final
matched unit Laplace result is 0.413 µs. Stage differences are not additive
attributions of CPU costs. The main win at fine precision is **avoiding wide gcd
normalization**, not merely drawing fewer random bytes.

### Rand fallback

Buffering remains OpenSSL-only because the fallback already amortizes its RNG.
The direct rand fallback also benefits from the arithmetic changes:

| Scale/output | Law | Main µs | Optimized µs |
| :--- | :--- | ---: | ---: |
| 1/integer | Laplace | 1.065 | 0.319 |
| 1/integer | Gaussian | 2.935 | 0.933 |
| 2^1074/integer | Laplace | 7.921 | 1.050 |
| 2^1074/integer | Gaussian | 26.756 | 4.092 |
| 2^1074/f64_grid | Laplace | 10.986 | 4.415 |
| 2^1074/f64_grid | Gaussian | 30.333 | 7.505 |

The existing fallback security caveat is unchanged; this is not a recommendation
to replace OpenSSL. Raw data: `results/final-rand*.csv`.

## Changes and exactness argument

1. **Invocation-local entropy.** Port the already-reviewed 256-byte OpenSSL
   reservoir from `perf/continuous-sampler-entropy`. Each public discrete sample
   creates one lazy buffer; all internal sign, proposal and rejection draws share
   it. The rand backend stays direct. No global/TLS cache, cloneable state, new
   PRNG, retry/precision cap, or source substitution. Failed refills are never
   published; errors propagate rather than becoming rejection events.
2. **Wide uniforms.** For positive bound d, draw `bit_len(d-1)` bits, mask padding
   in the first big-endian byte, and accept iff `candidate < d`. Every retained
   integer has the same probability; acceptance is greater than 1/2 and powers
   of two never reject. Bound one returns zero without entropy. Bound zero now
   returns an error instead of the old remainder-by-zero panic; native bounds
   receive the same zero/one guard. Public wrappers remain system-backed.
3. **Lazy rational Bernoulli.** For unreduced a/d in [0,1], compare uniform-prefix
   interval `[p/2^b,(p+1)/2^b)` exactly to a/d. Return false if `p*d >= a*2^b`,
   true if `(p+1)*d <= a*2^b`, otherwise extend the **same prefix** by 64 bits.
   Arbitrary-precision integers preserve the continuation beyond native widths.
   Finite positive denominators have zero probability of an indefinitely equal
   boundary. Zero/one are deterministic; native-size denominators use unbiased
   bounded native draws. No approximate floating-point probability is used.
4. **Unreduced exponential ratios.** The existing continuation sequence has
   probabilities `x/k = a/(d*k)`. Use those integer pairs directly instead of
   constructing and reducing RBig at every step. Parity on the first failure is
   unchanged, so the success probability remains exp(-x). For x>1, factor unit
   exp(-1) coins with early failure, then use the exact residual. Unit ratios d/d
   use 1/k directly rather than carrying a wide common factor.
5. **Geometric proposal.** CKS's uniform integer u in [0,d) already provides the
   ratio u/d; the exp(-u/d) acceptance coin does not need RBig normalization.
   Quotient/output and the rest of the existing geometric construction remain
   unchanged, including its explicitly tested zero-rate convention.
6. **Integer Gaussian envelope.** For sigma=a/b>0 and t=floor(sigma)+1, the
   unchanged CKS exponent is

   ```text
   (|c| - sigma²/t)² / (2sigma²)
     = (|c| b² t - a²)² / (2 a² b² t²).
   ```

   Precompute a², b²t, 2a²b²t², and the proposal inverse once per sample call.
   Candidate evaluation needs signed-integer subtraction/square and an unreduced
   exponential coin, rather than rational gcds and division. The original
   signed Laplace proposal and **negative-zero rejection** remain unchanged;
   keeping both signs of zero would change the target law. No continuous-noise
   rounding shortcut was introduced.

Negative discrete scales now return errors explicitly. The Laplace PMF's reversed
numerator in its source documentation was corrected; the distribution is unchanged.
Existing proof-linked scalar entry points remain, but **historical proof documents
need revision**: in particular Bernoulli and wide-uniform `.tex` proofs describe
the previous integer-draw/modulo implementations. Tests, algebraic correspondence,
and source review here do not constitute formal OpenDP proof approval.

The entropy buffer retains unused future randomness in process memory; stack drop
does not guarantee erasure. Invocation-local state does not survive between calls,
but does not protect against snapshots/forks of an already-active sampling stack.
No constant-time or stronger RNG-security claim is made.

## Validation

Both OpenSSL and rand configurations pass the full minimal Rust library suite:
**280 passed, 0 failed, 2 ignored**, plus 5 tooling tests pass. Unmodified main
passes 258, fails zero, ignores two with the same configuration. With normal
default features plus `contrib`, **297 pass, 0 fail, 2 are ignored** (plus the
5 tooling tests). This main-based worktree does not have the continuous branch's
three known baseline failures.

New deterministic and regression tests cover:

- Exhaustive masked candidates at byte boundaries and wide 2^k−1/2^k/2^k+1 bounds.
- Native bounded u8 acceptance counts, zero/one bounds, and rejection followed by
  success or error for native/wide draws.
- Bernoulli exact boundaries, unreduced ratios, prefixes forced through 1,075
  bits, and error propagation while overlapping a boundary.
- Exact Gaussian envelope equality to the old rational expression at fractional
  and fine-grid scales, on positive/negative/zero candidates.
- Laplace sign and negative-zero correction, scripted Gaussian rejection/retry,
  and fault injection at **each successive fill** of complete sample traces.
- Identical outputs and consumed-byte positions for direct/7-byte/256-byte
  scripted sources, plus ported buffer-order/refill-failure tests.
- Fractional-scale Gaussian moments/zero mass against the discrete PMF, in
  addition to existing exponential/geometric/Laplace/Gaussian statistical tests.

Rust formatting and whitespace checks pass. Independent read-only implementation
review found no blockers; it flagged proof-document updates and careful separation
of raw integer vs float-output benchmark claims, both recorded above.

## Reproduce

From `rust/` in this worktree:

```sh
CARGO_TARGET_DIR=/tmp/discrete-opt-target cargo run --release --locked \
  --no-default-features --features contrib,use-openssl \
  --example discrete_sampler_bench -- 10000 5

CARGO_TARGET_DIR=/tmp/discrete-opt-target cargo test --release --locked \
  --no-default-features --features contrib,use-openssl --lib
```

For baseline, create a detached worktree at `ef1a23863`, copy only
`rust/examples/discrete_sampler_bench.rs` (create the directory) and the ignored
`rust/Cargo.lock`, then run the same commands with a **different target directory**.
To reproduce fallback runs, omit `use-openssl`. Optional default values are 5,000
samples per batch and five batches. Raw CSV is in nanoseconds.

No scalar/vector measurement API was refactored to retain prepared parameters
between calls. A future vector-level prepared sampler could amortize the envelope
setup and entropy reservoir across outputs; it needs a distinct API/integration
change and is not included in these results.
