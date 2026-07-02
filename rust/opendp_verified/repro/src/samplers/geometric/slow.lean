import Generated.OpenDP
import SampCert.Samplers.Geometric.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.uniform.pmf
import src.samplers.bernoulli.exp

/-!
# `sample_geometric_exp_slow` — geometric via `Bernoulli(e^{-x})` (roadmap stage 6)

Target: `⟦sample_geometric_exp_slow x⟧ₙ = SLang.probGeometric (Bernoulli(e^{-x})) (· + 1)` for
`x = numer/denom ≥ 0`. The extracted function runs a `probWhile` loop that draws
`Bernoulli(e^{-x})` (stage 5, `sample_bernoulli_exp`), increments an opaque `UBig` counter on
success, and returns the counter at the first failure. SampCert's `probGeometric` counts the
*total* number of draws (successes plus the final failure), so the extracted output `n`
corresponds to SampCert's `n + 1`.

Structure mirrors `bernoulli/exp1.lean`: factor the loop body through the proved stage-5 draw
(`geo_step`), compute the body's `cont`/`done` fiber distributions, match every `probWhileCut`
depth against SampCert's `geometric_succ_true`/`geometric_succ_false` recurrences
(`geo_slow_loop_cut_step`), lift through `⨆`/`tsum_iSup_commute`, and close with
`geometric_pwc_sup` / `probGeometric_apply`. The opaque-`UBig` output is pushed to `ℕ` with
`samplerDist_nat` (`uniform/pmf.lean`), summing the settle mass against the
`ubigToNat`-indicator.
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Classical

namespace OpenDP.samplers.geometric

open OpenDP.samplers.bernoulli (BernoulliExpSetup sample_bernoulli_exp_spec)
open OpenDP.samplers.uniform (samplerDist_nat)

/-- The SampCert Bernoulli(e^{-numer/denom}) trial driving the geometric loop. -/
noncomputable def geoTrial (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) : SLang Bool :=
  BernoulliExpNegSample (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩

/-- Stage 5 spec, pointwise at `Ok b`: the extracted Bernoulli draw realises `geoTrial`. -/
lemma samplerDistGen_bernoulli_exp (r : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup r)
    (b : Bool) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
      geoTrial setup.numer setup.denom setup.hdenom b := by
  have h := congrFun (sample_bernoulli_exp_spec r setup) b
  simpa [samplerDist, geoTrial] using h

/-! ### The loop body, factored through the Bernoulli draw -/

/-- The deterministic continuation of one slow-geometric loop step after the Bernoulli draw
returns `r1`: continue with the incremented counter on success, settle with the current counter
on failure, propagate errors. Naming it gives stable equation lemmas for `simp`. -/
noncomputable def geo_step (k : dashu_int.ubig.UBig) :
    core.result.Result Bool error.Error →
    Result (ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error)) :=
  fun r1 => do
    let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r1
    match cf with
    | core.ops.control_flow.ControlFlow.Continue val =>
      if val then do
        let u ← dashu_int.ubig.UBig.ONE
        let k1 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k u
        ok (cont k1)
      else ok (done (core.result.Result.Ok k))
    | core.ops.control_flow.ControlFlow.Break residual => do
      let r2 ← core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
        dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
      ok (done r2)

/-- Once the (deterministic) clone succeeds, the slow-geometric body factors through `geo_step`. -/
lemma geo_body_eq_step (x r : dashu_ratio.rbig.RBig) (k : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r) :
    samplers.geometric.sample_geometric_exp_slow_loop.body x k =
      samplers.bernoulli.sample_bernoulli_exp r >>= geo_step k := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop.body
  rw [hclone]
  rfl

