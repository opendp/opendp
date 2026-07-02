import Generated.OpenDP
import SampCert.Samplers.Laplace.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.bernoulli.rational
import src.samplers.bernoulli.pmf
import src.samplers.geometric.fast

/-!
# `sample_discrete_laplace` — discrete Laplace noise (roadmap stage 8)

Target: `⟦sample_discrete_laplace numer denom⟧ℤ = SLang.DiscreteLaplaceSample num den` for
scale `t = numer/denom` — the pure-DP noise mechanism. CKS construction: draw a geometric
magnitude with failure parameter `1 - e^{-denom/numer}` (stage 7), a fair sign bit (stage 3 at
`1/2`), reject the `(negative, 0)` outcome, and return the signed magnitude.

The extracted loop is a genuine Aeneas `loop` with `Unit` state (each iteration is
independent), so the rejection analysis is a *scalar* geometric series — simpler than stage 7.
Signed outputs are pushed to `ℤ` with `samplerDist_int` over the new `dashu.ibigToInt`
interpretation (positive-parts and negation axioms in `dashu.lean`).
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Classical

namespace OpenDP.samplers.laplace

open OpenDP.samplers.uniform (samplerDist_nat)
open OpenDP.samplers.bernoulli (BernoulliExpSetup RationalSetup sample_bernoulli_rational_pmf
  bernoulliPMF_eq_BernoulliSamplePMF)
open OpenDP.samplers.geometric (geoTrial sample_geometric_exp_fast_spec tsum_samplerDist_nat)

/-- The `ℤ`-valued SLang distribution of a `Result`-monad program that outputs `IBig`. -/
noncomputable def samplerDist_int
    (prog : Result (core.result.Result dashu_int.ibig.IBig error.Error)) : SLang ℤ :=
  SLang.probBind (samplerDist prog) (fun i => SLang.probPure (dashu.ibigToInt i))

/-- Push an `ibigToInt`-factored weight through an `IBig`-valued program. -/
lemma tsum_samplerDist_int
    (prog : Result (core.result.Result dashu_int.ibig.IBig error.Error)) (g : ℤ → ENNReal) :
    (∑' j : dashu_int.ibig.IBig, samplerDist prog j * g (dashu.ibigToInt j)) =
      ∑' z : ℤ, samplerDist_int prog z * g z := by
  symm
  simp only [samplerDist_int, SLang.probBind]
  have hpush : ∀ z : ℤ,
      (∑' j : dashu_int.ibig.IBig,
        samplerDist prog j * SLang.probPure (dashu.ibigToInt j) z) * g z =
      ∑' j : dashu_int.ibig.IBig,
        samplerDist prog j * SLang.probPure (dashu.ibigToInt j) z * g z :=
    fun z => (ENNReal.tsum_mul_right).symm
  simp_rw [hpush]
  rw [ENNReal.tsum_comm]
  refine tsum_congr fun j => ?_
  rw [tsum_eq_single (dashu.ibigToInt j) (fun z hz => by simp [SLang.probPure, hz])]
  simp [SLang.probPure]

/-- Index collapse for the fair coin: values `1`/`2` pin SampCert's `BernoulliSample 1 2`. -/
lemma bernoulliSample_collapse (mv nv : ℕ) (hn : 0 < nv) (wf : mv ≤ nv)
    (hm : mv = 1) (hn2 : nv = 2) :
    (SLang.BernoulliSamplePMF mv ⟨nv, hn⟩ wf : SLang Bool) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) := by
  subst hm; subst hn2
  funext b
  show (SLang.BernoulliSamplePMF 1 ⟨2, hn⟩ wf : SLang Bool) b = _
  unfold SLang.BernoulliSamplePMF
  rw [PMF.ofFintype_apply]
  rfl

/-- The extracted `1/2` constant drives SampCert's fair coin. -/
lemma coin_law (halfRat : dashu_ratio.rbig.RBig) (i2 : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (i2, two))
    (hsign : dashu_int.ibig.IBig.into_parts i2 = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2) (b : Bool) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_rational halfRat)
        (core.result.Result.Ok b) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) b := by
  have hvalid : dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt one two = ok false :=
    dashu.gt_false_of_le_spec one two (by rw [h1, h2]; norm_num)
  have h2pos : 0 < dashu.ubigToNat two := by rw [h2]; norm_num
  have h := congrFun (sample_bernoulli_rational_pmf halfRat
    ⟨i2, two, one, hparts, hsign, hvalid⟩ h2pos) b
  rw [bernoulliPMF_eq_BernoulliSamplePMF one two h2pos (by rw [h1, h2]; norm_num)] at h
  rw [bernoulliSample_collapse (dashu.ubigToNat one) (dashu.ubigToNat two) h2pos _ h1 h2] at h
  simpa [samplerDist] using h

