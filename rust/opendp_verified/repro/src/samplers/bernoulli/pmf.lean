import Generated.OpenDP
import SampCert.Samplers.Bernoulli.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.core.readable.notation
import src.samplers.uniform.pmf
import src.samplers.bernoulli.rational

/-!
# `sample_bernoulli_rational` — distributional correctness

Human proof (`traits/samplers/bernoulli/sample_bernoulli_rational.tex`): draw `k`
uniformly from `[0, denom)`, return `⊤` iff `k < numer`; hence `Pr[⊤] = numer/denom`.

Round 4 closes the chain opened by `rational.lean`:

* `rational.lean` proves the translation `sample_bernoulli_rational prob =
  (uniform-below denom) >>= map (numer > ·)` and defines `bernoulliPMF`.
* `uniform/pmf.lean` proves `samplerDist_nat (sample_uniform_ubig_below denom) =
  uniformNatBelowPMF denom`.

Here we push `samplerDist` through the deterministic comparison closure and bridge the
`UBig`-indexed sum to the `Nat`-indexed uniform law, obtaining
`⟦sample_bernoulli_rational prob⟧ = bernoulliPMF numer denom`.
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Finset Classical

namespace OpenDP.samplers.bernoulli

open OpenDP.samplers.uniform (uniformNatBelowPMF samplerDist_nat)

/-! ### Closure computation: the comparison `numer > ·` mapped over a `Result`. -/

private lemma closure_ok_map (numer u : dashu_int.ubig.UBig) :
    core.result.Result.map
      samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
      (core.result.Result.Ok (E := error.Error) u) numer =
    ok (core.result.Result.Ok (decide (dashu.ubigToNat numer > dashu.ubigToNat u))) := by
  simp only [core.result.Result.map,
    samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool.call_once,
    dashu.gt_spec_decide, bind_tc_ok]

private lemma closure_err_map (numer : dashu_int.ubig.UBig) (e : error.Error) :
    core.result.Result.map
      samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
      (core.result.Result.Err e : core.result.Result dashu_int.ubig.UBig error.Error) numer =
    ok (core.result.Result.Err e) := by
  simp only [core.result.Result.map]

private lemma rational_closure_ok_apply (numer u : dashu_int.ubig.UBig) :
    samplerDistGen
      (core.result.Result.map
        samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
        (core.result.Result.Ok (E := error.Error) u) numer)
      (core.result.Result.Ok (decide (dashu.ubigToNat numer > dashu.ubigToNat u))) = 1 := by
  rw [closure_ok_map, samplerDistGen_pure_ok, PMF.pure_apply, if_pos rfl]

private lemma rational_closure_ok_apply_ne (numer u : dashu_int.ubig.UBig) (b : Bool)
    (hb : b ≠ decide (dashu.ubigToNat numer > dashu.ubigToNat u)) :
    samplerDistGen
      (core.result.Result.map
        samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
        (core.result.Result.Ok (E := error.Error) u) numer)
      (core.result.Result.Ok b) = 0 := by
  rw [closure_ok_map, samplerDistGen_pure_ok, PMF.pure_apply]
  exact if_neg (fun h => hb (core.result.Result.Ok.inj h))

private lemma rational_closure_err_apply (numer : dashu_int.ubig.UBig) (e : error.Error) (b : Bool) :
    samplerDistGen
      (core.result.Result.map
        samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
        (core.result.Result.Err e : core.result.Result dashu_int.ubig.UBig error.Error) numer)
      (core.result.Result.Ok b) = 0 := by
  rw [closure_err_map, samplerDistGen_pure_ok, PMF.pure_apply]
  exact if_neg (fun h => by cases h)

/-! ### Bridge: `UBig`-indexed sum ↔ `Nat`-indexed uniform sum. -/

private lemma samplerDist_uniform_nat_bridge
    (denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (g : Nat → ENNReal) :
    ∑' u : dashu_int.ubig.UBig,
        samplerDist (samplers.uniform.sample_uniform_ubig_below denom) u * g (dashu.ubigToNat u) =
    ∑' k : Nat,
        (↑(uniformNatBelowPMF denom hdenom) : SLang Nat) k * g k := by
  have hnat := uniform.sample_uniform_ubig_below_pmf denom hdenom
  rw [← hnat]
  simp only [samplerDist_nat, SLang.bind_apply, SLang.pure_apply]
  simp_rw [← ENNReal.tsum_mul_right]
  conv_rhs => rw [ENNReal.tsum_comm]
  apply tsum_congr; intro u'
  simp_rw [mul_assoc]
  rw [ENNReal.tsum_mul_left]
  congr 1
  symm
  rw [tsum_eq_single (dashu.ubigToNat u') (fun k hk => by simp [hk])]
  simp

/-! ### Bridge: `bernoulliPMF` in product form. -/

private lemma bernoulliPMF_apply_prod
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (b : Bool) :
    (bernoulliPMF numer denom hdenom : SLang Bool) b =
    ∑' k : Nat, (uniformNatBelowPMF denom hdenom : SLang Nat) k *
        (if b = decide (k < dashu.ubigToNat numer) then 1 else 0) := by
  simp only [bernoulliPMF, uniformNatBelowPMF, PMF.map_apply, mul_ite, mul_one, mul_zero]
  congr 1; ext k
  split_ifs <;> rfl

/-! ### Main theorem -/

/-- **Distributional correctness.** On the valid-input branch, the extracted
`sample_bernoulli_rational` realises `bernoulliPMF numer denom` — i.e. `Bernoulli(numer/denom)`,
which `bernoulliPMF_eq_BernoulliSamplePMF` identifies with SampCert's canonical law. -/
theorem sample_bernoulli_rational_pmf
    (prob : dashu_ratio.rbig.RBig)
    (setup : RationalSetup prob)
    (hdenom : 0 < dashu.ubigToNat setup.denom) :
    samplerDist (samplers.bernoulli.sample_bernoulli_rational prob) =
      (bernoulliPMF setup.numer setup.denom hdenom : SLang Bool) := by
  funext b
  rw [bernoulliPMF_apply_prod]
  simp only [samplerDist]
  rw [sample_bernoulli_rational_eq_of_setup prob setup]
  simp only [samplerDistGen_bind, SLang.bind_apply]
  simp_rw [show ∀ r : core.result.Result dashu_int.ubig.UBig error.Error,
      samplerDistGen
        (core.result.Result.map
          samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
          r setup.numer)
        (core.result.Result.Ok b) =
      match r with
      | core.result.Result.Ok u =>
          if b = decide (dashu.ubigToNat setup.numer > dashu.ubigToNat u) then 1 else 0
      | core.result.Result.Err _ => 0
      from fun r => by
    cases r with
    | Ok u =>
      change samplerDistGen
          (core.result.Result.map
            samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
            (core.result.Result.Ok (E := error.Error) u) setup.numer)
          (core.result.Result.Ok b) =
          if b = decide (dashu.ubigToNat setup.numer > dashu.ubigToNat u) then 1 else 0
      by_cases hb : b = decide (dashu.ubigToNat setup.numer > dashu.ubigToNat u)
      · rw [if_pos hb, hb]; exact rational_closure_ok_apply setup.numer u
      · rw [if_neg hb]; exact rational_closure_ok_apply_ne setup.numer u b hb
    | Err e => exact rational_closure_err_apply setup.numer e b]
  rw [tsum_result_ok_eq (fun e => by simp)]
  exact samplerDist_uniform_nat_bridge setup.denom hdenom
      (fun k => if b = decide (k < dashu.ubigToNat setup.numer) then 1 else 0)

end OpenDP.samplers.bernoulli
