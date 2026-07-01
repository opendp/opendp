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

/-- Concrete witnesses for the nonnegative-input branch of `sample_bernoulli_exp`. Unlike
`RationalSetup` (`rational.lean`), this does NOT require `numer ≤ denom` — the integral part is
handled by the outer loop. -/
structure BernoulliExpSetup (x : dashu_ratio.rbig.RBig) where
  numerSigned : dashu_int.ibig.IBig
  denom : dashu_int.ubig.UBig
  numer : dashu_int.ubig.UBig
  hparts : dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom)
  hsign : dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer)
  hdenom : 0 < dashu.ubigToNat denom

/-- On `[0,1]` (`num ≤ den`) the full negative-exponential sampler is exactly the unit sampler. -/
lemma bernoulliExpNegSample_of_le (num : ℕ) (den : ℕ+) (h : num ≤ den) :
    BernoulliExpNegSample num den = BernoulliExpNegSampleUnit num den h := by
  unfold BernoulliExpNegSample
  rw [dif_pos h]
  funext b
  simp only [Bind.bind, SLang.bind_apply, tsum_bool, SLang.pure_apply]
  cases b <;> simp

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

/-- The unit sampler on the extracted `1/1` rational realises `BernoulliExpNegSampleUnit 1 1`
(reuses stage 4; the `ubigToNat one = 1` witnesses collapse the indices). -/
lemma samplerDist_exp1_one (oneRat : dashu_ratio.rbig.RBig) (oneI : dashu_int.ibig.IBig)
    (oneU : dashu_int.ubig.UBig)
    (hparts : dashu_ratio.rbig.RBig.into_parts oneRat = ok (oneI, oneU))
    (hsign : dashu_int.ibig.IBig.into_parts oneI = ok (dashu_base.sign.Sign.Positive, oneU))
    (honeval : dashu.ubigToNat oneU = 1) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1 oneRat) = BernoulliExpNegSampleUnit 1 1 (le_refl 1) := by
  have hdenom1 : 0 < dashu.ubigToNat oneU := by rw [honeval]; norm_num
  have hvalid : dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt oneU oneU = ok false :=
    dashu.gt_false_of_le_spec oneU oneU (le_refl _)
  rw [sample_bernoulli_exp1_spec oneRat ⟨oneI, oneU, oneU, hparts, hsign, hvalid⟩ hdenom1 (le_refl _)]
  -- collapse `ubigToNat oneU` to `1`.
  have : (⟨dashu.ubigToNat oneU, hdenom1⟩ : ℕ+) = 1 := Subtype.ext honeval
  rw [this]
  congr 1
  exact honeval

