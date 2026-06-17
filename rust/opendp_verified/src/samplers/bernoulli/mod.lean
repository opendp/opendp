import Aeneas
import Generated.OpenDP
import src.samplers.bernoulli.semantics
import src.samplers.uniform.mod

open Aeneas Aeneas.Std Result
open OpenDP

namespace OpenDP.samplers.bernoulli

/-- The extracted `numer > denom` check in `sample_bernoulli_rational` proves
the valid-input arithmetic side condition needed by the Bernoulli law. -/
theorem sample_bernoulli_rational_prob_le
    (prob : dashu_ratio.rbig.RBig)
    (setup : BernoulliRationalSetup prob) :
    dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom :=
  dashu.gt_false_spec setup.numer setup.denom setup.hvalid

/-- Once the deterministic validity checks of `sample_bernoulli_rational`
succeed, the extracted function reduces to uniform sampling below the
denominator followed by the Rust comparison closure. -/
theorem sample_bernoulli_rational_eq_of_setup
    (prob : dashu_ratio.rbig.RBig)
    (setup : BernoulliRationalSetup prob) :
    samplers.bernoulli.sample_bernoulli_rational prob =
      (do
        let r ← samplers.uniform.sample_uniform_ubig_below setup.denom
        core.result.Result.map
          samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
          r setup.numer) := by
  unfold samplers.bernoulli.sample_bernoulli_rational
  simp [setup.hparts, setup.hsign, setup.hvalid]

/-- Primary mathematical result for `sample_bernoulli_rational`: on the
valid-input branch, the extracted function reduces to a comparison against a
uniform sample below the denominator, and the target success law is exactly
SampCert's canonical Bernoulli PMF. -/
theorem sample_bernoulli_rational_spec
    (prob : dashu_ratio.rbig.RBig)
    (setup : BernoulliRationalSetup prob)
    (hdenom : 0 < dashu.ubigToNat setup.denom) :
    ∃ uniformSetup : OpenDP.samplers.uniform.UniformBelowSetup setup.denom,
      OpenDP.samplers.uniform.sample_uniform_ubig_below_setup setup.denom =
        ok uniformSetup ∧
      samplers.bernoulli.sample_bernoulli_rational prob =
        (do
          let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len
          let r ←
            samplers.uniform.sample_uniform_ubig_below_loop setup.denom
              uniformSetup.threshold buffer
          core.result.Result.map
            samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
            r setup.numer) ∧
      dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom ∧
      bernoulliPMF setup.numer setup.denom hdenom =
        SLang.BernoulliSamplePMF
          (dashu.ubigToNat setup.numer)
          ⟨dashu.ubigToNat setup.denom, hdenom⟩
          (sample_bernoulli_rational_prob_le prob setup) := by
  rcases OpenDP.samplers.uniform.sample_uniform_ubig_below_spec setup.denom hdenom with
    ⟨uniformSetup, huniformSetup, hsample, _⟩
  refine ⟨uniformSetup, huniformSetup, ?_, sample_bernoulli_rational_prob_le prob setup, ?_⟩
  calc
    samplers.bernoulli.sample_bernoulli_rational prob
      =
        (do
          let r ← samplers.uniform.sample_uniform_ubig_below setup.denom
          core.result.Result.map
            samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
            r setup.numer) := sample_bernoulli_rational_eq_of_setup prob setup
    _ =
        (do
          let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len
          let r ←
            samplers.uniform.sample_uniform_ubig_below_loop setup.denom
              uniformSetup.threshold buffer
          core.result.Result.map
            samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
            r setup.numer) := by
          rw [hsample]
          simpa using
            (Aeneas.Std.bind_assoc_eq
              (alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len)
              (fun buffer =>
                samplers.uniform.sample_uniform_ubig_below_loop setup.denom
                  uniformSetup.threshold buffer)
              (fun r =>
                core.result.Result.map
                  samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
                  r setup.numer))
  exact bernoulliPMF_eq_BernoulliSamplePMF
    setup.numer setup.denom hdenom (sample_bernoulli_rational_prob_le prob setup)

