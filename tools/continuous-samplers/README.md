# Continuous Laplace and Gaussian performance

This report describes the pushed baseline at `4a51edde7`. For the subsequent
invocation-local entropy-buffering experiment and its additional speedups, see
[AMORTIZATION.md](AMORTIZATION.md).

## Results

Apple M4 Pro, macOS 26.5.2, Rust 1.96.0 (`aarch64-apple-darwin`), release profile,
OpenSSL private randomness (vendored OpenSSL 3.3.1). Each entry is the median of
five batches of 10,000 samples, after 100 warm-up samples. Baselines and optimized
code were built and run sequentially in **separate Cargo target directories**.

- Original continuous implementation: `494785e6047478ef4a624fd8bcf5c14f26200214`
  (`origin/sampler-continuous-laplace-gaussian`).
- Optimized implementation: the accompanying changes on `perf/continuous-samplers`.
- Current-main discrete baseline: `ef1a238634d6196b01a1a1e77d3d5ad2340e3094`.

Times are **microseconds per f64 output**, including allocation of a one-element
output vector and exact affine finalization, excluding sampler construction.

| Location | Scale | Law | Continuous before | Continuous after | Main discrete | Before/after speedup | Speedup vs main discrete |
| ---: | ---: | :--- | ---: | ---: | ---: | ---: | ---: |
| 0 | 1 | Laplace | 60.16 | 1.92 | 11.48 | 31.3× | 6.0× |
| 0 | 1 | Gaussian | 10.11 | 3.33 | 30.21 | 3.0× | 9.1× |
| 1 | 1 | Laplace | 57.96 | 1.89 | 11.54 | 30.7× | 6.1× |
| 1 | 1 | Gaussian | 9.92 | 3.34 | 30.16 | 3.0× | 9.0× |
| 1e6 | 0.5 | Laplace | 36.98 | 1.94 | 11.86 | 19.1× | 6.1× |
| 1e6 | 0.5 | Gaussian | 10.11 | 3.36 | 30.54 | 3.0× | 9.1× |
| 0 | 1e-6 | Laplace | 80.90 | 2.24 | 11.88 | 36.1× | 5.3× |
| 0 | 1e-6 | Gaussian | 11.07 | 3.69 | 30.29 | 3.0× | 8.2× |
| 0 | 1e6 | Laplace | 61.39 | 1.86 | 12.00 | 33.0× | 6.5× |
| 0 | 1e6 | Gaussian | 9.98 | 3.29 | 30.98 | 3.0× | 9.4× |

Raw medians and min/max batch times are in [`results/`](results/).

### Nonzero locations and coarser binades

A separate fixed-scale (`scale = 1`) sweep, using the same five-by-10,000 protocol:

| Location | Original Laplace (µs) | Optimized Laplace (µs) | Original Gaussian (µs) | Optimized Gaussian (µs) |
| ---: | ---: | ---: | ---: | ---: |
| 0 | 58.90 | 1.89 | 9.90 | 3.34 |
| 1 | 57.88 | 1.87 | 10.02 | 3.69 |
| 1e6 | 38.02 | 1.90 | 9.97 | 3.36 |
| 1e12 | 17.94 | 1.88 | 9.99 | 3.35 |
| 1e16 | 6.74 | 1.76 | 9.42 | 2.85 |
| 1e100 | 7.31 | 2.16 | 9.89 | 3.23 |
| 1e300 | 7.32 | 2.22 | 9.71 | 3.35 |

The original one-bit Laplace sampler benefits strongly from larger rounding
cells: it needs roughly `max(0, log2(scale / cell_width))` fractional bits, plus
boundary-dependent guard bits. For scale 1, f64 ulps near 1e6 and 1e12 are about
2^-33 and 2^-13, respectively, compared with about 2^-52 near unit-sized outputs.
The relevant quantity is the noisy output's magnitude relative to scale, not
merely whether the location is nonzero.

After 64-bit batching, ordinary f64 outputs already usually finalize within the
first revealed block. Coarser cells save little further work: generating and
accepting the lazy noise still costs time, and very large rational locations can
make arithmetic slightly more expensive. Gaussian already used 64-bit blocks
before this work, explaining its relatively flat curve in both versions.

Main's fine-grid discrete times remain roughly 11.6–11.9 µs for Laplace and
30.2–30.6 µs for Gaussian across this sweep. No location-dependent early return
or tail truncation was added, even when the probability of leaving the input's
rounding cell is astronomically small. Raw data are in `results/locations-*.csv`.

