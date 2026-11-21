# PCA eigenbasis sampler benchmark

`../../benchmark_pca_eigenvector.py` is standalone and runs against either checkout.
No instrumentation is added to the production sampler.

## Reproduction

Use one Python environment with NumPy, randomgen, SciPy and OpenDP's Python
requirements installed. Build the base revision's shared library and generated
Python bindings, then use that same library and bindings for both checkouts (the
optimization changes Python only):

```sh
git worktree add --detach /tmp/opendp-pca-baseline 9e02e1bd6143a17fa42d8a558a40ef81b42a751a
# In a checkout of the base revision:
(cd rust && cargo build --release --no-default-features --features bindings,untrusted)
export OPENDP_LIB_DIR="$PWD/rust/target/release"
# Generated Python bindings are ignored by git; copy these to the other worktree:
for name in _data _internal accuracy combinators core domains measurements measures metrics transformations; do
    cp "python/src/opendp/$name.py" "/tmp/opendp-pca-baseline/python/src/opendp/$name.py"
done
PYTHON="$PWD/.venv-bench/bin/python"  # or your environment's absolute Python path
SCRIPT="$PWD/tools/benchmark_pca_eigenvector.py"
"$PYTHON" "$SCRIPT" --source /tmp/opendp-pca-baseline --output baseline.json
"$PYTHON" "$SCRIPT" --source "$PWD" --output optimized.json
```

All BLAS thread limits are set to one before importing NumPy, in both processes.
The script records Python/NumPy/BLAS/CPU details, source/library/script hashes,
matrix hashes, revision and any uncommitted sampler diff. The recorded source
hash allows identifying the implementation even after amending the results into
the same development commit.

## Method

- Full Cartesian product of dimensions 4, 16, 64, 128, 256; concentrations
  0, 10, 1000; isotropic, moderate, strong spectra.
- Isotropic: identity. Moderate: eigenvalues linearly spaced from 0 to 1.
  Strong: one eigenvalue 1, all others linearly spaced from 0 to 0.01.
- Nonisotropic matrices use a dense orthogonal basis obtained by QR of a public,
  seeded Gaussian matrix (`default_rng(d)`). Matrices are explicitly symmetrized.
  Matrix construction is outside timing; hashes must match between runs.
- Three warmups, then 31 end-to-end **measurement invocations**, with the genuine
  OpenDP CSPRNG and no counting wrapper. Includes eigenanalysis, parameter search,
  all rejection attempts, rotation and Python/FFI output construction. Excludes
  measurement constructor and imports. Report median and interquartile range
  (Q25–Q75), in milliseconds. Raw timings are retained in JSON.
- Separately, 128 invocations use a counting wrapper over NumPy `default_rng`,
  seeded with `2026 + d` per case. Record every proposal count and the aggregate
  acceptance rate (128 / total proposals). This is diagnostic only; production
  randomness is unchanged. The random streams are not coupled across samplers:
  eigenbasis/SVD coordinates differ, so samples or counts need not be identical.
- Each case has a 120-second budget, recorded explicitly if exceeded rather than
  extrapolated. POSIX alarms are delivered after a running native call returns.
- Cases run sequentially, baseline first and optimized second, on the same host.
  Results describe this machine/environment, not universal performance bounds.

## Distribution and numerical qualification

In an orthonormal eigenbasis, `A = diag(a)` with
`a = concentration * (lambda_max - lambda)` and `Omega = diag(1 + 2*a/b)`.
Scaling independent standard normals by `1/sqrt(omega)` and normalizing gives
exactly the original ACG proposal after rotation. Its inverse density contributes
`(w.T @ Omega @ w) ** (d/2)` to the rejection probability. The envelope constant
is unchanged; computing its logarithm avoids overflow in its power/exponential
representation. Rotating only the accepted vector preserves the Bingham target.

