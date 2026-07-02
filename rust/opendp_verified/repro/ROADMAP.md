# Sampler pipeline: bit flips → discrete Gaussian

This is the end-to-end chain the `opendp_verified` proofs are climbing, from raw
hardware randomness up to the discrete Gaussian mechanism used for
(zero-concentrated) differential privacy. Each stage is built **only** from the
stage(s) above it — the Canonne–Kamath–Steinke construction — so correctness
composes upward:

```
bit flips (OpenSSL entropy)
  → rand_bytes            uniform bytes
  → fill_bytes            uniform nat in [0, 256^n)
  → sample_uniform_ubig   uniform on [0, upper)                 (rejection)
  → sample_bernoulli_fraction   Bernoulli(numer/denom)          (compare < numer)
  → sample_bernoulli_exp1       Bernoulli(e^{-x}),  x ∈ [0,1]
  → sample_bernoulli_exp        Bernoulli(e^{-x}),  x ≥ 0
  → sample_geometric_exp_slow   Geometric via Bernoulli(e^{-x})
  → sample_geometric_exp_fast   Geometric (optimized; "dast")
  → sample_discrete_laplace      DiscreteLaplace(t)
  → sample_discrete_gaussian     DiscreteGaussian(σ²)
```

Everything is verified against **SampCert** reference distributions: for each
extracted Rust function `f`, the theorem is `samplerDist f = <SampCert PMF>`.

## Status legend

| mark | meaning |
|------|---------|
| ✅ | proved in this repro (`repro/src/`), zero `sorry` |
| 🟡 | Rust **extracted** into `repro/Generated/`, proof **not yet ported** here (reference proof may exist in `repro/proofs_legacy/`) |
| ⚪ | **not extracted** into Lean at all — no `Generated/` code exists yet; needs a Charon/Aeneas pass first |
| 🔒 | axiom / trust boundary (not a theorem) |

## The chain, stage by stage

### 0. bit flips → `rand_bytes` 🔒
Raw entropy from OpenSSL; each byte is 8 uniform bits.
- **Contract:** `openssl.rand.rand_bytes_uniform_spec` (axiom) — the buffer is filled with independent uniform bytes.
- This is the *only* hardware assumption; everything downstream is a theorem on top of it.

### 1. `rand_bytes` → `fill_bytes` ✅
Fills an `n`-byte buffer and reads it big-endian → uniform on `[0, 256^n)`.
- **Rust:** `samplers.fill_bytes` · **repro:** `OpenDP.Core.Bytes.uniformByteNatPMF`, `OpenDP.Core.Semantics.samplerDistGen_fill_bytes_nat` (the "hardware theorem"), `fill_bytes_nat_bridge`.
- The single stochastic primitive: all randomness in the crate flows through this bridge.

### 2. `fill_bytes` → `sample_uniform_ubig_below` ✅
Rejection sampling to get an exact uniform on `[0, upper)` (draw enough bytes, reject the tail above the largest multiple of `upper`).
- **Rust:** `samplers.uniform.sample_uniform_ubig_below` (+ `_usize_below`, `sample_from_uniform_bytes`).
- **repro:** `sample_uniform_ubig_below_pmf`, end-to-end `sample_uniform_ubig_below_spec` = `uniformNatBelowPMF`. The a14083a6 `deref_mut` loop-body reduction lives in `repro/src/samplers/uniform/blockers.lean` (`body_eq`).

### 3. `sample_uniform_ubig_below` → `sample_bernoulli_fraction` ✅
Draw `k` uniform on `[0, denom)`; return `⊤` iff `k < numer` → `Bernoulli(numer/denom)`.
- **Rust:** `samplers.bernoulli.sample_bernoulli_rational` · **repro:** `sample_bernoulli_rational_pmf`; `bernoulliPMF = SLang.BernoulliSamplePMF` grounds it against the trusted reference.

### 4. `sample_bernoulli_fraction` → `sample_bernoulli_exp1` ✅
`Bernoulli(e^{-x})` for `x ∈ [0,1]`. CKS unit construction: repeatedly draw
`Bernoulli(x/k)` for `k = 1,2,…` until one fails; return the parity of `k`.
`Pr[⊤] = e^{-x}`.
- **Rust:** `samplers.bernoulli.sample_bernoulli_exp1` (+ `_loop`, `_loop.body`) — a `probWhile` loop.
- **SampCert target:** `SLang.BernoulliExpNegSampleUnit`.
- **repro:** `sample_bernoulli_exp1_spec` = `BernoulliExpNegSampleUnit` (`src/samplers/bernoulli/exp1.lean`), via `exp1_loop_cut_step` (cut-depth ↔ SampCert `BESL` loop), `exp1_loop_probWhile` (`⨆`/`tsum_iSup_commute` lift), and `probWhile_besl_eq_aux` (SampCert `..._sup`/`..._apply`) + `ℕ+`→`ℕ` reindex. One temporary axiom `div_rbig_by_ubig_exact_bernoulli_setup` (Dashu exact-division boundary).