/-! ### The loop body, factored through the fast-geometric draw -/

/-- The post-magnitude continuation of one Laplace loop iteration: draw the fair coin, reject
`(negative, 0)`, otherwise settle with the signed magnitude. -/
noncomputable def lap_step (val_unused : Unit) :
    core.result.Result dashu_int.ubig.UBig error.Error →
    Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :=
  fun r1 => do
    let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r1
    match cf with
    | core.ops.control_flow.ControlFlow.Continue val =>
      let i ← lift (UScalar.cast .U128 1#u32)
      let i1 ← lift (UScalar.cast .U128 2#u32)
      let r2 ←
        dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i1
      let r3 ← samplers.bernoulli.sample_bernoulli_rational r2
      let cf1 ←
        core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r3
      match cf1 with
      | core.ops.control_flow.ControlFlow.Continue val1 =>
        if val1
        then
          let b ← dashu_int.ubig.UBig.is_zero val
          if b
          then ok (cont ())
          else
            let i2 ← dashu_int.convert.UBig.as_ibig val
            let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
            let i4 ← dashu_int.ibig.IBig.Insts.CoreOpsArithNegIBig.neg i3
            ok (done (core.result.Result.Ok i4))
        else
          let i2 ← dashu_int.convert.UBig.as_ibig val
          let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
          ok (done (core.result.Result.Ok i3))
      | core.ops.control_flow.ControlFlow.Break residual =>
        let r4 ←
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ibig.IBig (core.convert.FromSame error.Error) residual
        ok (done r4)
    | core.ops.control_flow.ControlFlow.Break residual =>
      let r2 ←
        core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
          dashu_int.ibig.IBig (core.convert.FromSame error.Error) residual
      ok (done r2)

/-- Once the (deterministic) clone succeeds, the Laplace body factors through `lap_step`. -/
lemma lap_body_eq_step (x r : dashu_ratio.rbig.RBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r) :
    samplers.laplace.sample_discrete_laplace_loop.body x =
      samplers.geometric.sample_geometric_exp_fast r >>= lap_step () := by
  unfold samplers.laplace.sample_discrete_laplace_loop.body
  rw [hclone]
  rfl

/-- Fixed witnesses for the extracted `1/2` constant. -/
lemma half_const_exists :
    ∃ (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
      (one two : dashu_int.ubig.UBig),
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
        (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat ∧
      dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two) ∧
      dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one) ∧
      dashu.ubigToNat one = 1 ∧
      dashu.ubigToNat two = 2 :=
  dashu.rbig_from_parts_const_half_exists_spec _ _
    (by simp [UScalar.cast_val_eq]) (by simp [UScalar.cast_val_eq])

/-- Step on `Err e`: a point mass at `done (Err e)`. -/
lemma lap_step_err (e : error.Error)
    (out : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :
    samplerDistGen (lap_step () (core.result.Result.Err e)) out =
      (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [lap_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    from_residual_err_ok, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step's `cont` mass on a drawn magnitude: negative sign times the zero indicator. -/
lemma lap_step_cont (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hhalf : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
      (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two))
    (hsign : dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2)
    (val : dashu_int.ubig.UBig) :
    samplerDistGen (lap_step () (core.result.Result.Ok val)) (cont ()) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (if 0 = dashu.ubigToNat val then 1 else 0) := by
  have hstep : lap_step () (core.result.Result.Ok val) =
      (samplers.bernoulli.sample_bernoulli_rational halfRat >>= fun r3 =>
        match r3 with
        | core.result.Result.Ok val1 =>
          if val1
          then
            (do
              let b ← dashu_int.ubig.UBig.is_zero val
              if b
              then ok (cont ())
              else do
                let i2 ← dashu_int.convert.UBig.as_ibig val
                let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
                let i4 ← dashu_int.ibig.IBig.Insts.CoreOpsArithNegIBig.neg i3
                ok (done (core.result.Result.Ok i4)))
          else do
            let i2 ← dashu_int.convert.UBig.as_ibig val
            let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
            ok (done (core.result.Result.Ok i3))
        | core.result.Result.Err e => ok (done (core.result.Result.Err e))) := by
    unfold lap_step
    simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
      bind_tc_ok, lift]
    rw [hhalf]
    simp only [bind_tc_ok]
    congr 1
    funext r3
    rcases r3 with val1 | e
    · rfl
    · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
        from_residual_err_ok]
  rw [hstep, samplerDistGen_bind, SLang.probBind,
    tsum_result_ok_eq (fun e => by simp [samplerDistGen_pure_ok, PMF.pure_apply]),
    tsum_bool]
  rw [coin_law halfRat iH one two hparts hsign h1 h2 false,
    coin_law halfRat iH one two hparts hsign h1 h2 true]
  by_cases hv : dashu.ubigToNat val = 0
  · -- zero magnitude: negate branch continues, positive branch settles.
    have hz : dashu_int.ubig.UBig.is_zero val = ok true := by
      obtain ⟨b, hb⟩ := dashu.is_zero_exists_spec val
      cases b
      · exact absurd (dashu.is_zero_false_spec val hb) (by omega)
      · exact hb
    obtain ⟨i2, hI, _⟩ := dashu.as_ibig_exists_spec val
    obtain ⟨i3, hC⟩ := dashu.ibig_clone_exists_spec i2
    simp [hz, hI, hC, hv, samplerDistGen_pure_ok, PMF.pure_apply]
  · -- nonzero magnitude: both branches settle; `cont` carries no mass.
    have hz : dashu_int.ubig.UBig.is_zero val = ok false :=
      dashu.is_zero_of_pos_spec val (Nat.pos_of_ne_zero hv)
    obtain ⟨i2, hI, _⟩ := dashu.as_ibig_exists_spec val
    obtain ⟨i3, hC⟩ := dashu.ibig_clone_exists_spec i2
    obtain ⟨i4, hN, _⟩ := dashu.ibig_neg_exists_spec i3
    simp [hz, hI, hC, hN, Ne.symm hv, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step's settle mass on a drawn magnitude, summed against the signed-output indicator. -/
lemma lap_step_done_summed (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hhalf : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
      (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two))
    (hsign : dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2)
    (val : dashu_int.ubig.UBig) (z : ℤ) :
    (∑' j : dashu_int.ibig.IBig,
      samplerDistGen (lap_step () (core.result.Result.Ok val))
          (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (if 0 = dashu.ubigToNat val then 0
         else if z = -(dashu.ubigToNat val : ℤ) then 1 else 0) +
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
        (if z = (dashu.ubigToNat val : ℤ) then 1 else 0) := by
  obtain ⟨i2, hI, hIparts⟩ := dashu.as_ibig_exists_spec val
  obtain ⟨i3, hC⟩ := dashu.ibig_clone_exists_spec i2
  have hI3parts : dashu_int.ibig.IBig.into_parts i3 =
      ok (dashu_base.sign.Sign.Positive, val) :=
    dashu.ibig_clone_parts_spec i2 i3 _ hC hIparts
  have hI3val : dashu.ibigToInt i3 = (dashu.ubigToNat val : ℤ) :=
    dashu.ibigToInt_pos_spec i3 val hI3parts
  obtain ⟨i4, hN, hNval⟩ := dashu.ibig_neg_exists_spec i3
  have hI4val : dashu.ibigToInt i4 = -(dashu.ubigToNat val : ℤ) := by rw [hNval, hI3val]
  have hstep : samplerDistGen (lap_step () (core.result.Result.Ok val)) =
      samplerDistGen ((samplers.bernoulli.sample_bernoulli_rational halfRat >>= fun r3 =>
        match r3 with
        | core.result.Result.Ok val1 =>
          if val1
          then
            (do
              let b ← dashu_int.ubig.UBig.is_zero val
              if b
              then ok (cont ())
              else do
                let i2 ← dashu_int.convert.UBig.as_ibig val
                let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
                let i4 ← dashu_int.ibig.IBig.Insts.CoreOpsArithNegIBig.neg i3
                ok (done (core.result.Result.Ok i4)))
          else do
            let i2 ← dashu_int.convert.UBig.as_ibig val
            let i3 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i2
            ok (done (core.result.Result.Ok i3))
        | core.result.Result.Err e => ok (done (core.result.Result.Err e)))) := by
    congr 1
    unfold lap_step
    simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
      bind_tc_ok, lift]
    rw [hhalf]
    simp only [bind_tc_ok]
    congr 1
    funext r3
    rcases r3 with val1 | e
    · rfl
    · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
        from_residual_err_ok]
  simp_rw [hstep, samplerDistGen_bind, SLang.probBind]
  simp_rw [← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  rw [tsum_result_ok_eq (fun e => by
    refine ENNReal.tsum_eq_zero.mpr fun j => ?_
    simp [samplerDistGen_pure_ok, PMF.pure_apply])]
  rw [tsum_bool]
  rw [coin_law halfRat iH one two hparts hsign h1 h2 false,
    coin_law halfRat iH one two hparts hsign h1 h2 true]
  by_cases hv : dashu.ubigToNat val = 0
  · have hz : dashu_int.ubig.UBig.is_zero val = ok true := by
      obtain ⟨b, hb⟩ := dashu.is_zero_exists_spec val
      cases b
      · exact absurd (dashu.is_zero_false_spec val hb) (by omega)
      · exact hb
    have hfalse : (∑' j : dashu_int.ibig.IBig,
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          samplerDistGen (ok (done (core.result.Result.Ok i3)) :
              Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
            (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0)) =
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          (if z = (dashu.ubigToNat val : ℤ) then 1 else 0) := by
      simp_rw [samplerDistGen_pure_ok, PMF.pure_apply, mul_assoc]
      rw [ENNReal.tsum_mul_left]
      congr 1
      rw [tsum_eq_single i3 (fun j hj => by
        rw [if_neg (fun h => by
          injection h with hh
          injection hh with hh2
          exact hj hh2), zero_mul]),
        if_pos rfl, one_mul, hI3val]
    simp only [hz, hI, hC, bind_tc_ok, if_true, Bool.false_eq_true, if_false]
    rw [show (∑' j : dashu_int.ibig.IBig,
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          samplerDistGen (ok (cont ()) :
              Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
            (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0)) = 0 from
      ENNReal.tsum_eq_zero.mpr fun j => by simp [samplerDistGen_pure_ok, PMF.pure_apply]]
    rw [hfalse]
    simp [hv]
  · have hz : dashu_int.ubig.UBig.is_zero val = ok false :=
      dashu.is_zero_of_pos_spec val (Nat.pos_of_ne_zero hv)
    simp only [hz, hI, hC, hN, bind_tc_ok, if_true, Bool.false_eq_true, if_false]
    have htrue : (∑' j : dashu_int.ibig.IBig,
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          samplerDistGen (ok (done (core.result.Result.Ok i4)) :
              Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
            (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0)) =
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          (if z = -(dashu.ubigToNat val : ℤ) then 1 else 0) := by
      simp_rw [samplerDistGen_pure_ok, PMF.pure_apply, mul_assoc]
      rw [ENNReal.tsum_mul_left]
      congr 1
      rw [tsum_eq_single i4 (fun j hj => by
        rw [if_neg (fun h => by
          injection h with hh
          injection hh with hh2
          exact hj hh2), zero_mul]),
        if_pos rfl, one_mul, hI4val]
    have hfalse : (∑' j : dashu_int.ibig.IBig,
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          samplerDistGen (ok (done (core.result.Result.Ok i3)) :
              Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
            (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0)) =
        BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          (if z = (dashu.ubigToNat val : ℤ) then 1 else 0) := by
      simp_rw [samplerDistGen_pure_ok, PMF.pure_apply, mul_assoc]
      rw [ENNReal.tsum_mul_left]
      congr 1
      rw [tsum_eq_single i3 (fun j hj => by
        rw [if_neg (fun h => by
          injection h with hh
          injection hh with hh2
          exact hj hh2), zero_mul]),
        if_pos rfl, one_mul, hI3val]
    rw [htrue, hfalse]
    simp [hv]
    ring

/-! ### Body fiber laws (abstract in the magnitude law) -/

/-- The body's `cont` mass: negative sign times the magnitude law at `0`. -/
lemma lap_body_cont (x r : dashu_ratio.rbig.RBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (magLaw : SLang ℕ)
    (hfast : samplerDist_nat (samplers.geometric.sample_geometric_exp_fast r) = magLaw)
    (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hhalf : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
      (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two))
    (hsign : dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2) :
    samplerDistGen (samplers.laplace.sample_discrete_laplace_loop.body x) (cont ()) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true * magLaw 0 := by
  rw [lap_body_eq_step x r hclone, samplerDistGen_bind, SLang.probBind,
    tsum_result_ok_eq (fun e => by rw [lap_step_err]; simp)]
  simp_rw [lap_step_cont halfRat iH one two hhalf hparts hsign h1 h2]
  have hre : ∀ val : dashu_int.ubig.UBig,
      samplerDistGen (samplers.geometric.sample_geometric_exp_fast r)
          (core.result.Result.Ok val) *
        (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          (if 0 = dashu.ubigToNat val then 1 else 0)) =
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (samplerDistGen (samplers.geometric.sample_geometric_exp_fast r)
            (core.result.Result.Ok val) *
          (if 0 = dashu.ubigToNat val then 1 else 0)) := fun val => by ring
  simp_rw [hre]
  rw [ENNReal.tsum_mul_left]
  congr 1
  refine Eq.trans (tsum_samplerDist_nat (samplers.geometric.sample_geometric_exp_fast r)
    (fun n => if 0 = n then 1 else 0)) ?_
  rw [hfast]
  rw [tsum_eq_single 0 (fun n hn => by rw [if_neg (fun h => hn h.symm), mul_zero]),
    if_pos rfl, mul_one]

/-- The body's settle mass summed against the signed-output indicator. -/
lemma lap_body_done_summed (x r : dashu_ratio.rbig.RBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (magLaw : SLang ℕ)
    (hfast : samplerDist_nat (samplers.geometric.sample_geometric_exp_fast r) = magLaw)
    (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hhalf : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
      (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two))
    (hsign : dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2) (z : ℤ) :
    (∑' j : dashu_int.ibig.IBig,
      samplerDistGen (samplers.laplace.sample_discrete_laplace_loop.body x)
          (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) =
      ∑' m : ℕ, magLaw m *
        (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          (if 0 = m then 0 else if z = -(m : ℤ) then 1 else 0) +
         BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          (if z = (m : ℤ) then 1 else 0)) := by
  simp_rw [lap_body_eq_step x r hclone, samplerDistGen_bind, SLang.probBind,
    ← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  simp_rw [mul_assoc, ENNReal.tsum_mul_left]
  rw [tsum_result_ok_eq (fun e => by simp [lap_step_err])]
  simp_rw [lap_step_done_summed halfRat iH one two hhalf hparts hsign h1 h2]
  refine Eq.trans (tsum_samplerDist_nat (samplers.geometric.sample_geometric_exp_fast r)
    (fun m => BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (if 0 = m then 0 else if z = -(m : ℤ) then 1 else 0) +
      BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
        (if z = (m : ℤ) then 1 else 0))) ?_
  rw [hfast]

/-! ### Scalar rejection series and the `probWhile` limit -/

/-- Truncation closed form for a unit-state rejection loop with per-iteration settle law `A`
and self-loop mass `ρ`. -/
private lemma lap_cut_closed
    (cond : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) → Bool)
    (bd : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) →
      SLang (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
    (hcc : ∀ a, cond (cont a) = true)
    (hcd : ∀ w, cond (done w) = false)
    (ρ : ENNReal) (A : ℤ → ENNReal)
    (hρ : bd (cont ()) (cont ()) = ρ)
    (hA : ∀ z : ℤ, (∑' j : dashu_int.ibig.IBig,
      bd (cont ()) (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) = A z) :
    ∀ (k : ℕ) (z : ℤ),
      (∑' j : dashu_int.ibig.IBig,
        probWhileCut cond bd (k + 1) (cont ()) (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0)) =
      A z * ∑ i ∈ Finset.range k, ρ ^ i := by
  intro k
  induction k with
  | zero =>
    intro z
    have h1 : ∀ j : dashu_int.ibig.IBig,
        probWhileCut cond bd 1 (cont ()) (done (core.result.Result.Ok j)) = 0 := by
      intro j
      rw [probWhileCut, probWhileFunctional, if_pos (hcc ())]
      simp only [Bind.bind, SLang.bind_apply, probWhileCut, SLang.probZero, mul_zero,
        tsum_zero]
    simp [h1]
  | succ k ih =>
    intro z
    have hunf : ∀ j : dashu_int.ibig.IBig,
        probWhileCut cond bd (k + 1 + 1) (cont ()) (done (core.result.Result.Ok j)) =
          ρ * probWhileCut cond bd (k + 1) (cont ()) (done (core.result.Result.Ok j)) +
          bd (cont ()) (done (core.result.Result.Ok j)) := by
      intro j
      rw [probWhileCut, probWhileFunctional, if_pos (hcc ())]
      simp only [Bind.bind, SLang.bind_apply]
      rw [tsum_controlFlow]
      congr 1
      · rw [tsum_eq_single () (fun a ha => absurd (Subsingleton.elim a ()) ha), hρ]
      · simp_rw [probWhileCut_done_pt cond bd hcd k, SLang.pure_apply]
        rw [tsum_eq_single (core.result.Result.Ok j) (fun r' hr' => by
          rw [if_neg (fun h => hr' ((ControlFlow.done.inj h).symm)), mul_zero]),
          if_pos rfl, mul_one]
    simp_rw [hunf, add_mul]
    rw [ENNReal.tsum_add]
    simp_rw [mul_assoc]
    rw [ENNReal.tsum_mul_left, ih z, hA z]
    have hgeom : (∑ i ∈ Finset.range (k + 1), ρ ^ i) =
        1 + ρ * ∑ i ∈ Finset.range k, ρ ^ i := by
      rw [Finset.sum_range_succ']
      simp_rw [pow_succ']
      rw [← Finset.mul_sum, pow_zero]
      ring
    rw [hgeom]
    ring

/-- `probWhile` limit of the unit-state rejection loop: `A(z) / (1 - ρ)`. -/
private lemma lap_probWhile_closed
    (cond : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) → Bool)
    (bd : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) →
      SLang (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)))
    (hcc : ∀ a, cond (cont a) = true)
    (hcd : ∀ w, cond (done w) = false)
    (ρ : ENNReal) (A : ℤ → ENNReal)
    (hρ : bd (cont ()) (cont ()) = ρ)
    (hA : ∀ z : ℤ, (∑' j : dashu_int.ibig.IBig,
      bd (cont ()) (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) = A z) (z : ℤ) :
    (∑' j : dashu_int.ibig.IBig,
      probWhile cond bd (cont ()) (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) = A z * (1 - ρ)⁻¹ := by
  simp only [probWhile]
  simp_rw [ENNReal.iSup_mul]
  rw [tsum_iSup_commute (fun j k => probWhileCut cond bd k (cont ())
      (done (core.result.Result.Ok j)) * (if z = dashu.ibigToInt j then 1 else 0))
    (fun j => (probWhileCut_monotonic cond bd (cont ())
      (done (core.result.Result.Ok j))).mul_const (zero_le _))]
  have hmono : Monotone (fun k => ∑' j : dashu_int.ibig.IBig,
      probWhileCut cond bd k (cont ()) (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) := by
    intro k1 k2 hk
    exact ENNReal.tsum_le_tsum fun j =>
      mul_le_mul_right' (probWhileCut_monotonic cond bd (cont ())
        (done (core.result.Result.Ok j)) hk) _
  have hshift : (⨆ k : ℕ, ∑' j : dashu_int.ibig.IBig,
      probWhileCut cond bd k (cont ()) (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) =
      ⨆ k : ℕ, ∑' j : dashu_int.ibig.IBig,
        probWhileCut cond bd (k + 1) (cont ()) (done (core.result.Result.Ok j)) *
          (if z = dashu.ibigToInt j then 1 else 0) := by
    refine le_antisymm (iSup_le fun k => ?_) (iSup_le fun k => le_iSup_of_le (k + 1) le_rfl)
    exact le_iSup_of_le k (hmono (Nat.le_succ k))
  rw [hshift]
  simp_rw [lap_cut_closed cond bd hcc hcd ρ A hρ hA]
  rw [← ENNReal.mul_iSup, ← ENNReal.tsum_eq_iSup_nat, ENNReal.tsum_geometric]

/-! ### Lift, wrapper, and the SampCert equality -/

/-- Lift: the extracted Laplace loop's `ℤ`-law is the scalar rejection closed form. -/
private lemma lap_loop_lift (x r : dashu_ratio.rbig.RBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (magLaw : SLang ℕ)
    (hfast : samplerDist_nat (samplers.geometric.sample_geometric_exp_fast r) = magLaw)
    (halfRat : dashu_ratio.rbig.RBig) (iH : dashu_int.ibig.IBig)
    (one two : dashu_int.ubig.UBig)
    (hhalf : dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
      (UScalar.cast .U128 1#u32) (UScalar.cast .U128 2#u32) = ok halfRat)
    (hparts : dashu_ratio.rbig.RBig.into_parts halfRat = ok (iH, two))
    (hsign : dashu_int.ibig.IBig.into_parts iH = ok (dashu_base.sign.Sign.Positive, one))
    (h1 : dashu.ubigToNat one = 1) (h2 : dashu.ubigToNat two = 2) (z : ℤ) :
    samplerDist_int (samplers.laplace.sample_discrete_laplace_loop x) z =
      (∑' m : ℕ, magLaw m *
        (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
          (if 0 = m then 0 else if z = -(m : ℤ) then 1 else 0) +
         BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
          (if z = (m : ℤ) then 1 else 0))) *
      (1 - BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true * magLaw 0)⁻¹ := by
  let cond : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) → Bool :=
    fun cf => match cf with | cont _ => true | done _ => false
  let bd : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) →
      SLang (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :=
    fun cf => match cf with
      | cont _ => samplerDistGen (samplers.laplace.sample_discrete_laplace_loop.body x)
      | done _ => PMF.pure cf
  have hcc : ∀ a, cond (cont a) = true := fun _ => rfl
  have hcd : ∀ w, cond (done w) = false := fun _ => rfl
  have hstep1 : ∀ j : dashu_int.ibig.IBig,
      samplerDist (samplers.laplace.sample_discrete_laplace_loop x) j =
        probWhile cond bd (cont ()) (done (core.result.Result.Ok j)) := by
    intro j
    simp only [samplerDist, samplers.laplace.sample_discrete_laplace_loop,
      samplerDistGen_loop]
    congr 1 <;> (funext cf; cases cf <;> rfl)
  have hexpand : samplerDist_int (samplers.laplace.sample_discrete_laplace_loop x) z =
      ∑' j : dashu_int.ibig.IBig,
        samplerDist (samplers.laplace.sample_discrete_laplace_loop x) j *
          (if z = dashu.ibigToInt j then 1 else 0) := by
    simp only [samplerDist_int, SLang.probBind, SLang.probPure]
    refine tsum_congr fun j => ?_
    by_cases h : z = dashu.ibigToInt j <;> simp [h]
  rw [hexpand]
  simp_rw [hstep1]
  exact lap_probWhile_closed cond bd hcc hcd
    (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true * magLaw 0)
    (fun z => ∑' m : ℕ, magLaw m *
      (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (if 0 = m then 0 else if z = -(m : ℤ) then 1 else 0) +
       BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
        (if z = (m : ℤ) then 1 else 0)))
    (lap_body_cont x r hclone magLaw hfast halfRat iH one two hhalf hparts hsign h1 h2)
    (fun z => lap_body_done_summed x r hclone magLaw hfast halfRat iH one two hhalf hparts
      hsign h1 h2 z) z

/-- The scalar closed form equals SampCert's `DiscreteLaplaceSample` (pure SLang algebra). -/
private lemma lap_closed_form_eq (num den : ℕ+) (z : ℤ) :
    (∑' m : ℕ, probGeometric (BernoulliExpNegSample (den : ℕ) num) (m + 1) *
      (BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
        (if 0 = m then 0 else if z = -(m : ℤ) then 1 else 0) +
       BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false *
        (if z = (m : ℤ) then 1 else 0))) *
    (1 - BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true *
      probGeometric (BernoulliExpNegSample (den : ℕ) num) 1)⁻¹ =
    DiscreteLaplaceSample num den z := by
  -- Common constant: the SampCert Laplace-loop parameter.
  set p : ENNReal := ENNReal.ofReal (Real.exp (-((den : ℕ+) / (num : ℕ+) : ℝ))) with hp
  -- The coin values.
  have hcT : BernoulliSample 1 2 (Nat.le.step Nat.le.refl) true = 2⁻¹ := by
    simp [BernoulliSample_apply, one_div]
  have hcF : BernoulliSample 1 2 (Nat.le.step Nat.le.refl) false = 2⁻¹ := by
    simp [BernoulliSample_apply, one_div, ENNReal.one_sub_inv_two]
  -- The geometric masses in terms of `p`.
  have hmag : ∀ m : ℕ, probGeometric (BernoulliExpNegSample (den : ℕ) num) (m + 1) =
      p ^ m * (1 - p) := by
    intro m
    rw [probGeometric_apply]
    rw [if_neg (Nat.succ_ne_zero m), Nat.add_sub_cancel]
    rw [BernoulliExpNegSample_apply_true, BernoulliExpNegSample_apply_false]
    rw [hp]
    congr 2 <;> · congr 1; push_cast; ring
  -- RHS: unfold the sampler through the normalized `probUntil`.
  simp only [DiscreteLaplaceSample, Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  simp_rw [probUntil_apply_norm _ _ _ (DiscreteLaplaceSampleLoop_normalizes num den)]
  have hcomm : ∀ (f g : (Bool × ℕ) → ENNReal) (c : ENNReal),
      (∑' st : Bool × ℕ, f st * c * g st) = (∑' st : Bool × ℕ, f st * g st) * c := by
    intro f g c
    rw [← ENNReal.tsum_mul_right]
    exact tsum_congr fun st => by ring
  rw [hcomm]
  congr 1
  · -- numerator
    rw [ENNReal.tsum_prod', tsum_bool]
    simp_rw [hmag, hcT, hcF, mul_add]
    rw [ENNReal.tsum_add, add_comm]
    congr 1
    · -- positive outputs (coin false)
      refine tsum_congr fun n => ?_
      rw [DiscreteLaplaceSampleLoop_apply]
      by_cases h : z = (n : ℤ) <;> simp [h] <;> ring
    · -- negative outputs (coin true), zero rejected
      refine tsum_congr fun n => ?_
      rw [DiscreteLaplaceSampleLoop_apply]
      by_cases h0 : n = 0
      · subst h0
        simp
      · by_cases h : z = -(n : ℤ) <;> simp [h, h0] <;> ring
  · -- denominator
    congr 1
    rw [hmag 0, hcT, pow_zero, one_mul]
    symm
    refine Eq.trans (tsum_congr (g := fun st => if st = ((true, 0) : Bool × ℕ) then 0
      else DiscreteLaplaceSampleLoop num den st) fun st => ?_) ?_
    · by_cases hst : st = ((true, 0) : Bool × ℕ)
      · subst hst; simp
      · have hne : ¬(st.1 = true ∧ st.2 = 0) := fun h => hst (Prod.ext h.1 h.2)
        simp [hst, hne]
    · have hnorm := DiscreteLaplaceSampleLoop_normalizes num den
      have hsplit := ENNReal.tsum_eq_add_tsum_ite
        (f := fun st : Bool × ℕ => DiscreteLaplaceSampleLoop num den st)
        ((true, 0) : Bool × ℕ)
      rw [hnorm] at hsplit
      have hLzero : DiscreteLaplaceSampleLoop num den ((true, 0) : Bool × ℕ) =
          (1 - p) * 2⁻¹ := by
        rw [DiscreteLaplaceSampleLoop_apply, pow_zero, one_mul, ← hp]
        congr 1 <;> simp
      have hfin : DiscreteLaplaceSampleLoop num den ((true, 0) : Bool × ℕ) ≠ ⊤ := by
        rw [hLzero]
        exact ENNReal.mul_ne_top (by simp) (by simp)
      have hrest : (∑' st : Bool × ℕ,
          if st = ((true, 0) : Bool × ℕ) then 0 else DiscreteLaplaceSampleLoop num den st) =
          1 - DiscreteLaplaceSampleLoop num den ((true, 0) : Bool × ℕ) := by
        rw [hsplit, ENNReal.add_sub_cancel_left hfin]
        refine tsum_congr fun st => ?_
        by_cases hst : st = ((true, 0) : Bool × ℕ) <;> simp [hst]
      rw [hrest, hLzero, mul_comm (1 - p) 2⁻¹]

/-- **Distributional correctness (roadmap stage 8).** On strictly positive scale
`numer/denom`, the extracted `sample_discrete_laplace` realises SampCert's
`DiscreteLaplaceSample` — the pure-DP discrete Laplace noise mechanism on `ℤ`. -/
theorem sample_discrete_laplace_spec (numer denom : dashu_int.ubig.UBig)
    (hnum : 0 < dashu.ubigToNat numer) (hdenom : 0 < dashu.ubigToNat denom) :
    samplerDist_int (samplers.laplace.sample_discrete_laplace numer denom) =
      DiscreteLaplaceSample ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩ := by
  obtain ⟨iD, hI, hIparts⟩ := dashu.as_ibig_exists_spec denom
  obtain ⟨i1, hC⟩ := dashu.ibig_clone_exists_spec iD
  have hi1parts : dashu_int.ibig.IBig.into_parts i1 =
      ok (dashu_base.sign.Sign.Positive, denom) :=
    dashu.ibig_clone_parts_spec iD i1 _ hC hIparts
  obtain ⟨x, hF, hXparts⟩ :=
    dashu.rbig_from_parts_positive_exists_spec denom numer i1 hnum hi1parts
  obtain ⟨r, hclone⟩ := dashu.rbig_clone_exists_spec x
  have hRparts := dashu.rbig_clone_parts_spec x r _ hclone hXparts
  have hfast := sample_geometric_exp_fast_spec r
    ⟨i1, numer, denom, hRparts, hi1parts, hnum⟩ hdenom
  obtain ⟨halfRat, iH, one, two, hhalf, hparts, hsign, h1, h2⟩ := half_const_exists
  have hprog : samplers.laplace.sample_discrete_laplace numer denom =
      samplers.laplace.sample_discrete_laplace_loop x := by
    unfold samplers.laplace.sample_discrete_laplace
    rw [hI]; simp only [bind_tc_ok]
    rw [hC]; simp only [bind_tc_ok]
    rw [hF]; simp only [bind_tc_ok]
  funext z
  rw [hprog]
  rw [lap_loop_lift x r hclone
    (fun v => probGeometric (BernoulliExpNegSample
      ((⟨dashu.ubigToNat denom, hdenom⟩ : ℕ+) : ℕ)
      ⟨dashu.ubigToNat numer, hnum⟩) (v + 1))
    hfast halfRat iH one two hhalf hparts hsign h1 h2 z]
  exact lap_closed_form_eq ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩ z

end OpenDP.samplers.laplace
