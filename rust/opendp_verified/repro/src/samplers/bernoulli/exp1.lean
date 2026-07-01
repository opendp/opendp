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

end OpenDP.samplers.bernoulli