/-- One `exp1` Bernoulli step reduces to a valid rational Bernoulli subproblem
whose success law is the canonical SampCert Bernoulli step at denominator
`k * denom`. -/
theorem sample_bernoulli_exp1_step_spec
    (numer denom k : dashu_int.ubig.UBig)
    (hk : 0 < dashu.ubigToNat k)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    ∃ x_div_k : dashu_ratio.rbig.RBig,
      ∃ setup : BernoulliRationalSetup x_div_k,
        ∃ hsetupDenom : 0 < dashu.ubigToNat setup.denom,
          ∃ uniformSetup : OpenDP.samplers.uniform.UniformBelowSetup setup.denom,
            utilities.div_rbig_by_ubig_exact numer denom k = ok x_div_k ∧
            OpenDP.samplers.uniform.sample_uniform_ubig_below_setup setup.denom =
              ok uniformSetup ∧
            samplers.bernoulli.sample_bernoulli_rational x_div_k =
              (do
                let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len
                let r ←
                  samplers.uniform.sample_uniform_ubig_below_loop setup.denom
                    uniformSetup.threshold buffer
                core.result.Result.map
                  samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
                  r setup.numer) ∧
            bernoulliPMF setup.numer setup.denom hsetupDenom =
              SLang.BernoulliSamplePMF
                (dashu.ubigToNat numer)
                ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
                (bernoulli_step_wf hfrac hk) := by
  rcases div_rbig_by_ubig_exact_bernoulli_setup numer denom k hk hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, hdiv, hpmf⟩
  rcases sample_bernoulli_rational_spec x_div_k setup hsetupDenom with
    ⟨uniformSetup, huniformSetup, hrat, _, hcanon⟩
  refine ⟨x_div_k, setup, hsetupDenom, uniformSetup, hdiv, huniformSetup, hrat, ?_⟩
  calc
    bernoulliPMF setup.numer setup.denom hsetupDenom
      =
        SLang.BernoulliSamplePMF
          (dashu.ubigToNat setup.numer)
          ⟨dashu.ubigToNat setup.denom, hsetupDenom⟩
          (sample_bernoulli_rational_prob_le x_div_k setup) := hcanon
    _ =
        SLang.BernoulliSamplePMF
          (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
          (bernoulli_step_wf hfrac hk) := hpmf

/-- Extracted one-step `exp1` law, stated directly against SampCert's
`BernoulliExpNegSampleUnitLoop` transition. -/
theorem sample_bernoulli_exp1_step_sampcert_spec
    (numer denom k : dashu_int.ubig.UBig)
    (hk : 0 < dashu.ubigToNat k)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    ∃ x_div_k : dashu_ratio.rbig.RBig,
      ∃ setup : BernoulliRationalSetup x_div_k,
        ∃ hsetupDenom : 0 < dashu.ubigToNat setup.denom,
          ∃ uniformSetup : OpenDP.samplers.uniform.UniformBelowSetup setup.denom,
            utilities.div_rbig_by_ubig_exact numer denom k = ok x_div_k ∧
            OpenDP.samplers.uniform.sample_uniform_ubig_below_setup setup.denom =
              ok uniformSetup ∧
            samplers.bernoulli.sample_bernoulli_rational x_div_k =
              (do
                let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len
                let r ←
                  samplers.uniform.sample_uniform_ubig_below_loop setup.denom
                    uniformSetup.threshold buffer
                core.result.Result.map
                  samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
                  r setup.numer) ∧
            ∀ b : Bool,
              bernoulliPMF setup.numer setup.denom hsetupDenom b =
                SLang.BernoulliExpNegSampleUnitLoop
                  (dashu.ubigToNat numer)
                  ⟨dashu.ubigToNat denom, hdenom⟩
                  hfrac
                  (true, ⟨dashu.ubigToNat k, hk⟩)
                  (b, ⟨dashu.ubigToNat k + 1, Nat.succ_pos _⟩) := by
  rcases sample_bernoulli_exp1_step_spec numer denom k hk hdenom hfrac with
    ⟨x_div_k, setup, hsetupDenom, uniformSetup, hdiv, huniformSetup, hrat, hpmf⟩
  refine ⟨x_div_k, setup, hsetupDenom, uniformSetup, hdiv, huniformSetup, hrat, ?_⟩
  intro b
  calc
    bernoulliPMF setup.numer setup.denom hsetupDenom b
      =
        (SLang.BernoulliSamplePMF
          (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
          (bernoulli_step_wf hfrac hk)) b := by rw [hpmf]
    _ =
        SLang.BernoulliSample
          (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat k * dashu.ubigToNat denom, Nat.mul_pos hk hdenom⟩
          (bernoulli_step_wf hfrac hk) b := by
          unfold SLang.BernoulliSamplePMF
          rw [PMF.ofFintype_apply]
    _ =
        SLang.BernoulliExpNegSampleUnitLoop
          (dashu.ubigToNat numer)
          ⟨dashu.ubigToNat denom, hdenom⟩
          hfrac
          (true, ⟨dashu.ubigToNat k, hk⟩)
          (b, ⟨dashu.ubigToNat k + 1, Nat.succ_pos _⟩) :=
          (bernoulliExp1_sampcert_step_apply
            (dashu.ubigToNat numer)
            (dashu.ubigToNat denom)
            (dashu.ubigToNat k)
            hdenom hfrac hk b).symm

/-- On the positive-input branch, the extracted `sample_bernoulli_exp1`
function reduces to the expected loop initialized with `k = 1`. -/
theorem sample_bernoulli_exp1_eq_of_positive_parts
    (x : dashu_ratio.rbig.RBig)
    (numerSigned : dashu_int.ibig.IBig)
    (denom numer k : dashu_int.ubig.UBig)
    (hparts : dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom))
    (hsign :
      dashu_int.ibig.IBig.into_parts numerSigned =
        ok (dashu_base.sign.Sign.Positive, numer))
    (hk : dashu_int.ubig.UBig.ONE = ok k) :
    samplers.bernoulli.sample_bernoulli_exp1 x =
      samplers.bernoulli.sample_bernoulli_exp1_loop k denom numer k := by
  unfold samplers.bernoulli.sample_bernoulli_exp1
  simp [hparts, hsign, hk]

/-- Valid-input specification for `sample_bernoulli_exp1`: the extracted Rust
function starts the generated `exp1` loop at `k = 1`, and the canonical
probabilistic target is SampCert's negative-exponential unit sampler. -/
theorem sample_bernoulli_exp1_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    samplers.bernoulli.sample_bernoulli_exp1 x =
      samplers.bernoulli.sample_bernoulli_exp1_loop
        setup.one setup.denom setup.numer setup.one ∧
    dashu.ubigToNat setup.one = 1 ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac =
      SLang.BernoulliExpNegSampleUnit
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
        setup.hfrac := by
  refine ⟨?_, dashu.one_spec setup.one setup.hone, ?_⟩
  · exact sample_bernoulli_exp1_eq_of_positive_parts
      x setup.numerSigned setup.denom setup.numer setup.one
      setup.hparts setup.hsign setup.hone
  · rfl

/-- The first extracted `exp1` loop step, initialized from a valid
`sample_bernoulli_exp1` input, has the same one-step law as SampCert's
negative-exponential unit loop. -/
theorem sample_bernoulli_exp1_initial_step_sampcert_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    ∃ x_div_k : dashu_ratio.rbig.RBig,
      ∃ rationalSetup : BernoulliRationalSetup x_div_k,
        ∃ hsetupDenom : 0 < dashu.ubigToNat rationalSetup.denom,
          ∃ uniformSetup : OpenDP.samplers.uniform.UniformBelowSetup rationalSetup.denom,
            utilities.div_rbig_by_ubig_exact setup.numer setup.denom setup.one = ok x_div_k ∧
            OpenDP.samplers.uniform.sample_uniform_ubig_below_setup rationalSetup.denom =
              ok uniformSetup ∧
            samplers.bernoulli.sample_bernoulli_rational x_div_k =
              (do
                let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 uniformSetup.byte_len
                let r ←
                  samplers.uniform.sample_uniform_ubig_below_loop rationalSetup.denom
                    uniformSetup.threshold buffer
                core.result.Result.map
                  samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
                  r rationalSetup.numer) ∧
            ∀ b : Bool,
              bernoulliPMF rationalSetup.numer rationalSetup.denom hsetupDenom b =
                SLang.BernoulliExpNegSampleUnitLoop
                  (dashu.ubigToNat setup.numer)
                  ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
                  setup.hfrac
                  (true, ⟨dashu.ubigToNat setup.one, by
                    rw [dashu.one_spec setup.one setup.hone]
                    decide⟩)
                  (b, ⟨dashu.ubigToNat setup.one + 1, Nat.succ_pos _⟩) := by
  have honeNat : dashu.ubigToNat setup.one = 1 :=
    dashu.one_spec setup.one setup.hone
  have honePos : 0 < dashu.ubigToNat setup.one := by
    rw [honeNat]
    decide
  exact sample_bernoulli_exp1_step_sampcert_spec
    setup.numer setup.denom setup.one honePos setup.hdenom setup.hfrac

/-- One extracted `exp1` loop step continues with the incremented counter when
the rational Bernoulli subcall succeeds with `true`. -/
theorem sample_bernoulli_exp1_loop_body_eq_continue
    (k denom numer k1 k2 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Ok true))
  (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.cont k2) := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop.body
  simp [hdiv, hrat, hadd,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- Strengthened `true` branch: when the loop counter increment is by a value
with mathematical value one, the extracted continuation state is `k1 + 1`. -/
theorem sample_bernoulli_exp1_loop_body_continue_counter
    (k denom numer k1 k2 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Ok true))
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k1 k = ok k2)
    (hone : dashu.ubigToNat k = 1) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.cont k2) ∧
    dashu.ubigToNat k2 = dashu.ubigToNat k1 + 1 := by
  refine ⟨?_, ?_⟩
  · exact sample_bernoulli_exp1_loop_body_eq_continue
      k denom numer k1 k2 x_div_k hdiv hrat hadd
  · calc
      dashu.ubigToNat k2 = dashu.ubigToNat k1 + dashu.ubigToNat k :=
        dashu.add_assign_spec k1 k k2 hadd
      _ = dashu.ubigToNat k1 + 1 := by rw [hone]

