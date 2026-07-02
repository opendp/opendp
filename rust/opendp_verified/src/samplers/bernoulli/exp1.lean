import Generated.OpenDP
import SampCert.Samplers.BernoulliNegativeExponential.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.bernoulli.rational
import src.samplers.bernoulli.pmf

/-!
# `sample_bernoulli_exp1` — unit negative-exponential (roadmap stage 4)

Target: `⟦sample_bernoulli_exp1 x⟧ = SLang.BernoulliExpNegSampleUnit` for `x ∈ [0,1]`
(`Pr[⊤] = e^{-x}`). The extracted function runs a `probWhile` loop that, at step `k1`,
draws `Bernoulli((numer/denom)/k1)` (via the proved `sample_bernoulli_rational`), continues
with `k1 + k` on success, and returns the parity of `k1` at the first failure — matching
SampCert's `BernoulliExpNegSampleUnitLoop`.

**Increment 1 (this file so far):** the per-step machinery — factor the loop body through the
proved rational draw (`exp1_step`) and compute each step's point-mass distribution. The
`probWhile`/`probWhileCut` limit-equivalence to SampCert's loop is the remaining work (it will
reuse `samplerDistGen_loop` from the core and the `bernoulliPMF = BernoulliSamplePMF` bridge).
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Classical

namespace OpenDP.samplers.bernoulli

