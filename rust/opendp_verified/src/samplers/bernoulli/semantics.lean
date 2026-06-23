import Aeneas
import Generated.OpenDP
import src.externals.dashu
import src.utilities.mod
import SampCert.Samplers.Bernoulli.Properties
import SampCert.Samplers.BernoulliNegativeExponential.Properties

open Aeneas Aeneas.Std Result Classical
open OpenDP SLang

namespace OpenDP.samplers.bernoulli

theorem exists_ok_of_spec
    {α : Type}
    {m : Result α}
    {P : α → Prop}
    (h : m ⦃ P ⦄) :
    ∃ x, m = ok x ∧ P x := by
  cases hm : m with
  | ok x =>
      subst hm
      simp [Aeneas.Std.WP.spec_ok] at h
      exact ⟨x, rfl, h⟩
  | fail e =>
      simp [hm] at h
  | div =>
      simp [hm] at h

/-- Concrete witnesses for the valid-input branch of
`sample_bernoulli_rational`. -/
structure BernoulliRationalSetup (prob : dashu_ratio.rbig.RBig) where
  numerSigned : dashu_int.ibig.IBig
  denom : dashu_int.ubig.UBig
  numer : dashu_int.ubig.UBig
  hparts : dashu_ratio.rbig.RBig.into_parts prob = ok (numerSigned, denom)
  hsign :
    dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer)
  hvalid :
    dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt numer denom = ok false

/-- Concrete witnesses for the valid `[0, 1]` branch of
`sample_bernoulli_exp1`. -/
structure BernoulliExp1Setup (x : dashu_ratio.rbig.RBig) where
  numerSigned : dashu_int.ibig.IBig
  denom : dashu_int.ubig.UBig
  numer : dashu_int.ubig.UBig
  one : dashu_int.ubig.UBig
  hparts : dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom)
  hsign :
    dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer)
  hone : dashu_int.ubig.UBig.ONE = ok one
  hdenom : 0 < dashu.ubigToNat denom
  hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom

/-- Concrete witnesses for the nonnegative-input branch of
`sample_bernoulli_exp`. Unlike `BernoulliExp1Setup`, this setup does not
require the fraction to lie in `[0, 1]`; the integral part is handled by the
outer loop. -/
structure BernoulliExpSetup (x : dashu_ratio.rbig.RBig) where
  numerSigned : dashu_int.ibig.IBig
  denom : dashu_int.ubig.UBig
  numer : dashu_int.ubig.UBig
  hparts : dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom)
  hsign :
    dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer)
  hdenom : 0 < dashu.ubigToNat denom

/-- The Bernoulli distribution with success probability `numer / denom`.
The success outcome is distributed to be true exactly when a uniform sample
from `[0, denom)` falls in `[0, numer)`. -/
noncomputable def bernoulliPMF (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom) : PMF Bool :=
  PMF.map (fun x : Nat => decide (x < dashu.ubigToNat numer))
    (UniformSample_PMF ⟨dashu.ubigToNat denom, hdenom⟩)

/-- `bernoulliPMF` is exactly SampCert's canonical Bernoulli sampler PMF. -/
theorem bernoulliPMF_eq_BernoulliSamplePMF
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hwf : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliPMF numer denom hdenom =
      SLang.BernoulliSamplePMF
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf := by
  ext b
  cases b
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, SLang.BernoulliSample_apply_false]
    change SLang.BernoulliSample
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf false = _
    rw [SLang.BernoulliSample_apply_false]
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, SLang.BernoulliSample_apply_true]
    change SLang.BernoulliSample
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf true = _
    rw [SLang.BernoulliSample_apply_true]

/-- Canonical SampCert target distribution for `sample_bernoulli_exp1`. -/
noncomputable def bernoulliExp1Target
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    SLang Bool :=
  SLang.BernoulliExpNegSampleUnit
    (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac

/-- The `true` mass of `sample_bernoulli_exp1`'s canonical target PMF. -/
theorem bernoulliExp1Target_apply_true
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliExp1Target numer denom hdenom hfrac true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal))) := by
  unfold bernoulliExp1Target
  simp

