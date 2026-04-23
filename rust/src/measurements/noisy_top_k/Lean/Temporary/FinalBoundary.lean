import Hax
import opendp
import ControlFlow.ExtractedFunctionReduction
import Temporary.RuntimeSemantics

noncomputable section
open scoped Classical

/-!
# Permute-and-Flip: final hax workaround boundary (**temporary scaffolding**)

This file contains only the obligations that exist because the current hax Lean
output does not yet give a complete checked semantics for the sampler/runtime
layer.

## Temporary status

Everything here is intended to be replaced as hax improves. In particular, this
file isolates:
- the dependence on a law interpreter for the extracted `RustM` computation,
- semantic replacement of extracted sampler calls,
- the explicit branch-selection hypothesis for `scale ≠ 0`, and
- the final assumption packaging used to state the privacy theorem for the
  actual extracted function.

The *permanent* extracted-function reduction facts live in
`ControlFlow/ExtractedFunctionReduction.lean`, and the permanent semantic bridge
to SampCert lives in `Semantic/SampCertBridge.lean`.
-/

namespace opendp.measurements.noisy_top_k

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

/--
Privacy for the actual extracted `permute_and_flip_without_replacement` function,
assuming:
* the extracted zero-test reduces to `false`,
* sampler replacement holds,
* the semantic branch matches SampCert's executable mechanism.

This theorem simply transports the already-proved positive-scale bridge back to
`permute_and_flip_without_replacement` itself.
-/
theorem permute_and_flip_without_replacement_runtime_range_privacy_of_sampler_laws
    (lawBranch : BranchLaw)
    (hlaw : LawRespectsEq lawBranch)
    {n : CandidateCount}
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool))
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below usize ((8 : usize)) rem = choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap)
    (H : PNFWRSamplerLawBridge choose_right accept_gap (fun _ _ => 0) (fun _ _ => 0) lawBranch x x' scale q q' embed ε₁ ε₂ encodeGap)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      ≤
      lawBranch (permute_and_flip_without_replacement x' scale) (embed r) := by
  have hx : lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      = lawBranch (pnfwr_positive_scale x scale) (embed r) := by
    exact pnfwr_law_eq_positive_scale_of_is_zero_false lawBranch hlaw x scale hzero (embed r)
  have hx' : lawBranch (permute_and_flip_without_replacement x' scale) (embed r)
      = lawBranch (pnfwr_positive_scale x' scale) (embed r) := by
    exact pnfwr_law_eq_positive_scale_of_is_zero_false lawBranch hlaw x' scale hzero (embed r)
  calc
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      = (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawBranch (pnfwr_positive_scale x scale) (embed r) := by
          rw [hx]
    _ ≤ lawBranch (pnfwr_positive_scale x' scale) (embed r) := by
          exact pnfwr_runtime_range_privacy_of_sampler_laws
            lawBranch hlaw choose_right accept_gap x x' scale q q' embed ε₁ ε₂ encodeGap
            hchoose haccept H r
    _ = lawBranch (permute_and_flip_without_replacement x' scale) (embed r) := by
          rw [hx']

/--
A final abstraction step: if you have an OpenDP-facing score distance `dist` on
input slices and can show it coincides with SampCert's `rangeDistance` after your
normalization/encoding step, then the privacy theorem transports to that metric.
-/
theorem permute_and_flip_without_replacement_runtime_privacy_on_metric
    (lawBranch : BranchLaw)
    (hlaw : LawRespectsEq lawBranch)
    {n : CandidateCount}
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ)
    (dist : RustSlice dashu_ratio.rbig.RBig -> RustSlice dashu_ratio.rbig.RBig -> ℕ)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool))
    (hdist : dist x x' = rangeDistance q q')
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below usize ((8 : usize)) rem = choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap)
    (H : PNFWRSamplerLawBridge choose_right accept_gap (fun _ _ => 0) (fun _ _ => 0) lawBranch x x' scale q q' embed ε₁ ε₂ encodeGap)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ dist x x' * lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      ≤
      lawBranch (permute_and_flip_without_replacement x' scale) (embed r) := by
  rw [hdist]
  exact permute_and_flip_without_replacement_runtime_range_privacy_of_sampler_laws
    lawBranch hlaw choose_right accept_gap x x' scale q q' embed ε₁ ε₂ encodeGap
    hzero hchoose haccept H r

/--
This is the clean remaining obligation set for a full OpenDP-facing theorem on the
extracted `permute_and_flip_without_replacement` function:
* a law interpreter for the extracted `RustM` computation,
* a proof that nonzero `scale` selects the positive-scale branch,
* checked semantic models for the two samplers,
* a normalization from rational OpenDP scores to SampCert's natural-valued scores,
* and a proof that the semantic branch law matches `permuteAndFlipSLang`.
-/

/-!
## Final temporary packaging

This final section packages every remaining obligation into a single assumption
structure. Instantiating that structure is equivalent to finishing the proof for
the extracted OpenDP function.
-/

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

/--
A final single-package assumption set for privacy of the *actual extracted*
`permute_and_flip_without_replacement` function.

This packages every remaining obligation after the hax loop/control-flow bridge:
* a law interpreter for the extracted Rust computation,
* law extensionality for that interpreter,
* a proof that the extracted zero test selects the positive-scale branch,
* semantic replacements for the two sampler calls,
* and the SampCert executable-mechanism bridge assumptions.
-/
structure PNFWRFinalAssumptions
    {n : CandidateCount}
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ) where
  lawBranch : BranchLaw
  lawRespectsEq : LawRespectsEq lawBranch
  hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool)
  choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error)
  accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error)
  hchoose : ∀ rem,
    opendp.traits.samplers.uniform.sample_uniform_uint_below usize ((8 : usize)) rem = choose_right rem
  haccept : ∀ gap,
    opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap
  bridge : PNFWRSamplerLawBridge choose_right accept_gap (fun _ _ => 0) (fun _ _ => 0)
    lawBranch x x' scale q q' embed ε₁ ε₂ encodeGap

