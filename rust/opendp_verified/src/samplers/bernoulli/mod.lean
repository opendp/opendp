import Aeneas
import Generated.OpenDP
import src.samplers.bernoulli.semantics
import src.samplers.uniform.mod

open Aeneas Aeneas.Std Result
open OpenDP

namespace OpenDP.samplers.bernoulli

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

/-- The extracted `sample_bernoulli_exp` is definitionally its generated loop
wrapper. -/
theorem sample_bernoulli_exp_eq
    (x : dashu_ratio.rbig.RBig) :
    samplers.bernoulli.sample_bernoulli_exp x =
      samplers.bernoulli.sample_bernoulli_exp_loop x := by
  rfl

end OpenDP.samplers.bernoulli