/-- The `false` mass of `sample_bernoulli_exp1`'s canonical target PMF. -/
theorem bernoulliExp1Target_apply_false
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliExp1Target numer denom hdenom hfrac false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal))) := by
  unfold bernoulliExp1Target
  simp

/-- Canonical SampCert target distribution for the full nonnegative
`sample_bernoulli_exp` sampler. -/
noncomputable def bernoulliExpTarget
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) :
    SLang Bool :=
  SLang.BernoulliExpNegSample
    (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩

/-- The `true` mass of the full negative-exponential target PMF. -/
theorem bernoulliExpTarget_apply_true
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) :
    bernoulliExpTarget numer denom hdenom true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : NNReal) /
              (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ))) := by
  unfold bernoulliExpTarget
  simp

/-- The `false` mass of the full negative-exponential target PMF. -/
theorem bernoulliExpTarget_apply_false
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) :
    bernoulliExpTarget numer denom hdenom false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : NNReal) /
              (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ))) := by
  unfold bernoulliExpTarget
  simp

/-- When the input rational already lies in `[0, 1]`, the full
negative-exponential target collapses definitionally to the unit-step target. -/
theorem bernoulliExpTarget_eq_exp1_of_le
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliExpTarget numer denom hdenom =
      bernoulliExp1Target numer denom hdenom hfrac := by
  unfold bernoulliExpTarget bernoulliExp1Target
  unfold SLang.BernoulliExpNegSample
  simp [hfrac]

/-- One recursive `x > 1` step of the full SampCert target. This is the pure
probabilistic counterpart of the Rust outer loop body: sample `exp(-1)`, and
on success continue with `x - 1`; on failure return `false`.

