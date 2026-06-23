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

/-- SampCert tail target for the `exp1` loop when started from an arbitrary
positive counter `k`. This is the clean mathematical object that the extracted
loop should eventually be shown to realize operationally. -/
noncomputable def bernoulliExp1LoopTailTarget
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom)
    (k : Nat)
    (hk : 0 < k) :
    SLang Bool := do
  let K ←
    (do
      let st ←
        SLang.probWhile
          (fun state : Bool × PNat => state.1)
          (SLang.BernoulliExpNegSampleUnitLoop
            (dashu.ubigToNat numer)
            ⟨dashu.ubigToNat denom, hdenom⟩
            hfrac)
          (true, ⟨k, hk⟩)
      pure (st.2 : Nat))
  if K % 2 = 0 then pure true else pure false

/-- Starting the `exp1` tail target at counter `1` is definitionally SampCert's
canonical unit negative-exponential sampler. -/
theorem bernoulliExp1LoopTailTarget_one_eq
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliExp1LoopTailTarget numer denom hdenom hfrac 1 (by decide) =
      bernoulliExp1Target numer denom hdenom hfrac := by
  unfold bernoulliExp1LoopTailTarget bernoulliExp1Target
  unfold SLang.BernoulliExpNegSampleUnit SLang.BernoulliExpNegSampleUnitAux
  simp

/-- Target-side PMF theorem for `sample_bernoulli_exp1`. This is the final
mathematical statement we want the extracted loop to realize: the success mass
is exactly `exp (-numer / denom)`, and the whole target is SampCert's
`BernoulliExpNegSampleUnit`. -/
theorem sample_bernoulli_exp1_pmf_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : ENNReal) /
              (dashu.ubigToNat setup.denom : ENNReal)).toReal))) ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : ENNReal) /
              (dashu.ubigToNat setup.denom : ENNReal)).toReal))) ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac =
      SLang.BernoulliExpNegSampleUnit
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
        setup.hfrac := by
  refine ⟨?_, ?_, rfl⟩
  · exact bernoulliExp1Target_apply_true setup.numer setup.denom setup.hdenom setup.hfrac
  · exact bernoulliExp1Target_apply_false setup.numer setup.denom setup.hdenom setup.hfrac

/-- The unit-step negative-exponential target is a proper distribution. -/
theorem bernoulliExp1Target_normalizes
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hfrac : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliExp1Target numer denom hdenom hfrac true +
      bernoulliExp1Target numer denom hdenom hfrac false = 1 := by
  rw [bernoulliExp1Target_apply_true, bernoulliExp1Target_apply_false]
  have hle :
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal))) ≤ 1 := by
    have hexp_le :
        Real.exp
          (-(((dashu.ubigToNat numer : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal)) ≤ 1 := by
      apply (Real.exp_le_one_iff).2
      have hnonneg :
          0 ≤
            (((dashu.ubigToNat numer : ENNReal) /
                (dashu.ubigToNat denom : ENNReal)).toReal) := by
        positivity
      linarith
    simpa using ENNReal.ofReal_le_ofReal hexp_le
  simpa [add_comm, add_left_comm, add_assoc] using
    (tsub_add_cancel_of_le hle :
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat numer : ENNReal) /
              (dashu.ubigToNat denom : ENNReal)).toReal))) +
        ENNReal.ofReal
          (Real.exp
            (-(((dashu.ubigToNat numer : ENNReal) /
                (dashu.ubigToNat denom : ENNReal)).toReal))) = 1)

/-- Intended final end-to-end correctness theorem for the extracted
`sample_bernoulli_exp1`.

This is the theorem we ultimately care about: the stochastic behavior induced
by the extracted Rust function should coincide exactly with SampCert's
`BernoulliExpNegSampleUnit`, equivalently with the explicit PMF of
`exp (-numer / denom)`.