This remains a continuous sampler under OpenDP's **idealized-numerics** feature.
It is not a finite-precision DP-safe sampler. Eigenanalysis, normalization, normal
and uniform sampling and acceptance comparisons all retain floating-point error.
Explicit checks can detect invalid numerical states, but do not certify privacy.

## Results

See [RESULTS.md](RESULTS.md) for all 45 cases and numerical qualifications, and
`baseline.json` / `optimized.json` for raw measurements and full environment
configuration. The baseline was captured before editing the sampler. The same
script, shared-library and public-matrix hashes were verified across both runs.
`baseline.txt` / `optimized.txt` retain run transcripts, including baseline warnings.

## Validation

With the shared library built above, using the same Python environment:

```sh
export PYTHONPATH="$PWD/python/src"
"$PYTHON" -m pytest python/test/extras/test_eigenvector_numpy.py python/test/extras/test_pca_numpy.py -q
.venv-bench/bin/ruff check python/src/opendp/extras/sklearn/_make_eigenvector/__init__.py python/test/extras/test_eigenvector_numpy.py tools/benchmark_pca_eigenvector.py
(cd python && ../.venv-bench/bin/mypy --follow-imports=silent src/opendp/extras/sklearn/_make_eigenvector/__init__.py test/extras/test_eigenvector_numpy.py)
cargo fmt --manifest-path=rust/Cargo.toml --all --check
git diff --check
```

- Base: **101 tests passed**; optimized: **109 tests passed**. Coverage includes
  the retained von Mises distributional test, deterministic acceptance and
  rejection, zero concentration, repeated eigenvalues, dense rotation, float32
  output conversion, large isotropic matrices, log-envelope overflow avoidance,
  and explicit failure on invalid envelopes.
- Expanded validation also ran `test_eigenvalues_numpy.py`,
  `test_ext_domains_numpy.py`, and `test_xTx_numpy.py`: **144 passed, 1 failed**
  including the 109 targeted tests. The failure is pre-existing and reproduced
  on the untouched base (**35 passed, 1 failed** for those three extra files):
  `test_eigenvalues_nuclear_lipschitz` subtracts two list outputs in
  `test_eigenvalues_numpy.py:38`, raising `TypeError`. Neither that test nor the
  eigenvalue transformation is modified by this sampler optimization.
- Ruff, targeted mypy with imports followed silently, cargo formatting and diff
  checks pass. No type/lint suppressions were added.
- Full mypy 1.18.2 (`cd python && ../.venv-bench/bin/mypy .`), following all
  imports, has **one identical pre-existing error on both checkouts**:
  `src/opendp/extras/sklearn/linear_model/_logistic_regression.py:110`, incompatible
  assignment of `type[float64]` to `type[signedinteger[_64Bit]]`. It is unrelated
  to PCA and is not changed here. Targeted mypy with normal import following
  encounters that same dependency error, with none in the changed files.
- Pyright 1.1.411 with `python/pyproject.toml` reports 24 existing diagnostics on
  the base targeted files and 15 on the optimized files: untyped dynamic Domain
  descriptors and optional-dependency return types in unchanged projection and
  composition code. The sampler narrows its NumPy dependency without ignores.
- The local pre-commit hook invokes system mypy 2.3.0 rather than the benchmark
  environment and initially reported 23 errors (missing stubs/dependencies and
  unrelated files). Rechecking in the actual environment reduced this to the
  single independently reproduced base error above. Commits used a one-command
  hook override after manual checks; no hooks or persistent configuration changed.
- Independent read-only review found no production correctness blockers and
  identified the baseline overflow qualification now recorded in RESULTS.md.

The first optimized benchmark exposed roundoff amplified by `d/2` for large
isotropic inputs. The acceptance validity tolerance now includes this scale;
large isotropic tests and the final full benchmark pass without warnings. The
checks do not certify numerical privacy: tiny positive log probabilities within
roundoff are allowed, larger positives/non-finite probabilities fail explicitly.
Extreme spectra/concentrations can still overflow eigenvalue-gap/proposal
arithmetic (which raises rather than clipping or retrying indefinitely).
