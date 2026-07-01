import Generated.OpenDP
import SampCert.Samplers.BernoulliNegativeExponential.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.bernoulli.rational
import src.samplers.bernoulli.exp1

/-!
# `sample_bernoulli_exp` — negative exponential for `x ≥ 0` (roadmap stage 5)

Target: `⟦sample_bernoulli_exp x⟧ = SLang.BernoulliExpNegSample` for `x = numer/denom ≥ 0`
(`Pr[⊤] = e^{-x}`). CKS construction: while `x > 1`, draw `Bernoulli(e^{-1})` and continue with
`x - 1` on success; once `x ≤ 1`, draw `Bernoulli(e^{-x})` (stage 4).

Unlike the exp1 loop, this loop **terminates** (the rational state strictly decreases by `1` each
step), so it is handled at the *program level* by the Aeneas `loop` fixpoint (`loop.eq_def`) plus a
**strong induction on `ubigToNat numer`** — no `probWhile` cut-depth machinery. The base case reuses
stage 4 (`sample_bernoulli_exp1_spec`); the recursive step reuses it for the per-iteration
`Bernoulli(e^{-1})` draw and closes against SampCert via the recursive law below.
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang ENNReal Classical

namespace OpenDP.samplers.bernoulli

/-- **SampCert-side recursive law.** For `x = num/den > 1`, `Bernoulli(e^{-num/den})` factors as one
`Bernoulli(e^{-1})` draw followed (on success) by `Bernoulli(e^{-(num-den)/den})`. Proved via the
closed form `e^{-num/den} = e^{-1} · e^{-(num-den)/den}` (`Real.exp_add`) plus normalization for the
`false` mass. This is the pure counterpart of one extracted loop iteration. -/
theorem bernoulliExpNegSample_succ (num : ℕ) (den : ℕ+) (hgt : (den : ℕ) < num) :
    BernoulliExpNegSample num den =
      (BernoulliExpNegSampleUnit 1 1 (le_refl 1) >>= fun b =>
        if b then BernoulliExpNegSample (num - den) den else pure false) := by
  have hden_le : (den : ℕ) ≤ num := le_of_lt hgt
  set RHS := (BernoulliExpNegSampleUnit 1 1 (le_refl 1) >>= fun b =>
    if b then BernoulliExpNegSample (num - den) den else pure false) with hRHSdef
  -- The real exponents split: `1 + (num-den)/den = num/den`.
  have hden0 : ((den : ℕ) : ℝ) ≠ 0 := by exact_mod_cast (PNat.ne_zero den)
  have hreal : (1 : ℝ) + (((num - den : ℕ) : ℝ) / ((den : ℕ) : ℝ))
      = ((num : ℕ) : ℝ) / ((den : ℕ) : ℝ) := by
    rw [Nat.cast_sub hden_le]
    field_simp
    ring
  -- `true` mass via the closed form `e^{-num/den} = e^{-1} · e^{-(num-den)/den}`.
  have htrue : RHS true = BernoulliExpNegSample num den true := by
    have hRtrue : RHS true =
        BernoulliExpNegSampleUnit 1 1 (le_refl 1) true * BernoulliExpNegSample (num - den) den true := by
      rw [hRHSdef]
      simp only [Bind.bind, SLang.bind_apply, tsum_bool, Bool.false_eq_true, if_false, if_true]
      rw [show ((pure false : SLang Bool) true) = 0 from by simp [SLang.pure_apply], mul_zero, zero_add]
    rw [hRtrue, BernoulliExpNegSampleUnit_apply_true 1 1 (le_refl 1) 1 (by simp),
      BernoulliExpNegSample_apply_true, BernoulliExpNegSample_apply_true, ENNReal.toReal_one,
      ← ENNReal.ofReal_mul (Real.exp_nonneg _), ← Real.exp_add]
    congr 2
    simp only [NNReal.coe_natCast]
    linarith [hreal]
  -- RHS normalizes, so the `false` mass is determined by the `true` mass.
  have hnorm : (∑' b : Bool, RHS b) = 1 := by
    rw [hRHSdef]
    simp only [Bind.bind, SLang.bind_apply]
    rw [ENNReal.tsum_comm]
    have hb : ∀ b0 : Bool, (∑' b : Bool,
        BernoulliExpNegSampleUnit 1 1 (le_refl 1) b0 *
          (if b0 then BernoulliExpNegSample (num - den) den else pure false) b) =
        BernoulliExpNegSampleUnit 1 1 (le_refl 1) b0 := by
      intro b0
      rw [ENNReal.tsum_mul_left]
      have hinner : (∑' b : Bool,
          (if b0 then BernoulliExpNegSample (num - den) den else pure false) b) = 1 := by
        cases b0
        · rw [show (if (false : Bool) then BernoulliExpNegSample (num - den) den
              else pure false) = (pure false : SLang Bool) from by simp]
          simp [SLang.pure_apply, tsum_bool]
        · rw [show (if (true : Bool) then BernoulliExpNegSample (num - den) den
              else pure false) = BernoulliExpNegSample (num - den) den from by simp]
          exact BernoulliExpNegSample_normalizes (num - den) den
      rw [hinner, mul_one]
    simp_rw [hb]
    exact BernoulliExpNegSampleUnit_normalizes 1 1 (le_refl 1) 1 (by simp)
  funext b
  cases b
  · rw [BernoulliExpNegSample_apply_false]
    have hsum := hnorm
    rw [tsum_bool, htrue, BernoulliExpNegSample_apply_true] at hsum
    rw [← hsum]
    simp
  · exact htrue.symm

/-- **One-step loop unfold at the distribution level.** For a *terminating* Aeneas `loop`, the
program-level fixpoint (`loop.eq_def`) plus `samplerDistGen_bind` gives a clean recursion: run the
body once, then either recurse (`cont`) or settle (`done`). This replaces the `probWhile` cut-depth
machinery needed for the unbounded exp1 loop. -/
lemma samplerDistGen_loop_unfold {α β : Type} (body : α → Result (ControlFlow α β)) (x : α) :
    samplerDistGen (loop body x) =
      SLang.probBind (samplerDistGen (body x)) (fun cf => match cf with
        | ControlFlow.cont x' => samplerDistGen (loop body x')
        | ControlFlow.done r => SLang.probPure r) := by
  have hprog : loop body x =
      (body x >>= fun r => match r with
        | ControlFlow.cont x' => loop body x'
        | ControlFlow.done v => ok v) := by
    conv_lhs => rw [loop.eq_def]
    cases body x <;> rfl
  rw [hprog, samplerDistGen_bind]
  congr 1
  funext cf
  cases cf with
  | cont x' => rfl
  | done v => exact samplerDistGen_pure_ok v

end OpenDP.samplers.bernoulli