/-- Step on `Ok true`: a point mass at `cont k1'` (where `k1' = k + 1`). -/
lemma geo_step_ok_true (k one k1' : dashu_int.ubig.UBig)
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one = ok k1')
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result dashu_int.ubig.UBig error.Error)) :
    samplerDistGen (geo_step k (core.result.Result.Ok true)) out =
      (if out = cont k1' then 1 else 0) := by
  simp [geo_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    hone, hadd, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Ok false`: a point mass at `done (Ok k)` (settle with the current counter). -/
lemma geo_step_ok_false (k : dashu_int.ubig.UBig)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result dashu_int.ubig.UBig error.Error)) :
    samplerDistGen (geo_step k (core.result.Result.Ok false)) out =
      (if out = done (core.result.Result.Ok k) then 1 else 0) := by
  simp [geo_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    samplerDistGen_pure_ok, PMF.pure_apply]

/-- Step on `Err e`: a point mass at `done (Err e)`. -/
lemma geo_step_err (k : dashu_int.ubig.UBig) (e : error.Error)
    (out : ControlFlow dashu_int.ubig.UBig (core.result.Result dashu_int.ubig.UBig error.Error)) :
    samplerDistGen (geo_step k (core.result.Result.Err e)) out =
      (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [geo_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual,
    samplerDistGen_pure_ok, PMF.pure_apply]

/-! ### Body fiber distributions -/

/-- The loop body's `cont` mass: `trial(true)` times the point mass at the incremented counter. -/
lemma geo_body_cont_apply (x r : dashu_ratio.rbig.RBig)
    (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hbern : ∀ b : Bool,
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
        geoTrial numer denom hdenom b)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (k k2 one k1' : dashu_int.ubig.UBig)
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd : dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one = ok k1') :
    samplerDistGen (samplers.geometric.sample_geometric_exp_slow_loop.body x k) (cont k2) =
      geoTrial numer denom hdenom true * (if k2 = k1' then 1 else 0) := by
  rw [geo_body_eq_step x r k hclone, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [geo_step_err]; simp)]
  rw [tsum_bool, geo_step_ok_false k, geo_step_ok_true k one k1' hone hadd]
  rw [hbern true]
  simp [ControlFlow.cont.injEq]

/-- The loop body's `done (Ok u)` mass: `trial(false)` times the point mass at the current
counter. -/
lemma geo_body_done_ok_apply (x r : dashu_ratio.rbig.RBig)
    (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hbern : ∀ b : Bool,
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
        geoTrial numer denom hdenom b)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (k u : dashu_int.ubig.UBig) :
    samplerDistGen (samplers.geometric.sample_geometric_exp_slow_loop.body x k)
        (done (core.result.Result.Ok u)) =
      geoTrial numer denom hdenom false * (if u = k then 1 else 0) := by
  obtain ⟨one, hone, _⟩ := dashu.one_exists_spec
  obtain ⟨k1', hadd, _⟩ := dashu.add_assign_exists_spec k one
  rw [geo_body_eq_step x r k hclone, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by rw [geo_step_err]; simp)]
  rw [tsum_bool, geo_step_ok_false k, geo_step_ok_true k one k1' hone hadd]
  rw [hbern false]
  simp [ControlFlow.done.injEq, core.result.Result.Ok.injEq]

/-! ### Cut-depth correspondence with SampCert's geometric loop -/

/-- SampCert's one-step geometric recurrence from a live state, restated with `fuel + 1` (the
shape the induction produces) instead of `Nat.succ fuel`. -/
lemma geo_cut_succ_true (trial : SLang Bool) (fuel n : ℕ) (st : Bool × ℕ) :
    probWhileCut geoLoopCond (geoLoopBody trial) (fuel + 1) (true, n) st =
      trial false * probWhileCut geoLoopCond (geoLoopBody trial) fuel (false, n + 1) st +
      trial true * probWhileCut geoLoopCond (geoLoopBody trial) fuel (true, n + 1) st :=
  SLang.geometric_succ_true trial fuel n st

/-- SampCert's settle recurrence from a dead state, restated with `fuel + 1`. -/
lemma geo_cut_succ_false (trial : SLang Bool) (fuel n : ℕ) (st : Bool × ℕ) :
    probWhileCut geoLoopCond (geoLoopBody trial) (fuel + 1) (false, n) st =
      if st = (false, n) then 1 else 0 :=
  SLang.geometric_succ_false trial fuel n st

/-- **Cut-depth correspondence (the crux).** At every cut depth `n`, the extracted loop's
`probWhileCut` mass on outputs `done (Ok u)` with `ubigToNat u = v` (started from counter `k`)
equals SampCert's geometric loop's mass on the terminal state `(false, v + 1)` started from
`(true, ubigToNat k)` — the `+ 1` accounts for SampCert counting the final failing draw.
Induction on `n`; the step splits the extracted body into its `cont` (recurse via `ih`) and
`done` (settle) fibers and matches them against `geometric_succ_true` / `geometric_succ_false`. -/
private lemma geo_slow_loop_cut_step (x r : dashu_ratio.rbig.RBig)
    (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hbern : ∀ b : Bool,
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
        geoTrial numer denom hdenom b)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (cond : ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error) → Bool)
    (bd : ControlFlow dashu_int.ubig.UBig
        (core.result.Result dashu_int.ubig.UBig error.Error) →
      SLang (ControlFlow dashu_int.ubig.UBig
        (core.result.Result dashu_int.ubig.UBig error.Error)))
    (hcc : ∀ a, cond (cont a) = true)
    (hcd : ∀ w, cond (done w) = false)
    (hbc : ∀ a, bd (cont a) =
      samplerDistGen (samplers.geometric.sample_geometric_exp_slow_loop.body x a))
    (n : ℕ) :
    ∀ (k : dashu_int.ubig.UBig) (v : ℕ),
      (∑' u : dashu_int.ubig.UBig,
        probWhileCut cond bd n (cont k) (done (core.result.Result.Ok u)) *
          (if v = dashu.ubigToNat u then 1 else 0)) =
      probWhileCut geoLoopCond (geoLoopBody (geoTrial numer denom hdenom)) n
        (true, dashu.ubigToNat k) (false, v + 1) := by
  induction n with
  | zero =>
    intro k v
    simp only [probWhileCut, SLang.probZero, zero_mul, tsum_zero]
  | succ n ih =>
    intro k v
    obtain ⟨one, hone, honeval⟩ := dashu.one_exists_spec
    obtain ⟨k1', hadd, hk1'nat⟩ := dashu.add_assign_exists_spec k one
    have hk1'val : dashu.ubigToNat k1' = dashu.ubigToNat k + 1 := by rw [hk1'nat, honeval]
    cases n with
    | zero =>
      -- Depth 1: reaching a `done` output takes at least two cut steps on both sides.
      rw [geo_cut_succ_true]
      simp only [probWhileCut, probWhileFunctional, hcc k, if_true, Bind.bind,
        SLang.bind_apply, SLang.probZero, mul_zero, tsum_zero, zero_mul, add_zero]
    | succ n' =>
      -- One-step unfolding of the extracted cut, pointwise in the output `u`.
      have hpt : ∀ u : dashu_int.ubig.UBig,
          probWhileCut cond bd (n' + 1 + 1) (cont k) (done (core.result.Result.Ok u)) =
            geoTrial numer denom hdenom true *
              probWhileCut cond bd (n' + 1) (cont k1') (done (core.result.Result.Ok u)) +
            geoTrial numer denom hdenom false * (if u = k then 1 else 0) := by
        intro u
        rw [probWhileCut, probWhileFunctional, if_pos (hcc k)]
        simp only [Bind.bind, SLang.bind_apply]
        rw [tsum_controlFlow]
        congr 1
        · -- CONT fiber: only the incremented counter `k1'` survives.
          simp_rw [hbc k,
            geo_body_cont_apply (x := x) (r := r) (numer := numer) (denom := denom)
              (hdenom := hdenom) (hbern := hbern) (hclone := hclone) (k := k) (one := one)
              (k1' := k1') (hone := hone) (hadd := hadd),
            mul_assoc]
          rw [ENNReal.tsum_mul_left]
          congr 1
          rw [tsum_eq_single k1' (fun a ha => by rw [if_neg ha, zero_mul]), if_pos rfl, one_mul]
        · -- DONE fiber: only `done (Ok u)` survives; the body settles at the current counter.
          simp_rw [probWhileCut_done_pt cond bd hcd n', SLang.pure_apply]
          rw [tsum_eq_single (core.result.Result.Ok u) (fun w hw => by
              rw [if_neg (fun h => by injection h with h'; exact hw h'.symm), mul_zero]),
            if_pos rfl, mul_one, hbc k,
            geo_body_done_ok_apply x r numer denom hdenom hbern hclone k u]
      -- Assemble: distribute the `ubigToNat` indicator, recurse via `ih`, match SampCert.
      simp_rw [hpt, add_mul, mul_assoc]
      rw [ENNReal.tsum_add, ENNReal.tsum_mul_left, ENNReal.tsum_mul_left,
        ih k1' v, hk1'val,
        tsum_eq_single k (fun u hu => by rw [if_neg hu, zero_mul]), if_pos rfl, one_mul]
      conv_rhs => rw [geo_cut_succ_true, geo_cut_succ_false]
      have hind : (if ((false : Bool), v + 1) = ((false : Bool), dashu.ubigToNat k + 1)
            then (1 : ENNReal) else 0) =
          (if v = dashu.ubigToNat k then 1 else 0) := by
        by_cases h : v = dashu.ubigToNat k <;> simp [h]
      rw [hind]
      ring

/-! ### Full `probWhile` lift and the SampCert equality -/

/-- Lift the cut-depth correspondence to the full `probWhile`: the extracted slow-geometric
loop, started at counter `k0`, outputs a `UBig` with value `v` with exactly the mass SampCert's
geometric loop assigns to terminating at `(false, v + 1)` from `(true, ubigToNat k0)`. -/
lemma geo_slow_loop_probWhile (x r : dashu_ratio.rbig.RBig)
    (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom)
    (hbern : ∀ b : Bool,
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
        geoTrial numer denom hdenom b)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (k0 : dashu_int.ubig.UBig) (v : ℕ) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_slow_loop x k0) v =
      probWhile geoLoopCond (geoLoopBody (geoTrial numer denom hdenom))
        (true, dashu.ubigToNat k0) (false, v + 1) := by
  let cond : ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error) → Bool :=
    fun cf => match cf with | cont _ => true | done _ => false
  let bd : ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error) →
      SLang (ControlFlow dashu_int.ubig.UBig
        (core.result.Result dashu_int.ubig.UBig error.Error)) :=
    fun cf => match cf with
      | cont a => samplerDistGen (samplers.geometric.sample_geometric_exp_slow_loop.body x a)
      | done _ => PMF.pure cf
  have hcc : ∀ a, cond (cont a) = true := fun _ => rfl
  have hcd : ∀ w, cond (done w) = false := fun _ => rfl
  have hbc : ∀ a, bd (cont a) =
      samplerDistGen (samplers.geometric.sample_geometric_exp_slow_loop.body x a) := fun _ => rfl
  -- Step 1: `samplerDist` of the extracted loop is `probWhile` of the body distribution.
  have hstep1 : ∀ u : dashu_int.ubig.UBig,
      samplerDist (samplers.geometric.sample_geometric_exp_slow_loop x k0) u =
        probWhile cond bd (cont k0) (done (core.result.Result.Ok u)) := by
    intro u
    simp only [samplerDist, samplers.geometric.sample_geometric_exp_slow_loop,
      samplerDistGen_loop]
    congr 1 <;> (funext cf; cases cf <;> rfl)
  -- Step 2: expand the nat-pushforward into the settle sum.
  have hexpand : samplerDist_nat (samplers.geometric.sample_geometric_exp_slow_loop x k0) v =
      ∑' u : dashu_int.ubig.UBig,
        samplerDist (samplers.geometric.sample_geometric_exp_slow_loop x k0) u *
          (if v = dashu.ubigToNat u then 1 else 0) := by
    simp only [samplerDist_nat, SLang.probBind, SLang.probPure]
    refine tsum_congr fun u => ?_
    by_cases h : v = dashu.ubigToNat u <;> simp [h]
  rw [hexpand]
  simp_rw [hstep1]
  -- Step 3: unfold every `probWhile` to `⨆ probWhileCut`, pull the indicator into each sup,
  -- then commute `∑' u` past the `⨆ n`.
  simp only [probWhile]
  simp_rw [ENNReal.iSup_mul]
  rw [tsum_iSup_commute _ (fun u => (probWhileCut_monotonic cond bd (cont k0)
      (done (core.result.Result.Ok u))).mul_const (zero_le _))]
  -- Step 4: match each cut via the correspondence lemma.
  refine iSup_congr (fun n => ?_)
  exact geo_slow_loop_cut_step x r numer denom hdenom hbern hclone cond bd hcc hcd hbc n k0 v

/-- The extracted slow-geometric wrapper reduces to its loop started at counter `0`. -/
lemma sample_geometric_exp_slow_eq_loop (x : dashu_ratio.rbig.RBig)
    (z : dashu_int.ubig.UBig) (hz : dashu_int.ubig.UBig.ZERO = ok z) :
    samplers.geometric.sample_geometric_exp_slow x =
      samplers.geometric.sample_geometric_exp_slow_loop x z := by
  unfold samplers.geometric.sample_geometric_exp_slow
  rw [hz]
  rfl

/-- **Distributional correctness (roadmap stage 6).** On the nonnegative-input branch, the
extracted `sample_geometric_exp_slow` realises SampCert's geometric law over the
`Bernoulli(e^{-x})` trial: the value-level output `v` carries the mass `probGeometric` assigns
to `v + 1` total draws (`v` successes plus the final failure). -/
theorem sample_geometric_exp_slow_spec (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_slow x) =
      fun v => probGeometric (geoTrial setup.numer setup.denom setup.hdenom) (v + 1) := by
  obtain ⟨z, hz, hzval⟩ := dashu.zero_exists_spec
  obtain ⟨r, hclone⟩ := dashu.rbig_clone_exists_spec x
  have hparts_r := dashu.rbig_clone_parts_spec x r _ hclone setup.hparts
  have hbern : ∀ b : Bool,
      samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
        geoTrial setup.numer setup.denom setup.hdenom b :=
    samplerDistGen_bernoulli_exp r
      ⟨setup.numerSigned, setup.denom, setup.numer, hparts_r, setup.hsign, setup.hdenom⟩
  funext v
  rw [sample_geometric_exp_slow_eq_loop x z hz,
    geo_slow_loop_probWhile x r setup.numer setup.denom setup.hdenom hbern hclone z v,
    hzval, SLang.probGeometric_apply]
  show (⨆ i, probWhileCut geoLoopCond
      (geoLoopBody (geoTrial setup.numer setup.denom setup.hdenom)) i (true, 0)
      (false, v + 1)) = _
  rw [SLang.geometric_pwc_sup]

/-- Closed form (CKS): `P[out = v] = e^{-x·v} · (1 - e^{-x})`, as trial powers. -/
theorem sample_geometric_exp_slow_closed_form (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) (v : ℕ) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_slow x) v =
      (geoTrial setup.numer setup.denom setup.hdenom true) ^ v *
        (geoTrial setup.numer setup.denom setup.hdenom false) := by
  rw [sample_geometric_exp_slow_spec x setup]
  simp

end OpenDP.samplers.geometric