/-- **Distributional correctness (roadmap stage 5).** On the nonnegative-input branch, the extracted
`sample_bernoulli_exp` realises SampCert's `BernoulliExpNegSample` — i.e. `Bernoulli(e^{-x})` for
`x = numer/denom ≥ 0`. Strong induction on `ubigToNat numer`: the base case (`≤ denom`) reuses
stage 4 via `sample_bernoulli_exp1`; the step case unfolds one loop iteration, distributes with
`samplerDist_bind`, and closes with the recursive law + IH. -/
theorem sample_bernoulli_exp_spec (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp x) =
      BernoulliExpNegSample (dashu.ubigToNat setup.numer) ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ := by
  suffices h : ∀ (n : ℕ) (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x),
      dashu.ubigToNat setup.numer = n →
      samplerDist (samplers.bernoulli.sample_bernoulli_exp x) =
        BernoulliExpNegSample (dashu.ubigToNat setup.numer) ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ from
    h _ x setup rfl
  intro n
  induction n using Nat.strong_induction_on with
  | _ n ih =>
    intro x setup hn
    obtain ⟨oneRat, oneI, oneU, honeRat, honeU, honePartsRat, honeSignRat⟩ := dashu.rbig_one_setup_spec
    have honeval : dashu.ubigToNat oneU = 1 := dashu.one_spec oneU honeU
    by_cases hle : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom
    · -- Base case `x ≤ 1`: the loop body settles immediately to `sample_bernoulli_exp1 x`.
      have hgt_false : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok false :=
        dashu.rbig_gt_one_false_of_le_spec x setup.numerSigned setup.denom setup.numer oneRat
          setup.hparts setup.hsign hle
      have hbody_le : samplers.bernoulli.sample_bernoulli_exp_loop.body x =
          (samplers.bernoulli.sample_bernoulli_exp1 x >>= fun r => ok (done r)) := by
        unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
        simp [honeRat, hgt_false]
      have hprog_le : samplers.bernoulli.sample_bernoulli_exp x = samplers.bernoulli.sample_bernoulli_exp1 x := by
        unfold samplers.bernoulli.sample_bernoulli_exp samplers.bernoulli.sample_bernoulli_exp_loop
        rw [loop.eq_def, hbody_le]
        rcases samplers.bernoulli.sample_bernoulli_exp1 x with r | e | _ <;> simp
      have hvalid : dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt setup.numer setup.denom = ok false :=
        dashu.gt_false_of_le_spec setup.numer setup.denom hle
      rw [hprog_le,
        sample_bernoulli_exp1_spec x
          ⟨setup.numerSigned, setup.denom, setup.numer, setup.hparts, setup.hsign, hvalid⟩ setup.hdenom hle,
        bernoulliExpNegSample_of_le (dashu.ubigToNat setup.numer) ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ hle]
    · -- Step case `x > 1`: one `Bernoulli(e^{-1})` draw, then recurse on `x - 1`.
      push_neg at hle
      have hgt_true : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok true :=
        dashu.rbig_gt_one_true_of_gt_spec x setup.numerSigned setup.denom setup.numer oneRat
          setup.hparts setup.hsign hle
      have hi : Aeneas.Std.lift (UScalar.cast UScalarTy.U128 1#u32) = ok (UScalar.cast UScalarTy.U128 1#u32) := by
        simp [Aeneas.Std.lift]
      have hfromparts : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
          (UScalar.cast UScalarTy.U128 1#u32) (UScalar.cast UScalarTy.U128 1#u32) = ok oneRat :=
        dashu.rbig_from_parts_const_eq_one (UScalar.cast UScalarTy.U128 1#u32) oneRat honeRat hi
      obtain ⟨xMinusOne, hsub⟩ := dashu.rbig_sub_assign_one_exists x oneRat setup.numerSigned setup.denom
        setup.numer setup.hparts setup.hsign (le_of_lt hle)
      obtain ⟨numer', _, hnumer'val⟩ := dashu.sub_exists_spec setup.numer setup.denom (le_of_lt hle)
      obtain ⟨i', _, hpartsX', hsignX'⟩ := dashu.rbig_sub_one_positive_spec x oneRat xMinusOne setup.numerSigned
        oneI setup.numer setup.denom numer' oneU setup.hparts setup.hsign honeU honePartsRat honeSignRat
        (le_of_lt hle) hsub
      let setupMinusOne : BernoulliExpSetup xMinusOne :=
        ⟨i', setup.denom, numer', hpartsX', hsignX', setup.hdenom⟩
      have hlt : dashu.ubigToNat numer' < n := by rw [hnumer'val, hn]; omega
      have hih := ih (dashu.ubigToNat numer') hlt xMinusOne setupMinusOne rfl
      -- One extracted loop iteration draws `exp1 oneRat`, continues (`x-1`) or settles.
      have hbody_gt : samplers.bernoulli.sample_bernoulli_exp_loop.body x =
          (samplers.bernoulli.sample_bernoulli_exp1 oneRat >>= fun r =>
            match r with
            | core.result.Result.Ok true => ok (cont xMinusOne)
            | core.result.Result.Ok false => ok (done (core.result.Result.Ok false))
            | core.result.Result.Err e => ok (done (core.result.Result.Err e))) := by
        unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
        simp [honeRat, hgt_true, hi, hfromparts, hsub,
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]
        congr 1; funext r; rcases r with (b | e) <;> simp
      have hprog_gt : samplers.bernoulli.sample_bernoulli_exp x =
          (do let r ← samplers.bernoulli.sample_bernoulli_exp1 oneRat
              match r with
              | core.result.Result.Ok v =>
                  (if v then samplers.bernoulli.sample_bernoulli_exp xMinusOne
                   else ok (core.result.Result.Ok false))
              | core.result.Result.Err e => ok (core.result.Result.Err e)) := by
        unfold samplers.bernoulli.sample_bernoulli_exp samplers.bernoulli.sample_bernoulli_exp_loop
        rw [loop.eq_def, hbody_gt]
        rcases samplers.bernoulli.sample_bernoulli_exp1 oneRat with (b | e) | _
        · cases b <;> rfl
        · rfl
        · rfl
      rw [hprog_gt, samplerDist_bind]
      simp only [apply_ite samplerDist, samplerDist_pure_ok,
        samplerDist_exp1_one oneRat oneI oneU honePartsRat honeSignRat honeval, hih]
      rw [bernoulliExpNegSample_succ (dashu.ubigToNat setup.numer) ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ hle,
        hnumer'val]
      congr 1
      funext v
      cases v <;> rfl

end OpenDP.samplers.bernoulli