/--
Final theorem schema for the actual extracted `permute_and_flip_without_replacement`.

Instantiating `PNFWRFinalAssumptions` is now equivalent to finishing the proof.
-/
theorem permute_and_flip_without_replacement_privacy_of_final_assumptions
    {n : CandidateCount}
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ)
    (A : PNFWRFinalAssumptions x x' scale q q' embed ε₁ ε₂ encodeGap)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' *
      A.lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      ≤
      A.lawBranch (permute_and_flip_without_replacement x' scale) (embed r) := by
  exact permute_and_flip_without_replacement_runtime_range_privacy_of_sampler_laws
    A.lawBranch
    A.lawRespectsEq
    A.choose_right
    A.accept_gap
    x x' scale q q' embed ε₁ ε₂ encodeGap
    A.hzero
    A.hchoose
    A.haccept
    A.bridge
    r

/--
A final theorem schema transported to any OpenDP-facing metric that coincides with
SampCert's `rangeDistance` after your normalization.
-/
theorem permute_and_flip_without_replacement_privacy_on_metric_of_final_assumptions
    {n : CandidateCount}
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ)
    (dist : RustSlice dashu_ratio.rbig.RBig -> RustSlice dashu_ratio.rbig.RBig -> ℕ)
    (hdist : dist x x' = rangeDistance q q')
    (A : PNFWRFinalAssumptions x x' scale q q' embed ε₁ ε₂ encodeGap)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ dist x x' *
      A.lawBranch (permute_and_flip_without_replacement x scale) (embed r)
      ≤
      A.lawBranch (permute_and_flip_without_replacement x' scale) (embed r) := by
  rw [hdist]
  exact permute_and_flip_without_replacement_privacy_of_final_assumptions
    x x' scale q q' embed ε₁ ε₂ encodeGap A r

end opendp.measurements.noisy_top_k