At the moment, all target-side PMF facts and the local control-flow / setup
bridges are in place; the remaining work is the operational probabilistic
bridge from the extracted Aeneas loop to `bernoulliExp1LoopTailTarget`. -/
theorem sample_bernoulli_exp1_end_to_end_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    ∃ x_div_k : dashu_ratio.rbig.RBig,
      ∃ rationalSetup : BernoulliRationalSetup x_div_k,
        ∃ _hsetupDenom : 0 < dashu.ubigToNat rationalSetup.denom,
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
            samplers.bernoulli.sample_bernoulli_exp1 x =
              samplers.bernoulli.sample_bernoulli_exp1_loop
                setup.one setup.denom setup.numer setup.one ∧
            bernoulliExp1LoopTailTarget
              setup.numer setup.denom setup.hdenom setup.hfrac 1 (by decide) =
              bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac ∧
            bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac =
              SLang.BernoulliExpNegSampleUnit
                (dashu.ubigToNat setup.numer)
                ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
                setup.hfrac ∧
            bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac true =
              ENNReal.ofReal
                (Real.exp
                  (-(((dashu.ubigToNat setup.numer : ENNReal) /
                      (dashu.ubigToNat setup.denom : ENNReal)).toReal))) ∧
            bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac false =
              1 - ENNReal.ofReal
                (Real.exp
                  (-(((dashu.ubigToNat setup.numer : ENNReal) /
                      (dashu.ubigToNat setup.denom : ENNReal)).toReal))) := by
  rcases sample_bernoulli_exp1_initial_step_sampcert_spec x setup with
    ⟨x_div_k, rationalSetup, hsetupDenom, uniformSetup, hdiv, huniform, hrat, hstep⟩
  rcases sample_bernoulli_exp1_spec x setup with ⟨hextract, hone, htarget⟩
  rcases sample_bernoulli_exp1_pmf_spec x setup with ⟨htrue, hfalse, hpmf⟩
  refine ⟨x_div_k, rationalSetup, hsetupDenom, uniformSetup, ?_⟩
  refine ⟨hdiv, huniform, hrat, ?_⟩
  refine ⟨hextract, ?_⟩
  refine ⟨bernoulliExp1LoopTailTarget_one_eq setup.numer setup.denom setup.hdenom setup.hfrac, ?_⟩
  refine ⟨hpmf, ?_⟩
  exact ⟨htrue, hfalse⟩

/-- Structural packaging for `sample_bernoulli_exp1`: the extracted loop,
the unit-step PMF facts, and the SampCert target are all aligned in one
place. -/
theorem sample_bernoulli_exp1_structural_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExp1Setup x) :
    samplers.bernoulli.sample_bernoulli_exp1 x =
      samplers.bernoulli.sample_bernoulli_exp1_loop
        setup.one setup.denom setup.numer setup.one ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : ENNReal) /
              (dashu.ubigToNat setup.denom : ENNReal)).toReal))) ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : ENNReal) /
              (dashu.ubigToNat setup.denom : ENNReal)).toReal))) ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac =
      SLang.BernoulliExpNegSampleUnit
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
        setup.hfrac ∧
    bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac true +
      bernoulliExp1Target setup.numer setup.denom setup.hdenom setup.hfrac false = 1 := by
  rcases sample_bernoulli_exp1_spec x setup with ⟨hexp, hone, htarget⟩
  rcases sample_bernoulli_exp1_pmf_spec x setup with ⟨htrue, hfalse, hpmf⟩
  refine ⟨hexp, htrue, hfalse, hpmf, ?_⟩
  exact bernoulliExp1Target_normalizes
    setup.numer setup.denom setup.hdenom setup.hfrac

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

/-- Primary PMF theorem for the full Bernoulli negative-exponential sampler.
For any nonnegative rational input, the verified target induced by the Rust
sampler has the same pointwise mass function as SampCert's
`BernoulliExpNegSample`, and therefore samples `true` with probability
`exp (-numer / denom)` and `false` with the complementary mass. -/
theorem sample_bernoulli_exp_pmf_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    bernoulliExpTarget setup.numer setup.denom setup.hdenom true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      SLang.BernoulliExpNegSample
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ := by
  refine ⟨?_, ?_, rfl⟩
  · exact bernoulliExpTarget_apply_true setup.numer setup.denom setup.hdenom
  · exact bernoulliExpTarget_apply_false setup.numer setup.denom setup.hdenom

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