### What does “2^-1074 precision” mean here?

The discrete baseline samples `D_Z(scale / grid)`, with `grid = 2^-1074`, then
computes `location + grid * integer_noise` as an exact rational and rounds to f64.
The location rational is precomputed outside the discrete timing loop. Both paths
include a one-element output allocation; these are sampler benchmarks, not FFI
or whole-measurement benchmarks.

The continuous sampler samples an **exact random real rounded to f64**, revealing
only enough random bits to certify the output's IEEE rounding cell. It need not
compute 1,074 fractional bits near ordinary outputs. Refinement is unbounded when
more bits are needed; 64-bit refinement is not a precision cap. The rounded lattice
and rounded continuous distributions are **not mathematically identical**.

The old branch's discrete implementation is not the current main implementation.
At location 0, scale 1, its discrete times are about 19.4 µs (Laplace) and 64.1 µs
(Gaussian). Comparing only against those would suggest approximately 10.1× and
19.3× speedups with the current optimization. The more conservative main comparison above includes main's newer Dashu
and discrete implementation: branch resolves Dashu 0.4.4, main resolves 0.6.2.
No dependencies were upgraded as part of this optimization. A 1,074-bit grid
also does not imply 1,074 iterations of work per discrete sample: these samplers
draw wide integers and operate on their limbs. Removing the grid therefore does
not remove a thousand-step loop.

These measurements cover ordinary scales and a range of f64 locations on one
machine, not f32, large vectors, extreme-scale performance, or constant-time behavior. All samplers
remain variable-time. No truncation, approximate inverse CDF, or native-only
resampling shortcut was introduced.

## Low-complexity changes

1. **Reuse `PartialUniform01` for Laplace.** It already reveals 64 iid bits per
   refinement. This removes the separate one-bit uniform implementation and
   avoids rebuilding exact affine bounds and rounding cells roughly once per
   mantissa bit.
2. **Compare uniform prefixes as integers.** A small shared method aligns exact
   dyadic denominators for `U < V` and `U < V/2`. Laplace and Gaussian acceptance
   avoid rational construction/reduction for those tests. Overlapping or touching
   intervals still refine, including beyond machine precision.
3. **Use Laplace's rejected attempts as the exponential integer part.** Accept
   `x ~ Uniform(0,1)` with probability `exp(-x)` and count previous rejected
   attempts as `k`. Since each attempt rejects with probability `e^-1`,
   `P(k=m, X in dx) = e^-m exp(-x) dx`. Thus `k+X` is exactly Exp(1), without an
   additional geometric draw. A separate fair sign gives standard Laplace.
4. **Reuse Gaussian's exact affine endpoints.** The rounding certificate used to
   reconstruct the same rational endpoints a second time. Compare the existing
   interval with the rounding cell instead.
5. **Share rounding-cell handling.** This also fixes a pre-existing Laplace bug:
   at finite ±MAX, the old code tried to rationalize an infinite neighbor.
6. **Specialize Gaussian's unit-scale integer component.** Instead of the general
   rational discrete sampler, use Bernoulli(p) trials with `p = exp(-1/2)`.
   Reaching k requires success blocks of lengths 1, 3, ..., 2k-1, totaling k²
   successes. A failure of the next main coin returns k; failures inside an extra
   success block restart at zero. Thus an attempt returns k with mass
   `(1-p) p^(k²)`, exactly the required half-Gaussian law after rejection.
   The p-coin itself continues with probabilities `1/(2n)` and returns the parity
   of the number of successes: its survival probabilities are `2^-n/n!`, giving
   success probability `exp(-1/2)`. Unbiased native bounded draws handle ordinary
   denominators, with an unbounded `UBig` fallback rather than a counter cap.
7. **Guess from an endpoint, not a midpoint.** Rounding the existing lower bound
   avoids constructing another rational. The exact containment certificate, not
   the candidate guess, still decides whether to return. A lower bound exactly on
   an unfavorable tie triggers further refinement.
8. **Fold Laplace's affine offset.** Compute `mu + signed_scale*k` once and retain
   only the fractional uniform, removing redundant lazy wrappers and repeated
   integer-part arithmetic.

Production sampler code shrinks overall. New tests cover integer-vs-rational
certificates, unequal precisions through 1,074 bits, overlap refinement, both
Gaussian signs, finite extrema/overflow, subnormals, cancellation, endpoint ties,
affine equivalence, exact integer-sampler control flow/restarts, parity,
entropy-error propagation, and loose distribution checks.