/-- The deterministic continuation of one exp1 loop step after the rational draw returns `r`:
branch on the control flow, then continue with `k1 + k` (success), or terminate with the parity
of `k1` (failure). Naming it gives stable equation lemmas so `simp` can reduce the `match`. -/
noncomputable def exp1_step (k k1 : dashu_int.ubig.UBig) :
    core.result.Result Bool error.Error →
    Result (ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :=
  fun r => do
    let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r
    match cf with
    | core.ops.control_flow.ControlFlow.Continue val =>
      if val then do
        let k2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k
        ok (cont k2)
      else do
        let i ← dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8
        ok (done (core.result.Result.Ok (i = 1#u8)))
    | core.ops.control_flow.ControlFlow.Break residual => do
      let r1 ← core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
        Bool (core.convert.FromSame error.Error) residual
      ok (done r1)

/-- Once the (deterministic) division succeeds, the exp1 body factors through `exp1_step`. -/
lemma exp1_body_eq_step
    (k denom numer k1 : dashu_int.ubig.UBig) (x_div_k : dashu_ratio.rbig.RBig)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
    sample_bernoulli_rational x_div_k >>= exp1_step k k1 := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop.body
  rw [hdiv]
  rfl

/-- Step on `Ok true`: a point mass at `cont k2'` (where `k2' = k1 + k`). -/
lemma exp1_step_ok_true
    (k k1 k2' : dashu_int.ubig.UBig)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2')
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Ok true)) out =
    (if out = cont k2' then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    hadd, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Ok false`: a point mass at `done (Ok parity)` (where `parity = decide (i = 1)`). -/
lemma exp1_step_ok_false
    (k k1 : dashu_int.ubig.UBig) (i : Std.U8)
    (hrem : dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Ok false)) out =
    (if out = done (core.result.Result.Ok (decide (i = 1#u8))) then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    hrem, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Err e`: a point mass at `done (Err e)`; `0` at any `cont` or `done (Ok _)`. -/
lemma exp1_step_err
    (k k1 : dashu_int.ubig.UBig) (e : error.Error)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Err e)) out =
    (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual,
    samplerDistGen_pure_ok, PMF.pure_apply]

/-! ### Increment 2 — per-step ↔ SampCert bridge and body distribution -/

/-- Well-formedness of the per-step Bernoulli numerator: `numer ≤ k·denom`. -/
theorem bernoulli_step_wf {numer denom : Nat} (hfrac : numer ≤ denom) {k : Nat}
    (hk : 0 < k) : numer ≤ k * denom := by
  have hk1 : 1 ≤ k := Nat.succ_le_of_lt hk
  calc numer ≤ denom := hfrac
    _ ≤ k * denom := by simpa [one_mul] using Nat.mul_le_mul_right denom hk1

/-- Rust checks the *current* counter `k` for oddness; SampCert's loop has advanced the
final counter to `k+1` and checks it for evenness. These agree. -/
theorem decide_odd_current_eq_even_successor (k : Nat) :
    (decide (k % 2 = 1) : Bool) = decide ((k + 1) % 2 = 0) := by
  rw [Nat.mod_two_of_bodd, Nat.mod_two_of_bodd]
  cases h : Nat.bodd k <;> simp [Nat.bodd_succ, h]

/-- Pointwise value of SampCert's one-step neg-exponential loop at the successor state —
the canonical step law the extracted `exp1` loop must match. -/
theorem bernoulliExp1_sampcert_step_apply
    (numer denom k : Nat) (hdenom : 0 < denom) (hfrac : numer ≤ denom) (hk : 0 < k) (b : Bool) :
    SLang.BernoulliExpNegSampleUnitLoop numer ⟨denom, hdenom⟩ hfrac
        (true, ⟨k, hk⟩) (b, ⟨k + 1, Nat.succ_pos k⟩) =
      SLang.BernoulliSample numer ⟨k * denom, Nat.mul_pos hk hdenom⟩ (bernoulli_step_wf hfrac hk) b := by
  have hsucc : (⟨k + 1, Nat.succ_pos k⟩ : PNat) = (⟨k, hk⟩ : ℕ+) + 1 := by
    ext; change k + 1 = k + 1; rfl
  rw [hsucc]
  unfold SLang.BernoulliExpNegSampleUnitLoop
  cases b <;> simp <;> intro h <;> exact False.elim (h rfl)

/-- The Aeneas `k1 rem 2 = 1` parity check equals SampCert's `(k1+1) % 2 = 0`. -/
lemma rem_u8_sampcert_parity (k1 : dashu_int.ubig.UBig) (i : Std.U8)
    (hrem : dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i) :
    (decide (i = 1#u8) : Bool) = decide ((dashu.ubigToNat k1 + 1) % 2 = 0) := by
  have hremNat := dashu.rem_u8_spec k1 2#u8 i hrem
  have hi : (i = 1#u8) ↔ i.val = 1 := by rw [Aeneas.Std.UScalar.eq_equiv]; simp
  have hiff : (i = 1#u8) ↔ dashu.ubigToNat k1 % 2 = 1 := by rw [hi, hremNat]; simp
  calc (decide (i = 1#u8) : Bool) = decide (dashu.ubigToNat k1 % 2 = 1) := by
        by_cases h : i = 1#u8
        · simp [h, hiff.mp h]
        · have hp : ¬ dashu.ubigToNat k1 % 2 = 1 := fun hp => h (hiff.mpr hp)
          simp [h, hp]
    _ = decide ((dashu.ubigToNat k1 + 1) % 2 = 0) := decide_odd_current_eq_even_successor _

/-- **Temporary semantic boundary for `div_rbig_by_ubig_exact`** while Dashu's exact-division
path is unstable — the sole exp1-specific axiom. It asserts that at each step the exact division
yields a valid rational-Bernoulli setup whose per-step law is SampCert's `BernoulliSample` at
`(numer, k·denom)`. To be discharged once the upstream Dashu division is fixed. -/
axiom div_rbig_by_ubig_exact_bernoulli_setup
    (numer denom k : dashu_int.ubig.UBig)
    (hk : 0 < dashu.ubigToNat k) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    ∃ x_div_k : dashu_ratio.rbig.RBig, ∃ setup : RationalSetup x_div_k,
      ∃ hsetupDenom : 0 < dashu.ubigToNat setup.denom,
        utilities.div_rbig_by_ubig_exact numer denom k = ok x_div_k ∧
        bernoulliPMF setup.numer setup.denom hsetupDenom =
          SLang.BernoulliSamplePMF (dashu.ubigToNat numer)
            ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
            (bernoulli_step_wf hfrac hk)

/-- SampCert's negative-exponential unit loop body, at the extracted numer/denom. -/
noncomputable def BESL (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) : Bool × PNat → SLang (Bool × PNat) :=
  SLang.BernoulliExpNegSampleUnitLoop (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac

/-- Per-step law of the extracted loop, stated against SampCert's `BESL` transition (lean form:
just the division witness + the per-step PMF match, which is all the body analysis needs). -/
lemma exp1_step_sampcert_spec (numer denom k : dashu_int.ubig.UBig)
    (hk : 0 < dashu.ubigToNat k) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    ∃ x_div_k : dashu_ratio.rbig.RBig, ∃ setup : RationalSetup x_div_k,
      ∃ hsetupDenom : 0 < dashu.ubigToNat setup.denom,
        utilities.div_rbig_by_ubig_exact numer denom k = ok x_div_k ∧
        ∀ b : Bool, bernoulliPMF setup.numer setup.denom hsetupDenom b =
          BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k, hk⟩)
            (b, ⟨dashu.ubigToNat k + 1, Nat.succ_pos _⟩) := by
  rcases div_rbig_by_ubig_exact_bernoulli_setup numer denom k hk hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, hdiv, hpmf⟩
  refine ⟨x_div_k, setup, hsetupDenom, hdiv, fun b => ?_⟩
  calc bernoulliPMF setup.numer setup.denom hsetupDenom b
      = SLang.BernoulliSamplePMF (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
          (bernoulli_step_wf hfrac hk) b := by rw [hpmf]
    _ = SLang.BernoulliSample (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
          (bernoulli_step_wf hfrac hk) b := by unfold SLang.BernoulliSamplePMF; rw [PMF.ofFintype_apply]
    _ = BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k, hk⟩)
          (b, ⟨dashu.ubigToNat k + 1, Nat.succ_pos _⟩) := by
          rw [BESL]; exact (bernoulliExp1_sampcert_step_apply _ _ _ hdenom hfrac hk b).symm

/-- The loop body's `cont` mass matches BESL at `(true, k1+1)`. -/
lemma exp1_body_cont_apply
    (k denom numer k1 k2 k2' : dashu_int.ubig.UBig)
    (hk1 : 0 < dashu.ubigToNat k1) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2') :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1) (cont k2) =
    BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) * (if k2 = k2' then 1 else 0) := by
  rcases exp1_step_sampcert_spec numer denom k1 hk1 hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, hdiv, hstep⟩
  obtain ⟨i, hrem, _⟩ := dashu.rem_u8_exists_spec k1 2#u8 (by decide)
  rw [exp1_body_eq_step k denom numer k1 x_div_k hdiv, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [exp1_step_err]; simp)]
  rw [tsum_bool, exp1_step_ok_true k k1 k2' hadd, exp1_step_ok_false k k1 i hrem]
  have hrat_true : samplerDistGen (sample_bernoulli_rational x_div_k) (core.result.Result.Ok true) =
      bernoulliPMF setup.numer setup.denom hsetupDenom true := by
    simpa [samplerDist] using congrFun (sample_bernoulli_rational_pmf x_div_k setup hsetupDenom) true
  rw [hrat_true, hstep true]
  simp [ControlFlow.cont.injEq]

/-- The loop body's `done (Ok b)` mass matches BESL at `(false, k1+1)` times the parity indicator. -/
lemma exp1_body_done_ok_apply
    (k denom numer k1 : dashu_int.ubig.UBig)
    (hk1 : 0 < dashu.ubigToNat k1) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (b : Bool) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1)
        (done (core.result.Result.Ok b)) =
    BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
    (if decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b then 1 else 0) := by
  rcases exp1_step_sampcert_spec numer denom k1 hk1 hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, hdiv, hstep⟩
  obtain ⟨i, hrem, _⟩ := dashu.rem_u8_exists_spec k1 2#u8 (by decide)
  have hparity := rem_u8_sampcert_parity k1 i hrem
  rcases dashu.add_assign_exists_spec k1 k with ⟨k2', hadd, _⟩
  rw [exp1_body_eq_step k denom numer k1 x_div_k hdiv, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [exp1_step_err]; simp)]
  rw [tsum_bool, exp1_step_ok_true k k1 k2' hadd, exp1_step_ok_false k k1 i hrem]
  have hrat_false : samplerDistGen (sample_bernoulli_rational x_div_k) (core.result.Result.Ok false) =
      bernoulliPMF setup.numer setup.denom hsetupDenom false := by
    simpa [samplerDist] using congrFun (sample_bernoulli_rational_pmf x_div_k setup hsetupDenom) false
  rw [hrat_false, hstep false]
  cases b <;> simp [hparity]

/-! ### Increment 3 — operational loop bridge (`probWhile` limit ↔ SampCert)

The per-step body distribution (increment 2) is now lifted to the whole `probWhile`, closing the
gap between the extracted Aeneas loop and SampCert's `BernoulliExpNegSampleUnit`. The technique
mirrors the uniform sampler's `samplerDist_loop_rejection_uniform` (`samplers/uniform/pmf.lean`):
`samplerDistGen_loop` turns the loop into a `probWhile` over `ControlFlow`, then a cut-depth
induction (`exp1_loop_cut_step`) matches each `probWhileCut` against SampCert's own `BESL` loop —
here at the *same* cut depth (both loops draw-and-transition identically, so there is no `n+1 ↔ n`
offset), and finally `tsum_iSup_commute` pulls the supremum through the counter sum.

The extracted loop's `done (Ok b)` state records only the parity bit `b`; SampCert's terminal
states `(false, m)` record the full counter. So the correspondence sends each SampCert final state
`(false, m)` to the extracted `done (Ok (decide (m % 2 = 0)))` via the parity indicator. -/

/-- PNat successor bridge: the `⟨k+1, _⟩` shape produced by the extracted body lemmas equals the
`⟨k, _⟩ + 1` shape produced by SampCert's `..._succ_true`/`_succ_false` recurrences. -/
lemma pnat_succ_eq (k : Nat) (hk : 0 < k) :
    (⟨k, hk⟩ : ℕ+) + 1 = ⟨k + 1, Nat.succ_pos k⟩ := by
  ext; rfl

/-- One `BESL` step from `(true, k)` to `(true, k+1)` has mass `numer / (k · denom)` — the same
coefficient SampCert's `BernoulliExpNegSampleUnitAux_succ_true` produces. -/
lemma besl_true_coeff (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (k : Nat) (hk : 0 < k) :
    BESL numer denom hdenom hfrac (true, ⟨k, hk⟩) (true, ⟨k + 1, Nat.succ_pos k⟩) =
      (dashu.ubigToNat numer : ENNReal) /
        (((⟨k, hk⟩ : ℕ+) : ENNReal) * ((⟨dashu.ubigToNat denom, hdenom⟩ : ℕ+) : ENNReal)) := by
  rw [BESL, bernoulliExp1_sampcert_step_apply _ _ _ hdenom hfrac hk true, BernoulliSample_apply_true]
  congr 1
  show ((k * dashu.ubigToNat denom : ℕ) : ENNReal)
      = ((k : ℕ) : ENNReal) * ((dashu.ubigToNat denom : ℕ) : ENNReal)
  exact Nat.cast_mul k (dashu.ubigToNat denom)

/-- One `BESL` step from `(true, k)` to `(false, k+1)` has mass `1 - numer / (k · denom)`. -/
lemma besl_false_coeff (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (k : Nat) (hk : 0 < k) :
    BESL numer denom hdenom hfrac (true, ⟨k, hk⟩) (false, ⟨k + 1, Nat.succ_pos k⟩) =
      1 - (dashu.ubigToNat numer : ENNReal) /
        (((⟨k, hk⟩ : ℕ+) : ENNReal) * ((⟨dashu.ubigToNat denom, hdenom⟩ : ℕ+) : ENNReal)) := by
  rw [BESL, bernoulliExp1_sampcert_step_apply _ _ _ hdenom hfrac hk false, BernoulliSample_apply_false]
  congr 2
  show ((k * dashu.ubigToNat denom : ℕ) : ENNReal)
      = ((k : ℕ) : ENNReal) * ((dashu.ubigToNat denom : ℕ) : ENNReal)
  exact Nat.cast_mul k (dashu.ubigToNat denom)

/-- SampCert's one-step `true` recurrence, re-exposed with `BESL` folded. -/
lemma besl_cut_succ_true (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (fuel : Nat) (r : ℕ+)
    (st : Bool × ℕ+) :
    probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) (fuel + 1) (true, r) st =
      (dashu.ubigToNat numer : ENNReal) / ((r : ENNReal) * ((⟨dashu.ubigToNat denom, hdenom⟩ : ℕ+) : ENNReal)) *
        probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) fuel (true, r + 1) st +
      (1 - (dashu.ubigToNat numer : ENNReal) / ((r : ENNReal) * ((⟨dashu.ubigToNat denom, hdenom⟩ : ℕ+) : ENNReal))) *
        probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) fuel (false, r + 1) st := by
  simp only [BESL]
  exact BernoulliExpNegSampleUnitAux_succ_true (dashu.ubigToNat numer)
    ⟨dashu.ubigToNat denom, hdenom⟩ fuel st r hfrac

/-- SampCert's one-step `false` (settle) recurrence, re-exposed with `BESL` folded. -/
lemma besl_cut_succ_false (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (fuel : Nat) (r : ℕ+)
    (st : Bool × ℕ+) :
    probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) (fuel + 1) (false, r) st =
      if st = (false, r) then 1 else 0 := by
  simp only [BESL]
  exact BernoulliExpNegSampleUnitAux_succ_false (dashu.ubigToNat numer)
    ⟨dashu.ubigToNat denom, hdenom⟩ fuel st r hfrac

/-- **Cut-depth correspondence (the crux of increment 3).** At every cut depth `n`, the extracted
loop's `probWhileCut` mass on `done (Ok b)` (started from counter `k1`) equals SampCert's `BESL`
loop's mass on all terminal states `(false, m)` whose counter has parity `b`, summed against the
parity indicator. Proved by induction on `n`; the step splits the extracted body into its `cont`
(recurse via `ih`) and `done` (settle) fibers and matches them against SampCert's `_succ_true` /
`_succ_false` recurrences. -/
private lemma exp1_loop_cut_step
    (numer denom kOne : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) (hkOne : dashu.ubigToNat kOne = 1)
    (cond : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) → Bool)
    (bd : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) →
          SLang (ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)))
    (hcc : ∀ a, cond (cont a) = true)
    (hcd : ∀ w, cond (done w) = false)
    (hbc : ∀ a, bd (cont a) =
        samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body kOne denom numer a))
    (n : Nat) :
    ∀ (k1 : dashu_int.ubig.UBig) (hk1 : 0 < dashu.ubigToNat k1) (b : Bool),
      probWhileCut cond bd n (cont k1) (done (core.result.Result.Ok b)) =
      ∑' m : ℕ+, probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) n
          (true, ⟨dashu.ubigToNat k1, hk1⟩) (false, m) *
        (if decide ((m : ℕ) % 2 = 0) = b then 1 else 0) := by
  induction n with
  | zero =>
    intro k1 hk1 b
    simp only [probWhileCut, SLang.probZero, zero_mul, tsum_zero]
  | succ n ih =>
    intro k1 hk1 b
    obtain ⟨k2', hadd, hk2'nat⟩ := dashu.add_assign_exists_spec k1 kOne
    have hk2'val : dashu.ubigToNat k2' = dashu.ubigToNat k1 + 1 := by rw [hk2'nat, hkOne]
    have hk2'pos : 0 < dashu.ubigToNat k2' := by rw [hk2'val]; exact Nat.succ_pos _
    have hstateEq : (⟨dashu.ubigToNat k2', hk2'pos⟩ : ℕ+) = ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩ :=
      Subtype.ext hk2'val
    -- CONT fiber: only the incremented counter `k2'` survives; recurse via `ih`.
    have hCONT :
        (∑' a : dashu_int.ubig.UBig, bd (cont k1) (cont a) *
            probWhileCut cond bd n (cont a) (done (core.result.Result.Ok b))) =
          BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
              (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
            ∑' m : ℕ+, probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) n
                (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) (false, m) *
              (if decide ((m : ℕ) % 2 = 0) = b then 1 else 0) := by
      simp_rw [hbc k1, exp1_body_cont_apply (k := kOne) (denom := denom) (numer := numer) (k1 := k1)
        (k2' := k2') (hk1 := hk1) (hdenom := hdenom) (hfrac := hfrac) (hadd := hadd), mul_assoc]
      rw [ENNReal.tsum_mul_left]
      congr 1
      rw [tsum_eq_single k2' (fun a ha => by rw [if_neg ha, zero_mul]), if_pos rfl, one_mul,
        ih k2' hk2'pos b, hstateEq]
    -- DONE fiber: only `done (Ok b)` survives; matches SampCert's settle step.
    have hDONE :
        (∑' w : core.result.Result Bool error.Error, bd (cont k1) (done w) *
            probWhileCut cond bd n (done w) (done (core.result.Result.Ok b))) =
          BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
              (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
            ∑' m : ℕ+, probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) n
                (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) (false, m) *
              (if decide ((m : ℕ) % 2 = 0) = b then 1 else 0) := by
      cases n with
      | zero => simp only [probWhileCut, SLang.probZero, mul_zero, zero_mul, tsum_zero]
      | succ n' =>
        -- RHS settle FIRST (as a standalone `∑'` sub-goal so `tsum_eq_single` can resolve its
        -- summand function from the goal head before its zero-proof elaborates): the BESL loop
        -- from `(false, ⟨k1+1⟩)` collapses to a parity point mass.
        rw [show (∑' m : ℕ+, probWhileCut (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) (n' + 1)
                (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) (false, m) *
                (if decide ((m : ℕ) % 2 = 0) = b then 1 else 0)) =
              (if decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b then 1 else 0) from by
          -- `tsum_eq_single` is unavailable over `ℕ+` here (its `SummationFilter.LeAtTop` instance
          -- won't synthesize), so evaluate the point-mass sum with `simp`, matching SampCert's own
          -- idiom for settling these `∑' : ℕ+` sums (cf. `BernoulliExpNegSampleUnitAux_succ_false`).
          -- First push the settle indicator (`ite_mul`) *outside* the parity indicator, so the point
          -- mass at `m = ⟨k1+1⟩` is the outer `if` and `simp` can collapse the sum.
          simp_rw [besl_cut_succ_false numer denom hdenom hfrac n' ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩,
            ite_mul, one_mul, zero_mul]
          simp
          rfl]
        -- LHS settle: only `w = Ok b` survives.
        simp_rw [probWhileCut_done_pt cond bd hcd n', SLang.pure_apply]
        rw [tsum_eq_single (core.result.Result.Ok b) (fun w hw => by
            rw [if_neg (fun h => by injection h with h'; exact hw h'.symm), mul_zero]),
          if_pos rfl, mul_one, hbc k1,
          exp1_body_done_ok_apply kOne denom numer k1 hk1 hdenom hfrac b]
    -- Assemble: unfold one extracted step, split fibers, match against SampCert's `_succ_true`.
    rw [probWhileCut, probWhileFunctional, if_pos (hcc k1)]
    simp only [Bind.bind, SLang.bind_apply]
    rw [tsum_controlFlow, hCONT, hDONE,
      besl_true_coeff numer denom hdenom hfrac (dashu.ubigToNat k1) hk1,
      besl_false_coeff numer denom hdenom hfrac (dashu.ubigToNat k1) hk1]
    simp_rw [besl_cut_succ_true numer denom hdenom hfrac n ⟨dashu.ubigToNat k1, hk1⟩,
      add_mul, mul_assoc]
    rw [ENNReal.tsum_add, ENNReal.tsum_mul_left, ENNReal.tsum_mul_left]
    -- The two sides now differ only up to defeq: SampCert emits the successor counter as
    -- `⟨k1,hk1⟩ + 1` (defeq to `⟨k1+1,_⟩` via `ℕ+` addition) and its coefficient counter as the
    -- double coercion `↑↑⟨k1,hk1⟩` (defeq to `↑k1`), so `rfl` discharges the whole equation.
    rfl

/-! ### Increment 4 — full `probWhile` lift and the SampCert equality

The cut-depth correspondence (`exp1_loop_cut_step`) is lifted to the whole `probWhile` (via the
`⨆`/`tsum_iSup_commute` skeleton of `samplers/uniform/pmf.lean`), then bridged to SampCert's
`BernoulliExpNegSampleUnit`: SampCert's `..._sup`/`..._apply` identify the per-counter `probWhile`
mass with `BernoulliExpNegSampleUnitAux`, and `..._at_zero` lets the extracted `∑' : ℕ+` be reindexed
against SampCert's `∑' : ℕ`. -/

/-- Lift the cut-depth correspondence to the full `probWhile`: the extracted exp1 loop, started at
counter `kOne` (= 1), outputs `Ok b` with exactly the mass SampCert's `BESL` loop assigns to
terminating at some counter `m` with parity `b`. -/
lemma exp1_loop_probWhile (numer denom kOne : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (hkOne : dashu.ubigToNat kOne = 1) (b : Bool) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1_loop kOne denom numer kOne) b =
    ∑' m : ℕ+, probWhile (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac)
        (true, (1 : ℕ+)) (false, m) * (if decide ((m : ℕ) % 2 = 0) = b then 1 else 0) := by
  have hk1 : 0 < dashu.ubigToNat kOne := by rw [hkOne]; exact Nat.one_pos
  have hstate : (⟨dashu.ubigToNat kOne, hk1⟩ : ℕ+) = (1 : ℕ+) := Subtype.ext hkOne
  let cond : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) → Bool :=
    fun cf => match cf with | cont _ => true | done _ => false
  let bd : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) →
      SLang (ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :=
    fun cf => match cf with
      | cont a => samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body kOne denom numer a)
      | done _ => PMF.pure cf
  have hcc : ∀ a, cond (cont a) = true := fun _ => rfl
  have hcd : ∀ w, cond (done w) = false := fun _ => rfl
  have hbc : ∀ a, bd (cont a) =
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body kOne denom numer a) := fun _ => rfl
  -- Step 1: `samplerDist` of the extracted loop is `probWhile` of the body distribution.
  have hstep1 : samplerDist (samplers.bernoulli.sample_bernoulli_exp1_loop kOne denom numer kOne) b
      = probWhile cond bd (cont kOne) (done (core.result.Result.Ok b)) := by
    simp only [samplerDist, samplers.bernoulli.sample_bernoulli_exp1_loop, samplerDistGen_loop]
    congr 1 <;> (funext cf; cases cf <;> rfl)
  rw [hstep1]
  -- Step 2: unfold every `probWhile` to `⨆ probWhileCut`, pull the parity out of each sup, then
  -- commute `∑' m` past the `⨆ n` on the RHS.
  simp only [probWhile]
  simp_rw [ENNReal.iSup_mul]
  rw [tsum_iSup_commute _ (fun m => (probWhileCut_monotonic (fun s : Bool × ℕ+ => s.1)
      (BESL numer denom hdenom hfrac) (true, (1 : ℕ+)) (false, m)).mul_const (zero_le _))]
  -- Step 3: match each cut on the LHS via `exp1_loop_cut_step`.
  refine iSup_congr (fun n => ?_)
  rw [exp1_loop_cut_step numer denom kOne hdenom hfrac hkOne cond bd hcc hcd hbc n kOne hk1 b, hstate]

/-- The per-counter `probWhile` mass of the `BESL` loop is SampCert's `BernoulliExpNegSampleUnitAux`
at that counter (`..._sup` computes the `⨆ probWhileCut`, `..._apply` the closed form; they agree). -/
lemma probWhile_besl_eq_aux (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (n : ℕ+) :
    probWhile (fun s : Bool × ℕ+ => s.1) (BESL numer denom hdenom hfrac) (true, 1) (false, n) =
    BernoulliExpNegSampleUnitAux (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac ↑n := by
  rw [BernoulliExpNegSampleUnitAux_apply]
  simp only [BESL, probWhile]
  rw [BernoulliExpNegSampleUnitAux_sup]

/-- **The exp1 loop realises SampCert's unit negative-exponential sampler.** Combining the
`probWhile` lift, the per-counter `Aux` identification, and the `ℕ+`→`ℕ` reindex (valid since
`Aux 0 = 0`) with the parity split of `BernoulliExpNegSampleUnit`. -/
lemma sample_bernoulli_exp1_loop_spec (numer denom kOne : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (hkOne : dashu.ubigToNat kOne = 1) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1_loop kOne denom numer kOne) =
    BernoulliExpNegSampleUnit (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac := by
  funext b
  rw [exp1_loop_probWhile numer denom kOne hdenom hfrac hkOne b]
  simp_rw [probWhile_besl_eq_aux numer denom hdenom hfrac]
  -- RHS: unfold `BernoulliExpNegSampleUnit` into `∑' K:ℕ, Aux K * parity`.
  have hRHS : BernoulliExpNegSampleUnit (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac b =
      ∑' K : ℕ, BernoulliExpNegSampleUnitAux (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac K *
        (if decide (K % 2 = 0) = b then 1 else 0) := by
    simp only [BernoulliExpNegSampleUnit, Bind.bind, SLang.bind_apply]
    refine tsum_congr (fun K => ?_)
    congr 1
    by_cases hK : K % 2 = 0 <;> cases b <;> simp [hK, SLang.pure_apply]
  rw [hRHS]
  -- Reindex `∑' K:ℕ` to `∑' m:ℕ+` (the `K = 0` term vanishes since `Aux 0 = 0`). The vanishing
  -- proof is hoisted into a typed `have` so the summand function (hence `ENNReal`) is pinned before
  -- `Function.Injective.tsum_eq` needs the topology instance.
  have hbij : Function.support
        (fun K : ℕ => BernoulliExpNegSampleUnitAux (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac K *
          (if decide (K % 2 = 0) = b then 1 else 0)) ⊆ Set.range PNat.val := by
    intro K hK
    rcases Nat.eq_zero_or_pos K with h0 | hpos
    · exact absurd (by subst h0; simp [BernoulliExpNegSampleUnitAux_at_zero]) hK
    · exact ⟨⟨K, hpos⟩, rfl⟩
  exact Function.Injective.tsum_eq PNat.coe_injective hbij

/-- On the valid-input branch (captured by `RationalSetup`, reused from `rational.lean`), the
deterministic destructuring of `x` — `into_parts`, the positive-sign match, and `UBig.ONE` — reduces
`sample_bernoulli_exp1 x` to its loop started at counter `1`. -/
theorem sample_bernoulli_exp1_eq_of_setup (x : dashu_ratio.rbig.RBig) (setup : RationalSetup x) :
    ∃ kOne, dashu.ubigToNat kOne = 1 ∧
      samplers.bernoulli.sample_bernoulli_exp1 x =
        samplers.bernoulli.sample_bernoulli_exp1_loop kOne setup.denom setup.numer kOne := by
  obtain ⟨one, hone, honeval⟩ := dashu.one_exists_spec
  refine ⟨one, honeval, ?_⟩
  unfold samplers.bernoulli.sample_bernoulli_exp1
  simp [setup.hparts, setup.hsign, hone]

/-- **Distributional correctness (roadmap stage 4).** On the valid-input branch, the extracted
`sample_bernoulli_exp1` realises SampCert's `BernoulliExpNegSampleUnit` — i.e. `Bernoulli(e^{-x})`
for `x = numer/denom ∈ [0,1]`. -/
theorem sample_bernoulli_exp1_spec (x : dashu_ratio.rbig.RBig) (setup : RationalSetup x)
    (hdenom : 0 < dashu.ubigToNat setup.denom)
    (hfrac : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1 x) =
      BernoulliExpNegSampleUnit (dashu.ubigToNat setup.numer) ⟨dashu.ubigToNat setup.denom, hdenom⟩ hfrac := by
  obtain ⟨kOne, hkOne, heq⟩ := sample_bernoulli_exp1_eq_of_setup x setup
  rw [heq]
  exact sample_bernoulli_exp1_loop_spec setup.numer setup.denom kOne hdenom hfrac hkOne

end OpenDP.samplers.bernoulli
