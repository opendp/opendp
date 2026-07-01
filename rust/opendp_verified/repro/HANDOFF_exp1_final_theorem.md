# Handoff — `sample_bernoulli_exp1` final theorem (roadmap stage 4)

## ✅ DONE (2026-07-01) — stage 4 is fully proved

The final end-to-end theorem **`sample_bernoulli_exp1_spec`** (`= BernoulliExpNegSampleUnit`)
is proved and the project builds GREEN. The plan below was executed as Increment 4 in
`repro/src/samplers/bernoulli/exp1.lean`:
- `exp1_loop_probWhile` — lift `exp1_loop_cut_step` to the full `probWhile` (via
  `simp only [probWhile]` + `simp_rw [ENNReal.iSup_mul]` + `tsum_iSup_commute` + `iSup_congr`;
  NOTE: `conv … ext` cannot descend into `⨆`/`∑'` on this Mathlib — use `iSup_congr` / `simp_rw`).
- `probWhile_besl_eq_aux` — per-counter mass = `BernoulliExpNegSampleUnitAux` via SampCert's
  `..._sup` + `..._apply`.
- `sample_bernoulli_exp1_loop_spec` — reindex `∑' : ℕ+` → `∑' : ℕ` with
  `Function.Injective.tsum_eq PNat.coe_injective` (needs a `Function.support f ⊆ Set.range PNat.val`
  arg, and `f` pinned via a typed `have` or instance search gets stuck on `TopologicalSpace ?`) +
  parity split of `BernoulliExpNegSampleUnit` + `BernoulliExpNegSampleUnitAux_at_zero`.
- `sample_bernoulli_exp1_eq_of_setup` / `sample_bernoulli_exp1_spec` — outer `x`-destructuring,
  reusing `RationalSetup` + `dashu.one_exists_spec`.

Axiom footprint of the final theorem: the expected boundary (`openssl.rand.rand_bytes`,
`samplerDistGen_exists`, dashu specs, `div_rbig_by_ubig_exact_bernoulli_setup`) plus `sorryAx`,
which is **pre-existing Aeneas `Std` infrastructure** (the accepted uniform theorem carries it too),
not from this proof. Next up: roadmap stage 5 (`sample_bernoulli_exp`).

---

_Original handoff (pre-session) below, for reference:_

**Status as of end of 2026-06-30 session:** `repro/` builds **GREEN** (exit 0, 2866/2866
jobs, no `sorry` in `exp1.lean`). Increment 3 of
`repro/src/samplers/bernoulli/exp1.lean` is fully proved. The **final end-to-end
theorem is not yet written** — that is the next task.

## Build / iterate

- Fast leaf rebuild (deps cached, ~10 s, only recompiles `exp1`):
  `cd rust/opendp_verified/repro && lake build OpenDPVerified`  (exit 0 + zero `error:` = green)
- Full reliable verification (guard → cache → build):
  `cd <repo> && ./rust/opendp_verified/repro/tools/build_lean.sh > /tmp/lb.log 2>&1`
  then `TaskOutput`/read `<exit_code>`; `grep -nE "error:" /tmp/lb.log`.
- Debugging idiom that works well here: replace a stuck closing tactic with `rfl` — if the
  goal is defeq it closes; if not, the error dumps both sides so you can see the mismatch.

## What is already proved in `exp1.lean` (build on these)

- `exp1_loop_cut_step` (the crux, now GREEN): at every cut depth `n`,
  ```
  probWhileCut cond bd n (cont k1) (done (Ok b))
    = ∑' m:ℕ+, probWhileCut (·.1) (BESL numer denom hdenom hfrac) n
                 (true, ⟨ubigToNat k1, hk1⟩) (false, m) * (if decide (↑m % 2 = 0) = b then 1 else 0)
  ```
  where `cond`/`bd` are the extracted loop's control-flow condition/body distribution,
  `BESL = BernoulliExpNegSampleUnitLoop (ubigToNat numer) ⟨ubigToNat denom, hdenom⟩ hfrac`.
- Supporting: `besl_true_coeff`, `besl_false_coeff`, `besl_cut_succ_true/false`,
  `exp1_body_cont_apply`, `exp1_body_done_ok_apply`, `pnat_succ_eq`,
  `bernoulliExp1_sampcert_step_apply`, `rem_u8_sampcert_parity`.
- **One axiom** (documented temporary Dashu boundary):
  `div_rbig_by_ubig_exact_bernoulli_setup` — discharge once upstream Dashu exact-division
  is fixed. Not part of the final-theorem work; it will show up under `#print axioms`.

## The target (Increment 4)

Prove `samplerDist (sample_bernoulli_exp1 x) = BernoulliExpNegSampleUnit num den wf`
(as `SLang Bool`) on the valid-input branch, mirroring
`repro/src/samplers/uniform/pmf.lean` → `samplerDist_loop_rejection_uniform`.

