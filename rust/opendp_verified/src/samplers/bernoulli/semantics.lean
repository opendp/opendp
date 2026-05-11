import Aeneas
import Generated.OpenDP.FunsExternal
import src.externals.dashu
import SampCert.Samplers.Bernoulli.Properties

open Aeneas Aeneas.Std Result Classical
open OpenDP SLang

namespace OpenDP.samplers.bernoulli

/-- The Bernoulli distribution with success probability `numer / denom`.
The success outcome is distributed to be true exactly when a uniform sample
from `[0, denom)` falls in `[0, numer)`. -/
noncomputable def bernoulliPMF (numer denom : dashu_int.ubig.UBig) (hdenom : 0 < dashu.ubigToNat denom) : PMF Bool :=
  PMF.map (fun x : Nat => decide (x < dashu.ubigToNat numer))
    (UniformSample_PMF ⟨dashu.ubigToNat denom, hdenom⟩)

/-- `bernoulliPMF` is exactly SampCert's canonical Bernoulli sampler PMF. -/
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

end OpenDP.samplers.bernoulli
