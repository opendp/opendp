import Generated.OpenDP
import SampCert.Samplers.Gaussian.Code
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.laplace

/-!
# `sample_discrete_gaussian` — discrete Gaussian noise (roadmap stage 9, **final target**)

Target: `⟦sample_discrete_gaussian numer denom⟧ℤ = SLang.DiscreteGaussianSample num den mix` —
the zCDP noise mechanism. CKS rejection construction: propose a discrete Laplace candidate `Y`
with scale `t = ⌊numer/denom⌋ + 1` (stage 8), accept with probability
`e^{-(|Y|·t·den − num)²/(2·num·t²·den)}` over `σ²`'s parts `num = numer²`, `den = denom²`
(stage 5), reject otherwise. The extracted loop has `Unit` state, so the rejection analysis
reuses the stage-8 scalar-series lemmas (`lap_cut_closed`/`lap_probWhile_closed`) verbatim; the
closed form meets SampCert through `probUntil_apply_norm`, `bind_bind_pair_apply_true`, and
`DiscreteLaplaceSampleMixed_equiv` (which also makes the result independent of SampCert's `mix`
parameter, as it must be).
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Classical

namespace OpenDP.samplers.gaussian

open OpenDP.samplers.bernoulli (BernoulliExpSetup sample_bernoulli_exp_spec)
open OpenDP.samplers.laplace (samplerDist_int tsum_samplerDist_int sample_discrete_laplace_spec
  lap_cut_closed lap_probWhile_closed)