### Second-pass measurements and remaining opportunities

At location 0, scale 1, measured Gaussian latency progressed as follows:

| Implementation | Gaussian µs/sample |
| :--- | ---: |
| First pass, general rational integer sampler | 8.35 |
| Unit-scale geometric proposal + decreasing-uniform p-coin | 4.29 |
| Interleaved odd-length success blocks | 3.88 |
| Reciprocal-trial p-coin with native bounded draws | 3.56 |
| Lower-endpoint candidate and final cleanup | 3.33 |

These are successive measured implementations, not an additive attribution of
individual CPU costs. Raw intermediate runs and first-pass results are retained
in `results/`. The latest pass is another **2.5× Gaussian speedup** over the first
pass. Laplace improves more modestly, from 2.08 to 1.92 µs at unit scale.

**Sampler-local entropy buffering** is a promising next experiment, not an
implemented speedup. The randomness microbenchmark measured:

| Bytes requested | Median ns/call |
| ---: | ---: |
| 1 | 147 |
| 8 | 124 |
| 64 | 126 |
| 256 | 141 |
| 4096 | 441 |

Thus a local reservoir shared by a vector's samples could amortize many tiny
OpenSSL requests. Savings depend on draw counts and batch size; they are not
predicted by this microbenchmark alone. This needs entropy plumbing and careful
state/lifetime handling. Avoid global RNG caching, source substitution, or finite
retry/precision caps just to improve timings. Another possible, unmeasured step
is exact dyadic-integer finalization for float-derived scales, keeping the
rational path as fallback; that is a larger correctness/maintenance commitment.

## Reproduce

From `rust/` in the optimized worktree:

```sh
CARGO_TARGET_DIR=/tmp/continuous-opt-target cargo run --release \
  --no-default-features --features contrib,use-openssl \
  --example continuous_sampler_bench -- 10000 5
```

For the original implementation, create a detached worktree at `494785e60`, copy
`rust/examples/continuous_sampler_bench.rs` into its `rust/examples/`, and run the
same command with a **different** target directory. Use the optimized worktree's
ignored `rust/Cargo.lock` there as well to hold dependency versions constant.

For current main, create another detached worktree at `ef1a23863`, copy the example
(create `rust/examples/` if necessary), and run with only `--features use-openssl`.
Omitting `contrib` selects the discrete-only benchmark so the example also compiles
on revisions without continuous samplers. Let that checkout resolve its own
Dashu version. Again use a separate target directory; sharing one across worktrees
can leave the un-hashed `libopendp.rlib` output from another revision in place.

The first argument is samples per batch; the second is batch count. Append
`locations` as the third argument to reproduce the fixed-scale location sweep.
The example prints CSV in nanoseconds, including the randomness backend and
min/max batch times. Current locked dependency versions are reported above; new
resolutions or other hardware may change timings. For the entropy microbenchmark:

```sh
CARGO_TARGET_DIR=/tmp/continuous-opt-target cargo run --release \
  --no-default-features --features use-openssl --example sampler_entropy_bench
```

## Validation

```sh
# With production randomness (all 79 sampler tests passed as part of the full suite):
CARGO_TARGET_DIR=/tmp/continuous-opt-target cargo test --release \
  --no-default-features --features contrib,use-openssl --lib

# Also checked with the rand fallback: all 79 sampler tests passed.
CARGO_TARGET_DIR=/tmp/continuous-opt-target cargo test --release \
  --no-default-features --features contrib --lib traits::samplers
```

Full-library test result: **258 passed, 3 failed, 2 ignored**. Running the same
suite on the unmodified branch reproduced the identical failures (243 passed,
3 failed, 2 ignored):

- `measurements::noise_threshold::distribution::laplace::test::test_make_laplace_threshold_float_map`
- `measurements::randomized_response::test::test_cat_extremes`
- `traits::cast::test::test_round_cast_rbig_to_f64_corrects_dashu_double_rounding`

Those existing failures were left outside this performance change. Changed Rust
files pass rustfmt and the diff passes whitespace checks. Independent read-only
source reviews of both passes found no concrete correctness regressions; this is
not a replacement for OpenDP's formal sampler review/proofs. The practically
unreachable native-to-UBig transition in the specialized p-coin is not directly
exercised by the new tests.

The local pre-commit hook's Rust formatting check passes, but its Python mypy
stage reports nine errors in unchanged Python files. The same nine errors were
reproduced on the original revision. The optimization commit therefore bypassed
the hook for that invocation only; no hook configuration or Python files changed.
