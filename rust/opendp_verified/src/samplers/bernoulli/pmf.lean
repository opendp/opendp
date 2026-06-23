import Aeneas
import Generated.OpenDP
import SampCert.SLang
import SampCert.Samplers.Bernoulli.Properties
import SampCert.Samplers.BernoulliNegativeExponential.Properties
import src.samplers.bytes
import src.samplers.functor
import src.samplers.uniform.mod
import src.samplers.uniform.pmf
import src.samplers.bernoulli.semantics
import src.samplers.bernoulli.mod

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.samplers
open SLang PMF ENNReal Finset Classical

namespace OpenDP.samplers.bernoulli

open OpenDP.samplers.uniform (uniformNatBelowPMF samplerDist_nat)

/-! ### Closure computation lemmas -/

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

/-! ### Bridge: UBig-indexed sum ↔ Nat-indexed sum -/

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
  -- Goal: ∑' u, S u * g (nat_u) = ∑' k, (∑' u', S u' * (if k = nat_u' then 1 else 0)) * g k
  simp_rw [← ENNReal.tsum_mul_right]
  conv_rhs => rw [ENNReal.tsum_comm]
  -- Goal per u': S u' * g (nat_u') = ∑' k, S u' * (if k = nat_u' then 1 else 0) * g k
  apply tsum_congr; intro u'
  simp_rw [mul_assoc]  -- (a * b) * c → a * (b * c)
  rw [ENNReal.tsum_mul_left]
  congr 1
  symm
  rw [tsum_eq_single (dashu.ubigToNat u') (fun k hk => by simp [hk])]
  simp

/-! ### Bridge from bernoulliPMF to product form -/

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

theorem sample_bernoulli_rational_pmf
    (prob : dashu_ratio.rbig.RBig)
    (setup : BernoulliRationalSetup prob)
    (hdenom : 0 < dashu.ubigToNat setup.denom) :
    samplerDist (samplers.bernoulli.sample_bernoulli_rational prob) =
      (bernoulliPMF setup.numer setup.denom hdenom : SLang Bool) := by
  funext b
  -- Set up the RHS in product form matching the bridge
  rw [bernoulliPMF_apply_prod]
  -- Unfold LHS via Aeneas bind decomposition
  simp only [samplerDist]
  rw [sample_bernoulli_rational_eq_of_setup prob setup]
  simp only [samplerDistGen_bind, SLang.bind_apply]
  -- Simplify the closure application for each r : Result UBig error.Error
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
  -- Eliminate Err terms
  rw [tsum_result_ok_eq (fun e => by simp)]
  -- Apply the UBig→Nat bridge
  -- LHS is definitionally equal to the bridge's LHS (samplerDist vs samplerDistGen + Ok, and GT vs LT)
  exact samplerDist_uniform_nat_bridge setup.denom hdenom
      (fun k => if b = decide (k < dashu.ubigToNat setup.numer) then 1 else 0)

/-! ### Exp1 loop: body distribution -/

section Exp1Loop

open samplers.bernoulli

/-- The SampCert Bernoulli negative-exponential unit loop body. -/
private noncomputable def BESL
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    Bool × PNat → SLang (Bool × PNat) :=
  SLang.BernoulliExpNegSampleUnitLoop
    (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hfrac

/-- Control-flow guard for the Aeneas exp1 loop. -/
private abbrev exp1_guard :
    ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) → Bool :=
  fun cf => match cf with | cont _ => true | done _ => false

/-- Control-flow body distribution for the Aeneas exp1 loop. -/
private noncomputable def exp1_body_dist
    (k denom numer : dashu_int.ubig.UBig) :
    ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error) →
    SLang (ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :=
  fun cf => match cf with
    | cont k2 => samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k2)
    | done v => PMF.pure (done v)

private lemma exp1_guard_cont (k : dashu_int.ubig.UBig) :
    exp1_guard (cont k) = true := rfl
private lemma exp1_guard_done (v : core.result.Result Bool error.Error) :
    exp1_guard (done v) = false := rfl

/-- At the `cont k2` state, the body distribution equals the Aeneas body distribution. -/
private lemma exp1_body_dist_cont_eq
    (k denom numer k2 : dashu_int.ubig.UBig) :
    exp1_body_dist k denom numer (cont k2) =
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k2) := rfl

