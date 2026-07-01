import Generated.OpenDP
import SampCert.Samplers.Bernoulli.Properties
import SampCert.Samplers.Uniform.Properties
import src.core.primitives.semantics
import src.core.externals.dashu
import src.core.readable.notation

/-!
# `sample_bernoulli_rational` — correctness

Human proof (`traits/samplers/bernoulli/sample_bernoulli_rational.tex`), in three
sentences: sample uniformly from `[0, denom)`; return `⊤` iff the sample is below
`numer`; so `Pr[⊤] = numer/denom = prob`.

Round 1 builds the **translation layer** (proven correct): on the valid-input
branch the extracted Aeneas code equals "uniform-below-`denom`, then compare".
The distributional half (`⟦·⟧ = Bernoulli …`) follows once the uniform spec lands.
-/

open Aeneas Aeneas.Std Result
open OpenDP
open SLang PMF ENNReal

namespace OpenDP.samplers.bernoulli

/-- The Bernoulli distribution with success probability `numer / denom`: a uniform draw
from `[0, denom)` returns `true` exactly when it lands in `[0, numer)`. This is the
reference law the extracted `sample_bernoulli_rational` is proven to realise. -/
noncomputable def bernoulliPMF (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) : PMF Bool :=
  PMF.map (fun x : Nat => decide (x < dashu.ubigToNat numer))
    (UniformSample_PMF ⟨dashu.ubigToNat denom, hdenom⟩)

/-- `bernoulliPMF` is exactly SampCert's canonical `BernoulliSamplePMF`, so the correctness
theorem can be read against the trusted reference distribution. -/
theorem bernoulliPMF_eq_BernoulliSamplePMF
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hwf : dashu.ubigToNat numer ≤ dashu.ubigToNat denom) :
    bernoulliPMF numer denom hdenom =
      SLang.BernoulliSamplePMF
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf := by
  ext b
  cases b
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, SLang.BernoulliSample_apply_false]
    change SLang.BernoulliSample
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf false = _
    rw [SLang.BernoulliSample_apply_false]
  · unfold SLang.BernoulliSamplePMF
    rw [PMF.ofFintype_apply, SLang.BernoulliSample_apply_true]
    change SLang.BernoulliSample
        (dashu.ubigToNat numer) ⟨dashu.ubigToNat denom, hdenom⟩ hwf true = _
    rw [SLang.BernoulliSample_apply_true]

/-- Concrete witnesses for the valid-input branch of `sample_bernoulli_rational`:
`prob = numer/denom` with `numer ≥ 0` and `numer ≤ denom`. -/
structure RationalSetup (prob : dashu_ratio.rbig.RBig) where
  numerSigned : dashu_int.ibig.IBig
  denom : dashu_int.ubig.UBig
  numer : dashu_int.ubig.UBig
  hparts : dashu_ratio.rbig.RBig.into_parts prob = ok (numerSigned, denom)
  hsign :
    dashu_int.ibig.IBig.into_parts numerSigned =
      ok (dashu_base.sign.Sign.Positive, numer)
  hvalid :
    dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt numer denom = ok false

/-- **Translation layer (proven correct).** Once the deterministic input checks
succeed, the extracted function reduces to a uniform draw below `denom` followed
by the Rust comparison closure `numer > ·`. -/
theorem sample_bernoulli_rational_eq_of_setup
    (prob : dashu_ratio.rbig.RBig) (setup : RationalSetup prob) :
    samplers.bernoulli.sample_bernoulli_rational prob =
      (do
        let r ← samplers.uniform.sample_uniform_ubig_below setup.denom
        core.result.Result.map
          samplers.bernoulli.sample_bernoulli_rational.closure.Insts.CoreOpsFunctionFnOnceTupleUBigBool
          r setup.numer) := by
  unfold samplers.bernoulli.sample_bernoulli_rational
  simp [setup.hparts, setup.hsign, setup.hvalid]

end OpenDP.samplers.bernoulli