### 5. `sample_bernoulli_exp1` → `sample_bernoulli_exp` ✅
`Bernoulli(e^{-x})` for arbitrary `x ≥ 0`: `⌊x⌋` independent `Bernoulli(e^{-1})` all true, then `Bernoulli(e^{-frac(x)})` via stage 4.
- **Rust:** `samplers.bernoulli.sample_bernoulli_exp` (+ `_loop`). **SampCert target:** `SLang.BernoulliExpNegSample`.
- **repro:** `sample_bernoulli_exp_spec` (`src/samplers/bernoulli/exp.lean`) — strong induction on
  `ubigToNat numer` at the *program level* (`loop.eq_def`; the loop terminates, so no `probWhile`
  cut machinery), closed by the SampCert-side recursive law `bernoulliExpNegSample_succ`.

### 6. `sample_bernoulli_exp` → `sample_geometric_exp_slow` ✅
Count consecutive successes of `Bernoulli(e^{-x})` → a geometric law.
- **Rust:** `samplers.geometric.sample_geometric_exp_slow` (+ `_loop`, `_loop.body`). **SampCert target:** `SLang.probGeometric` over the `BernoulliExpNegSample` trial.
- **repro:** `sample_geometric_exp_slow_spec` (`src/samplers/geometric/slow.lean`):
  `samplerDist_nat ⟦slow x⟧ = fun v => probGeometric (Bernoulli(e^{-x})) (v+1)` (the `+1` is
  SampCert counting the final failing draw), plus the closed form
  `sample_geometric_exp_slow_closed_form` (`P[v] = q^v·(1-q)`, `q = e^{-x}`). Technique mirrors
  exp1: body factored through the stage-5 draw (`geo_step`), cut-depth induction against
  SampCert's `geometric_succ_true/false`, `tsum_iSup_commute` lift, `geometric_pwc_sup` finish.
  The opaque-`UBig` counter is pushed to `ℕ` with `samplerDist_nat`. Two new dashu axioms:
  `rbig_clone_exists_spec`, `rbig_clone_parts_spec` (the loop clones `x` each iteration).

### 7. `sample_geometric_exp_slow` → `sample_geometric_exp_fast` ✅  (the "dast"/fast variant)
Optimized geometric that avoids the linear loop (uniform residue + acceptance test + one slow
geometric at `1`, combined as `⌊(v·denom + u) / numer⌋`).
- **Rust:** `samplers.geometric.sample_geometric_exp_fast` (+ `_loop`). **SampCert target:** `DiscreteLaplaceSampleLoopIn1`/`In2` (the Laplace inner loop) and `SLang.Geo`.
- **repro:** `sample_geometric_exp_fast_spec` (`src/samplers/geometric/fast.lean`):
  `samplerDist_nat ⟦fast x⟧ = fun v => probGeometric (Bernoulli(e^{-x})) (v+1)` — the *same law
  as stage 6* — plus `…_zero_spec` (point mass at `0`) for the `x = 0` branch. Proof pieces:
  1. **`partial_fixpoint` → `loop` bridge** (`sample_geometric_exp_fast_loop_eq_loop`): Aeneas
     extracts this loop as genuine Lean recursion, outside the loop-semantics axiom; proved
     equal to `Aeneas.Std.loop (fast_body …)` by two-sided least-fixpoint induction in the flat
     `Result` order (generated `fixpoint_induct` principles; **no new axiom**).
  2. Fiber laws of `fast_body`: accept = stage-6 slow at `1` through the floor-division
     arithmetic (new dashu axioms `div_ubig_floor_spec`/`…_exists_spec`, `as_ibig_exists_spec`,
     `ibig_clone_exists_spec`); reject = stage-2 uniform.
  3. Cut-depth induction against a SampCert-side model loop (`fastLoopBody`), lifted via
     `tsum_iSup_commute`.
  4. Rejection-sampling closed form as a geometric series over the reject mass
     (`fast_mixed_probWhile`), identified with `DiscreteLaplaceSampleLoopIn1`'s normalized law
     (`In1_apply_form`), and closed by the ported legacy `Geo` algebra
     (`fastTarget_pmf`, `fastTarget_eq_slowLaw` via `DiscreteLaplaceSampleLoop_equiv`).

### 8. `sample_geometric_exp_fast` → `sample_discrete_laplace` ✅
`DiscreteLaplace(t)`: a sign times a geometric magnitude, rejecting `(negative, 0)` (CKS).
- **Rust:** `samplers.laplace.sample_discrete_laplace` (`src/samplers/laplace/mod.rs`, extracted
  via the standard Charon→Aeneas refresh — a genuine Aeneas `loop` with `Unit` state).