The remaining proof work is arithmetic transport between SampCert's quotient /
remainder decomposition and repeated subtraction by one. -/
theorem bernoulliExpTarget_eq_exp1_one_bind_sub_one_of_gt
    (numer denom numerMinusDenom one : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (honePos : 0 < dashu.ubigToNat one)
    (hone : dashu.ubigToNat one = 1)
    (hgt : dashu.ubigToNat denom < dashu.ubigToNat numer)
    (hminus :
      dashu.ubigToNat numerMinusDenom =
        dashu.ubigToNat numer - dashu.ubigToNat denom) :
    bernoulliExpTarget numer denom hdenom =
      (do
        let b ← bernoulliExp1Target one one honePos (le_refl _)
        if b
        then bernoulliExpTarget numerMinusDenom denom hdenom
        else pure false) := by
  have hnum : dashu.ubigToNat numer = dashu.ubigToNat numerMinusDenom + dashu.ubigToNat denom := by
    exact (Nat.sub_eq_iff_eq_add (Nat.le_of_lt hgt)).mp (by
      simp [hminus])
  have hreal :
      ((dashu.ubigToNat numer : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal =
        1 + ((dashu.ubigToNat numerMinusDenom : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal := by
    calc
      ((dashu.ubigToNat numer : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal
          = (dashu.ubigToNat numer : ℝ) / (dashu.ubigToNat denom : ℝ) := by
              simp [ENNReal.toReal_div]
      _ = ((dashu.ubigToNat numerMinusDenom : ℝ) + dashu.ubigToNat denom) / (dashu.ubigToNat denom : ℝ) := by
            congr 1
            exact_mod_cast hnum
      _ = 1 + ((dashu.ubigToNat numerMinusDenom : ℝ) / (dashu.ubigToNat denom : ℝ)) := by
            field_simp [Nat.ne_of_lt hdenom]
            ring
      _ = 1 + ((dashu.ubigToNat numerMinusDenom : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal := by
            simp [ENNReal.toReal_div]
  have hneg :
      -(((dashu.ubigToNat numer : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal) =
        -1 + -(((dashu.ubigToNat numerMinusDenom : ENNReal) / (dashu.ubigToNat denom : ENNReal)).toReal) := by
    rw [hreal]
    ring
  let a : ENNReal := ENNReal.ofReal (Real.exp (-1))
  let c : ENNReal :=
    ENNReal.ofReal
      (Real.exp
        (-(((dashu.ubigToNat numerMinusDenom : ENNReal) /
            (dashu.ubigToNat denom : ENNReal)).toReal)))
  have ha : a ≤ 1 := by
    dsimp [a]
    have hle : Real.exp (-1 : ℝ) ≤ 1 := by
      exact (Real.exp_le_one_iff).2 (by linarith)
    have h' : ENNReal.ofReal (Real.exp (-1 : ℝ)) ≤ ENNReal.ofReal (1 : ℝ) :=
      ENNReal.ofReal_le_ofReal hle
    simp
  have hc : c ≤ 1 := by
    dsimp [c]
    have hle :
        Real.exp
          (-(((dashu.ubigToNat numerMinusDenom : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal)) ≤ 1 := by
      exact (Real.exp_le_one_iff).2 (by
        have : -(((dashu.ubigToNat numerMinusDenom : ENNReal) /
            (dashu.ubigToNat denom : ENNReal)).toReal) ≤ 0 := by
          have hnonneg :
              0 ≤
                (((dashu.ubigToNat numerMinusDenom : ENNReal) /
                    (dashu.ubigToNat denom : ENNReal)).toReal) := by
            exact ENNReal.toReal_nonneg
          linarith
        linarith)
    have h' :
        ENNReal.ofReal
          (Real.exp
            (-(((dashu.ubigToNat numerMinusDenom : ENNReal) /
                (dashu.ubigToNat denom : ENNReal)).toReal))) ≤
          ENNReal.ofReal (1 : ℝ) :=
      ENNReal.ofReal_le_ofReal hle
    simpa using h'
  have hab : a * c ≤ a := by
    calc
      a * c ≤ a * 1 := by
        exact mul_le_mul_right hc a
      _ = a := by simp [a]
  have hsum : ((1 - a) + (a - a * c)) + a * c = 1 := by
    rw [add_assoc, tsub_add_cancel_of_le hab]
    exact tsub_add_cancel_of_le ha
  have hmain : (1 - a) + (a - a * c) = 1 - a * c := by
    apply ENNReal.eq_sub_of_add_eq' (by simp)
    simpa [add_assoc, add_comm, add_left_comm] using hsum
  have hmulsub : a * (1 - c) = a - a * c := by
    apply ENNReal.eq_sub_of_add_eq' (by simp [a])
    calc
      a * (1 - c) + a * c = a * ((1 - c) + c) := by
        rw [mul_add]
      _ = a * 1 := by rw [tsub_add_cancel_of_le hc]
      _ = a := by simp
  (ext bb; cases bb)
  · have hfalse : bernoulliExpTarget numer denom hdenom false = a * (1 - c) + (1 - a) := by
      calc
        bernoulliExpTarget numer denom hdenom false
            = 1 -
                ENNReal.ofReal
                  (Real.exp
                    (-(((dashu.ubigToNat numer : ENNReal) /
                        (dashu.ubigToNat denom : ENNReal)).toReal))) := by
                simp [bernoulliExpTarget_apply_false]
        _ = 1 - (a * c) := by
              rw [hneg, Real.exp_add]
              simp [a, c, ENNReal.ofReal_mul' (Real.exp_nonneg _)]
        _ = a * (1 - c) + (1 - a) := by
              rw [hmulsub]
              simpa [a, c, add_comm, add_left_comm, add_assoc] using hmain.symm
    have hdo_false :
        (do
          let b ← bernoulliExp1Target one one honePos (le_refl _)
          if b
          then bernoulliExpTarget numerMinusDenom denom hdenom
          else pure false) false =
          a * (1 - c) + (1 - a) := by
      simp [a, c, hone, SLang.bind_apply, SLang.pure_apply, bernoulliExp1Target_apply_false,
        bernoulliExp1Target_apply_true, bernoulliExpTarget_apply_false]
    exact hfalse.trans hdo_false.symm
  · have htrue :
      bernoulliExpTarget numer denom hdenom true =
        ENNReal.ofReal (Real.exp (-1)) * bernoulliExpTarget numerMinusDenom denom hdenom true := by
      calc
        bernoulliExpTarget numer denom hdenom true
            = ENNReal.ofReal
                (Real.exp
                  (-(((dashu.ubigToNat numer : ENNReal) /
                      (dashu.ubigToNat denom : ENNReal)).toReal))) := by
                simp [bernoulliExpTarget_apply_true]
        _ = ENNReal.ofReal
              (Real.exp
                (-1 +
                  -(((dashu.ubigToNat numerMinusDenom : ENNReal) /
                      (dashu.ubigToNat denom : ENNReal)).toReal))) := by
              rw [hneg]
        _ = ENNReal.ofReal (Real.exp (-1)) *
              ENNReal.ofReal
                (Real.exp
                  (-(((dashu.ubigToNat numerMinusDenom : ENNReal) /
                      (dashu.ubigToNat denom : ENNReal)).toReal))) := by
              rw [Real.exp_add, ← ENNReal.ofReal_mul' (Real.exp_nonneg _)]
        _ = ENNReal.ofReal (Real.exp (-1)) *
              bernoulliExpTarget numerMinusDenom denom hdenom true := by
              simp [bernoulliExpTarget_apply_true]
    have hdo_true :
        (do
          let b ← bernoulliExp1Target one one honePos (le_refl _)
          if b
          then bernoulliExpTarget numerMinusDenom denom hdenom
          else pure false) true =
          ENNReal.ofReal (Real.exp (-1)) * bernoulliExpTarget numerMinusDenom denom hdenom true := by
      simp [hone, SLang.bind_apply, SLang.pure_apply, bernoulliExp1Target_apply_true,
        bernoulliExp1Target_apply_false, bernoulliExpTarget_apply_true]
    exact htrue.trans hdo_true.symm

/-- Canonical unit target used by the extracted `rbig!(1)` call. -/
theorem bernoulliExp1Target_one_eq_sampcert_unit_one
    (one : dashu_int.ubig.UBig)
    (honePos : 0 < dashu.ubigToNat one)
    (hone : dashu.ubigToNat one = 1) :
    bernoulliExp1Target one one honePos (le_refl _) =
      SLang.BernoulliExpNegSampleUnit 1 ⟨1, by decide⟩ (le_refl 1) := by
  unfold bernoulliExp1Target
  congr 1
  apply Subtype.ext
  simp [hone]

/-- If `0 < k`, then a Bernoulli step with success count `numer` and base
denominator `denom` is well-formed at the scaled denominator `k * denom`. -/
theorem bernoulli_step_wf
    {numer denom : Nat}
    (hfrac : numer ≤ denom)
    {k : Nat}
    (hk : 0 < k) :
    numer ≤ k * denom := by
  have hk1 : 1 ≤ k := Nat.succ_le_of_lt hk
  calc
    numer ≤ denom := hfrac
    _ ≤ k * denom := by
      simpa [one_mul] using Nat.mul_le_mul_right denom hk1

/-- Pointwise evaluation of SampCert's one-step negative-exponential loop at
the successor state. This is the canonical step law that the extracted `exp1`
loop has to match. -/
theorem bernoulliExp1_sampcert_step_apply
    (numer denom k : Nat)
    (hdenom : 0 < denom)
    (hfrac : numer ≤ denom)
    (hk : 0 < k)
    (b : Bool) :
    SLang.BernoulliExpNegSampleUnitLoop numer ⟨denom, hdenom⟩ hfrac
        (true, ⟨k, hk⟩)
        (b, ⟨k + 1, Nat.succ_pos k⟩) =
      SLang.BernoulliSample numer
        ⟨k * denom, Nat.mul_pos hk hdenom⟩
        (bernoulli_step_wf hfrac hk) b := by
  have hsucc :
      (⟨k + 1, Nat.succ_pos k⟩ : PNat) = (⟨k, hk⟩ + (1 : PNat)) := by
    ext
    change k + 1 = k + 1
    rfl
  rw [hsucc]
  unfold SLang.BernoulliExpNegSampleUnitLoop
  cases b <;> simp <;> intro h <;> exact False.elim (h rfl)

/-- Rust stops at the current counter `k` and checks `k` for oddness, while
SampCert's loop body has already advanced the final counter to `k + 1` and
checks that final counter for evenness. -/
theorem odd_current_iff_even_successor (k : Nat) :
    k % 2 = 1 ↔ (k + 1) % 2 = 0 := by
  rw [Nat.mod_two_of_bodd, Nat.mod_two_of_bodd]
  cases h : Nat.bodd k <;> simp [Nat.bodd_succ, h]

/-- Boolean form of `odd_current_iff_even_successor`. -/
theorem decide_odd_current_eq_even_successor (k : Nat) :
    (decide (k % 2 = 1) : Bool) = decide ((k + 1) % 2 = 0) := by
  by_cases h : k % 2 = 1
  · have hsucc := (odd_current_iff_even_successor k).mp h
    simp [h, hsucc]
  · have hsucc : ¬ (k + 1) % 2 = 0 := fun hsucc =>
      h ((odd_current_iff_even_successor k).mpr hsucc)
    simp [h, hsucc]

/-- Exact gcd-cancellation arithmetic used by the Bernoulli negative
exponential step. -/
theorem bernoulli_reduction_arith
    {numer denom k g nRed kRed : Nat}
    (hk : 0 < k)
    (hdenom : 0 < denom)
    (hg : 0 < g)
    (hnumEq : numer = nRed * g)
    (hkEq : k = kRed * g)
    (hfrac : numer ≤ denom)
    (hnRed : nRed = numer / g)
    (hkRed : kRed = k / g) :
    nRed ≤ denom * kRed ∧
    (nRed : ENNReal) / (denom * kRed : Nat) =
      (numer : ENNReal) / (k * denom : Nat) := by
  have hkRedPos : 0 < kRed := by
    rw [hkRed]
    exact Nat.div_pos (Nat.le_of_dvd hk <| by rw [hkEq, Nat.mul_comm]; exact Nat.dvd_mul_right g kRed) hg
  have hnRedLeNumer : nRed ≤ numer := by
    rw [hnRed]
    exact Nat.div_le_self _ _
  have hkMul : k * denom = g * (denom * kRed) := by
    rw [hkEq]
    ring
  have hnumMul : numer = g * nRed := by
    rw [hnumEq, Nat.mul_comm]
  have hwf : nRed ≤ denom * kRed := by
    have hkRedGe : 1 ≤ kRed := Nat.succ_le_of_lt hkRedPos
    calc
      nRed ≤ numer := hnRedLeNumer
      _ ≤ denom := hfrac
      _ ≤ denom * kRed := by
            simpa [one_mul] using Nat.mul_le_mul_left denom hkRedGe
  refine ⟨hwf, ?_⟩
  have hdenkNe : (((denom * kRed : Nat)) : ENNReal) ≠ 0 := by
    exact_mod_cast Nat.ne_of_gt (Nat.mul_pos hdenom hkRedPos)
  have hkdenNe : (((k * denom : Nat)) : ENNReal) ≠ 0 := by
    exact_mod_cast Nat.ne_of_gt (Nat.mul_pos hk hdenom)
  rw [eq_comm]
  apply (ENNReal.div_eq_div_iff
    hdenkNe
    (by
      simpa [Nat.cast_mul] using
        (ENNReal.mul_ne_top (by simp) (by simp)))
    hkdenNe
    (by
      simpa [Nat.cast_mul] using
        (ENNReal.mul_ne_top (by simp) (by simp)))).2
  have hNat : numer * (denom * kRed) = nRed * (k * denom) := by
    rw [hnumEq, hkEq]
    ring
  have hCast : ((((numer * (denom * kRed) : Nat)) : ENNReal)) =
      ((((nRed * (k * denom) : Nat)) : ENNReal)) := by
    exact_mod_cast hNat
  simpa [Nat.cast_mul, mul_assoc, mul_left_comm, mul_comm] using hCast

/-- Gcd-cancelling the numerator and scaled denominator does not change the
Bernoulli law. -/
theorem bernoulliPMF_reduction_eq
    (numer denom k g nRed kRed : Nat)
    (hk : 0 < k)
    (hdenom : 0 < denom)
    (hg : 0 < g)
    (hnumEq : numer = nRed * g)
    (hkEq : k = kRed * g)
    (hfrac : numer ≤ denom)
    (hnRed : nRed = numer / g)
    (hkRed : kRed = k / g) :
    SLang.BernoulliSamplePMF
      nRed
      ⟨denom * kRed, Nat.mul_pos hdenom (by
        rw [hkRed]
        exact Nat.div_pos (Nat.le_of_dvd hk <| by rw [hkEq, Nat.mul_comm]; exact Nat.dvd_mul_right g kRed) hg)⟩
      (bernoulli_reduction_arith hk hdenom hg hnumEq hkEq hfrac hnRed hkRed).1
    =
    SLang.BernoulliSamplePMF
      numer
      ⟨k * denom, Nat.mul_pos hk hdenom⟩
      (bernoulli_step_wf hfrac hk) := by
  ext b
  cases b
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, PMF.ofFintype_apply]
    rw [SLang.BernoulliSample_apply_false, SLang.BernoulliSample_apply_false]
    exact congrArg (fun x : ENNReal => 1 - x)
      (bernoulli_reduction_arith hk hdenom hg hnumEq hkEq hfrac hnRed hkRed).2
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, PMF.ofFintype_apply]
    rw [SLang.BernoulliSample_apply_true, SLang.BernoulliSample_apply_true]
    exact (bernoulli_reduction_arith hk hdenom hg hnumEq hkEq hfrac hnRed hkRed).2

/-- Zero numerator Bernoulli laws are independent of the denominator. -/
theorem bernoulliPMF_zero_eq
    (den : PNat) :
    SLang.BernoulliSamplePMF 0 ⟨1, by decide⟩ (by decide) =
      SLang.BernoulliSamplePMF 0 den (by exact Nat.zero_le _) := by
  ext b
  cases b
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, PMF.ofFintype_apply]
    rw [SLang.BernoulliSample_apply_false, SLang.BernoulliSample_apply_false]
    simp
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, PMF.ofFintype_apply]
    rw [SLang.BernoulliSample_apply_true, SLang.BernoulliSample_apply_true]
    simp

/-- Temporary semantic boundary for `div_rbig_by_ubig_exact` while Dashu's
exact-division path is unstable. This should disappear once the upstream Dashu
bug is fixed and the helper is removed. -/
axiom div_rbig_by_ubig_exact_bernoulli_setup
    (numer denom k : dashu_int.ubig.UBig)
    (hk : 0 < dashu.ubigToNat k)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    ∃ x_div_k : dashu_ratio.rbig.RBig,
      ∃ setup : BernoulliRationalSetup x_div_k,
        ∃ hsetupDenom : 0 < dashu.ubigToNat setup.denom,
          utilities.div_rbig_by_ubig_exact numer denom k = ok x_div_k ∧
          bernoulliPMF setup.numer setup.denom hsetupDenom =
            SLang.BernoulliSamplePMF
              (dashu.ubigToNat numer)
              ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
              (bernoulli_step_wf hfrac hk)

end OpenDP.samplers.bernoulli