/-- The candidate-dependent acceptance trial of the Gaussian rejection loop, at the
mathematical values `tN = ⌊σ⌋+1`, `nN = numer²`, `dN = denom²`. -/
noncomputable def gaussAccept (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (Y : ℤ) : SLang Bool :=
  BernoulliExpNegSample
    ((((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs *
      (((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs)
    ⟨2 * nN * tN * tN * dN, hD⟩

/-- The post-candidate continuation of one Gaussian loop iteration (mirrors the extracted body
after the Laplace draw). -/
noncomputable def gauss_step (u t num den u1 : dashu_int.ubig.UBig) :
    core.result.Result dashu_int.ibig.IBig error.Error →
    Result (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :=
  fun r => do
    let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r
    match cf with
    | core.ops.control_flow.ControlFlow.Continue val =>
      let i ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone val
      let u2 ← dashu_int.ibig.IBig.Insts.Dashu_baseSignUnsignedAbsUBig.unsigned_abs i
      let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u2 u1
      let u4 ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone den
      let lhs ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u3 u4
      let i1 ← dashu_int.convert.UBig.as_ibig lhs
      let i2 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i1
      let i3 ← dashu_int.convert.UBig.as_ibig num
      let i4 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i3
      let diff ← dashu_int.ibig.IBig.Insts.CoreOpsArithSubIBigIBig.sub i2 i4
      let n_abs ← dashu_int.ibig.IBig.Insts.Dashu_baseSignUnsignedAbsUBig.unsigned_abs diff
      let u5 ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone n_abs
      let n ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u5 n_abs
      let u6 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u u
      let u7 ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone num
      let u8 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u6 u7
      let u9 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u8 u1
      let u10 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u9 u1
      let d ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul u10 u4
      let i5 ← dashu_int.convert.UBig.as_ibig n
      let i6 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i5
      let r1 ← dashu_ratio.rbig.RBig.from_parts i6 d
      let r2 ← samplers.bernoulli.sample_bernoulli_exp r1
      let cf1 ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r2
      match cf1 with
      | core.ops.control_flow.ControlFlow.Continue val1 =>
        if val1
        then ok (done (core.result.Result.Ok val))
        else ok (cont ())
      | core.ops.control_flow.ControlFlow.Break residual =>
        let r3 ←
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ibig.IBig (core.convert.FromSame error.Error) residual
        ok (done r3)
    | core.ops.control_flow.ControlFlow.Break residual =>
      let r1 ←
        core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
          dashu_int.ibig.IBig (core.convert.FromSame error.Error) residual
      ok (done r1)

/-- Once the (deterministic) scale clone succeeds, the Gaussian body factors through
`gauss_step`. -/
lemma gauss_body_eq_step (u t num den u1 : dashu_int.ubig.UBig)
    (hcloneT : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone t = ok u1) :
    samplers.gaussian.sample_discrete_gaussian_loop.body u t num den =
      samplers.laplace.sample_discrete_laplace u1 u >>= gauss_step u t num den u1 := by
  unfold samplers.gaussian.sample_discrete_gaussian_loop.body
  rw [hcloneT]
  rfl

/-- Step on `Err e`: a point mass at `done (Err e)`. -/
lemma gauss_step_err (u t num den u1 : dashu_int.ubig.UBig) (e : error.Error)
    (out : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :
    samplerDistGen (gauss_step u t num den u1 (core.result.Result.Err e)) out =
      (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [gauss_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    from_residual_err_ok, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Index collapse for the acceptance trial. -/
lemma bernExpNeg_cast (n1 n2 d1 d2 : ℕ) (h1 : 0 < d1) (h2 : 0 < d2)
    (hn : n1 = n2) (hd : d1 = d2) :
    BernoulliExpNegSample n1 ⟨d1, h1⟩ = BernoulliExpNegSample n2 ⟨d2, h2⟩ := by
  subst hn; subst hd; rfl

/-- The deterministic arithmetic chain of `gauss_step` on a drawn candidate, packaged: the step
factors through the acceptance trial at the candidate's integer value. -/
private lemma gauss_step_chain (u t num den u1 : dashu_int.ubig.UBig)
    (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (huv : dashu.ubigToNat u = 1)
    (hu1v : dashu.ubigToNat u1 = tN)
    (hnumv : dashu.ubigToNat num = nN)
    (hdenv : dashu.ubigToNat den = dN)
    (val : dashu_int.ibig.IBig) :
    ∃ (r1 : dashu_ratio.rbig.RBig),
      gauss_step u t num den u1 (core.result.Result.Ok val) =
        (samplers.bernoulli.sample_bernoulli_exp r1 >>= fun r2 =>
          match r2 with
          | core.result.Result.Ok true => ok (done (core.result.Result.Ok val))
          | core.result.Result.Ok false => ok (cont ())
          | core.result.Result.Err e => ok (done (core.result.Result.Err e))) ∧
      ∀ b : Bool,
        samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r1) (core.result.Result.Ok b) =
          gaussAccept tN nN dN hD (dashu.ibigToInt val) b := by
  obtain ⟨i, hi⟩ := dashu.ibig_clone_exists_spec val
  have hiv : dashu.ibigToInt i = dashu.ibigToInt val := dashu.ibig_clone_int_spec val i hi
  obtain ⟨u2, hu2, hu2v⟩ := dashu.ibig_unsigned_abs_exists_spec i
  obtain ⟨u3, hu3, hu3v⟩ := dashu.mul_ubig_exists_spec u2 u1
  obtain ⟨u4, hu4, hu4v⟩ := dashu.clone_exists_spec den
  obtain ⟨lhs, hlhs, hlhsv⟩ := dashu.mul_ubig_exists_spec u3 u4
  obtain ⟨i1, hi1, hi1parts⟩ := dashu.as_ibig_exists_spec lhs
  have hi1v : dashu.ibigToInt i1 = (dashu.ubigToNat lhs : ℤ) :=
    dashu.ibigToInt_pos_spec i1 lhs hi1parts
  obtain ⟨i2, hi2⟩ := dashu.ibig_clone_exists_spec i1
  have hi2v : dashu.ibigToInt i2 = dashu.ibigToInt i1 := dashu.ibig_clone_int_spec i1 i2 hi2
  obtain ⟨i3, hi3, hi3parts⟩ := dashu.as_ibig_exists_spec num
  have hi3v : dashu.ibigToInt i3 = (dashu.ubigToNat num : ℤ) :=
    dashu.ibigToInt_pos_spec i3 num hi3parts
  obtain ⟨i4, hi4⟩ := dashu.ibig_clone_exists_spec i3
  have hi4v : dashu.ibigToInt i4 = dashu.ibigToInt i3 := dashu.ibig_clone_int_spec i3 i4 hi4
  obtain ⟨diff, hdiff, hdiffv⟩ := dashu.ibig_sub_exists_spec i2 i4
  obtain ⟨n_abs, hnabs, hnabsv⟩ := dashu.ibig_unsigned_abs_exists_spec diff
  obtain ⟨u5, hu5, hu5v⟩ := dashu.clone_exists_spec n_abs
  obtain ⟨n, hn, hnv⟩ := dashu.mul_ubig_exists_spec u5 n_abs
  obtain ⟨u6, hu6, hu6v⟩ := dashu.add_ubig_exists_spec u u
  obtain ⟨u7, hu7, hu7v⟩ := dashu.clone_exists_spec num
  obtain ⟨u8, hu8, hu8v⟩ := dashu.mul_ubig_exists_spec u6 u7
  obtain ⟨u9, hu9, hu9v⟩ := dashu.mul_ubig_exists_spec u8 u1
  obtain ⟨u10, hu10, hu10v⟩ := dashu.mul_ubig_exists_spec u9 u1
  obtain ⟨d, hd, hdv⟩ := dashu.mul_ubig_exists_spec u10 u4
  obtain ⟨i5, hi5, hi5parts⟩ := dashu.as_ibig_exists_spec n
  obtain ⟨i6, hi6⟩ := dashu.ibig_clone_exists_spec i5
  have hi6parts : dashu_int.ibig.IBig.into_parts i6 =
      ok (dashu_base.sign.Sign.Positive, n) :=
    dashu.ibig_clone_parts_spec i5 i6 _ hi6 hi5parts
  -- value of the denominator
  have hdval : dashu.ubigToNat d = 2 * nN * tN * tN * dN := by
    rw [hdv, hu10v, hu9v, hu8v, hu6v, hu7v, hu4v, huv, hu1v, hnumv, hdenv]
  have hdpos : 0 < dashu.ubigToNat d := by rw [hdval]; exact hD
  obtain ⟨r1, hr1, hr1parts⟩ :=
    dashu.rbig_from_parts_positive_exists_spec n d i6 hdpos hi6parts
  refine ⟨r1, ?_, ?_⟩
  · unfold gauss_step
    simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
      bind_tc_ok]
    rw [hi]; simp only [bind_tc_ok]
    rw [hu2]; simp only [bind_tc_ok]
    rw [hu3]; simp only [bind_tc_ok]
    rw [hu4]; simp only [bind_tc_ok]
    rw [hlhs]; simp only [bind_tc_ok]
    rw [hi1]; simp only [bind_tc_ok]
    rw [hi2]; simp only [bind_tc_ok]
    rw [hi3]; simp only [bind_tc_ok]
    rw [hi4]; simp only [bind_tc_ok]
    rw [hdiff]; simp only [bind_tc_ok]
    rw [hnabs]; simp only [bind_tc_ok]
    rw [hu5]; simp only [bind_tc_ok]
    rw [hn]; simp only [bind_tc_ok]
    rw [hu6]; simp only [bind_tc_ok]
    rw [hu7]; simp only [bind_tc_ok]
    rw [hu8]; simp only [bind_tc_ok]
    rw [hu9]; simp only [bind_tc_ok]
    rw [hu10]; simp only [bind_tc_ok]
    rw [hd]; simp only [bind_tc_ok]
    rw [hi5]; simp only [bind_tc_ok]
    rw [hi6]; simp only [bind_tc_ok]
    rw [hr1]; simp only [bind_tc_ok]
    congr 1
    funext r2
    rcases r2 with b' | e
    · cases b' <;> rfl
    · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
        from_residual_err_ok]
  · intro b
    have h := congrFun (sample_bernoulli_exp_spec r1 ⟨i6, d, n, hr1parts, hi6parts, hdpos⟩) b
    have hnvalue : dashu.ubigToNat n =
        ((((dashu.ibigToInt val).natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs *
          ((((dashu.ibigToInt val).natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs := by
      rw [hnv, hu5v, hnabsv, hdiffv, hi2v, hi1v, hi4v, hi3v, hlhsv, hu3v, hu2v, hiv,
        hu1v, hu4v, hnumv, hdenv]
    rw [show BernoulliExpNegSample (dashu.ubigToNat n) ⟨dashu.ubigToNat d, hdpos⟩ =
        gaussAccept tN nN dN hD (dashu.ibigToInt val) from
      bernExpNeg_cast _ _ _ _ hdpos hD hnvalue hdval] at h
    simpa [samplerDist] using h

/-! ### Ported SampCert facts (from `Gaussian/Properties`, which needs the analytic
`Util.Gaussian` layer that does not build on this pinned stack; these two lemmas only need
the sampler `Code` plus the Laplace/BNE properties, so they are re-proved here verbatim) -/

/-- Factored form of the Gaussian candidate loop (port of the `htrue`/`hfalse` sub-proofs of
SampCert's `DiscreteGaussianSampleLoop_normalizes`). -/
lemma gaussLoop_apply (num den t : ℕ+) (mix : ℕ) (a : ℤ) (b : Bool) :
    DiscreteGaussianSampleLoop num den t mix (a, b) =
      DiscreteLaplaceSample t 1 a *
        BernoulliExpNegSample (Int.natAbs (Int.sub (|a| * ↑↑t * ↑↑den) ↑↑num) ^ 2)
          (2 * num * t ^ 2 * den) b := by
  unfold DiscreteGaussianSampleLoop
  simp only [Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  rw [DiscreteLaplaceSampleMixed_equiv]
  rw [tsum_eq_single a]
  · cases b <;> simp
  · intro a₁ ha₁
    have hne : ¬ a = a₁ := by simpa [eq_comm] using ha₁
    cases b <;> simp [hne]

/-- Port of SampCert's `DiscreteGaussianSampleLoop_normalizes`. -/
theorem gaussLoop_normalizes (num den t : ℕ+) (mix : ℕ) :
    ∑' x, (DiscreteGaussianSampleLoop num den t mix) x = 1 := by
  rw [ENNReal.tsum_prod']
  have hsplit : ∀ a : ℤ,
      (∑' b : Bool, DiscreteGaussianSampleLoop num den t mix (a, b)) =
        DiscreteLaplaceSample t 1 a := by
    intro a
    rw [tsum_bool, gaussLoop_apply, gaussLoop_apply, ← mul_add, ← tsum_bool,
      BernoulliExpNegSample_normalizes]
    simp
  simp_rw [hsplit]
  exact DiscreteLaplaceSample_normalizes (num := t) (den := 1)

/-! ### Body fiber laws and the rejection closed form -/

/-- Step masses on a drawn candidate: settle-at-the-candidate with the accept probability,
continue with the reject probability. -/
private lemma gauss_step_masses (u t num den u1 : dashu_int.ubig.UBig)
    (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (huv : dashu.ubigToNat u = 1)
    (hu1v : dashu.ubigToNat u1 = tN)
    (hnumv : dashu.ubigToNat num = nN)
    (hdenv : dashu.ubigToNat den = dN)
    (val : dashu_int.ibig.IBig) :
    (∀ j : dashu_int.ibig.IBig,
      samplerDistGen (gauss_step u t num den u1 (core.result.Result.Ok val))
          (done (core.result.Result.Ok j)) =
        gaussAccept tN nN dN hD (dashu.ibigToInt val) true * (if j = val then 1 else 0)) ∧
    samplerDistGen (gauss_step u t num den u1 (core.result.Result.Ok val)) (cont ()) =
      gaussAccept tN nN dN hD (dashu.ibigToInt val) false := by
  obtain ⟨r1, heq, htrial⟩ :=
    gauss_step_chain u t num den u1 tN nN dN hD huv hu1v hnumv hdenv val
  constructor
  · intro j
    rw [heq, samplerDistGen_bind, SLang.probBind,
      tsum_result_ok_eq (fun e => by simp [samplerDistGen_pure_ok, PMF.pure_apply]),
      tsum_bool, htrial false, htrial true]
    simp only [samplerDistGen_pure_ok, PMF.pure_apply]
    by_cases hj : j = val
    · subst hj
      simp
    · have hne : (done (core.result.Result.Ok j) :
          ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) ≠
          done (core.result.Result.Ok val) := by
        intro h
        injection h with h1
        injection h1 with h2
        exact hj h2
      simp [hj, hne]
  · rw [heq, samplerDistGen_bind, SLang.probBind,
      tsum_result_ok_eq (fun e => by simp [samplerDistGen_pure_ok, PMF.pure_apply]),
      tsum_bool, htrial false, htrial true]
    simp [samplerDistGen_pure_ok, PMF.pure_apply]

/-- The body's `cont` mass: the candidate law mixed against the reject probability. -/
private lemma gauss_body_cont (u t num den u1 : dashu_int.ubig.UBig)
    (hcloneT : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone t = ok u1)
    (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (huv : dashu.ubigToNat u = 1)
    (hu1v : dashu.ubigToNat u1 = tN)
    (hnumv : dashu.ubigToNat num = nN)
    (hdenv : dashu.ubigToNat den = dN)
    (lapLaw : SLang ℤ)
    (hlap : samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u) = lapLaw) :
    samplerDistGen (samplers.gaussian.sample_discrete_gaussian_loop.body u t num den)
        (cont ()) =
      ∑' Y : ℤ, lapLaw Y * gaussAccept tN nN dN hD Y false := by
  rw [gauss_body_eq_step u t num den u1 hcloneT, samplerDistGen_bind, SLang.probBind,
    tsum_result_ok_eq (fun e => by rw [gauss_step_err]; simp)]
  simp_rw [fun val => (gauss_step_masses u t num den u1 tN nN dN hD huv hu1v hnumv hdenv
    val).2]
  refine Eq.trans (tsum_samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u)
    (fun Y => gaussAccept tN nN dN hD Y false)) ?_
  rw [hlap]

/-- The body's settle mass summed against the signed-output indicator: the candidate law times
the accept probability, at the output. -/
private lemma gauss_body_done_summed (u t num den u1 : dashu_int.ubig.UBig)
    (hcloneT : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone t = ok u1)
    (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (huv : dashu.ubigToNat u = 1)
    (hu1v : dashu.ubigToNat u1 = tN)
    (hnumv : dashu.ubigToNat num = nN)
    (hdenv : dashu.ubigToNat den = dN)
    (lapLaw : SLang ℤ)
    (hlap : samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u) = lapLaw)
    (z : ℤ) :
    (∑' j : dashu_int.ibig.IBig,
      samplerDistGen (samplers.gaussian.sample_discrete_gaussian_loop.body u t num den)
          (done (core.result.Result.Ok j)) *
        (if z = dashu.ibigToInt j then 1 else 0)) =
      lapLaw z * gaussAccept tN nN dN hD z true := by
  simp_rw [gauss_body_eq_step u t num den u1 hcloneT, samplerDistGen_bind, SLang.probBind,
    ← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  simp_rw [mul_assoc, ENNReal.tsum_mul_left]
  rw [tsum_result_ok_eq (fun e => by
    simp_rw [gauss_step_err]
    simp)]
  simp_rw [fun val => (gauss_step_masses u t num den u1 tN nN dN hD huv hu1v hnumv hdenv
    val).1]
  have hcol : ∀ val : dashu_int.ibig.IBig,
      (∑' j : dashu_int.ibig.IBig,
        gaussAccept tN nN dN hD (dashu.ibigToInt val) true * (if j = val then 1 else 0) *
          (if z = dashu.ibigToInt j then 1 else 0)) =
      gaussAccept tN nN dN hD (dashu.ibigToInt val) true *
        (if z = dashu.ibigToInt val then 1 else 0) := by
    intro val
    rw [tsum_eq_single val (fun j hj => by rw [if_neg hj, mul_zero, zero_mul])]
    rw [if_pos rfl, mul_one]
  simp_rw [hcol]
  refine Eq.trans (tsum_samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u)
    (fun Y => gaussAccept tN nN dN hD Y true * (if z = Y then 1 else 0))) ?_
  rw [hlap]
  rw [tsum_eq_single z (fun Y hY => by
    rw [if_neg (fun h => hY h.symm), mul_zero, mul_zero])]
  rw [if_pos rfl, mul_one]

/-- Lift: the extracted Gaussian loop's `ℤ`-law is the scalar rejection closed form. -/
private lemma gauss_loop_lift (u t num den u1 : dashu_int.ubig.UBig)
    (hcloneT : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone t = ok u1)
    (tN nN dN : ℕ) (hD : 0 < 2 * nN * tN * tN * dN)
    (huv : dashu.ubigToNat u = 1)
    (hu1v : dashu.ubigToNat u1 = tN)
    (hnumv : dashu.ubigToNat num = nN)
    (hdenv : dashu.ubigToNat den = dN)
    (lapLaw : SLang ℤ)
    (hlap : samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u) = lapLaw)
    (z : ℤ) :
    samplerDist_int (samplers.gaussian.sample_discrete_gaussian_loop u t num den) z =
      (lapLaw z * gaussAccept tN nN dN hD z true) *
      (1 - ∑' Y : ℤ, lapLaw Y * gaussAccept tN nN dN hD Y false)⁻¹ := by
  let cond : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) → Bool :=
    fun cf => match cf with | cont _ => true | done _ => false
  let bd : ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error) →
      SLang (ControlFlow Unit (core.result.Result dashu_int.ibig.IBig error.Error)) :=
    fun cf => match cf with
      | cont _ => samplerDistGen
          (samplers.gaussian.sample_discrete_gaussian_loop.body u t num den)
      | done _ => PMF.pure cf
  have hcc : ∀ a, cond (cont a) = true := fun _ => rfl
  have hcd : ∀ w, cond (done w) = false := fun _ => rfl
  have hstep1 : ∀ j : dashu_int.ibig.IBig,
      samplerDist (samplers.gaussian.sample_discrete_gaussian_loop u t num den) j =
        probWhile cond bd (cont ()) (done (core.result.Result.Ok j)) := by
    intro j
    simp only [samplerDist, samplers.gaussian.sample_discrete_gaussian_loop,
      samplerDistGen_loop]
    congr 1 <;> (funext cf; cases cf <;> rfl)
  have hexpand : samplerDist_int (samplers.gaussian.sample_discrete_gaussian_loop u t num den)
      z = ∑' j : dashu_int.ibig.IBig,
        samplerDist (samplers.gaussian.sample_discrete_gaussian_loop u t num den) j *
          (if z = dashu.ibigToInt j then 1 else 0) := by
    simp only [samplerDist_int, SLang.probBind, SLang.probPure]
    refine tsum_congr fun j => ?_
    by_cases h : z = dashu.ibigToInt j <;> simp [h]
  rw [hexpand]
  simp_rw [hstep1]
  exact lap_probWhile_closed cond bd hcc hcd
    (∑' Y : ℤ, lapLaw Y * gaussAccept tN nN dN hD Y false)
    (fun z => lapLaw z * gaussAccept tN nN dN hD z true)
    (gauss_body_cont u t num den u1 hcloneT tN nN dN hD huv hu1v hnumv hdenv lapLaw hlap)
    (fun z => gauss_body_done_summed u t num den u1 hcloneT tN nN dN hD huv hu1v hnumv hdenv
      lapLaw hlap z) z

/-! ### The SampCert equality -/

/-- Two-index collapse for the Laplace candidate law. -/
private lemma dlap_cast (a b : ℕ) (ha : 0 < a) (hb : 0 < b) (c : ℕ) (hc : 0 < c)
    (hac : a = c) (hb1 : b = 1) :
    DiscreteLaplaceSample ⟨a, ha⟩ ⟨b, hb⟩ = DiscreteLaplaceSample ⟨c, hc⟩ 1 := by
  subst hac; subst hb1; rfl

/-- Index collapse into an arbitrary `ℕ+` denominator. -/
private lemma bernExpNeg_cast' (n1 d1 : ℕ) (h1 : 0 < d1) (n2 : ℕ) (d2 : ℕ+)
    (hn : n1 = n2) (hd : d1 = (d2 : ℕ)) :
    BernoulliExpNegSample n1 ⟨d1, h1⟩ = BernoulliExpNegSample n2 d2 := by
  subst hn
  cases d2 with
  | mk v hv => cases hd; rfl

/-- The rejection closed form equals SampCert's `DiscreteGaussianSample`
(pure SLang algebra). -/
private lemma gauss_closed_form_eq (num den : ℕ+) (mix : ℕ)
    (tN nN dN : ℕ) (htN : 0 < tN) (hD : 0 < 2 * nN * tN * tN * dN)
    (htval : tN = (num : ℕ) / (den : ℕ) + 1)
    (hnval : nN = (num : ℕ) * (num : ℕ))
    (hdval : dN = (den : ℕ) * (den : ℕ)) (z : ℤ) :
    (DiscreteLaplaceSample ⟨tN, htN⟩ 1 z * gaussAccept tN nN dN hD z true) *
      (1 - ∑' Y : ℤ,
        DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y false)⁻¹ =
    DiscreteGaussianSample num den mix z := by
  -- The Gaussian candidate loop at any scale of the right value, in my factored form.
  have hGL : ∀ (T : ℕ+), (T : ℕ) = tN → ∀ (Y : ℤ) (b : Bool),
      DiscreteGaussianSampleLoop (num ^ 2) (den ^ 2) T mix (Y, b) =
        DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y b := by
    intro T hT Y b
    have hTeq : T = (⟨tN, htN⟩ : ℕ+) := Subtype.ext hT
    subst hTeq
    rw [gaussLoop_apply]
    congr 1
    unfold gaussAccept
    refine congrFun (Eq.symm (bernExpNeg_cast' _ _ _ _ _ ?_ ?_)) b
    · rw [pow_two]
      congr 1 <;>
      · congr 1
        change _ = (_ : ℤ) - (_ : ℤ)
        show ((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ) =
          |Y| * (tN : ℤ) * (((den ^ 2 : ℕ+) : ℕ) : ℤ) - (((num ^ 2 : ℕ+) : ℕ) : ℤ)
        rw [Int.abs_eq_natAbs, hnval, hdval]
        push_cast
        ring
    · show 2 * nN * tN * tN * dN = ((2 : ℕ+) : ℕ) * ((num ^ 2 : ℕ+) : ℕ) * (tN ^ 2) *
        ((den ^ 2 : ℕ+) : ℕ)
      push_cast
      rw [hnval, hdval]
      ring
  -- RHS: unfold the sampler through the normalized `probUntil`.
  simp only [DiscreteGaussianSample, Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  simp_rw [probUntil_apply_norm _ _ _ (gaussLoop_normalizes _ _ _ mix)]
  have hcomm : ∀ (f g : (ℤ × Bool) → ENNReal) (c : ENNReal),
      (∑' st : ℤ × Bool, f st * c * g st) = (∑' st : ℤ × Bool, f st * g st) * c := by
    intro f g c
    rw [← ENNReal.tsum_mul_right]
    exact tsum_congr fun st => by ring
  rw [hcomm]
  congr 1
  · -- numerator
    rw [ENNReal.tsum_prod']
    simp_rw [tsum_bool]
    refine Eq.trans (Eq.symm ?_) (tsum_congr
      (f := fun Y : ℤ => DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y *
        gaussAccept tN nN dN hD Y true * (if z = Y then 1 else 0)) fun Y => ?_)
    · rw [tsum_eq_single z (fun Y hY => by
        rw [if_neg (fun h => hY h.symm), mul_zero])]
      rw [if_pos rfl, mul_one]
    · symm
      simp only [Bool.false_eq_true, if_false, zero_mul, zero_add, if_true]
      rw [hGL _ (by rw [htval]; exact rfl) Y true]
      by_cases h : z = Y <;> simp [h]
  · -- denominator
    congr 1
    have hρ_fin : (∑' Y : ℤ,
        DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y false) ≠ ⊤ := by
      refine ne_top_of_le_ne_top ENNReal.one_ne_top ?_
      calc (∑' Y : ℤ, DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y false)
          ≤ ∑' Y : ℤ, DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y := by
            refine ENNReal.tsum_le_tsum fun Y => ?_
            conv_rhs => rw [← mul_one (DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y)]
            refine mul_le_mul_left' ?_ _
            have hbY := BernoulliExpNegSample_normalizes
              ((((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs *
                (((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs)
              ⟨2 * nN * tN * tN * dN, hD⟩
            rw [tsum_bool] at hbY
            calc gaussAccept tN nN dN hD Y false
                ≤ gaussAccept tN nN dN hD Y false + gaussAccept tN nN dN hD Y true :=
                  le_self_add
              _ = 1 := hbY
        _ = 1 := DiscreteLaplaceSample_normalizes _ _
    have hsum1 : (∑' Y : ℤ,
        DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y false) +
        (∑' Y : ℤ,
          DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y true) = 1 := by
      rw [← ENNReal.tsum_add]
      have hnormpt : ∀ Y : ℤ,
          DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y false +
            DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y * gaussAccept tN nN dN hD Y true =
          DiscreteLaplaceSample ⟨tN, htN⟩ 1 Y := by
        intro Y
        rw [← mul_add]
        have hbY := BernoulliExpNegSample_normalizes
          ((((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs *
            (((Y.natAbs * tN * dN : ℕ) : ℤ) - (nN : ℤ)).natAbs)
          ⟨2 * nN * tN * tN * dN, hD⟩
        rw [tsum_bool] at hbY
        show _ * (gaussAccept tN nN dN hD Y false + gaussAccept tN nN dN hD Y true) = _
        unfold gaussAccept
        rw [hbY, mul_one]
      simp_rw [hnormpt]
      exact DiscreteLaplaceSample_normalizes _ _
    rw [← hsum1, ENNReal.add_sub_cancel_left hρ_fin]
    rw [ENNReal.tsum_prod']
    simp_rw [tsum_bool]
    refine tsum_congr fun Y => ?_
    simp only [Bool.false_eq_true, if_false, zero_add, if_true]
    rw [hGL _ (by rw [htval]; exact rfl) Y true]

/-- **Distributional correctness (roadmap stage 9 — the final target).** On strictly positive
`numer, denom`, the extracted `sample_discrete_gaussian` realises SampCert's
`DiscreteGaussianSample` — the zCDP discrete Gaussian noise mechanism on `ℤ` — for every value
of SampCert's `mix` parameter. -/
theorem sample_discrete_gaussian_spec (numer denom : dashu_int.ubig.UBig)
    (hnum : 0 < dashu.ubigToNat numer) (hdenom : 0 < dashu.ubigToNat denom) (mix : ℕ) :
    samplerDist_int (samplers.gaussian.sample_discrete_gaussian numer denom) =
      DiscreteGaussianSample ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩
        mix := by
  obtain ⟨u, hcu, huv'⟩ := dashu.clone_exists_spec numer
  obtain ⟨u1', hcu1, hu1v'⟩ := dashu.clone_exists_spec denom
  have hu1pos' : 0 < dashu.ubigToNat u1' := by rw [hu1v']; exact hdenom
  obtain ⟨u2, hdiv, hu2v⟩ := dashu.div_ubig_floor_exists_spec u u1' hu1pos'
  obtain ⟨u3, hone, hu3v⟩ := dashu.one_exists_spec
  obtain ⟨t, ht, htv⟩ := dashu.add_ubig_exists_spec u2 u3
  obtain ⟨num, hnm, hnmv⟩ := dashu.mul_ubig_exists_spec u numer
  obtain ⟨den, hdn, hdnv⟩ := dashu.mul_ubig_exists_spec u1' denom
  have htNv : dashu.ubigToNat t =
      dashu.ubigToNat numer / dashu.ubigToNat denom + 1 := by
    rw [htv, hu2v, hu3v, huv', hu1v']
  have htN : 0 < dashu.ubigToNat t := by rw [htNv]; omega
  have hnNv : dashu.ubigToNat num = dashu.ubigToNat numer * dashu.ubigToNat numer := by
    rw [hnmv, huv']
  have hdNv : dashu.ubigToNat den = dashu.ubigToNat denom * dashu.ubigToNat denom := by
    rw [hdnv, hu1v']
  have hD : 0 < 2 * dashu.ubigToNat num * dashu.ubigToNat t * dashu.ubigToNat t *
      dashu.ubigToNat den := by
    have h1 : 0 < dashu.ubigToNat num := by rw [hnNv]; exact Nat.mul_pos hnum hnum
    have h2 : 0 < dashu.ubigToNat den := by rw [hdNv]; exact Nat.mul_pos hdenom hdenom
    positivity
  obtain ⟨u1, hcloneT, hu1v⟩ := dashu.clone_exists_spec t
  have hu1pos : 0 < dashu.ubigToNat u1 := by rw [hu1v]; exact htN
  have hu3pos : 0 < dashu.ubigToNat u3 := by rw [hu3v]; norm_num
  have hlap : samplerDist_int (samplers.laplace.sample_discrete_laplace u1 u3) =
      DiscreteLaplaceSample ⟨dashu.ubigToNat t, htN⟩ 1 :=
    (sample_discrete_laplace_spec u1 u3 hu1pos hu3pos).trans
      (dlap_cast _ _ _ _ _ htN hu1v hu3v)
  have hprog : samplers.gaussian.sample_discrete_gaussian numer denom =
      samplers.gaussian.sample_discrete_gaussian_loop u3 t num den := by
    unfold samplers.gaussian.sample_discrete_gaussian
    rw [hcu]; simp only [bind_tc_ok]
    rw [hcu1]; simp only [bind_tc_ok]
    rw [hdiv]; simp only [bind_tc_ok]
    rw [hone]; simp only [bind_tc_ok]
    rw [ht]; simp only [bind_tc_ok]
    rw [hnm]; simp only [bind_tc_ok]
    rw [hdn]; simp only [bind_tc_ok]
  funext z
  rw [hprog, gauss_loop_lift u3 t num den u1 hcloneT (dashu.ubigToNat t)
    (dashu.ubigToNat num) (dashu.ubigToNat den) hD hu3v hu1v rfl rfl
    (DiscreteLaplaceSample ⟨dashu.ubigToNat t, htN⟩ 1) hlap z]
  refine gauss_closed_form_eq ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩
    mix (dashu.ubigToNat t) (dashu.ubigToNat num) (dashu.ubigToNat den) htN hD
    htNv hnNv hdNv z

end OpenDP.samplers.gaussian