/-- Setup bridge for the extracted `rbig!(1)` call in `sample_bernoulli_exp`.
The scalar cast and Dashu constructor should together produce the same
positive `1 / 1` rational expected by SampCert's generator loop. -/
theorem sample_bernoulli_exp_one_setup_of_from_parts_const
    (i : Std.U128)
    (oneRat : dashu_ratio.rbig.RBig)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok oneRat) :
    ∃ setup : BernoulliExp1Setup oneRat,
      setup.numer = setup.one ∧
      setup.denom = setup.one ∧
      dashu.ubigToNat setup.numer = 1 ∧
      dashu.ubigToNat setup.denom = 1 := by
  have hiEq : (UScalar.cast .U128 (1#u32)) = i := by
    simpa [Aeneas.Std.lift] using hi
  have hiNat : i.val = 1 := by
    simpa [hiEq] using (U32.cast_U128_val_eq (1#u32))
  rcases dashu.rbig_from_parts_const_one_spec i oneRat hiNat honeRat with
    ⟨numerSigned, one, hone, hparts, hsign⟩
  refine ⟨
    { numerSigned := numerSigned
      denom := one
      numer := one
      one := one
      hparts := hparts
      hsign := hsign
      hone := hone
      hdenom := by
        rw [dashu.one_spec one hone]
        norm_num
      hfrac := by simp },
    ?_⟩
  constructor
  · rfl
  · constructor
    · rfl
    · constructor <;> simpa using dashu.one_spec one hone

/-- Subtract-one state bridge for the outer `sample_bernoulli_exp` loop.
On the `x > 1` branch, the extracted `x -= RBig::ONE` state represents the
same denominator with numerator decreased by one denominator. -/
theorem sample_bernoulli_exp_sub_one_setup
    (x one xMinusOne : dashu_ratio.rbig.RBig)
    (i : Std.U128)
    (setup : BernoulliExpSetup x)
    (hlt :
      dashu.ubigToNat setup.denom < dashu.ubigToNat setup.numer)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok one)
    (hsub :
      dashu_ratio.rbig.RBig.Insts.CoreOpsArithSubAssignRBig.sub_assign x one =
        ok xMinusOne) :
    ∃ setupMinusOne : BernoulliExpSetup xMinusOne,
      setupMinusOne.denom = setup.denom ∧
      dashu.ubigToNat setupMinusOne.numer =
        dashu.ubigToNat setup.numer - dashu.ubigToNat setup.denom := by
  rcases sample_bernoulli_exp_one_setup_of_from_parts_const i one hi honeRat with
    ⟨oneSetup, honeNumerEq, honeDenomEq, honeNumer, honeDenom⟩
  have honeParts :
      dashu_ratio.rbig.RBig.into_parts one =
        ok (oneSetup.numerSigned, oneSetup.one) := by
    simpa [honeDenomEq] using oneSetup.hparts
  have honeSign :
      dashu_int.ibig.IBig.into_parts oneSetup.numerSigned =
        ok (dashu_base.sign.Sign.Positive, oneSetup.one) := by
    simpa [honeNumerEq] using oneSetup.hsign
  have hle : dashu.ubigToNat setup.denom ≤ dashu.ubigToNat setup.numer :=
    Nat.le_of_lt hlt
  rcases dashu.sub_exists_spec setup.numer setup.denom hle with ⟨numer', hsubnumer, hsubnumerNat⟩
  rcases dashu.rbig_sub_one_positive_spec
      x one xMinusOne
      setup.numerSigned oneSetup.numerSigned
      setup.numer setup.denom numer' oneSetup.one
      setup.hparts setup.hsign oneSetup.hone
      honeParts honeSign hle hsub with
    ⟨numerSignedMinusOne, hsubEq, hpartsMinus, hsignMinus⟩
  refine ⟨
    { numerSigned := numerSignedMinusOne
      denom := setup.denom
      numer := numer'
      hparts := hpartsMinus
      hsign := hsignMinus
      hdenom := setup.hdenom },
    ?_⟩
  constructor
  · rfl
  · exact dashu.sub_spec setup.numer setup.denom numer' hsubnumer

/-- SampCert target bridge for one `x > 1` step. This is the mathematical
counterpart of the extracted outer loop body: `exp(-x)` is sampled by first
sampling `exp(-1)` and, on success, recurring on `x - 1`. -/
theorem sample_bernoulli_exp_target_step_of_gt
    (x oneRat xMinusOne : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x)
    (oneSetup : BernoulliExp1Setup oneRat)
    (honePos : 0 < dashu.ubigToNat oneSetup.one)
    (setupMinusOne : BernoulliExpSetup xMinusOne)
    (hlt :
      dashu.ubigToNat setup.denom < dashu.ubigToNat setup.numer)
    (hnumerMinus :
      dashu.ubigToNat setupMinusOne.numer =
        dashu.ubigToNat setup.numer - dashu.ubigToNat setup.denom) :
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      (do
        let b ←
          bernoulliExp1Target oneSetup.one oneSetup.one honePos (le_refl _)
        if b
        then bernoulliExpTarget setupMinusOne.numer setup.denom setup.hdenom
        else pure false) := by
  have honeNat : dashu.ubigToNat oneSetup.one = 1 := dashu.one_spec oneSetup.one oneSetup.hone
  have hstep :=
    bernoulliExpTarget_eq_exp1_one_bind_sub_one_of_gt
      setup.numer setup.denom setupMinusOne.numer oneSetup.one
      setup.hdenom honePos honeNat hlt hnumerMinus
  exact hstep

/-- Extracted control-flow bridge for one `x > 1` continuation step. This
packages the generated branch lemmas together with the semantic state updates
needed by the full induction. -/
theorem sample_bernoulli_exp_loop_body_continue_structural
    (x one xMinusOne : dashu_ratio.rbig.RBig)
    (i : Std.U128)
    (setup : BernoulliExpSetup x)
    (hone : dashu_ratio.rbig.RBig.ONE = ok one)
    (hgt :
      dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x one = ok true)
    (hi : lift (UScalar.cast .U128 1#u32) = ok i)
    (honeRat :
      dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i =
        ok one)
    (hexp1 :
      samplers.bernoulli.sample_bernoulli_exp1 one =
        ok (core.result.Result.Ok true))
    (hsub :
      dashu_ratio.rbig.RBig.Insts.CoreOpsArithSubAssignRBig.sub_assign x one =
        ok xMinusOne) :
    ∃ oneSetup : BernoulliExp1Setup one,
      ∃ setupMinusOne : BernoulliExpSetup xMinusOne,
        samplers.bernoulli.sample_bernoulli_exp_loop.body x =
          ok (ControlFlow.cont xMinusOne) ∧
        setupMinusOne.denom = setup.denom ∧
        dashu.ubigToNat setupMinusOne.numer =
          dashu.ubigToNat setup.numer - dashu.ubigToNat setup.denom ∧
        bernoulliExpTarget setup.numer setup.denom setup.hdenom =
          (do
            let b ←
              bernoulliExp1Target oneSetup.numer oneSetup.denom
                oneSetup.hdenom oneSetup.hfrac
            if b
            then bernoulliExpTarget setupMinusOne.numer setupMinusOne.denom setupMinusOne.hdenom
            else pure false) := by
  have hlt :
      dashu.ubigToNat setup.denom < dashu.ubigToNat setup.numer :=
    dashu.rbig_gt_one_true_spec
      x setup.numerSigned setup.denom setup.numer one
      setup.hparts setup.hsign hgt
  rcases sample_bernoulli_exp_one_setup_of_from_parts_const i one hi honeRat with
    ⟨oneSetup, honeNumerEq, honeDenomEq, honeNumer, honeDenom⟩
  let oneSetupCanon : BernoulliExp1Setup one :=
    { numerSigned := oneSetup.numerSigned
      denom := oneSetup.one
      numer := oneSetup.one
      one := oneSetup.one
      hparts := by
        simpa [honeDenomEq] using oneSetup.hparts
      hsign := by
        simpa [honeNumerEq] using oneSetup.hsign
      hone := oneSetup.hone
      hdenom := by
        simpa [honeDenomEq] using oneSetup.hdenom
      hfrac := le_refl _ }
  have honePos : 0 < dashu.ubigToNat oneSetup.one := by
    rw [dashu.one_spec oneSetup.one oneSetup.hone]
    decide
  rcases sample_bernoulli_exp_sub_one_setup
      x one xMinusOne i setup hlt hi honeRat hsub with
    ⟨setupMinusOne, hsetupMinusOne⟩
  let setupMinusOneCanon : BernoulliExpSetup xMinusOne :=
    { numerSigned := setupMinusOne.numerSigned
      denom := setup.denom
      numer := setupMinusOne.numer
      hparts := by
        simpa [hsetupMinusOne.1] using setupMinusOne.hparts
      hsign := by
        simpa [hsetupMinusOne.1] using setupMinusOne.hsign
      hdenom := by
        simpa [hsetupMinusOne.1] using setupMinusOne.hdenom }
  have hbody :
      samplers.bernoulli.sample_bernoulli_exp_loop.body x =
        ok (ControlFlow.cont xMinusOne) :=
    sample_bernoulli_exp_loop_body_eq_continue
      x one one xMinusOne i hone hgt hi honeRat hexp1 hsub
  have htarget :
      bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      (do
          let b ←
            bernoulliExp1Target oneSetupCanon.numer oneSetupCanon.denom
              oneSetupCanon.hdenom oneSetupCanon.hfrac
          if b then
            bernoulliExpTarget setupMinusOneCanon.numer
              setupMinusOneCanon.denom setupMinusOneCanon.hdenom
          else
            pure false) :=
      sample_bernoulli_exp_target_step_of_gt
      x one xMinusOne setup oneSetupCanon honePos setupMinusOneCanon hlt
      (by simpa using hsetupMinusOne.2)
  refine ⟨oneSetupCanon, setupMinusOneCanon, ?_, ?_⟩
  · exact hbody
  · constructor
    · rfl
    · constructor
      · simpa using hsetupMinusOne.2
      · simpa [oneSetupCanon, setupMinusOneCanon] using htarget

/-- Structural proposition for the outer `sample_bernoulli_exp` loop.
On inputs already in `[0, 1]`, the target collapses to the unit-step
negative-exponential sampler. On inputs `> 1`, the target factors into one
unit step followed by the subtract-one recursive target. -/
def sample_bernoulli_exp_loop_structural_prop
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) : Prop :=
    ∃ oneRat : dashu_ratio.rbig.RBig,
      dashu_ratio.rbig.RBig.ONE = ok oneRat ∧
      (if _hfrac : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom
       then
        ∃ exp1Setup : BernoulliExp1Setup x,
          exp1Setup.numer = setup.numer ∧
          exp1Setup.denom = setup.denom ∧
          bernoulliExpTarget setup.numer setup.denom setup.hdenom =
            bernoulliExp1Target exp1Setup.numer exp1Setup.denom
              exp1Setup.hdenom exp1Setup.hfrac
       else
        ∃ oneSetup : BernoulliExp1Setup oneRat,
          ∃ xMinusOne : dashu_ratio.rbig.RBig,
            ∃ setupMinusOne : BernoulliExpSetup xMinusOne,
              setupMinusOne.denom = setup.denom ∧
              dashu.ubigToNat setupMinusOne.numer =
                dashu.ubigToNat setup.numer - dashu.ubigToNat setup.denom ∧
              bernoulliExpTarget setup.numer setup.denom setup.hdenom =
                (do
                  let b ←
                    bernoulliExp1Target oneSetup.numer oneSetup.denom
                      oneSetup.hdenom oneSetup.hfrac
                  if b
                  then
                    bernoulliExpTarget setupMinusOne.numer setupMinusOne.denom
                      setupMinusOne.hdenom
                  else pure false))

/-- Complete structural specification of the outer `sample_bernoulli_exp`
loop: the handwritten model makes the same branch split and recursive target
decomposition as SampCert. -/
theorem sample_bernoulli_exp_loop_structural_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    sample_bernoulli_exp_loop_structural_prop x setup := by
  rcases dashu.rbig_one_setup_spec with
    ⟨oneRat, oneSigned, one, honeRat, hone, honeParts, honeSign⟩
  refine ⟨oneRat, honeRat, ?_⟩
  by_cases hfrac : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom
  · simp [hfrac]
    refine ⟨
      { numerSigned := setup.numerSigned
        denom := setup.denom
        numer := setup.numer
        one := one
        hparts := setup.hparts
        hsign := setup.hsign
        hone := hone
        hdenom := setup.hdenom
        hfrac := hfrac },
      rfl,
      rfl,
      ?_⟩
    simpa using
      (bernoulliExpTarget_eq_exp1_of_le
        setup.numer setup.denom setup.hdenom hfrac)
  · simp [hfrac]
    have hgt : dashu.ubigToNat setup.denom < dashu.ubigToNat setup.numer := by
      exact Nat.lt_of_not_ge hfrac
    have hle : dashu.ubigToNat setup.denom ≤ dashu.ubigToNat setup.numer :=
      Nat.le_of_lt hgt
    rcases dashu.sub_exists_spec setup.numer setup.denom hle with
      ⟨numer', hsub, hnumer'⟩
    rcases dashu.ibig_from_ubig_exists_spec numer' with
      ⟨numerSignedMinusOne, hconvMinus, hsignMinus⟩
    rcases dashu.rbig_from_parts_positive_exists_spec
        numer' setup.denom numerSignedMinusOne setup.hdenom hsignMinus with
      ⟨xMinusOne, hpartsMinus, hxMinusOne⟩
    let oneSetup : BernoulliExp1Setup oneRat :=
      { numerSigned := oneSigned
        denom := one
        numer := one
        one := one
        hparts := honeParts
        hsign := honeSign
        hone := hone
        hdenom := by
          have hnat : dashu.ubigToNat one = 1 := dashu.one_spec one hone
          simp [hnat]
        hfrac := by
          have hnat : dashu.ubigToNat one = 1 := dashu.one_spec one hone
          simp [hnat] }
    have honePos : 0 < dashu.ubigToNat one := by
      rw [dashu.one_spec one hone]
      decide
    let setupMinusOne : BernoulliExpSetup xMinusOne :=
      { numerSigned := numerSignedMinusOne
        denom := setup.denom
        numer := numer'
        hparts := hxMinusOne
        hsign := hsignMinus
        hdenom := setup.hdenom }
    have hnumerMinus :
        dashu.ubigToNat setupMinusOne.numer =
          dashu.ubigToNat setup.numer - dashu.ubigToNat setup.denom := by
      simpa [setupMinusOne] using
        (dashu.sub_spec setup.numer setup.denom numer' hsub)
    have htarget :=
      sample_bernoulli_exp_target_step_of_gt
        x oneRat xMinusOne setup oneSetup honePos setupMinusOne hgt hnumerMinus
    refine ⟨oneSetup, xMinusOne, setupMinusOne, ?_, ?_, ?_⟩
    · rfl
    · exact hnumerMinus
    · simpa [setupMinusOne, oneSetup] using htarget

/-- Structural specification of the extracted `sample_bernoulli_exp`.
This theorem pins down the exact recursive SampCert target used by the Rust
function, without yet proving the final distributional correctness bridge. -/
theorem sample_bernoulli_exp_structural_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp_loop x ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      SLang.BernoulliExpNegSample
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ ∧
    sample_bernoulli_exp_loop_structural_prop x setup := by
  rcases sample_bernoulli_exp_pmf_spec x setup with ⟨htrue, hfalse, hpmf⟩
  refine ⟨sample_bernoulli_exp_eq x, htrue, hfalse, hpmf, ?_⟩
  exact sample_bernoulli_exp_loop_structural_spec x setup

/-- End-to-end PMF specification for the extracted `sample_bernoulli_exp`.
This packages the extracted wrapper equality together with the closed-form
negative-exponential target law. -/
theorem sample_bernoulli_exp_end_to_end_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : BernoulliExpSetup x) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp_loop x ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom true =
      ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom false =
      1 - ENNReal.ofReal
        (Real.exp
          (-(((dashu.ubigToNat setup.numer : NNReal) /
              (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) ∧
    bernoulliExpTarget setup.numer setup.denom setup.hdenom =
      SLang.BernoulliExpNegSample
        (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ := by
  rcases sample_bernoulli_exp_structural_spec x setup with
    ⟨hloop, htrue, hfalse, hpmf, _hstruct⟩
  exact ⟨hloop, htrue, hfalse, hpmf⟩

end OpenDP.samplers.bernoulli