- **SampCert target:** `SLang.DiscreteLaplaceSample`.
- **repro:** `sample_discrete_laplace_spec` (`src/samplers/laplace.lean`):
  `samplerDist_int ⟦laplace numer denom⟧ = DiscreteLaplaceSample ⟨numer⟩ ⟨denom⟩` on `ℤ`
  (plus the new `samplerDist_int` pushforward over `dashu.ibigToInt`). Proof: body factored
  through the stage-7 magnitude draw and the stage-3 fair coin (`lap_step`); since each
  iteration is independent (`Unit` state), the rejection analysis is a *scalar* geometric
  series (`lap_cut_closed`/`lap_probWhile_closed`, abstract in the magnitude law); the closed
  form meets SampCert via `probUntil_apply_norm` + `DiscreteLaplaceSampleLoop_apply`
  (`lap_closed_form_eq`). New dashu axioms: `ibigToInt` interpretation (`ibigToInt_pos_spec`,
  `ibig_neg_exists_spec`) and the `1/2` constant (`rbig_from_parts_const_half_exists_spec`).

### 9. `sample_discrete_laplace` → `sample_discrete_gaussian` ✅  ← **final target, complete**
`DiscreteGaussian(σ²)` by rejection sampling from a discrete Laplace proposal (CKS Algorithm 3).
- **Rust:** `samplers.gaussian.sample_discrete_gaussian` (`src/samplers/gaussian/mod.rs`):
  propose `Y ~ DiscreteLaplace(⌊σ⌋+1)` (stage 8), accept with probability
  `e^{-(|Y|·t·den − num)²/(2·num·t²·den)}` over `σ²`'s parts (stage 5).
- **SampCert target:** `SLang.DiscreteGaussianSample` (for every `mix` parameter, via
  `DiscreteLaplaceSampleMixed_equiv`).
- **repro:** `sample_discrete_gaussian_spec` (`src/samplers/gaussian.lean`):
  `samplerDist_int ⟦gaussian numer denom⟧ = DiscreteGaussianSample ⟨numer⟩ ⟨denom⟩ mix` on `ℤ`.
  Proof: value-tracked 22-step deterministic body chain feeding the stage-5 acceptance trial
  (`gauss_step_chain`); `Unit`-state rejection → the stage-8 scalar-series lemmas verbatim;
  closed form meets SampCert via `probUntil_apply_norm` + a locally-ported factored loop law
  (`gaussLoop_apply` — SampCert's own `Gaussian/Properties` analytic layer does not compile on
  the pinned stack, so the two sampler-level facts it provides are re-proved here against
  `Gaussian/Code` only). New dashu axioms: `ibig_sub_exists_spec`,
  `ibig_unsigned_abs_exists_spec`, `ibig_clone_int_spec`.

## Where the frontier is today

**The chain is complete.** Every stage — bit flips → `fill_bytes` → uniform →
bernoulli-fraction → bernoulli-exp1 → bernoulli-exp → geometric-slow → geometric-fast →
discrete Laplace → **discrete Gaussian** — is a Lean theorem over the single randomness axiom
plus the deterministic dashu/usize external specs. The extracted Rust noise mechanisms for both
pure DP (`sample_discrete_laplace` = `DiscreteLaplaceSample`) and zCDP
(`sample_discrete_gaussian` = `DiscreteGaussianSample`) are verified against SampCert's
reference samplers end-to-end.

**Toward the DP guarantees themselves:** SampCert carries the privacy theorems for these
reference samplers (e.g. the zCDP bound for the discrete Gaussian mechanism); since the
extracted Rust samplers are now *equal* to SampCert's, those guarantees transfer by rewriting.
The remaining engineering is that SampCert's analytic Gaussian layer
(`SampCert/Util/Gaussian`, `Samplers/Gaussian/Properties`, and the `DifferentialPrivacy`
modules) does not compile on the pinned v4.30-rc2 stack — porting those files (or bumping the
SampCert pin) is the last step to state `⟦rust mechanism⟧ satisfies zCDP` as a single theorem.

## Notes

- The formal dependency graph for the ✅/🟡 stages is rendered by the blueprint
  (`repro/tools/build_blueprint.sh` → `repro/blueprint/web/dep_graph_document.html`),
  which carries the real Lean declaration names and colours nodes by proof status.
- Every stage's correctness is *relative to* the single randomness axiom (stage 0)
  plus the deterministic dashu/usize specs; run `#print axioms` on any end-to-end
  theorem to see exactly which it depends on.
- **Completion is machine-checked**: `tools/build_lean.sh` ends with
  `tools/check_verified_chain.sh`, which fails the build if any of the eight
  end-to-end theorems (stages 2–9) is missing, if any depends on an axiom outside
  the sanctioned trust surface (`tools/check_chain.lean`), or if any handwritten
  source contains a `sorry` token (`lake build` alone enforces none of these — a
  `sorry` is only a warning, and a deleted theorem doesn't break compilation).