### Extracted side (`Generated/OpenDP/Funs.lean:802–863`)
- `sample_bernoulli_exp1 x`: destructures `x` → `(numer_signed, denom)` → `(s, numer)`;
  on `Sign.Positive` sets `k := UBig.ONE` (= 1) and runs
  `sample_bernoulli_exp1_loop k denom numer k` (both loop args = 1);
  on `Sign.Negative` returns an `Err`.
- `sample_bernoulli_exp1_loop k denom numer k1 = loop (fun k2 => body k denom numer k2) k1`.

### SampCert side (`SampCert/Samplers/BernoulliNegativeExponential/Code.lean:44–78`)
- `BernoulliExpNegSampleUnit num den wf = do let K ← BernoulliExpNegSampleUnitAux …; if K%2=0 then pure true else pure false`
- `BernoulliExpNegSampleUnitAux = do let r ← probWhile (·.1) (BernoulliExpNegSampleUnitLoop num den wf) (true,1); pure r.2`
- `BernoulliExpNegSampleUnitLoop … state = do let A ← BernoulliSample num (state.2*den) …; pure (A, state.2+1)`

## Proof plan (bottom-up sub-lemmas)

1. **Lift `exp1_loop_cut_step` to `probWhile`** (`exp1_loop_probWhile`):
   - `samplerDist (sample_bernoulli_exp1_loop 1 denom numer 1) b`
     `= samplerDistGen (loop …) (Ok b)` `= probWhile cond bd (cont 1) (done (Ok b))`
     via `samplerDistGen_loop` (core, `semantics.lean:150`).
   - `probWhile = ⨆ n, probWhileCut n`; apply `exp1_loop_cut_step` (with `k1 = 1`,
     `hkOne : ubigToNat kOne = 1` from `UBig.ONE`); then commute `⨆`/`∑'` with
     `tsum_iSup_commute` + `probWhileCut_monotonic` (see the uniform proof, steps 2–5).
   - Result:
     `= ∑' m:ℕ+, probWhile (·.1) BESL (true, ⟨1,_⟩) (false, m) * (if decide (↑m%2=0)=b then 1 else 0)`.

2. **Bridge to `BernoulliExpNegSampleUnitAux`** (`ℕ+`→`ℕ` reindex + parity). Note:
   ```
   BernoulliExpNegSampleUnit num den wf b = ∑' K:ℕ, Aux K * (if decide (K%2=0)=b then 1 else 0)
   Aux K = ∑' st:Bool×ℕ+, probWhile (·.1) BESL (true,1) st * (if st.2 = K then 1 else 0)
   ```
   - `probWhile (·.1) BESL (true,1)` has mass only on terminal `(false, m)` states
     (cond false); `(true, _)` states get 0. (SampCert likely has this — check
     `BernoulliExpNegSampleUnitAux_monotone_counter` / around Properties.lean:131.)
   - So `Aux K = 0` at `K = 0`, and `Aux (↑m) = probWhile BESL (true,1) (false,m)` for `m:ℕ+`.
   - Reindex `∑' K:ℕ` (dropping `K=0`) to `∑' m:ℕ+` to match step 1's sum. Parity aligns
     (`↑m % 2` vs `K % 2`).
   - **CAUTION (from today's gotchas, see memory `lean-tsum-pnat-gotchas`):** `tsum_eq_single`
     does NOT work over `ℕ+` on this Mathlib (needs `SummationFilter.LeAtTop`, won't synth).
     Use `simp` / SampCert's own idioms for `∑' : ℕ+`. PNat coercions are defeq (`rfl` closes
     `↑↑⟨n,h⟩ = ↑n`, `⟨k,hk⟩+1 = ⟨k+1,_⟩`).
   - **TODO first thing tomorrow:** finish the interrupted lookup —
     `grep -nE "theorem BernoulliExpNegSampleUnit|_apply|probWhile.*BernoulliExpNegSampleUnitLoop" SampCert/SampCert/Samplers/BernoulliNegativeExponential/Properties.lean`
     to find any ready-made `BernoulliExpNegSampleUnit`/`Aux` distribution lemma to reuse
     instead of rebuilding the reindex.

3. **Outer `x`-destructuring** (`sample_bernoulli_exp1_spec`): push `samplerDist` through
   `into_parts`/sign match/`UBig.ONE` (deterministic — `samplerDist_bind`,
   `samplerDistGen_pure_ok`) on the `Positive` branch to reduce to the loop from step 1.
   Feed in `numer`/`denom`/`wf` from the rational setup (see how `sample_bernoulli_rational_pmf`
   in `bernoulli/pmf.lean` threads `RationalSetup`).

## Reference proof to mirror

`repro/src/samplers/uniform/pmf.lean` : `samplerDist_loop_rejection_uniform` (lines ~257–372)
— same `samplerDistGen_loop` → `⨆ probWhileCut` → `tsum_iSup_commute` → cut-step-match skeleton.
`loop_cut_step_aux` there is the analogue of `exp1_loop_cut_step`.