private lemma rem_u8_sampcert_parity
    (k1 : dashu_int.ubig.UBig)
    (i : Std.U8)
    (hrem :
      dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i) :
    (decide (i = 1#u8) : Bool) =
      decide ((dashu.ubigToNat k1 + 1) % 2 = 0) := by
  have hremNat := dashu.rem_u8_spec k1 2#u8 i hrem
  have hi : (i = 1#u8) ↔ i.val = 1 := by
    rw [Aeneas.Std.UScalar.eq_equiv]
    simp
  have hiff : (i = 1#u8) ↔ dashu.ubigToNat k1 % 2 = 1 := by
    rw [hi, hremNat]
    simp
  calc
    (decide (i = 1#u8) : Bool) = decide (dashu.ubigToNat k1 % 2 = 1) := by
      by_cases h : i = 1#u8
      · have hparity : dashu.ubigToNat k1 % 2 = 1 := hiff.mp h
        simp [h, hparity]
      · have hparity : ¬ dashu.ubigToNat k1 % 2 = 1 := fun hparity =>
          h (hiff.mpr hparity)
        simp [h, hparity]
    _ = decide ((dashu.ubigToNat k1 + 1) % 2 = 0) := decide_odd_current_eq_even_successor _

/-- The Aeneas exp1 loop-body continuation after the rational sample returns `r`:
    branch on the control flow, then either continue with `k1 + k` or terminate with the parity
    of `k1`. Naming this avoids repeating the (large) `do`-block inline and gives stable
    equation lemmas, so `simp` can reduce the `match` for a concrete `r`. -/
private noncomputable def exp1_step (k k1 : dashu_int.ubig.UBig) :
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
private lemma exp1_body_eq_step
    (k denom numer k1 : dashu_int.ubig.UBig) (x_div_k : dashu_ratio.rbig.RBig)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
    sample_bernoulli_rational x_div_k >>= exp1_step k k1 := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop.body
  rw [hdiv]
  rfl

/-- Step on `Ok true`: a point mass at `cont k2'` (where `k2' = k1 + k`). -/
private lemma exp1_step_ok_true
    (k k1 k2' : dashu_int.ubig.UBig)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2')
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Ok true)) out =
    (if out = cont k2' then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    hadd, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Ok false`: a point mass at `done (Ok parity)` (where `parity = decide (i = 1)`). -/
private lemma exp1_step_ok_false
    (k k1 : dashu_int.ubig.UBig) (i : Std.U8)
    (hrem : dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Ok false)) out =
    (if out = done (core.result.Result.Ok (decide (i = 1#u8))) then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    hrem, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Err e`: a point mass at `done (Err e)`; in particular `0` at any `cont` or `done (Ok _)`. -/
private lemma exp1_step_err
    (k k1 : dashu_int.ubig.UBig) (e : error.Error)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result Bool error.Error)) :
    samplerDistGen (exp1_step k k1 (core.result.Result.Err e)) out =
    (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [exp1_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual,
    samplerDistGen_pure_ok, PMF.pure_apply]

/-- The body assigns cont mass via k2', matching BESL at (true, k1+1). -/
private lemma exp1_body_cont_apply
    (k denom numer k1 k2 k2' : dashu_int.ubig.UBig)
    (_hk : dashu.ubigToNat k = 1)
    (hk1 : 0 < dashu.ubigToNat k1)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2')
    (_hk2nat : dashu.ubigToNat k2' = dashu.ubigToNat k1 + 1) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1) (cont k2) =
    BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
    (if k2 = k2' then 1 else 0) := by
  rcases sample_bernoulli_exp1_step_sampcert_spec numer denom k1 hk1 hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, _uniformSetup, hdiv, _huniformSetup, _hrat, hstep⟩
  obtain ⟨i, hrem, _⟩ := dashu.rem_u8_exists_spec k1 2#u8 (by decide)
  -- Factor the body through `exp1_step`, then split the rational sample over its result.
  rw [exp1_body_eq_step k denom numer k1 x_div_k hdiv, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [exp1_step_err]; simp)]
  rw [tsum_bool, exp1_step_ok_true k k1 k2' hadd, exp1_step_ok_false k k1 i hrem]
  -- The rational sample's `true` mass matches BESL at `(true, k1+1)`; `false` lands on a `done`.
  have hrat_true :
      samplerDistGen (sample_bernoulli_rational x_div_k) (core.result.Result.Ok true) =
      bernoulliPMF setup.numer setup.denom hsetupDenom true := by
    simpa [samplerDist] using congrFun (sample_bernoulli_rational_pmf x_div_k setup hsetupDenom) true
  have hpmf_true :
      bernoulliPMF setup.numer setup.denom hsetupDenom true =
      BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) := by
    simpa [BESL] using (hstep true)
  rw [hrat_true, hpmf_true]
  simp [ControlFlow.cont.injEq]

/-- The body assigns done-ok mass via parity of k1+1, matching BESL at (false, k1+1). -/
private lemma exp1_body_done_ok_apply
    (k denom numer k1 : dashu_int.ubig.UBig)
    (_hk : dashu.ubigToNat k = 1)
    (hk1 : 0 < dashu.ubigToNat k1)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (b : Bool) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1)
        (done (core.result.Result.Ok b)) =
    BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
    (if decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b then 1 else 0) := by
  rcases sample_bernoulli_exp1_step_sampcert_spec numer denom k1 hk1 hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, _uniformSetup, hdiv, _huniformSetup, _hrat, hstep⟩
  obtain ⟨i, hrem, _⟩ := dashu.rem_u8_exists_spec k1 2#u8 (by decide)
  have hparity : (decide (i = 1#u8) : Bool) = decide ((dashu.ubigToNat k1 + 1) % 2 = 0) :=
    rem_u8_sampcert_parity k1 i hrem
  rcases dashu.add_assign_exists_spec k1 k with ⟨k2', hadd, _⟩
  -- Factor the body through `exp1_step`, then split the rational sample over its result.
  rw [exp1_body_eq_step k denom numer k1 x_div_k hdiv, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [exp1_step_err]; simp)]
  rw [tsum_bool, exp1_step_ok_true k k1 k2' hadd, exp1_step_ok_false k k1 i hrem]
  -- Only the rational `false` outcome reaches a `done`; its mass matches BESL at `(false, k1+1)`.
  have hrat_false :
      samplerDistGen (sample_bernoulli_rational x_div_k) (core.result.Result.Ok false) =
      bernoulliPMF setup.numer setup.denom hsetupDenom false := by
    simpa [samplerDist] using congrFun (sample_bernoulli_rational_pmf x_div_k setup hsetupDenom) false
  have hpmf_false :
      bernoulliPMF setup.numer setup.denom hsetupDenom false =
      BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) := by
    simpa [BESL] using (hstep false)
  rw [hrat_false, hpmf_false]
  cases b <;> simp [hparity, eq_comm]

/-! ### Exp1 cut-step induction -/

/-- The BESL distribution at `(true, r)` is supported on exactly two points: `(true, r+1)` and
    `(false, r+1)`, with coefficients `num/(r*den)` and `1 - num/(r*den)`.
    Proved by point evaluation (finite Bool tsum), avoiding the whnf explosion that occurs
    when unfolding under `∑' st : Bool × PNat`. -/
private lemma BESL_two_point
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (k1 : dashu_int.ubig.UBig) (hk1 : 0 < dashu.ubigToNat k1)
    (st : Bool × PNat) :
    BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩) st =
    (if st = (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) then
       (dashu.ubigToNat numer : ENNReal) /
       ↑(⟨dashu.ubigToNat k1, hk1⟩ * ⟨dashu.ubigToNat denom, hdenom⟩ : PNat)
     else 0) +
    (if st = (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) then
       1 - (dashu.ubigToNat numer : ENNReal) /
       ↑(⟨dashu.ubigToNat k1, hk1⟩ * ⟨dashu.ubigToNat denom, hdenom⟩ : PNat)
     else 0) := by
  let curr : ℕ+ := ⟨dashu.ubigToNat k1, hk1⟩
  let next : ℕ+ := ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩
  have hsucc : curr + 1 = next := by
    apply Subtype.ext
    rfl
  rcases st with ⟨b, r⟩
  cases b
  · by_cases hr : r = next
    · subst hr
      unfold BESL SLang.BernoulliExpNegSampleUnitLoop
      simp [curr, next, hsucc]
    · unfold BESL SLang.BernoulliExpNegSampleUnitLoop
      have hr' : r ≠ curr + 1 := by
        intro h
        apply hr
        simpa [h] using hsucc
      simp [curr, next, hsucc, hr]
  · by_cases hr : r = next
    · subst hr
      unfold BESL SLang.BernoulliExpNegSampleUnitLoop
      simp [curr, next, hsucc]
    · unfold BESL SLang.BernoulliExpNegSampleUnitLoop
      have hr' : r ≠ curr + 1 := by
        intro h
        apply hr
        simpa [h] using hsucc
      simp [curr, next, hsucc, hr]

/-- The BESL loop only assigns mass to `(false, ...)` terminal states. -/
private lemma BESL_probWhileCut_true_zero
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (n : Nat) (init : Bool × PNat) (K : PNat) :
    probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) n init (true, K) = 0 := by
  simp [BESL, SLang.BernoulliExpNegSampleUnitAux_returns_false]

/-- The BESL `probWhileCut` at a (false, K) state is a point mass at (false, K). -/
private lemma BESL_probWhileCut_false_pure
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (n : Nat) (K K' : PNat) :
    probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) (n + 1) (false, K) (false, K') =
    if K' = K then 1 else 0 := by
  rw [probWhileCut_guard_false _ _ _ (by simp) n]
  simp [SLang.probPure]

/-- The key cut-step induction: Aeneas n-step truncation at `(cont k1)` equals
    BESL n-step truncation at `(true, k1)` projected through the parity indicator. -/
private lemma exp1_loop_cut_step
    (k denom numer : dashu_int.ubig.UBig)
    (hk : dashu.ubigToNat k = 1)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (n : Nat) (b : Bool) :
    ∀ (k1 : dashu_int.ubig.UBig) (hk1 : 0 < dashu.ubigToNat k1),
    probWhileCut exp1_guard (exp1_body_dist k denom numer) n (cont k1)
        (done (core.result.Result.Ok b)) =
    ∑' K : PNat,
        probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) n
            (true, ⟨dashu.ubigToNat k1, hk1⟩) (false, K) *
        (if decide (K.val % 2 = 0) = b then 1 else 0) := by
  induction n with
  | zero =>
    intro k1 hk1
    simp [probWhileCut, SLang.probZero]
  | succ n ih =>
    intro k1 hk1
    -- One-step unfold on the Aeneas side
    have hstep_lhs : probWhileCut exp1_guard (exp1_body_dist k denom numer) (n + 1) (cont k1)
        = exp1_body_dist k denom numer (cont k1) >>= probWhileCut exp1_guard (exp1_body_dist k denom numer) n := by
      rw [probWhileCut, probWhileFunctional, if_pos (exp1_guard_cont k1)]
    -- One-step unfold on the BESL side
    have hstep_rhs : ∀ K : PNat,
        probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) (n + 1)
            (true, ⟨dashu.ubigToNat k1, hk1⟩) (false, K) =
        (BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩) >>=
          probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) n) (false, K) := by
      intro K
      rw [probWhileCut, probWhileFunctional, if_pos (by simp)]
    simp only [hstep_lhs, exp1_body_dist_cont_eq, Bind.bind, SLang.bind_apply]
    simp_rw [hstep_rhs]
    -- Get the concrete k2' from add_assign; rewrite using hk : ubigToNat k = 1
    rcases dashu.add_assign_exists_spec k1 k with ⟨k2', hadd', hk2'nat⟩
    have hk2'nat1 : dashu.ubigToNat k2' = dashu.ubigToNat k1 + 1 :=
      hk2'nat.trans (by rw [hk])
    have hk2'pos : 0 < dashu.ubigToNat k2' := by rw [hk2'nat1]; exact Nat.succ_pos _
    -- Split Aeneas sum into cont and done fibers
    rw [tsum_controlFlow]
    -- The CONT fiber: use IH (k2' is the only reachable cont state)
    have lhs_cont_eq :
        ∑' k2 : dashu_int.ubig.UBig,
          samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1) (cont k2) *
          (probWhileCut exp1_guard (exp1_body_dist k denom numer) n (cont k2))
              (done (core.result.Result.Ok b)) =
        BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
            (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
        ∑' K : PNat,
          probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) n
              (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) (false, K) *
          (if decide (K.val % 2 = 0) = b then 1 else 0) := by
      simp_rw [exp1_body_cont_apply k denom numer k1 _ k2' hk hk1 hdenom hfrac hadd' hk2'nat1]
      simp_rw [mul_assoc]
      rw [← ENNReal.tsum_mul_left, tsum_eq_single k2' (fun k2 hk2 => by simp [hk2])]
      rw [ih k2' hk2'pos,
          show (⟨dashu.ubigToNat k2', hk2'pos⟩ : PNat) =
            ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩ from Subtype.ext hk2'nat1]
      simp only [if_true, one_mul, ENNReal.tsum_mul_left]
    -- The DONE fiber: BESL_false * ∑' K, pwc n BESL (false, k1+1) (false, K) * ind K
    -- (correct for all n: when n=0 both sides are 0; when n≥1 both equal BESL_false * ind(k1+1))
    have lhs_done_eq :
        ∑' v : core.result.Result Bool error.Error,
          samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1) (done v) *
          (probWhileCut exp1_guard (exp1_body_dist k denom numer) n (done v))
              (done (core.result.Result.Ok b)) =
        BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
            (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
        ∑' K : PNat,
          probWhileCut (fun s => s.1) (BESL numer denom hdenom hfrac) n
              (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) (false, K) *
          (if decide (K.val % 2 = 0) = b then 1 else 0) := by
      cases n with
      | zero => simp [probWhileCut, SLang.probZero]
      | succ n' =>
        -- (1) done state with guard=false gives probPure
        simp_rw [probWhileCut_guard_false _ _ _ (exp1_guard_done _) n', SLang.probPure]
        -- (2) Err terms vanish because a done-error state cannot equal done-(Ok b).
        rw [tsum_result_ok_eq (fun e => by simp)]
        -- (3) RHS: BESL false state with succ fuel is a point mass at (false, k1+1)
        simp_rw [BESL_probWhileCut_false_pure numer denom hdenom hfrac n']
        -- (4) LHS: split over Bool, apply the body lemma, combine.
        rw [tsum_bool]
        simp only [core.result.Result.Ok.injEq, ControlFlow.done.injEq]
        rw [exp1_body_done_ok_apply k denom numer k1 hk hk1 hdenom hfrac true,
            exp1_body_done_ok_apply k denom numer k1 hk hk1 hdenom hfrac false]
        have htsum_pt :
            (∑' K : PNat,
              (if K = ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩ then (1 : ENNReal) else 0) *
              (if decide ((K : ℕ) % 2 = 0) = b then 1 else 0)) =
            if decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b then 1 else 0 := by
          let pt : PNat := ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩
          have hrew :
              (∑' K : PNat,
                (if K = pt then (1 : ENNReal) else 0) *
                  (if decide ((K : ℕ) % 2 = 0) = b then 1 else 0)) =
              ∑' K : PNat,
                if K = pt then
                  (if decide ((K : ℕ) % 2 = 0) = b then 1 else 0)
                else 0 := by
            congr with K
            by_cases hK : K = pt <;> simp [hK]
          rw [hrew]
          rw [tsum_eq_single pt (fun K hK => by simp [hK])]
          simp [pt]
          rfl
        have hmul_pt :
            BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
                (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
              (∑' K : PNat,
                (if K = ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩ then 1 else 0) *
                (if decide ((K : ℕ) % 2 = 0) = b then 1 else 0)) =
            BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
                (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) *
              (if decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b then 1 else 0) := by
          exact congrArg
            (fun x =>
              BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
                (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) * x)
            htsum_pt
        rw [hmul_pt]
        by_cases hb : decide ((dashu.ubigToNat k1 + 1) % 2 = 0) = b
        · cases b <;> simp [hb]
        · cases b <;> simp [hb] at *
    -- Combine cont + done = RHS BESL bind expansion
    rw [lhs_cont_eq, lhs_done_eq]
    -- Evaluate BESL at the two specific output states (point evaluations via BESL_two_point;
    -- cheap because it avoids unfolding under ∑' st : Bool × PNat)
    have hc_true : BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (true, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) =
        (dashu.ubigToNat numer : ENNReal) /
        ↑(⟨dashu.ubigToNat k1, hk1⟩ * ⟨dashu.ubigToNat denom, hdenom⟩ : PNat) := by
      rw [BESL_two_point]; simp
    have hc_false : BESL numer denom hdenom hfrac (true, ⟨dashu.ubigToNat k1, hk1⟩)
        (false, ⟨dashu.ubigToNat k1 + 1, Nat.succ_pos _⟩) =
        1 - (dashu.ubigToNat numer : ENNReal) /
        ↑(⟨dashu.ubigToNat k1, hk1⟩ * ⟨dashu.ubigToNat denom, hdenom⟩ : PNat) := by
      rw [BESL_two_point]; simp
    rw [hc_true, hc_false]
    rw [← ENNReal.tsum_mul_left, ← ENNReal.tsum_mul_left, ← ENNReal.tsum_add]
    apply tsum_congr
    intro K
    by_cases hK : decide ((K : ℕ) % 2 = 0) = b
    · simp [hK, BESL_two_point, mul_comm]
    · simp [hK, BESL_two_point, mul_comm]

/-! ### Operational bridge: extracted loop ⇒ BESL tail target -/

/-- The distribution of the extracted `exp1` loop is `probWhile` of `exp1_body_dist`.
    (The loop theorem's body agrees with `exp1_body_dist` only up to `funext`/`cases`, not `rfl`,
    because its `done` branch returns the scrutinee rather than the reconstructed `done v`.) -/
private lemma samplerDistGen_exp1_loop
    (k denom numer k1 : dashu_int.ubig.UBig)
    (v : core.result.Result Bool error.Error) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1_loop k denom numer k1) v =
    probWhile exp1_guard (exp1_body_dist k denom numer) (cont k1) (done v) := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop
  rw [samplerDistGen_loop]
  congr 1 <;> (funext cf; cases cf <;> rfl)

/-- The extracted `sample_bernoulli_exp1` success mass at `b`, expressed as the supremum over
    fuel of the BESL truncation projected through the parity indicator. Combines the loop theorem,
    the `probWhile = ⨆ probWhileCut` characterization, and the cut-step induction. -/
private lemma exp1_loop_lhs_eq
    {x : dashu_ratio.rbig.RBig} (setup : BernoulliExp1Setup x) (b : Bool) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1 x) b =
    ⨆ n, ∑' K : PNat,
      probWhileCut (fun s : Bool × PNat => s.1)
        (BESL setup.numer setup.denom setup.hdenom setup.hfrac) n
        (true, (⟨1, Nat.one_pos⟩ : PNat)) (false, K) *
      (if decide (K.val % 2 = 0) = b then 1 else 0) := by
  have hone : dashu.ubigToNat setup.one = 1 := dashu.one_spec setup.one setup.hone
  have honePos : 0 < dashu.ubigToNat setup.one := by rw [hone]; decide
  have hpnat : (⟨dashu.ubigToNat setup.one, honePos⟩ : PNat) = ⟨1, Nat.one_pos⟩ := by
    apply Subtype.ext; exact hone
  -- Step 1-2: unwrap samplerDist, rewrite as the loop, apply the loop bridge.
  show samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1 x) (core.result.Result.Ok b) = _
  rw [sample_bernoulli_exp1_eq_of_positive_parts x setup.numerSigned setup.denom setup.numer
        setup.one setup.hparts setup.hsign setup.hone]
  rw [samplerDistGen_exp1_loop]
  -- Step 3: probWhile = ⨆ n probWhileCut (definitional).
  show (⨆ n, probWhileCut exp1_guard (exp1_body_dist setup.one setup.denom setup.numer) n
      (cont setup.one) (done (core.result.Result.Ok b))) = _
  -- Step 4: the cut-step induction, then normalize the PNat initial counter to 1.
  simp_rw [exp1_loop_cut_step setup.one setup.denom setup.numer hone setup.hdenom setup.hfrac _ b
    setup.one honePos, hpnat]

/-- Monotone convergence: for a sequence monotone in `n`, the supremum commutes with the sum. -/
private lemma tsum_iSup_commute (g : PNat → ℕ → ENNReal) (hmono : ∀ K, Monotone (g K)) :
    (∑' K : PNat, ⨆ n, g K n) = ⨆ n, ∑' K : PNat, g K n := by
  rw [ENNReal.tsum_eq_iSup_sum]
  simp_rw [ENNReal.finsetSum_iSup_of_monotone hmono]
  rw [iSup_comm]
  simp_rw [← ENNReal.tsum_eq_iSup_sum]

/-- The BESL loop assigns no `probWhile` mass to `(true, _)` states (the guard rejects them). -/
private lemma BESL_probWhile_true_zero
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (init : Bool × PNat) (K : PNat) :
    probWhile (fun s : Bool × PNat => s.1) (BESL numer denom hdenom hfrac) init (true, K) = 0 := by
  simp [SLang.probWhile, BESL_probWhileCut_true_zero]

/-- The BESL tail target at `b`, evaluated as a sum over terminal counters times the parity
    indicator. The do-block binds are fused before evaluation; `(true, _)` terms drop out. -/
private lemma exp1_tail_rhs_eq
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (b : Bool) :
    bernoulliExp1LoopTailTarget numer denom hdenom hfrac 1 (by decide) b =
    ∑' K : PNat,
      probWhile (fun s : Bool × PNat => s.1) (BESL numer denom hdenom hfrac)
        (true, (⟨1, Nat.one_pos⟩ : PNat)) (false, K) *
      (if decide (K.val % 2 = 0) = b then 1 else 0) := by
  unfold bernoulliExp1LoopTailTarget
  rw [show SLang.BernoulliExpNegSampleUnitLoop (dashu.ubigToNat numer)
        ⟨dashu.ubigToNat denom, hdenom⟩ hfrac = BESL numer denom hdenom hfrac from rfl]
  simp only [Bind.bind, Pure.pure, SLang.bind_bind, SLang.pure_bind]
  rw [SLang.bind_apply]
  rw [ENNReal.tsum_prod', tsum_bool]
  simp only [BESL_probWhile_true_zero, zero_mul, tsum_zero, add_zero]
  apply tsum_congr
  intro K
  congr 1
  by_cases h : (K : ℕ) % 2 = 0 <;> simp [h, SLang.pure_apply, eq_comm]

end Exp1Loop

/-! ### Main exp1 PMF theorem -/

theorem sample_bernoulli_exp1_pmf
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1 x) =
    (bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac : SLang Bool) := by
  funext b
  -- LHS: ⨆ fuel, ∑' K, BESL-cut * parity.  RHS: ∑' K, BESL-probWhile * parity.
  rw [exp1_loop_lhs_eq setup b,
    ← bernoulliExp1LoopTailTarget_one_eq setup.numer setup.denom setup.hdenom setup.hfrac,
    exp1_tail_rhs_eq setup.numer setup.denom setup.hdenom setup.hfrac b]
  -- Expand `probWhile = ⨆ probWhileCut`, pull the parity constant inside, and commute ⨆/∑'.
  simp only [SLang.probWhile, ENNReal.iSup_mul]
  rw [tsum_iSup_commute
    (fun K n => probWhileCut (fun s : Bool × PNat => s.1)
        (BESL setup.numer setup.denom setup.hdenom setup.hfrac) n
        (true, (⟨1, Nat.one_pos⟩ : PNat)) (false, K) *
      (if decide (K.val % 2 = 0) = b then 1 else 0))
    (fun K _ _ h => mul_le_mul_left (probWhileCut_monotonic _ _ _ _ h) _)]

/-! ### Outer exponential loop PMF -/

section ExpOuterLoop

/-- Main PMF theorem for the outer negative-exponential sampler.
For any nonneg rational input, the Rust sampler has the same distribution as
SampCert's `BernoulliExpNegSample`. -/
theorem sample_bernoulli_exp_pmf
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp x) =
    (bernoulliExpTarget setup.numer setup.denom setup.hdenom : SLang Bool) := by
  -- Reduce to strong induction on n = ubigToNat setup.numer
  suffices h : ∀ (n : ℕ) (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x),
      dashu.ubigToNat setup.numer = n →
      samplerDist (samplers.bernoulli.sample_bernoulli_exp x) =
        (bernoulliExpTarget setup.numer setup.denom setup.hdenom : SLang Bool) from
    h _ x setup rfl
  intro n
  induction n using Nat.strong_induction_on with
  | h n ih =>
    intro x setup hn
    by_cases hle : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom
    · -- ≤1 case
      rcases dashu.rbig_one_setup_spec with ⟨oneRat, _, one, honeRat, hone, _, _⟩
      have hgt_false : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok false :=
        dashu.rbig_gt_one_false_of_le_spec x setup.numerSigned setup.denom setup.numer oneRat
          setup.hparts setup.hsign hle
      -- body always produces done
      have hbody_le : samplers.bernoulli.sample_bernoulli_exp_loop.body x =
          samplers.bernoulli.sample_bernoulli_exp1 x >>= fun r => ok (ControlFlow.done r) := by
        unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
        simp [honeRat, hgt_false]
      -- Program equality exp x = exp1 x
      have hprog_le : samplers.bernoulli.sample_bernoulli_exp x =
          samplers.bernoulli.sample_bernoulli_exp1 x := by
        rw [sample_bernoulli_exp_eq]
        unfold samplers.bernoulli.sample_bernoulli_exp_loop
        rw [Aeneas.Std.loop.eq_def, hbody_le]
        rcases samplers.bernoulli.sample_bernoulli_exp1 x with hr | e | _
        · simp [bind_ok]
        · simp
        · simp
      -- Build exp1 setup and conclude
      -- Use `let` (not `have`) so hexp1setup.numer/denom reduce to setup.numer/denom by iota
      let hexp1setup : BernoulliExp1Setup x :=
        { numerSigned := setup.numerSigned, denom := setup.denom, numer := setup.numer,
          one := one, hparts := setup.hparts, hsign := setup.hsign, hone := hone,
          hdenom := setup.hdenom, hfrac := hle }
      rw [hprog_le, sample_bernoulli_exp1_pmf x hexp1setup,
          bernoulliExpTarget_eq_exp1_of_le setup.numer setup.denom setup.hdenom hle]
    · -- >1 case
      push_neg at hle
      -- hle : ubigToNat setup.denom < ubigToNat setup.numer
      -- 1. Get ONE rational for gt check and subtraction
      rcases dashu.rbig_one_setup_spec with
        ⟨oneRat_one, _, one_ubig, honeRat_one, hone_ubig, honeParts, honeSign⟩
      -- 2. Derive gt = true
      have hgt_true :
          dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat_one = ok true :=
        dashu.rbig_gt_one_true_of_gt_spec x setup.numerSigned setup.denom setup.numer oneRat_one
          setup.hparts setup.hsign hle
      -- 3. Build the 1/1 rational: lift and from_parts_const both give oneRat_one
      have hi : Aeneas.Std.lift (UScalar.cast UScalarTy.U128 1#u32) =
          ok (UScalar.cast UScalarTy.U128 1#u32) := by
        simp [Aeneas.Std.lift]
      have honeRat_from_parts :
          dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive
            (UScalar.cast UScalarTy.U128 1#u32) (UScalar.cast UScalarTy.U128 1#u32) =
          ok oneRat_one :=
        dashu.rbig_from_parts_const_eq_one
          (UScalar.cast UScalarTy.U128 1#u32) oneRat_one honeRat_one hi
      -- 4. Get subtraction result
      rcases dashu.rbig_sub_assign_one_exists x oneRat_one setup.numerSigned setup.denom setup.numer
          setup.hparts setup.hsign (Nat.le_of_lt hle) with ⟨xMinusOne, hsub⟩
      -- 5. Get oneSetup for exp1(oneRat_one)
      rcases sample_bernoulli_exp_one_setup_of_from_parts_const
          (UScalar.cast UScalarTy.U128 1#u32) oneRat_one hi honeRat_from_parts with
        ⟨oneSetup, honeNumerEq, honeDenomEq, honeNumerNat, honeDenomNat⟩
      -- 6. Get setupMinusOne
      rcases sample_bernoulli_exp_sub_one_setup x oneRat_one xMinusOne
          (UScalar.cast UScalarTy.U128 1#u32) setup hle hi honeRat_from_parts hsub with
        ⟨setupMinusOne, hdenomMinus, hnumerMinus⟩
      -- 7. IH applies: ubigToNat setupMinusOne.numer < n
      have hih_n : dashu.ubigToNat setupMinusOne.numer < n := by
        have hd := setup.hdenom  -- make standalone so omega can see it
        rw [hnumerMinus, hn]; omega
      have hih := ih _ hih_n xMinusOne setupMinusOne rfl
      -- 8. Outer honePos for exp1 setup
      have honePos : 0 < dashu.ubigToNat oneSetup.one := by
        rw [dashu.one_spec oneSetup.one oneSetup.hone]; norm_num
      -- 9. Body equation (when gt = true): exp1 is called on oneRat_one
      have hbody_gt : samplers.bernoulli.sample_bernoulli_exp_loop.body x =
          samplers.bernoulli.sample_bernoulli_exp1 oneRat_one >>= fun r =>
          match r with
          | core.result.Result.Ok true =>
              ok (ControlFlow.cont xMinusOne)
          | core.result.Result.Ok false =>
              ok (ControlFlow.done (core.result.Result.Ok false))
          | core.result.Result.Err e =>
              ok (ControlFlow.done (core.result.Result.Err e)) := by
        unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
        simp [honeRat_one, hgt_true, hi, honeRat_from_parts, hsub,
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]
        congr 1; funext r
        rcases r with ((_ | _) | e) <;> simp
      -- 10. Program equality: exp x = exp1 oneRat_one >>= step
      have hprog_gt : samplers.bernoulli.sample_bernoulli_exp x =
          samplers.bernoulli.sample_bernoulli_exp1 oneRat_one >>= fun r =>
          match r with
          | core.result.Result.Ok true =>
              samplers.bernoulli.sample_bernoulli_exp xMinusOne
          | core.result.Result.Ok false =>
              ok (core.result.Result.Ok false)
          | core.result.Result.Err e =>
              ok (core.result.Result.Err e) := by
        rw [sample_bernoulli_exp_eq]
        unfold samplers.bernoulli.sample_bernoulli_exp_loop
        rw [Aeneas.Std.loop.eq_def, hbody_gt]
        rcases samplers.bernoulli.sample_bernoulli_exp1 oneRat_one with hr | e | _
        · rcases hr with (b | e)
          · cases b <;> rfl
          · simp [bind_ok]
        · simp
        · simp
      -- 11. Distribution computation
      funext b
      show samplerDistGen (samplers.bernoulli.sample_bernoulli_exp x)
          (core.result.Result.Ok b) =
          bernoulliExpTarget setup.numer setup.denom setup.hdenom b
      rw [hprog_gt, samplerDistGen_bind, SLang.bind_apply]
      -- Eliminate Err mass (all those terms give 0 at Ok b)
      rw [tsum_result_ok_eq (fun e => by
          simp [samplerDistGen_pure_ok, PMF.pure_apply])]
      rw [tsum_bool]
      -- Separate false and true branches
      simp only [samplerDistGen_pure_ok, PMF.pure_apply]
      -- Use exp1 PMF and IH
      rw [show samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1 oneRat_one)
              (core.result.Result.Ok false) =
              bernoulliExp1Target oneSetup.numer oneSetup.denom oneSetup.hdenom oneSetup.hfrac false
          from congrFun (sample_bernoulli_exp1_pmf oneRat_one oneSetup) false,
          show samplerDistGen (samplers.bernoulli.sample_bernoulli_exp1 oneRat_one)
              (core.result.Result.Ok true) =
              bernoulliExp1Target oneSetup.numer oneSetup.denom oneSetup.hdenom oneSetup.hfrac true
          from congrFun (sample_bernoulli_exp1_pmf oneRat_one oneSetup) true,
          show samplerDistGen (samplers.bernoulli.sample_bernoulli_exp xMinusOne)
              (core.result.Result.Ok b) =
              bernoulliExpTarget setupMinusOne.numer setupMinusOne.denom setupMinusOne.hdenom b
          from congrFun hih b]
      -- Use the target step equation (pointed version)
      have htarget :=
        sample_bernoulli_exp_target_step_of_gt x oneRat_one xMinusOne setup oneSetup honePos
          setupMinusOne hle hnumerMinus
      rw [congrFun htarget b]
      simp only [Bind.bind, Pure.pure, SLang.bind_apply, tsum_bool]
      -- Simplify if-branches, probPure, bridge numer/denom→one, and setup.denom→setupMinusOne.denom
      simp [← hdenomMinus, SLang.pure_apply, honeNumerEq, honeDenomEq]

end ExpOuterLoop

end bernoulli
