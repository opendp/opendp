import Generated.OpenDP
import SampCert.Samplers.Laplace.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.samplers.uniform.pmf
import src.samplers.geometric.slow

/-!
# `sample_geometric_exp_fast` — optimized geometric ("dast", roadmap stage 7)

Target: the fast sampler realises the same geometric law as stage 6 —
`Geo(1 - e^{-numer/denom})` — via the CKS optimization: draw a uniform residue `u ∈ [0, denom)`,
accept it with probability `e^{-u/denom}` (rejection: redraw `u`), draw the unit geometric
`v ~ Geo(1 - e^{-1})` (stage 6 at `x = 1`), and output `(v·denom + u) / numer` (floor division).
The SampCert reference shape is `DiscreteLaplaceSampleLoop'` (Laplace inner loop): accepted
residue `DiscreteLaplaceSampleLoopIn1`, unit geometric `DiscreteLaplaceSampleLoopIn2 1 1`.

## Proof structure (complete — zero `sorry`)

1. **`partial_fixpoint` → `loop` bridge** (`sample_geometric_exp_fast_loop_eq_loop`): Aeneas
   extracts this loop as a **recursive function** (Lean `partial_fixpoint`), not the
   `Aeneas.Std.loop` combinator, so the loop-semantics axiom (iv) does not apply directly.
   Proved (no new axiom) by two-sided least-fixpoint induction in the flat `Result` order
   (`div` bottom) via the generated `fixpoint_induct` principles.
2. **Wrapper decomposition** (`…_eq_zero`, `…_eq_of_pos`): uniform residue, then the loop.
3. **SampCert-side model** (`fastLoopBody` over states `(live?, residue)`), its step laws, and
   the rejection closed form: a geometric series over the reject mass gives
   `Σ unif·probWhile = fastAcceptMixed / fastAcceptMass` (`fast_mixed_probWhile`).
4. **Extracted-side fiber laws**: the body's `cont` mass is the stage-2 uniform redraw; its
   settle mass (summed against the output indicator) is the stage-6 slow law at `1` pushed
   through the floor-division arithmetic (`fast_body_cont_mass`, `fast_body_done_summed`).
5. **Cut-depth induction + lift** (`geo_fast_loop_cut_step`, `geo_fast_loop_probWhile`).
6. **Identification with SampCert's Laplace inner loop**: `In1_apply_form` normalizes `In1` to
   `unif·accept / mass`; `fastTarget` (accepted residue + `In2 1 1` + quotient) then equals the
   rejection closed form, and the ported legacy `Geo` algebra (`fastTarget_pmf`,
   `fastTarget_eq_slowLaw`) closes the loop: **the fast sampler has exactly the stage-6 law**.
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Semantics
open SLang PMF ENNReal Classical
open Lean.Order

namespace OpenDP.samplers.geometric

/-! ### The fast loop as an Aeneas `ControlFlow` body -/

/-- One iteration of the fast geometric loop, reshaped as a `ControlFlow` step for the
`Aeneas.Std.loop` combinator: test the residue `u` with `Bernoulli(e^{-u/denom})`; on accept,
settle with `(v·denom + u) / numer` where `v` is the stage-6 slow geometric at `1`; on reject,
continue with a fresh uniform residue. Mirrors the extracted `sample_geometric_exp_fast_loop`
body verbatim, with the tail-recursive call replaced by `cont` and returns by `done`. -/
noncomputable def fast_body (denom : dashu_int.ubig.UBig) (numer : dashu_int.ibig.IBig)
    (u : dashu_int.ubig.UBig) :
    Result (ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error)) := do
  let i ← dashu_int.convert.UBig.as_ibig u
  let i1 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i
  let u1 ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom
  let r ← dashu_ratio.rbig.RBig.from_parts i1 u1
  let r1 ← samplers.bernoulli.sample_bernoulli_exp r
  let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r1
  match cf with
  | core.ops.control_flow.ControlFlow.Continue val =>
    if val
    then do
      let r2 ← dashu_ratio.rbig.RBig.ONE
      let r3 ← samplers.geometric.sample_geometric_exp_slow r2
      let cf1 ←
        core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r3
      match cf1 with
      | core.ops.control_flow.ControlFlow.Continue val1 => do
        let u2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
        let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
        let (_, u4) ← dashu_int.ibig.IBig.into_parts numer
        let u5 ← dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
        ok (done (core.result.Result.Ok u5))
      | core.ops.control_flow.ControlFlow.Break residual => do
        let w ←
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
        ok (done w)
    else do
      let r2 ← samplers.uniform.sample_uniform_ubig_below u1
      let cf1 ←
        core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r2
      match cf1 with
      | core.ops.control_flow.ControlFlow.Continue val1 => ok (cont val1)
      | core.ops.control_flow.ControlFlow.Break residual => do
        let w ←
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
        ok (done w)
  | core.ops.control_flow.ControlFlow.Break residual => do
    let w ←
      core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
        dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
    ok (done w)

/-! ### Flat-order helpers for `Result` -/

/-- `div` is the bottom of the flat `Result` order. -/
private lemma result_div_le {α : Type} (x : Result α) :
    (Result.div : Result α) ⊑ x := FlatOrder.rel.bot

/-- Reflexivity of the flat `Result` order. -/
private lemma result_le_refl {α : Type} (x : Result α) : x ⊑ x := FlatOrder.rel.refl

/-- One-step unfolding of `fast_body` at a specific residue, as a rewrite-friendly equation.
Rewriting with this (instead of `unfold fast_body`) exposes the head operations of the loop
*scrutinee only*, leaving the folded `loop (fast_body denom numer)` back-edge untouched so the
later `simp [h…]` steps cannot rewrite inside it. -/
private lemma fast_body_unfold (denom : dashu_int.ubig.UBig) (numer : dashu_int.ibig.IBig)
    (u : dashu_int.ubig.UBig) :
    fast_body denom numer u = (do
      let i ← dashu_int.convert.UBig.as_ibig u
      let i1 ← dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i
      let u1 ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom
      let r ← dashu_ratio.rbig.RBig.from_parts i1 u1
      let r1 ← samplers.bernoulli.sample_bernoulli_exp r
      let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r1
      match cf with
      | core.ops.control_flow.ControlFlow.Continue val =>
        if val
        then do
          let r2 ← dashu_ratio.rbig.RBig.ONE
          let r3 ← samplers.geometric.sample_geometric_exp_slow r2
          let cf1 ←
            core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r3
          match cf1 with
          | core.ops.control_flow.ControlFlow.Continue val1 => do
            let u2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
            let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
            let (_, u4) ← dashu_int.ibig.IBig.into_parts numer
            let u5 ← dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
            ok (done (core.result.Result.Ok u5))
          | core.ops.control_flow.ControlFlow.Break residual => do
            let w ←
              core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
                dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
            ok (done w)
        else do
          let r2 ← samplers.uniform.sample_uniform_ubig_below u1
          let cf1 ←
            core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r2
          match cf1 with
          | core.ops.control_flow.ControlFlow.Continue val1 => ok (cont val1)
          | core.ops.control_flow.ControlFlow.Break residual => do
            let w ←
              core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
                dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
            ok (done w)
      | core.ops.control_flow.ControlFlow.Break residual => do
        let w ←
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
        ok (done w)) := rfl

/-! ### The bridge -/