/-- One extracted `exp1` loop step stops with the parity test result when the
rational Bernoulli subcall succeeds with `false`. -/
theorem sample_bernoulli_exp1_loop_body_eq_done
    (k denom numer k1 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (i : Std.U8)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Ok false))
  (hrem :
      dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.done (core.result.Result.Ok (decide (i = 1#u8)))) := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop.body
  simp [hdiv, hrat, hrem,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- Strengthened `false` branch: the extracted return value is exactly the
parity test on the mathematical loop counter. -/
theorem sample_bernoulli_exp1_loop_body_done_parity
    (k denom numer k1 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (i : Std.U8)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Ok false))
    (hrem :
      dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.done (core.result.Result.Ok (decide (i = 1#u8)))) ∧
    (decide (i = 1#u8) : Bool) =
      decide (dashu.ubigToNat k1 % 2 = 1) := by
  refine ⟨?_, ?_⟩
  · exact sample_bernoulli_exp1_loop_body_eq_done
      k denom numer k1 x_div_k i hdiv hrat hrem
  · have hremNat := dashu.rem_u8_spec k1 2#u8 i hrem
    have hi : (i = 1#u8) ↔ i.val = 1 := by
      rw [Aeneas.Std.UScalar.eq_equiv]
      simp
    have hiff : (i = 1#u8) ↔ dashu.ubigToNat k1 % 2 = 1 := by
      rw [hi, hremNat]
      simp
    by_cases h : i = 1#u8
    · have hparity : dashu.ubigToNat k1 % 2 = 1 := hiff.mp h
      simp [h, hparity]
    · have hparity : ¬ dashu.ubigToNat k1 % 2 = 1 := fun hparity =>
        h (hiff.mpr hparity)
      simp [h, hparity]

/-- The extracted stopping branch, restated in SampCert's final-counter
convention: Rust checks that the current counter is odd, which is equivalent
to checking that the successor counter returned by the SampCert loop is even. -/
theorem sample_bernoulli_exp1_loop_body_done_sampcert_parity
    (k denom numer k1 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (i : Std.U8)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Ok false))
    (hrem :
      dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem k1 2#u8 = ok i) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.done
        (core.result.Result.Ok
          (decide ((dashu.ubigToNat k1 + 1) % 2 = 0)))) := by
  rcases sample_bernoulli_exp1_loop_body_done_parity
    k denom numer k1 x_div_k i hdiv hrat hrem with ⟨hbody, hparity⟩
  calc
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1
      =
        ok (ControlFlow.done (core.result.Result.Ok (decide (i = 1#u8)))) := hbody
    _ =
        ok (ControlFlow.done
          (core.result.Result.Ok
            (decide ((dashu.ubigToNat k1 + 1) % 2 = 0)))) := by
          rw [hparity]
          rw [decide_odd_current_eq_even_successor]

/-- One extracted `exp1` loop step stops with the propagated entropy/error
result when the rational Bernoulli subcall returns an error. -/
theorem sample_bernoulli_exp1_loop_body_eq_error
    (k denom numer k1 : dashu_int.ubig.UBig)
    (x_div_k : dashu_ratio.rbig.RBig)
    (err : error.Error)
    (hdiv : utilities.div_rbig_by_ubig_exact numer denom k1 = ok x_div_k)
    (hrat :
      samplers.bernoulli.sample_bernoulli_rational x_div_k =
        ok (core.result.Result.Err err)) :
    samplers.bernoulli.sample_bernoulli_exp1_loop.body k denom numer k1 =
      ok (ControlFlow.done (core.result.Result.Err err)) := by
  unfold samplers.bernoulli.sample_bernoulli_exp1_loop.body
  simp [hdiv, hrat,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- The extracted `sample_bernoulli_exp` is definitionally its generated loop
wrapper. -/
theorem sample_bernoulli_exp_eq
    (x : dashu_ratio.rbig.RBig) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp_loop x := by
  rfl

/-- Primary top-level target for the full Bernoulli negative-exponential
sampler: for any nonnegative rational input, the intended mathematical law is
SampCert's `BernoulliExpNegSample` at the corresponding numerator and
denominator. -/
theorem sample_bernoulli_exp_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp_loop x ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      SLang.BernoulliExpNegSample
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ := by
  exact ⟨rfl, rfl⟩

/-- When the extracted `x > 1` test fails, the outer Bernoulli-exp loop body
stops immediately and delegates to `sample_bernoulli_exp1 x`. -/
theorem sample_bernoulli_exp_loop_body_eq_done_le
    (x one : dashu_ratio.rbig.RBig)
    (r1 : core.result.Result Bool error.Error)
    (hone : dashu_ratio.rbig.RBig.ONE = ok one)
    (hgt : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x one = ok false)
    (hexp1 : samplers.bernoulli.sample_bernoulli_exp1 x = ok r1) :
    samplers.bernoulli.sample_bernoulli_exp_loop.body x =
      ok (ControlFlow.done r1) := by
  unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
  simp [hone, hgt, hexp1]

/-- On the `x > 1` branch, a successful `true` sample from `exp1(1)` continues
the loop with `x - 1`. -/
theorem sample_bernoulli_exp_loop_body_eq_continue
    (x one oneRat x1 : dashu_ratio.rbig.RBig)
    (i : Std.U128)
    (hone : dashu_ratio.rbig.RBig.ONE = ok one)
    (hgt : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x one = ok true)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok oneRat)
    (hexp1 :
      samplers.bernoulli.sample_bernoulli_exp1 oneRat =
        ok (core.result.Result.Ok true))
    (hsub :
      dashu_ratio.rbig.RBig.Insts.CoreOpsArithSubAssignRBig.sub_assign x one = ok x1) :
    samplers.bernoulli.sample_bernoulli_exp_loop.body x =
      ok (ControlFlow.cont x1) := by
  unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
  simp [hone, hgt, hi, honeRat, hexp1, hsub,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- On the `x > 1` branch, a successful `false` sample from `exp1(1)` stops
the loop with final answer `false`. -/
theorem sample_bernoulli_exp_loop_body_eq_done_false
    (x one oneRat : dashu_ratio.rbig.RBig)
    (i : Std.U128)
    (hone : dashu_ratio.rbig.RBig.ONE = ok one)
    (hgt : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x one = ok true)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok oneRat)
    (hexp1 :
      samplers.bernoulli.sample_bernoulli_exp1 oneRat =
        ok (core.result.Result.Ok false)) :
    samplers.bernoulli.sample_bernoulli_exp_loop.body x =
      ok (ControlFlow.done (core.result.Result.Ok false)) := by
  unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
  simp [hone, hgt, hi, honeRat, hexp1,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- On the `x > 1` branch, entropy failure or any propagated Bernoulli error
from `exp1(1)` stops the loop with that same error. -/
theorem sample_bernoulli_exp_loop_body_eq_error
    (x one oneRat : dashu_ratio.rbig.RBig)
    (i : Std.U128)
    (err : error.Error)
    (hone : dashu_ratio.rbig.RBig.ONE = ok one)
    (hgt : dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x one = ok true)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok oneRat)
    (hexp1 :
      samplers.bernoulli.sample_bernoulli_exp1 oneRat =
        ok (core.result.Result.Err err)) :
    samplers.bernoulli.sample_bernoulli_exp_loop.body x =
      ok (ControlFlow.done (core.result.Result.Err err)) := by
  unfold samplers.bernoulli.sample_bernoulli_exp_loop.body
  simp [hone, hgt, hi, honeRat, hexp1,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- If the loop body stops in one step, the extracted outer Bernoulli-exp loop
returns that same result. -/
theorem sample_bernoulli_exp_loop_eq_done
    (x : dashu_ratio.rbig.RBig)
    (r : core.result.Result Bool error.Error)
    (hbody :
      samplers.bernoulli.sample_bernoulli_exp_loop.body x =
        ok (ControlFlow.done r)) :
    samplers.bernoulli.sample_bernoulli_exp_loop x = ok r := by
  unfold samplers.bernoulli.sample_bernoulli_exp_loop
  simp [Aeneas.Std.loop, hbody]

/-- When the input rational already lies in `[0, 1]`, the extracted outer
sampler immediately delegates to `sample_bernoulli_exp1`. -/
theorem sample_bernoulli_exp_eq_of_le_parts
    (x oneRat : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x)
    (honeRat : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (hr : core.result.Result Bool error.Error)
    (hexp1 : samplers.bernoulli.sample_bernoulli_exp1 x = ok hr) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp1 x := by
  have hgt :
      dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok false :=
    dashu.rbig_gt_one_false_of_le_spec
      x setup.numerSigned setup.denom setup.numer oneRat
      setup.hparts setup.hsign setup.hfrac
  have hbody :=
    sample_bernoulli_exp_loop_body_eq_done_le
      x oneRat hr honeRat hgt hexp1
  have hloop := sample_bernoulli_exp_loop_eq_done x hr hbody
  rw [sample_bernoulli_exp_eq, hloop, hexp1]

/-- Fractional-branch end-to-end specification: when the input rational lies in
`[0, 1]`, the full sampler reduces to `sample_bernoulli_exp1` and therefore has
the canonical SampCert unit negative-exponential law. -/
theorem sample_bernoulli_exp_spec_of_le
    (x oneRat : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x)
    (honeRat : dashu_ratio.rbig.RBig.ONE = ok oneRat)
    (hr : core.result.Result Bool error.Error)
    (hexp1 : samplers.bernoulli.sample_bernoulli_exp1 x = ok hr) :
    samplers.bernoulli.sample_bernoulli_exp x = samplers.bernoulli.sample_bernoulli_exp1 x ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac := by
  refine ⟨?_, bernoulliExpTarget_eq_exp1_of_le setup.numer setup.denom setup.hdenom setup.hfrac⟩
  exact sample_bernoulli_exp_eq_of_le_parts x oneRat setup honeRat hr hexp1

end OpenDP.samplers.bernoulli