/-- **The `partial_fixpoint` → `loop` bridge.** The extracted fast loop (a genuine Lean
recursion) coincides with the `Aeneas.Std.loop` combinator over `fast_body` — so the
probabilistic loop semantics (`samplerDistGen_loop`) applies to it. Proved by two-sided
least-fixpoint induction in the flat `Result` order (`div` is bottom); no new axiom. Each
induction step case-bashes the (shared, opaque) head operations of one unfolding so that every
`match` scrutinee becomes a literal constructor, at which point both sides reduce in lockstep. -/
theorem sample_geometric_exp_fast_loop_eq_loop (denom : dashu_int.ubig.UBig)
    (numer : dashu_int.ibig.IBig) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer =
      Aeneas.Std.loop (fast_body denom numer) := by
  have h1 : ∀ u, samplers.geometric.sample_geometric_exp_fast_loop denom numer u ⊑
      Aeneas.Std.loop (fast_body denom numer) u := by
    apply samplers.geometric.sample_geometric_exp_fast_loop.fixpoint_induct denom numer
      (motive := fun g => ∀ u, g u ⊑ Aeneas.Std.loop (fast_body denom numer) u)
    · exact admissible_pi_apply
        (fun u (v : Result (core.result.Result dashu_int.ubig.UBig error.Error)) =>
          v ⊑ Aeneas.Std.loop (fast_body denom numer) u)
        (fun u => admissible_flatOrder _ (result_div_le _))
    · intro g hg u
      conv_rhs => rw [Aeneas.Std.loop.eq_def]
      rw [fast_body_unfold denom numer u]
      rcases hI : dashu_int.convert.UBig.as_ibig u with i | e | - <;>
        (try simp [hI]) <;> (try exact result_le_refl _)
      rcases hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i with i1 | e | - <;>
        (try simp [hC]) <;> (try exact result_le_refl _)
      rcases hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom with u1 | e | - <;>
        (try simp [hD]) <;> (try exact result_le_refl _)
      rcases hF : dashu_ratio.rbig.RBig.from_parts i1 u1 with r | e | - <;>
        (try simp [hF]) <;> (try exact result_le_refl _)
      rcases hB : samplers.bernoulli.sample_bernoulli_exp r with r1 | e | - <;>
        (try simp [hB]) <;> (try exact result_le_refl _)
      rcases r1 with b | e
      · cases b
        · -- reject: fresh uniform residue, then the recursive call vs the loop back-edge.
          simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
            bind_tc_ok, Bool.false_eq_true, if_false]
          rcases hU : samplers.uniform.sample_uniform_ubig_below u1 with r2 | e | - <;>
            (try simp [hU]) <;> (try exact result_le_refl _)
          rcases r2 with val1 | e
          · simpa [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]
              using hg val1
          · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              from_residual_err_ok]
            exact result_le_refl _
        · -- accept: both sides settle on the same arithmetic value.
          simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
            bind_tc_ok, if_true]
          rcases hO : dashu_ratio.rbig.RBig.ONE with r2 | e | - <;>
            (try simp [hO]) <;> (try exact result_le_refl _)
          rcases hS : samplers.geometric.sample_geometric_exp_slow r2 with r3 | e | - <;>
            (try simp [hS]) <;> (try exact result_le_refl _)
          rcases r3 with val1 | e
          · simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              bind_tc_ok]
            rcases hM : dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
                with u2 | e | - <;>
              (try simp [hM]) <;> (try exact result_le_refl _)
            rcases hA : dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
                with u3 | e | - <;>
              (try simp [hA]) <;> (try exact result_le_refl _)
            rcases hP : dashu_int.ibig.IBig.into_parts numer with ⟨sgn, u4⟩ | e | - <;>
              (try simp [hP]) <;> (try exact result_le_refl _)
            rcases hV : dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
                with u5 | e | - <;>
              (try simp [hV]) <;> exact result_le_refl _
          · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              from_residual_err_ok]
            exact result_le_refl _
      · -- Bernoulli error propagates identically.
        simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
          from_residual_err_ok]
        exact result_le_refl _
  have h2 : ∀ u, Aeneas.Std.loop (fast_body denom numer) u ⊑
      samplers.geometric.sample_geometric_exp_fast_loop denom numer u := by
    apply Aeneas.Std.loop.fixpoint_induct (fast_body denom numer)
      (motive := fun g => ∀ u,
        g u ⊑ samplers.geometric.sample_geometric_exp_fast_loop denom numer u)
    · exact admissible_pi_apply
        (fun u (v : Result (core.result.Result dashu_int.ubig.UBig error.Error)) =>
          v ⊑ samplers.geometric.sample_geometric_exp_fast_loop denom numer u)
        (fun u => admissible_flatOrder _ (result_div_le _))
    · intro g hg u
      conv_rhs => rw [samplers.geometric.sample_geometric_exp_fast_loop.eq_def]
      beta_reduce
      rw [fast_body_unfold denom numer u]
      rcases hI : dashu_int.convert.UBig.as_ibig u with i | e | - <;>
        (try simp [hI]) <;> (try exact result_le_refl _)
      rcases hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i with i1 | e | - <;>
        (try simp [hC]) <;> (try exact result_le_refl _)
      rcases hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom with u1 | e | - <;>
        (try simp [hD]) <;> (try exact result_le_refl _)
      rcases hF : dashu_ratio.rbig.RBig.from_parts i1 u1 with r | e | - <;>
        (try simp [hF]) <;> (try exact result_le_refl _)
      rcases hB : samplers.bernoulli.sample_bernoulli_exp r with r1 | e | - <;>
        (try simp [hB]) <;> (try exact result_le_refl _)
      rcases r1 with b | e
      · cases b
        · simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
            bind_tc_ok, Bool.false_eq_true, if_false]
          rcases hU : samplers.uniform.sample_uniform_ubig_below u1 with r2 | e | - <;>
            (try simp [hU]) <;> (try exact result_le_refl _)
          rcases r2 with val1 | e
          · simpa [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]
              using hg val1
          · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              from_residual_err_ok]
            exact result_le_refl _
        · simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
            bind_tc_ok, if_true]
          rcases hO : dashu_ratio.rbig.RBig.ONE with r2 | e | - <;>
            (try simp [hO]) <;> (try exact result_le_refl _)
          rcases hS : samplers.geometric.sample_geometric_exp_slow r2 with r3 | e | - <;>
            (try simp [hS]) <;> (try exact result_le_refl _)
          rcases r3 with val1 | e
          · simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              bind_tc_ok]
            rcases hM : dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
                with u2 | e | - <;>
              (try simp [hM]) <;> (try exact result_le_refl _)
            rcases hA : dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
                with u3 | e | - <;>
              (try simp [hA]) <;> (try exact result_le_refl _)
            rcases hP : dashu_int.ibig.IBig.into_parts numer with ⟨sgn, u4⟩ | e | - <;>
              (try simp [hP]) <;> (try exact result_le_refl _)
            rcases hV : dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
                with u5 | e | - <;>
              (try simp [hV]) <;> exact result_le_refl _
          · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
              from_residual_err_ok]
            exact result_le_refl _
      · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
          from_residual_err_ok]
        exact result_le_refl _
  funext u
  exact Lean.Order.PartialOrder.rel_antisymm (h1 u) (h2 u)

/-! ### Wrapper decomposition -/

/-- The extracted fast geometric wrapper returns zero on the zero-input branch. -/
theorem sample_geometric_exp_fast_eq_zero (x : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.RBig.is_zero x = ok true) :
    samplers.geometric.sample_geometric_exp_fast x =
      (do
        let u ← dashu_int.ubig.UBig.ZERO
        ok (core.result.Result.Ok u)) := by
  unfold samplers.geometric.sample_geometric_exp_fast
  simp [hzero]

/-- On positive inputs the extracted fast geometric wrapper reduces to a uniform residue draw
followed by the `loop`-form of the fast loop (via the `partial_fixpoint` bridge). -/
theorem sample_geometric_exp_fast_eq_of_pos (x : dashu_ratio.rbig.RBig)
    (setup : OpenDP.samplers.bernoulli.BernoulliExpSetup x)
    (hpos : 0 < dashu.ubigToNat setup.numer) :
    samplers.geometric.sample_geometric_exp_fast x =
      (do
        let u ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone setup.denom
        let r ← samplers.uniform.sample_uniform_ubig_below u
        let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r
        match cf with
        | core.ops.control_flow.ControlFlow.Continue val =>
          Aeneas.Std.loop (fast_body setup.denom setup.numerSigned) val
        | core.ops.control_flow.ControlFlow.Break residual =>
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual) := by
  have hzero : dashu_ratio.rbig.RBig.is_zero x = ok false :=
    dashu.rbig_is_zero_false_spec x setup.numerSigned setup.numer setup.denom
      setup.hparts setup.hsign hpos
  unfold samplers.geometric.sample_geometric_exp_fast
  rw [hzero, setup.hparts]
  simp
  rw [sample_geometric_exp_fast_loop_eq_loop setup.denom setup.numerSigned]
  all_goals refine congrArg _ (funext fun u => ?_)
  all_goals refine congrArg _ (funext fun r => ?_)
  all_goals refine congrArg _ (funext fun cf => ?_)
  all_goals rcases cf with val | residual <;> rfl

/-! ### The SampCert-side model of the fast loop

State `(true, u)` = live with untested residue `u`; `(false, m)` = settled with output `m`.
The guard is SampCert's `geoLoopCond` (`st.1`). One iteration: test the residue with
`Bernoulli(e^{-u/den})`; on accept, draw the unit slow geometric and combine; on reject, draw a
fresh uniform residue. -/

/-- The stage-6 slow-geometric law on `ℕ`: `v` successes then one failure
(`probGeometric` counts the final failing draw, hence the `+ 1`). -/
noncomputable def slowLaw (num : ℕ) (den : ℕ+) : SLang ℕ :=
  fun v => probGeometric (BernoulliExpNegSample num den) (v + 1)

/-- Accept-branch payload: draw the unit slow geometric `v`, output `(v·den + u) / num`. -/
noncomputable def fastAccept (den num : ℕ+) (u : ℕ) : SLang ℕ :=
  (slowLaw 1 1) >>= fun v => SLang.probPure ((v * (den : ℕ) + u) / (num : ℕ))

/-- One SampCert-side iteration of the fast rejection loop. -/
noncomputable def fastLoopBody (den num : ℕ+) (st : Bool × ℕ) : SLang (Bool × ℕ) := do
  let D ← BernoulliExpNegSample st.2 den
  if D then do
    let m ← fastAccept den num st.2
    return (false, m)
  else do
    let u' ← UniformSample den
    return (true, u')

lemma fastAccept_apply (den num : ℕ+) (u m : ℕ) :
    fastAccept den num u m =
      ∑' v : ℕ, slowLaw 1 1 v * (if m = (v * (den : ℕ) + u) / (num : ℕ) then 1 else 0) := by
  simp only [fastAccept, Bind.bind, SLang.bind_apply, SLang.pure_apply]
  exact tsum_congr fun v => by
    by_cases h : m = (v * (den : ℕ) + u) / (num : ℕ) <;> simp [h]

/-- Body mass on a settled state: accept probability times the payload law. -/
lemma fastLoopBody_apply_settled (den num : ℕ+) (c x : ℕ) :
    fastLoopBody den num (true, c) (false, x) =
      BernoulliExpNegSample c den true * fastAccept den num c x := by
  simp only [fastLoopBody, Bind.bind, Pure.pure, SLang.bind_apply, tsum_bool,
    Bool.false_eq_true, if_false, if_true]
  have hrej : (∑' u' : ℕ, UniformSample den u' *
      SLang.probPure ((true, u') : Bool × ℕ) (false, x)) = 0 := by
    simp [SLang.probPure]
  have hacc : (∑' m' : ℕ, fastAccept den num c m' *
      SLang.probPure ((false, m') : Bool × ℕ) (false, x)) = fastAccept den num c x := by
    simp only [SLang.probPure]
    rw [tsum_eq_single x (fun m' hm' => by
      rw [if_neg (fun h => hm' (by injection h with _ h2; exact h2.symm)), mul_zero])]
    rw [if_pos rfl, mul_one]
  rw [hrej, hacc, mul_zero, zero_add]

/-- Body mass on a live state: reject probability times the uniform redraw. -/
lemma fastLoopBody_apply_live (den num : ℕ+) (c x : ℕ) :
    fastLoopBody den num (true, c) (true, x) =
      BernoulliExpNegSample c den false * UniformSample den x := by
  simp only [fastLoopBody, Bind.bind, Pure.pure, SLang.bind_apply, tsum_bool,
    Bool.false_eq_true, if_false, if_true]
  have hacc : (∑' m' : ℕ, fastAccept den num c m' *
      SLang.probPure ((false, m') : Bool × ℕ) (true, x)) = 0 := by
    simp [SLang.probPure]
  have hrej : (∑' u' : ℕ, UniformSample den u' *
      SLang.probPure ((true, u') : Bool × ℕ) (true, x)) = UniformSample den x := by
    simp only [SLang.probPure]
    rw [tsum_eq_single x (fun u' hu' => by
      rw [if_neg (fun h => hu' (by injection h with _ h2; exact h2.symm)), mul_zero])]
    rw [if_pos rfl, mul_one]
  rw [hrej, hacc, mul_zero, add_zero]

/-- One live step of the SampCert-side loop (the analogue of `geometric_succ_true`). -/
lemma fast_cut_succ_live (den num : ℕ+) (fuel c : ℕ) (st : Bool × ℕ) :
    probWhileCut geoLoopCond (fastLoopBody den num) (fuel + 1) (true, c) st =
      BernoulliExpNegSample c den true *
        (∑' m' : ℕ, fastAccept den num c m' *
          probWhileCut geoLoopCond (fastLoopBody den num) fuel (false, m') st) +
      BernoulliExpNegSample c den false *
        (∑' u' : ℕ, UniformSample den u' *
          probWhileCut geoLoopCond (fastLoopBody den num) fuel (true, u') st) := by
  rw [probWhileCut, probWhileFunctional,
    if_pos (show geoLoopCond (true, c) = true from rfl)]
  simp only [Bind.bind, SLang.bind_apply]
  rw [ENNReal.tsum_prod', tsum_bool]
  simp_rw [fastLoopBody_apply_settled, fastLoopBody_apply_live, mul_assoc]
  rw [ENNReal.tsum_mul_left, ENNReal.tsum_mul_left]

/-- A settled state freezes (the analogue of `geometric_succ_false`). -/
lemma fast_cut_succ_settled (den num : ℕ+) (fuel m' : ℕ) (st : Bool × ℕ) :
    probWhileCut geoLoopCond (fastLoopBody den num) (fuel + 1) (false, m') st =
      if st = (false, m') then 1 else 0 := by
  cases st
  simp [probWhileCut, probWhileFunctional, geoLoopCond]

/-! ### Rejection-sampling closed form (geometric series over the reject mass) -/

/-- Per-iteration accept mass mixed over a fresh uniform residue — the numerator of the
rejection-sampling closed form. -/
noncomputable def fastAcceptMixed (den num : ℕ+) (m : ℕ) : ENNReal :=
  ∑' n : ℕ, UniformSample den n * (BernoulliExpNegSample n den true * fastAccept den num n m)

/-- Per-iteration total accept probability — the normalizer. -/
noncomputable def fastAcceptMass (den : ℕ+) : ENNReal :=
  ∑' n : ℕ, UniformSample den n * BernoulliExpNegSample n den true

/-- Per-iteration total reject probability. -/
noncomputable def fastRejectMass (den : ℕ+) : ENNReal :=
  ∑' n : ℕ, UniformSample den n * BernoulliExpNegSample n den false

lemma fastRejectMass_add_acceptMass (den : ℕ+) :
    fastRejectMass den + fastAcceptMass den = 1 := by
  rw [fastRejectMass, fastAcceptMass, ← ENNReal.tsum_add]
  have hb : ∀ n : ℕ,
      UniformSample den n * BernoulliExpNegSample n den false +
        UniformSample den n * BernoulliExpNegSample n den true = UniformSample den n := by
    intro n
    rw [← mul_add]
    have hnorm := BernoulliExpNegSample_normalizes n den
    rw [tsum_bool] at hnorm
    rw [hnorm, mul_one]
  simp_rw [hb]
  exact UniformSample_normalizes den

lemma fastRejectMass_le_one (den : ℕ+) : fastRejectMass den ≤ 1 := by
  conv_rhs => rw [← fastRejectMass_add_acceptMass den]
  exact le_self_add

lemma one_sub_fastRejectMass (den : ℕ+) :
    1 - fastRejectMass den = fastAcceptMass den := by
  have h := fastRejectMass_add_acceptMass den
  have hne : fastRejectMass den ≠ ⊤ :=
    ne_top_of_le_ne_top ENNReal.one_ne_top (fastRejectMass_le_one den)
  rw [← h, ENNReal.add_sub_cancel_left hne]

/-- The mixed truncation obeys the rejection recursion
`T (k+2) = fastAcceptMixed + fastRejectMass · T (k+1)`. -/
private lemma fast_mixed_cut_rec (den num : ℕ+) (m k : ℕ) :
    (∑' n : ℕ, UniformSample den n *
      probWhileCut geoLoopCond (fastLoopBody den num) (k + 1 + 1) (true, n) (false, m)) =
    fastAcceptMixed den num m +
      fastRejectMass den *
        (∑' n : ℕ, UniformSample den n *
          probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (true, n) (false, m)) := by
  have hsettle : ∀ c : ℕ,
      (∑' m' : ℕ, fastAccept den num c m' *
        probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (false, m') (false, m)) =
      fastAccept den num c m := by
    intro c
    simp_rw [fast_cut_succ_settled den num k]
    rw [tsum_eq_single m (fun m' hm' => by
      rw [if_neg (fun h => hm' (by injection h with _ h2; exact h2.symm)), mul_zero])]
    rw [if_pos rfl, mul_one]
  simp_rw [fast_cut_succ_live den num (k + 1), hsettle, mul_add]
  rw [ENNReal.tsum_add]
  have hrejg : (∑' n : ℕ, UniformSample den n * (BernoulliExpNegSample n den false *
      (∑' u' : ℕ, UniformSample den u' *
        probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (true, u') (false, m)))) =
      fastRejectMass den *
        (∑' u' : ℕ, UniformSample den u' *
          probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (true, u') (false, m)) := by
    simp_rw [← mul_assoc]
    rw [ENNReal.tsum_mul_right]
    rfl
  rw [hrejg]
  rfl

/-- Closed form of the mixed truncation: a finite geometric series in the reject mass. -/
private lemma fast_mixed_cut_closed (den num : ℕ+) (m : ℕ) : ∀ k : ℕ,
    (∑' n : ℕ, UniformSample den n *
      probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (true, n) (false, m)) =
    fastAcceptMixed den num m * ∑ j ∈ Finset.range k, fastRejectMass den ^ j := by
  intro k
  induction k with
  | zero =>
    have h1 : ∀ n : ℕ,
        probWhileCut geoLoopCond (fastLoopBody den num) 1 (true, n) (false, m) = 0 := by
      intro n
      rw [fast_cut_succ_live den num 0]
      simp only [probWhileCut, SLang.probZero, mul_zero, tsum_zero, add_zero, zero_add]
    simp [h1]
  | succ k ih =>
    rw [fast_mixed_cut_rec den num m k, ih]
    have hgeom : (∑ j ∈ Finset.range (k + 1), fastRejectMass den ^ j) =
        1 + fastRejectMass den * ∑ j ∈ Finset.range k, fastRejectMass den ^ j := by
      rw [Finset.sum_range_succ']
      simp_rw [pow_succ']
      rw [← Finset.mul_sum, pow_zero]
      ring
    rw [hgeom]
    ring

/-- The mixed `probWhile` limit: accept-mixed mass times the geometric-series normalizer. -/
private lemma fast_mixed_probWhile (den num : ℕ+) (m : ℕ) :
    (∑' n : ℕ, UniformSample den n *
      probWhile geoLoopCond (fastLoopBody den num) (true, n) (false, m)) =
    fastAcceptMixed den num m * (fastAcceptMass den)⁻¹ := by
  simp only [probWhile]
  simp_rw [ENNReal.mul_iSup]
  rw [tsum_iSup_commute (fun n k => UniformSample den n *
      probWhileCut geoLoopCond (fastLoopBody den num) k (true, n) (false, m))
    (fun n _ _ hk => mul_le_mul_left' (probWhileCut_monotonic geoLoopCond
      (fastLoopBody den num) (true, n) (false, m) hk) _)]
  have hmono : Monotone (fun k => ∑' n : ℕ, UniformSample den n *
      probWhileCut geoLoopCond (fastLoopBody den num) k (true, n) (false, m)) := by
    intro k1 k2 hk
    exact ENNReal.tsum_le_tsum fun n =>
      mul_le_mul_left' (probWhileCut_monotonic geoLoopCond (fastLoopBody den num)
        (true, n) (false, m) hk) _
  have hshift : (⨆ k : ℕ, ∑' n : ℕ, UniformSample den n *
      probWhileCut geoLoopCond (fastLoopBody den num) k (true, n) (false, m)) =
      ⨆ k : ℕ, ∑' n : ℕ, UniformSample den n *
        probWhileCut geoLoopCond (fastLoopBody den num) (k + 1) (true, n) (false, m) := by
    refine le_antisymm (iSup_le fun k => ?_) (iSup_le fun k => le_iSup_of_le (k + 1) le_rfl)
    exact le_iSup_of_le k (hmono (Nat.le_succ k))
  rw [hshift]
  simp_rw [fast_mixed_cut_closed den num m]
  rw [← ENNReal.mul_iSup, ← ENNReal.tsum_eq_iSup_nat, ENNReal.tsum_geometric,
    one_sub_fastRejectMass den]

/-! ### Identification with SampCert's Laplace inner loop -/

/-- `In1Aux` at an accepted pair is `uniform × Bernoulli-true` (SampCert proves this inline
inside `In1Aux_apply_true`; re-exposed here in unnormalized form). -/
lemma In1Aux_true_eq (den : ℕ+) (n : ℕ) :
    DiscreteLaplaceSampleLoopIn1Aux den (n, true) =
      UniformSample den n * BernoulliExpNegSample n den true := by
  unfold DiscreteLaplaceSampleLoopIn1Aux
  simp only [Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  rw [tsum_eq_single n]
  · simp
  · intro a ha
    have hne : ¬ n = a := by simpa [eq_comm] using ha
    simp [hne]

/-- SampCert's accepted-residue law, in the unnormalized `uniform × accept / mass` form. -/
lemma In1_apply_form (den : ℕ+) (n : ℕ) :
    DiscreteLaplaceSampleLoopIn1 den n =
      UniformSample den n * BernoulliExpNegSample n den true * (fastAcceptMass den)⁻¹ := by
  rw [DiscreteLaplaceSampleLoopIn1_apply_pre, In1Aux_true_eq]
  congr 2
  exact tsum_congr fun a => In1Aux_true_eq den a

/-- Mixing the accept payload over SampCert's accepted residue gives exactly the rejection
closed form. -/
lemma tsum_In1_fastAccept (den num : ℕ+) (m : ℕ) :
    (∑' n : ℕ, DiscreteLaplaceSampleLoopIn1 den n * fastAccept den num n m) =
      fastAcceptMixed den num m * (fastAcceptMass den)⁻¹ := by
  simp_rw [In1_apply_form, mul_right_comm _ ((fastAcceptMass den)⁻¹)]
  rw [ENNReal.tsum_mul_right]
  congr 1
  exact tsum_congr fun n => by rw [mul_assoc]

/-! ### The Laplace-loop-shaped reference target and its `Geo` law -/

/-- Rust-shaped SampCert target for the whole fast sampler: accepted residue, unit geometric
count (SampCert's one-based `In2 1 1`), arithmetic combination, quotient. -/
noncomputable def fastTarget (den num : ℕ+) : SLang ℕ := do
  let U ← DiscreteLaplaceSampleLoopIn1 den
  let v ← DiscreteLaplaceSampleLoopIn2 1 1
  return ((U + (den : ℕ) * (v - 1)) / (num : ℕ))

/-- Sampling an independent fair sign bit and then projecting it away leaves the original
natural-valued distribution unchanged (ported from `proofs_legacy`). -/
private theorem bind_fair_coin_project_second_eq (m : SLang ℕ) :
    ((m >>= fun y =>
        SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl) >>= fun b =>
          Pure.pure (b, y)) >>= fun st =>
      Pure.pure st.2) = m := by
  change
    ((m.probBind fun y =>
        (SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl)).probBind fun b =>
          SLang.probPure (b, y)).probBind fun st =>
      SLang.probPure st.2) = m
  rw [SLang.bind_bind]
  simp only [SLang.bind_bind, SLang.pure_bind]
  have hcoin_const :
      ∀ y : ℕ,
        (SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl)).probBind
            (fun _ => SLang.probPure y) =
          SLang.probPure y := by
    intro y
    apply SLang.ext
    intro z
    simp only [SLang.bind_apply, SLang.pure_apply]
    rw [tsum_bool]
    by_cases hzy : z = y
    · simp [hzy]
      simpa [one_div] using (ENNReal.add_halves (1 : ENNReal))
    · simp [hzy]
  simp [hcoin_const]

/-- The reference target is SampCert's optimized Laplace inner loop with the independent sign
bit marginalized away (ported from `proofs_legacy`). -/
lemma fastTarget_eq_loop' (den num : ℕ+) :
    fastTarget den num =
      (DiscreteLaplaceSampleLoop' den num >>= fun st => Pure.pure st.2) := by
  unfold fastTarget
  unfold SLang.DiscreteLaplaceSampleLoop'
  simpa using
    (bind_fair_coin_project_second_eq
      (SLang.DiscreteLaplaceSampleLoopIn1 den >>= fun u =>
        SLang.DiscreteLaplaceSampleLoopIn2 1 1 >>= fun v =>
        let V := v - 1
        let X := u + (den : ℕ) * V
        Pure.pure (X / (num : ℕ)))).symm

/-- Closed-form `Geo` law of the reference target (ported from `proofs_legacy`, which routes
through `DiscreteLaplaceSampleLoop_equiv` and `DiscreteLaplaceSampleLoop_apply`). -/
lemma fastTarget_pmf (den num : ℕ+) (n : ℕ) :
    fastTarget den num n =
      SLang.Geo (1 - ENNReal.ofReal (Real.exp (-(((num : ℕ) : ℝ) / ((den : ℕ) : ℝ))))) n := by
  let p : ENNReal := ENNReal.ofReal (Real.exp (-(((num : ℕ) : ℝ) / ((den : ℕ) : ℝ))))
  rw [fastTarget_eq_loop']
  show (SLang.DiscreteLaplaceSampleLoop' den num >>= fun st => Pure.pure st.2) n =
    SLang.Geo (1 - p) n
  rw [← SLang.DiscreteLaplaceSampleLoop_equiv den num]
  simp only [Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  rw [ENNReal.tsum_prod']
  rw [tsum_bool]
  simp only [SLang.DiscreteLaplaceSampleLoop_apply, SLang.Geo]
  have hsingle_false :
      (∑' b : ℕ, (p ^ b * (1 - p)) * ((2 : PNat) : ENNReal)⁻¹ *
          (@ite ENNReal (n = b) (Classical.propDecidable (n = (false, b).2)) 1 0)) =
        (p ^ n * (1 - p)) * ((2 : PNat) : ENNReal)⁻¹ := by
    rw [tsum_eq_single n]
    · simp
    · intro b hb
      have hne : n ≠ b := by exact fun h => hb h.symm
      simp [hne]
  rw [hsingle_false]
  have hhalf :
      ((2 : PNat) : ENNReal)⁻¹ + ((2 : PNat) : ENNReal)⁻¹ = 1 := by
    simpa [one_div] using (ENNReal.add_halves (1 : ENNReal))
  rw [← mul_add, hhalf, mul_one]
  rw [ENNReal.sub_sub_cancel]
  · simp
  · dsimp [p]
    apply ENNReal.ofReal_le_one.mpr
    apply Real.exp_le_one_iff.mpr
    have hnonneg : 0 ≤ (((num : ℕ) : ℝ) / ((den : ℕ) : ℝ)) := by positivity
    exact neg_nonpos.mpr hnonneg

/-- The reference target, unfolded as the accepted residue mixed against the accept payload. -/
lemma fastTarget_eq_tsum_In1 (den num : ℕ+) (m : ℕ) :
    fastTarget den num m =
      ∑' n : ℕ, DiscreteLaplaceSampleLoopIn1 den n * fastAccept den num n m := by
  simp only [fastTarget, Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  refine tsum_congr fun n => ?_
  congr 1
  rw [DiscreteLaplaceSampleLoopIn2_eq, fastAccept_apply]
  have hre : ∀ F : ℕ → ENNReal,
      (∑' v : ℕ, probGeometric (BernoulliExpNegSample 1 1) v * F v) =
      (∑' v : ℕ, probGeometric (BernoulliExpNegSample 1 1) (v + 1) * F (v + 1)) := by
    intro F
    refine (Function.Injective.tsum_eq (g := Nat.succ) Nat.succ_injective ?_).symm
    intro v hv
    rcases Nat.eq_zero_or_pos v with h0 | hp
    · exact absurd (by subst h0; simp [probGeometric_apply]) hv
    · exact ⟨v - 1, Nat.succ_pred_eq_of_pos hp⟩
  rw [hre]
  refine tsum_congr fun v => ?_
  rw [Nat.add_sub_cancel]
  rw [show n + (den : ℕ) * v = v * (den : ℕ) + n from by ring]
  by_cases h : m = (v * (den : ℕ) + n) / (num : ℕ) <;> simp [h, slowLaw]

/-- The accept probability of the trial is strictly below one (positivity of `num/den`). -/
lemma bernExpNeg_true_lt_one (den num : ℕ+) :
    BernoulliExpNegSample (num : ℕ) den true < 1 := by
  rw [BernoulliExpNegSample_apply_true]
  refine ENNReal.ofReal_lt_one.mpr (Real.exp_lt_one_iff.mpr ?_)
  have hratio_pos : (0 : ℝ) < ((((num : ℕ) : NNReal)) / (((den : ℕ+) : ℕ) : NNReal) : NNReal) := by
    refine div_pos ?_ ?_
    · exact_mod_cast num.pos
    · exact_mod_cast den.pos
  have : (0 : ℝ) < ((((num : ℕ) : NNReal)) / (((den : ℕ+) : ℕ) : NNReal) : NNReal) := hratio_pos
  push_cast at this ⊢
  linarith

/-- Boolean-sum form of the trial's normalization. -/
lemma bernExpNeg_norm_bool (num : ℕ) (den : ℕ+) :
    BernoulliExpNegSample num den false + BernoulliExpNegSample num den true = 1 := by
  have h := BernoulliExpNegSample_normalizes num den
  rwa [tsum_bool] at h

/-- **The reference target is the slow-geometric law** — the fast construction samples the same
distribution as stage 6. -/
lemma fastTarget_eq_slowLaw (den num : ℕ+) :
    fastTarget den num = slowLaw (num : ℕ) den := by
  funext m
  have hgeo := probGeometric_apply_Geo (BernoulliExpNegSample (num : ℕ) den)
    (bernExpNeg_norm_bool (num : ℕ) den) (bernExpNeg_true_lt_one den num) (m + 1)
  rw [if_neg (Nat.succ_ne_zero m), Nat.add_sub_cancel] at hgeo
  show fastTarget den num m = probGeometric (BernoulliExpNegSample (num : ℕ) den) (m + 1)
  rw [hgeo, fastTarget_pmf den num m, BernoulliExpNegSample_apply_true]
  congr 3 <;> (try push_cast) <;> (try ring)

/-! ### Extracted-side fiber laws -/

open OpenDP.samplers.uniform (samplerDist_nat uniformNatBelowPMF sample_uniform_ubig_below_pmf)
open OpenDP.samplers.bernoulli (BernoulliExpSetup sample_bernoulli_exp_spec)

/-- Push a `ubigToNat`-factored weight through a `UBig`-valued program: summing the raw
distribution against `g ∘ ubigToNat` equals summing the `ℕ`-pushforward against `g`. -/
lemma tsum_samplerDist_nat
    (prog : Result (core.result.Result dashu_int.ubig.UBig error.Error)) (g : ℕ → ENNReal) :
    (∑' w : dashu_int.ubig.UBig, samplerDist prog w * g (dashu.ubigToNat w)) =
      ∑' n : ℕ, samplerDist_nat prog n * g n :=
  tsum_samplerDist_comp prog dashu.ubigToNat g

/-- The mathematical value 1 collapses the trial indices to SampCert's `1 / 1`. -/
lemma bernExpNeg_collapse_one (mval : ℕ) (hpos : 0 < mval) (hm : mval = 1) :
    BernoulliExpNegSample mval ⟨mval, hpos⟩ = BernoulliExpNegSample 1 1 := by
  subst hm; rfl

/-- The stage-6 law of the extracted slow sampler at the constant rational `1`, collapsed to
`slowLaw 1 1`. -/
lemma samplerDist_nat_slow_one (oneRat : dashu_ratio.rbig.RBig)
    (oneI : dashu_int.ibig.IBig) (oneU : dashu_int.ubig.UBig)
    (hparts : dashu_ratio.rbig.RBig.into_parts oneRat = ok (oneI, oneU))
    (hsign : dashu_int.ibig.IBig.into_parts oneI = ok (dashu_base.sign.Sign.Positive, oneU))
    (honeval : dashu.ubigToNat oneU = 1) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat) = slowLaw 1 1 := by
  have h1pos : 0 < dashu.ubigToNat oneU := by rw [honeval]; norm_num
  rw [sample_geometric_exp_slow_spec oneRat ⟨oneI, oneU, oneU, hparts, hsign, h1pos⟩]
  funext v
  show probGeometric (geoTrial oneU oneU h1pos) (v + 1) = _
  rw [show geoTrial oneU oneU h1pos = BernoulliExpNegSample 1 1 from
    bernExpNeg_collapse_one (dashu.ubigToNat oneU) h1pos honeval]
  rfl

/-- The residue rational built at the head of the fast loop drives the SampCert
`Bernoulli(e^{-u/denom})` trial (with the trial's denominator collapsed to the loop's). -/
lemma residue_bern (denom u : dashu_int.ubig.UBig) (i i1 : dashu_int.ibig.IBig)
    (u1 : dashu_int.ubig.UBig) (r : dashu_ratio.rbig.RBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hIparts : dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u))
    (hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hF : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r) (b : Bool) :
    samplerDistGen (samplers.bernoulli.sample_bernoulli_exp r) (core.result.Result.Ok b) =
      BernoulliExpNegSample (dashu.ubigToNat u) ⟨dashu.ubigToNat denom, hdenom⟩ b := by
  have hsign_i1 : dashu_int.ibig.IBig.into_parts i1 =
      ok (dashu_base.sign.Sign.Positive, u) :=
    dashu.ibig_clone_parts_spec i i1 _ hC hIparts
  have hval_u1 : dashu.ubigToNat u1 = dashu.ubigToNat denom := dashu.clone_spec denom u1 hD
  have hd1 : 0 < dashu.ubigToNat u1 := by rw [hval_u1]; exact hdenom
  have hparts_r : dashu_ratio.rbig.RBig.into_parts r = ok (i1, u1) :=
    dashu.rbig_from_parts_positive_spec u u1 i1 r hd1 hsign_i1 hF
  have h := congrFun (sample_bernoulli_exp_spec r ⟨i1, u1, u, hparts_r, hsign_i1, hd1⟩) b
  have hcast : (⟨dashu.ubigToNat u1, hd1⟩ : ℕ+) = ⟨dashu.ubigToNat denom, hdenom⟩ :=
    Subtype.ext hval_u1
  rw [hcast] at h
  simpa [samplerDist] using h

/-- The post-Bernoulli continuation of one fast-loop iteration (mirrors `fast_body` after the
residue trial). -/
noncomputable def fast_step (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (u u1 : dashu_int.ubig.UBig) :
    core.result.Result Bool error.Error →
    Result (ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error)) :=
  fun r1 => do
    let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r1
    match cf with
    | core.ops.control_flow.ControlFlow.Continue val =>
      if val
      then do
        let r2 ← dashu_ratio.rbig.RBig.ONE
        let r3 ← samplers.geometric.sample_geometric_exp_slow r2
        let cf1 ←
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r3
        match cf1 with
        | core.ops.control_flow.ControlFlow.Continue val1 => do
          let u2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
          let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
          let (_, u4) ← dashu_int.ibig.IBig.into_parts numerSigned
          let u5 ← dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
          ok (done (core.result.Result.Ok u5))
        | core.ops.control_flow.ControlFlow.Break residual => do
          let w ←
            core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
              dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
          ok (done w)
      else do
        let r2 ← samplers.uniform.sample_uniform_ubig_below u1
        let cf1 ←
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r2
        match cf1 with
        | core.ops.control_flow.ControlFlow.Continue val1 => ok (cont val1)
        | core.ops.control_flow.ControlFlow.Break residual => do
          let w ←
            core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
              dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
          ok (done w)
    | core.ops.control_flow.ControlFlow.Break residual => do
      let w ←
        core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
          dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
      ok (done w)

/-- Once the deterministic head succeeds, `fast_body` factors through `fast_step`. -/
lemma fast_body_eq_step (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (u : dashu_int.ubig.UBig) (i i1 : dashu_int.ibig.IBig) (u1 : dashu_int.ubig.UBig)
    (r : dashu_ratio.rbig.RBig)
    (hI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hF : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r) :
    fast_body denom numerSigned u =
      samplers.bernoulli.sample_bernoulli_exp r >>= fast_step denom numerSigned u u1 := by
  unfold fast_body
  rw [hI]; simp only [bind_tc_ok]
  rw [hC]; simp only [bind_tc_ok]
  rw [hD]; simp only [bind_tc_ok]
  rw [hF]; simp only [bind_tc_ok]
  rfl

/-- Step on `Err e`: a point mass at `done (Err e)`. -/
lemma fast_step_err (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (u u1 : dashu_int.ubig.UBig) (e : error.Error)
    (out : ControlFlow dashu_int.ubig.UBig
      (core.result.Result dashu_int.ubig.UBig error.Error)) :
    samplerDistGen (fast_step denom numerSigned u u1 (core.result.Result.Err e)) out =
      (if out = done (core.result.Result.Err e) then 1 else 0) := by
  simp [fast_step, core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    from_residual_err_ok, samplerDistGen_pure_ok, PMF.pure_apply]

/-- Reject dispatch of the step, in clean `>>=` form. -/
private lemma fast_step_false_eq (denom : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig) (u u1 : dashu_int.ubig.UBig) :
    fast_step denom numerSigned u u1 (core.result.Result.Ok false) =
      (samplers.uniform.sample_uniform_ubig_below u1 >>= fun r2 =>
        match r2 with
        | core.result.Result.Ok val1 => ok (cont val1)
        | core.result.Result.Err e => ok (done (core.result.Result.Err e))) := by
  unfold fast_step
  simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    bind_tc_ok, Bool.false_eq_true, if_false]
  congr 1
  funext r2
  rcases r2 with val1 | e
  · rfl
  · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
      from_residual_err_ok]

/-- Accept dispatch of the step, in clean `>>=` form. -/
private lemma fast_step_true_eq (denom : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig) (u u1 : dashu_int.ubig.UBig)
    (oneRat : dashu_ratio.rbig.RBig) (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat) :
    fast_step denom numerSigned u u1 (core.result.Result.Ok true) =
      (samplers.geometric.sample_geometric_exp_slow oneRat >>= fun r3 =>
        match r3 with
        | core.result.Result.Ok val1 => do
          let u2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
          let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
          let (_, u4) ← dashu_int.ibig.IBig.into_parts numerSigned
          let u5 ← dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4
          ok (done (core.result.Result.Ok u5))
        | core.result.Result.Err e => ok (done (core.result.Result.Err e))) := by
  unfold fast_step
  simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    bind_tc_ok, if_true]
  rw [hone]
  simp only [bind_tc_ok]
  congr 1
  funext r3
  rcases r3 with val1 | e
  · rfl
  · simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
      from_residual_err_ok]

/-- Accept dispatch with the (deterministic) numerator destructuring pre-reduced — removes the
pattern-`let` on `into_parts numerSigned` that blocks `simp only`-driven reduction downstream. -/
private lemma fast_step_true_eq' (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (u u1 : dashu_int.ubig.UBig)
    (oneRat : dashu_ratio.rbig.RBig) (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat) :
    fast_step denom numerSigned u u1 (core.result.Result.Ok true) =
      (samplers.geometric.sample_geometric_exp_slow oneRat >>= fun r3 =>
        match r3 with
        | core.result.Result.Ok val1 => do
          let u2 ← dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom
          let u3 ← dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u
          let u5 ← dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 numer
          ok (done (core.result.Result.Ok u5))
        | core.result.Result.Err e => ok (done (core.result.Result.Err e))) := by
  rw [fast_step_true_eq denom numerSigned u u1 oneRat hone]
  congr 1
  funext r3
  rcases r3 with val1 | e
  · simp [hsign]
  · rfl

/-- Reject step's `cont` mass is the uniform redraw. -/
lemma fast_step_false_cont (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (u u1 a : dashu_int.ubig.UBig) :
    samplerDistGen (fast_step denom numerSigned u u1 (core.result.Result.Ok false)) (cont a) =
      samplerDist (samplers.uniform.sample_uniform_ubig_below u1) a := by
  rw [fast_step_false_eq, samplerDistGen_bind, SLang.probBind]
  rw [tsum_result_ok_eq (fun e => by simp [samplerDistGen_pure_ok, PMF.pure_apply])]
  simp only [samplerDistGen_pure_ok, PMF.pure_apply]
  rw [tsum_eq_single a (fun v hv => by
    rw [if_neg (fun h => hv (ControlFlow.cont.inj h).symm), mul_zero])]
  rw [if_pos rfl, mul_one]
  rfl

/-- Reject step never settles on an `Ok` value. -/
lemma fast_step_false_done (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (u u1 w : dashu_int.ubig.UBig) :
    samplerDistGen (fast_step denom numerSigned u u1 (core.result.Result.Ok false))
      (done (core.result.Result.Ok w)) = 0 := by
  rw [fast_step_false_eq, samplerDistGen_bind, SLang.probBind]
  refine ENNReal.tsum_eq_zero.mpr fun r2 => ?_
  rcases r2 with val1 | e <;> simp [samplerDistGen_pure_ok, PMF.pure_apply]

/-- Accept step never continues. -/
lemma fast_step_true_cont (denom : dashu_int.ubig.UBig) (numerSigned : dashu_int.ibig.IBig)
    (numer : dashu_int.ubig.UBig)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hpos : 0 < dashu.ubigToNat numer)
    (u u1 : dashu_int.ubig.UBig) (oneRat : dashu_ratio.rbig.RBig)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat) (a : dashu_int.ubig.UBig) :
    samplerDistGen (fast_step denom numerSigned u u1 (core.result.Result.Ok true))
      (cont a) = 0 := by
  rw [fast_step_true_eq denom numerSigned u u1 oneRat hone, samplerDistGen_bind, SLang.probBind]
  refine ENNReal.tsum_eq_zero.mpr fun r3 => ?_
  rcases r3 with val1 | e
  · obtain ⟨u2, hmul, _⟩ := dashu.mul_ubig_exists_spec val1 denom
    obtain ⟨u3, hadd, _⟩ := dashu.add_ubig_exists_spec u2 u
    obtain ⟨u5, hdiv, _⟩ := dashu.div_ubig_floor_exists_spec u3 numer hpos
    simp [hmul, hadd, hsign, hdiv, samplerDistGen_pure_ok, PMF.pure_apply]
  · simp [samplerDistGen_pure_ok, PMF.pure_apply]

/-- Accept step's settle mass, summed against the output-value indicator: the stage-6 slow law
at `1`, pushed through the floor-division arithmetic. -/
lemma fast_step_true_done_summed (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hpos : 0 < dashu.ubigToNat numer)
    (u u1 : dashu_int.ubig.UBig) (oneRat : dashu_ratio.rbig.RBig)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (den num : ℕ+) (hden : (den : ℕ) = dashu.ubigToNat denom)
    (hnum : (num : ℕ) = dashu.ubigToNat numer) (m : ℕ) :
    (∑' w : dashu_int.ubig.UBig,
      samplerDistGen (fast_step denom numerSigned u u1 (core.result.Result.Ok true))
          (done (core.result.Result.Ok w)) *
        (if m = dashu.ubigToNat w then 1 else 0)) =
      ∑' v : ℕ, samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat) v *
        (if m = (v * (den : ℕ) + dashu.ubigToNat u) / (num : ℕ) then 1 else 0) := by
  simp_rw [fast_step_true_eq' denom numer numerSigned hsign u u1 oneRat hone,
    samplerDistGen_bind, SLang.probBind, ← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  rw [tsum_result_ok_eq (fun e => by
    refine ENNReal.tsum_eq_zero.mpr fun w => ?_
    simp [samplerDistGen_pure_ok, PMF.pure_apply])]
  refine Eq.trans (tsum_congr fun val1 => ?_)
    (tsum_samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat)
      (fun v => if m = (v * (den : ℕ) + dashu.ubigToNat u) / (num : ℕ) then 1 else 0))
  obtain ⟨u2, hmul, hu2⟩ := dashu.mul_ubig_exists_spec val1 denom
  obtain ⟨u3, hadd, hu3⟩ := dashu.add_ubig_exists_spec u2 u
  obtain ⟨u5, hdiv, hu5⟩ := dashu.div_ubig_floor_exists_spec u3 numer hpos
  have hval : dashu.ubigToNat u5 =
      (dashu.ubigToNat val1 * (den : ℕ) + dashu.ubigToNat u) / (num : ℕ) := by
    rw [hu5, hu3, hu2, hden, hnum]
  simp only [hmul, hadd, hdiv, bind_tc_ok, samplerDistGen_pure_ok, PMF.pure_apply]
  rw [tsum_eq_single u5 (fun w hw => by
    rw [if_neg (fun h => by
      injection h with h1
      injection h1 with h2
      exact hw h2), mul_zero, zero_mul])]
  rw [if_pos rfl, mul_one, hval]
  rfl

/-- The body's `cont` mass: reject probability times the uniform redraw. -/
lemma fast_body_cont_mass (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hpos : 0 < dashu.ubigToNat numer)
    (u : dashu_int.ubig.UBig) (i i1 : dashu_int.ibig.IBig) (u1 : dashu_int.ubig.UBig)
    (r oneRat : dashu_ratio.rbig.RBig)
    (hI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hIparts : dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u))
    (hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hF : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (a : dashu_int.ubig.UBig) :
    samplerDistGen (fast_body denom numerSigned u) (cont a) =
      BernoulliExpNegSample (dashu.ubigToNat u) ⟨dashu.ubigToNat denom, hdenom⟩ false *
        samplerDist (samplers.uniform.sample_uniform_ubig_below u1) a := by
  rw [fast_body_eq_step denom numerSigned u i i1 u1 r hI hC hD hF,
    samplerDistGen_bind, SLang.probBind,
    tsum_result_ok_eq (fun e => by rw [fast_step_err]; simp),
    tsum_bool, fast_step_false_cont,
    fast_step_true_cont denom numerSigned numer hsign hpos u u1 oneRat hone a,
    residue_bern denom u i i1 u1 r hdenom hIparts hC hD hF false]
  rw [mul_zero, add_zero]

/-- The body's settle mass summed against the output-value indicator: accept probability times
the arithmetic-pushed slow law. -/
lemma fast_body_done_summed (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hpos : 0 < dashu.ubigToNat numer)
    (u : dashu_int.ubig.UBig) (i i1 : dashu_int.ibig.IBig) (u1 : dashu_int.ubig.UBig)
    (r oneRat : dashu_ratio.rbig.RBig)
    (hI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hIparts : dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u))
    (hC : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hF : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (den num : ℕ+) (hden : (den : ℕ) = dashu.ubigToNat denom)
    (hnum : (num : ℕ) = dashu.ubigToNat numer) (m : ℕ) :
    (∑' w : dashu_int.ubig.UBig,
      samplerDistGen (fast_body denom numerSigned u) (done (core.result.Result.Ok w)) *
        (if m = dashu.ubigToNat w then 1 else 0)) =
      BernoulliExpNegSample (dashu.ubigToNat u) ⟨dashu.ubigToNat denom, hdenom⟩ true *
        (∑' v : ℕ, samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat) v *
          (if m = (v * (den : ℕ) + dashu.ubigToNat u) / (num : ℕ) then 1 else 0)) := by
  simp_rw [fast_body_eq_step denom numerSigned u i i1 u1 r hI hC hD hF,
    samplerDistGen_bind, SLang.probBind, ← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  simp_rw [mul_assoc, ENNReal.tsum_mul_left]
  rw [tsum_result_ok_eq (fun e => by
    simp_rw [fast_step_err]
    simp)]
  rw [tsum_bool]
  simp_rw [fast_step_false_done, zero_mul, tsum_zero, mul_zero, zero_add]
  rw [fast_step_true_done_summed denom numer numerSigned hsign hpos u u1 oneRat hone
      den num hden hnum m,
    residue_bern denom u i i1 u1 r hdenom hIparts hC hD hF true]

/-- The uniform draw below a clone of `denom`, pushed to `ℕ`, is SampCert's `UniformSample`. -/
lemma samplerDist_nat_uniform_clone (denom u1 : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hu1pos : 0 < dashu.ubigToNat u1) (x : ℕ) :
    samplerDist_nat (samplers.uniform.sample_uniform_ubig_below u1) x =
      UniformSample ⟨dashu.ubigToNat denom, hdenom⟩ x := by
  rw [sample_uniform_ubig_below_pmf u1 hu1pos]
  have hcast : (⟨dashu.ubigToNat u1, hu1pos⟩ : ℕ+) = ⟨dashu.ubigToNat denom, hdenom⟩ :=
    Subtype.ext (dashu.clone_spec denom u1 hD)
  show (uniformNatBelowPMF u1 hu1pos : PMF ℕ) x = _
  rw [show uniformNatBelowPMF u1 hu1pos = SLang.UniformSample_PMF ⟨dashu.ubigToNat u1, hu1pos⟩
    from rfl, hcast]
  rfl

/-- **Cut-depth correspondence for the fast loop.** At every depth, the extracted loop's settle
mass (summed against the output-value indicator) equals the SampCert-side model's mass on the
settled state. -/
private lemma geo_fast_loop_cut_step (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig) (u1 : dashu_int.ubig.UBig)
    (oneRat : dashu_ratio.rbig.RBig)
    (hdenom : 0 < dashu.ubigToNat denom) (hpos : 0 < dashu.ubigToNat numer)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hu1pos : 0 < dashu.ubigToNat u1)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (hslowLaw : samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat) =
      slowLaw 1 1)
    (n : ℕ) :
    ∀ (u : dashu_int.ubig.UBig) (m : ℕ),
      (∑' w : dashu_int.ubig.UBig,
        probWhileCut loopCond (loopBd (fast_body denom numerSigned)) n (cont u)
          (done (core.result.Result.Ok w)) *
          (if m = dashu.ubigToNat w then 1 else 0)) =
      probWhileCut geoLoopCond
        (fastLoopBody ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩) n
        (true, dashu.ubigToNat u) (false, m) := by
  induction n with
  | zero =>
    intro u m
    simp only [probWhileCut, SLang.probZero, zero_mul, tsum_zero]
  | succ n ih =>
    intro u m
    obtain ⟨i, hI, hIparts⟩ := dashu.as_ibig_exists_spec u
    obtain ⟨i1, hC⟩ := dashu.ibig_clone_exists_spec i
    have hsign_i1 : dashu_int.ibig.IBig.into_parts i1 =
        ok (dashu_base.sign.Sign.Positive, u) :=
      dashu.ibig_clone_parts_spec i i1 _ hC hIparts
    obtain ⟨r, hF, hRparts⟩ :=
      dashu.rbig_from_parts_positive_exists_spec u u1 i1 hu1pos hsign_i1
    cases n with
    | zero =>
      have hz : ∀ w : dashu_int.ubig.UBig,
          probWhileCut loopCond (loopBd (fast_body denom numerSigned)) 1 (cont u)
            (done (core.result.Result.Ok w)) = 0 := by
        intro w
        rw [probWhileCut, probWhileFunctional, if_pos (loopCond_cont u)]
        simp only [Bind.bind, SLang.bind_apply, probWhileCut, SLang.probZero, mul_zero,
          tsum_zero]
      simp only [hz, zero_mul, tsum_zero]
      rw [fast_cut_succ_live]
      simp only [probWhileCut, SLang.probZero, mul_zero, tsum_zero, add_zero]
    | succ n' =>
      have hunf : ∀ w : dashu_int.ubig.UBig,
          probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1 + 1) (cont u)
              (done (core.result.Result.Ok w)) =
            (∑' a : dashu_int.ubig.UBig,
              loopBd (fast_body denom numerSigned) (cont u) (cont a) *
              probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1) (cont a)
                (done (core.result.Result.Ok w))) +
            (∑' r' : core.result.Result dashu_int.ubig.UBig error.Error,
              loopBd (fast_body denom numerSigned) (cont u) (done r') *
                probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1)
                  (done r') (done (core.result.Result.Ok w))) := by
        intro w
        rw [probWhileCut, probWhileFunctional, if_pos (loopCond_cont u)]
        simp only [Bind.bind, SLang.bind_apply]
        rw [tsum_controlFlow]
      simp_rw [hunf, add_mul]
      rw [ENNReal.tsum_add]
      have hcont : (∑' w : dashu_int.ubig.UBig,
          (∑' a : dashu_int.ubig.UBig, loopBd (fast_body denom numerSigned) (cont u) (cont a) *
            probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1) (cont a)
              (done (core.result.Result.Ok w))) *
            (if m = dashu.ubigToNat w then 1 else 0)) =
          BernoulliExpNegSample (dashu.ubigToNat u) ⟨dashu.ubigToNat denom, hdenom⟩ false *
            (∑' x : ℕ, UniformSample ⟨dashu.ubigToNat denom, hdenom⟩ x *
              probWhileCut geoLoopCond
                (fastLoopBody ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩)
                (n' + 1) (true, x) (false, m)) := by
        simp_rw [← ENNReal.tsum_mul_right]
        rw [ENNReal.tsum_comm]
        simp_rw [loopBd_cont,
          fast_body_cont_mass denom numer numerSigned hdenom hsign hpos u i i1 u1 r oneRat
            hI hIparts hC hD hF hone,
          mul_assoc, ENNReal.tsum_mul_left, ih]
        congr 1
        rw [tsum_samplerDist_nat (samplers.uniform.sample_uniform_ubig_below u1)
          (fun x => probWhileCut geoLoopCond
            (fastLoopBody ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩)
            (n' + 1) (true, x) (false, m))]
        exact tsum_congr fun x => by
          rw [samplerDist_nat_uniform_clone denom u1 hdenom hD hu1pos x]
      have hdone : (∑' w : dashu_int.ubig.UBig,
          (∑' r' : core.result.Result dashu_int.ubig.UBig error.Error,
            loopBd (fast_body denom numerSigned) (cont u) (done r') *
              probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1) (done r')
                (done (core.result.Result.Ok w))) *
            (if m = dashu.ubigToNat w then 1 else 0)) =
          BernoulliExpNegSample (dashu.ubigToNat u) ⟨dashu.ubigToNat denom, hdenom⟩ true *
            fastAccept ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩
              (dashu.ubigToNat u) m := by
        have hcol : ∀ w : dashu_int.ubig.UBig,
            (∑' r' : core.result.Result dashu_int.ubig.UBig error.Error,
              loopBd (fast_body denom numerSigned) (cont u) (done r') *
                probWhileCut loopCond (loopBd (fast_body denom numerSigned)) (n' + 1) (done r')
                  (done (core.result.Result.Ok w))) =
            loopBd (fast_body denom numerSigned) (cont u) (done (core.result.Result.Ok w)) := by
          intro w
          simp_rw [probWhileCut_done_pt loopCond (loopBd (fast_body denom numerSigned)) (fun _ => rfl) n',
            SLang.pure_apply]
          rw [tsum_eq_single (core.result.Result.Ok w) (fun r' hr' => by
            rw [if_neg (fun h => hr' ((ControlFlow.done.inj h).symm)), mul_zero]),
            if_pos rfl, mul_one]
        simp_rw [hcol, loopBd_cont]
        rw [fast_body_done_summed denom numer numerSigned hdenom hsign hpos u i i1 u1 r oneRat
          hI hIparts hC hD hF hone ⟨dashu.ubigToNat denom, hdenom⟩
          ⟨dashu.ubigToNat numer, hpos⟩ rfl rfl m]
        congr 1
        rw [hslowLaw, ← fastAccept_apply]
      rw [hcont, hdone]
      conv_rhs => rw [fast_cut_succ_live]
      have hsettle : (∑' m' : ℕ,
          fastAccept ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩
            (dashu.ubigToNat u) m' *
          probWhileCut geoLoopCond
            (fastLoopBody ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩)
            (n' + 1) (false, m') (false, m)) =
          fastAccept ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩
            (dashu.ubigToNat u) m := by
        simp_rw [fast_cut_succ_settled]
        rw [tsum_eq_single m (fun m' hm' => by
          rw [if_neg (fun h => hm' (by injection h with _ h2; exact h2.symm)), mul_zero]),
          if_pos rfl, mul_one]
      rw [hsettle]
      ring

/-- Lift of the cut correspondence to the full `probWhile`. -/
private lemma geo_fast_loop_probWhile (denom numer : dashu_int.ubig.UBig)
    (numerSigned : dashu_int.ibig.IBig) (u1 : dashu_int.ubig.UBig)
    (oneRat : dashu_ratio.rbig.RBig)
    (hdenom : 0 < dashu.ubigToNat denom) (hpos : 0 < dashu.ubigToNat numer)
    (hsign : dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer))
    (hD : dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hu1pos : 0 < dashu.ubigToNat u1)
    (hone : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (hslowLaw : samplerDist_nat (samplers.geometric.sample_geometric_exp_slow oneRat) =
      slowLaw 1 1)
    (u0 : dashu_int.ubig.UBig) (m : ℕ) :
    samplerDist_nat (Aeneas.Std.loop (fast_body denom numerSigned) u0) m =
      probWhile geoLoopCond
        (fastLoopBody ⟨dashu.ubigToNat denom, hdenom⟩ ⟨dashu.ubigToNat numer, hpos⟩)
        (true, dashu.ubigToNat u0) (false, m) := by
  have hexpand : samplerDist_nat (Aeneas.Std.loop (fast_body denom numerSigned) u0) m =
      ∑' w : dashu_int.ubig.UBig,
        samplerDist (Aeneas.Std.loop (fast_body denom numerSigned) u0) w *
          (if m = dashu.ubigToNat w then 1 else 0) :=
    probBind_pure_apply _ _ m
  rw [hexpand, tsum_samplerDist_loop]
  simp only [probWhile]
  exact iSup_congr fun n =>
    geo_fast_loop_cut_step denom numer numerSigned u1 oneRat hdenom hpos hsign hD hu1pos
      hone hslowLaw n u0 m

/-- Push `samplerDist_nat` through a bind whose continuation propagates errors. -/
private lemma samplerDist_nat_bind
    (p : Result (core.result.Result dashu_int.ubig.UBig error.Error))
    (k : core.result.Result dashu_int.ubig.UBig error.Error →
      Result (core.result.Result dashu_int.ubig.UBig error.Error))
    (hkerr : ∀ e, k (core.result.Result.Err e) = ok (core.result.Result.Err e)) (m : ℕ) :
    samplerDist_nat (p >>= k) m =
      ∑' v : dashu_int.ubig.UBig,
        samplerDist p v * samplerDist_nat (k (core.result.Result.Ok v)) m := by
  simp only [samplerDist_nat, SLang.probBind, samplerDist, samplerDistGen_bind]
  simp_rw [← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  rw [tsum_result_ok_eq (fun e => by
    refine ENNReal.tsum_eq_zero.mpr fun w => ?_
    rw [hkerr e]
    simp [samplerDistGen_pure_ok, PMF.pure_apply])]
  simp_rw [mul_assoc, ENNReal.tsum_mul_left]

/-- **Distributional correctness (roadmap stage 7).** On strictly positive inputs, the extracted
`sample_geometric_exp_fast` realises the *same* geometric law as the slow sampler (stage 6):
`probGeometric` over the `Bernoulli(e^{-x})` trial, shifted by the final failing draw. -/
theorem sample_geometric_exp_fast_spec (x : dashu_ratio.rbig.RBig)
    (setup : OpenDP.samplers.bernoulli.BernoulliExpSetup x)
    (hpos : 0 < dashu.ubigToNat setup.numer) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_fast x) =
      fun m => probGeometric (geoTrial setup.numer setup.denom setup.hdenom) (m + 1) := by
  obtain ⟨u1, hD, hu1val⟩ := dashu.clone_exists_spec setup.denom
  have hu1pos : 0 < dashu.ubigToNat u1 := by rw [hu1val]; exact setup.hdenom
  obtain ⟨oneRat, oneI, oneU, honeRat, honeU, honeParts, honeSign⟩ := dashu.rbig_one_setup_spec
  have honeval : dashu.ubigToNat oneU = 1 := dashu.one_spec oneU honeU
  have hslowLaw := samplerDist_nat_slow_one oneRat oneI oneU honeParts honeSign honeval
  funext m
  rw [sample_geometric_exp_fast_eq_of_pos x setup hpos, hD]
  simp only [bind_tc_ok]
  rw [samplerDist_nat_bind (samplers.uniform.sample_uniform_ubig_below u1) _
    (fun e => by
      simp [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
        from_residual_err_ok]) m]
  simp only [core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    bind_tc_ok]
  simp_rw [geo_fast_loop_probWhile setup.denom setup.numer setup.numerSigned u1 oneRat
    setup.hdenom hpos setup.hsign hD hu1pos honeRat hslowLaw]
  rw [tsum_samplerDist_nat (samplers.uniform.sample_uniform_ubig_below u1)
    (fun n => probWhile geoLoopCond
      (fastLoopBody ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
        ⟨dashu.ubigToNat setup.numer, hpos⟩) (true, n) (false, m))]
  simp_rw [samplerDist_nat_uniform_clone setup.denom u1 setup.hdenom hD hu1pos]
  rw [fast_mixed_probWhile, ← tsum_In1_fastAccept, ← fastTarget_eq_tsum_In1,
    fastTarget_eq_slowLaw]
  rfl

/-- Zero-input branch: the fast sampler is a point mass at `0` (the slow sampler does not
terminate at `x = 0`, so the Rust implementation special-cases it). -/
theorem sample_geometric_exp_fast_zero_spec (x : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.RBig.is_zero x = ok true) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_fast x) = SLang.probPure 0 := by
  rw [sample_geometric_exp_fast_eq_zero x hzero]
  obtain ⟨z, hz, hzval⟩ := dashu.zero_exists_spec
  rw [hz]
  simp only [bind_tc_ok]
  funext m
  simp only [samplerDist_nat, SLang.probBind, samplerDist, samplerDistGen_pure_ok,
    PMF.pure_apply, SLang.probPure]
  rw [tsum_eq_single z (fun w hw => by
    rw [if_neg (fun h => hw (core.result.Result.Ok.inj h)), zero_mul])]
  rw [if_pos rfl, one_mul, hzval]

end OpenDP.samplers.geometric
