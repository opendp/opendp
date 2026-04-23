
import Hax
import opendp
import Temporary.SamplerReplacement
import SampCert.DifferentialPrivacy.PermuteAndFlip.Mechanism.Selector
import SampCert.DifferentialPrivacy.PermuteAndFlip.Privacy

noncomputable section
open scoped Classical

/-!
# Permute-and-Flip: runtime laws and sampler laws (**temporary scaffolding**)

This file introduces an abstract output-law semantics for the extracted `RustM`
programs and packages the remaining sampler-law assumptions needed to compose
with SampCert's executable privacy theorem.

## Temporary status

This is temporary until the extracted sampler/runtime layer can be given a fully
checked Lean semantics. It is intentionally explicit about every remaining law
needed to complete the proof.
-/

namespace opendp.measurements.noisy_top_k

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

abbrev PNFWRComp := RustM (core_models.result.Result usize opendp.error.Error)

/--
An abstract output-law interpreter for the extracted Rust computation.
This is the only place where we still hide the semantics of `RustM` as a law on
returned indices.
-/
abbrev PNFWRLaw := PNFWRComp -> usize -> ENNReal

/-- Equality of Rust computations transports to equality of their interpreted laws. -/
def LawRespectsEq (lawOf : PNFWRLaw) : Prop :=
  ∀ {m₁ m₂ : PNFWRComp}, m₁ = m₂ -> ∀ r : usize, lawOf m₁ r = lawOf m₂ r

/--
Sampler replacement at the RustM level lifts immediately to equality of output
laws for the extracted positive-scale branch.
-/
theorem pnfwr_positive_scale_law_eq_semantic_fold_of_sampler_eq
    (lawOf : PNFWRLaw)
    (hlaw : LawRespectsEq lawOf)
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below
        usize
        ((8 : usize))
        rem
        =
      choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap) :
    ∀ r : usize,
      lawOf (pnfwr_positive_scale x scale) r
        =
      lawOf (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale) r := by
  intro r
  apply hlaw
  exact pnfwr_positive_scale_eq_semantic_fold_of_sampler_eq
    choose_right accept_gap x scale hchoose haccept

/--
A direct runtime-to-SampCert bridge for the positive-scale no-replacement
implementation. `embed` identifies the Rust `usize` returned by the extracted
code with the `Fin` candidate returned by SampCert.
-/
structure PNFWRRuntimeBridgeAssumptions
    (lawOf : PNFWRLaw)
    (mx mx' : PNFWRComp)
    {n : CandidateCount}
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+) where
  law_x : ∀ r : Fin n.succ, lawOf mx (embed r) = permuteAndFlipSLang n q ε₁ ε₂ r
  law_x' : ∀ r : Fin n.succ, lawOf mx' (embed r) = permuteAndFlipSLang n q' ε₁ ε₂ r

/--
Once the interpreted output law of the extracted computation is identified with
SampCert's exact executable mechanism, privacy follows immediately.
-/
theorem pnfwr_runtime_range_privacy_of_bridge
    (lawOf : PNFWRLaw)
    (mx mx' : PNFWRComp)
    {n : CandidateCount}
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (H : PNFWRRuntimeBridgeAssumptions lawOf mx mx' q q' embed ε₁ ε₂)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawOf mx (embed r)
      ≤
      lawOf mx' (embed r) := by
  calc
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawOf mx (embed r)
      = (privacyBase ε₁ ε₂) ^ rangeDistance q q' * permuteAndFlipSLang n q ε₁ ε₂ r := by
          rw [H.law_x r]
    _ ≤ permuteAndFlipSLang n q' ε₁ ε₂ r := by
          exact permuteAndFlipSLang_range_privacy q q' r ε₁ ε₂
    _ = lawOf mx' (embed r) := by
          rw [H.law_x' r]

/--
Combined last-mile theorem:
1. replace the extracted sampler calls by semantic samplers,
2. interpret the resulting Rust computation as an output law,
3. bridge that law to SampCert's exact executable mechanism.
-/
theorem pnfwr_runtime_range_privacy_of_sampler_and_bridge
    (lawOf : PNFWRLaw)
    (hlaw : LawRespectsEq lawOf)
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    {n : CandidateCount}
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below
        usize
        ((8 : usize))
        rem
        =
      choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap)
    (H : PNFWRRuntimeBridgeAssumptions
      lawOf
      (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale)
      (pnfwr_positive_scale_semantic_fold choose_right accept_gap x' scale)
      q q' embed ε₁ ε₂)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawOf (pnfwr_positive_scale x scale) (embed r)
      ≤
      lawOf (pnfwr_positive_scale x' scale) (embed r) := by
  have hx : lawOf (pnfwr_positive_scale x scale) (embed r)
      = lawOf (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale) (embed r) := by
    exact pnfwr_positive_scale_law_eq_semantic_fold_of_sampler_eq
      lawOf hlaw choose_right accept_gap x scale hchoose haccept (embed r)
  have hx' : lawOf (pnfwr_positive_scale x' scale) (embed r)
      = lawOf (pnfwr_positive_scale_semantic_fold choose_right accept_gap x' scale) (embed r) := by
    exact pnfwr_positive_scale_law_eq_semantic_fold_of_sampler_eq
      lawOf hlaw choose_right accept_gap x' scale hchoose haccept (embed r)
  calc
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawOf (pnfwr_positive_scale x scale) (embed r)
      = (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawOf (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale) (embed r) := by
          rw [hx]
    _ ≤ lawOf (pnfwr_positive_scale_semantic_fold choose_right accept_gap x' scale) (embed r) := by
          exact pnfwr_runtime_range_privacy_of_bridge
            lawOf
            (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale)
            (pnfwr_positive_scale_semantic_fold choose_right accept_gap x' scale)
            q q' embed ε₁ ε₂ H r
    _ = lawOf (pnfwr_positive_scale x' scale) (embed r) := by
          rw [hx']

/-!
## Temporary sampler-law interface

The declarations below record the remaining law-level assumptions for the
sampler primitives. This is the narrowest temporary interface needed to compose
the extracted OpenDP branch with SampCert's executable theorem.
-/

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

/-- A law interpreter for Rust computations returning `Result α Error`. -/
abbrev ResultLaw (α : Type) := RustM (core_models.result.Result α opendp.error.Error) -> α -> ENNReal

/-- A law interpreter for the extracted no-replacement positive-scale branch. -/
abbrev BranchLaw := PNFWRLaw

/-- Uniform draw from `[0, rem)` at the law level. -/
def IsUniformBelow (lawNat : ResultLaw usize)
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error)) : Prop :=
  ∀ rem j,
    lawNat (choose_right rem) j = if h : j < rem then (rem : ENNReal)⁻¹ else 0

/-- Bernoulli with exact exponential-gap acceptance law. -/
def IsBernoulliExpLaw
    (lawBool : ResultLaw Bool)
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ)
    (ε₂ : ℕ+) : Prop :=
  ∀ gap b,
    lawBool (accept_gap gap) b = exactCoinPMF (encodeGap gap) ε₂ b

/--
A concrete last-mile package for the semantic no-replacement branch.

The intended instantiation is:
* `lawNat` interprets `sample_uniform_uint_below`
* `lawBool` interprets `sample_bernoulli_exp`
* `lawBranch` interprets the full extracted branch
* `encodeGap` turns the rational OpenDP gap into the normalized natural SampCert gap
* `embed` identifies Rust `usize` indices with `Fin n.succ`
-/
structure PNFWRSamplerLawBridge
    {n : CandidateCount}
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (lawNat : ResultLaw usize)
    (lawBool : ResultLaw Bool)
    (lawBranch : BranchLaw)
    (x x' : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (q q' : Scores n)
    (embed : Fin n.succ -> usize)
    (ε₁ : ℕ)
    (ε₂ : ℕ+)
    (encodeGap : dashu_ratio.rbig.RBig -> ℕ) where
  law_choose : IsUniformBelow lawNat choose_right
  law_accept : IsBernoulliExpLaw lawBool accept_gap encodeGap ε₂
  /-- The semantic no-replacement branch on `x` has exactly the SampCert executable law. -/
  branch_x : ∀ r : Fin n.succ,
    lawBranch (pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale) (embed r)
      = permuteAndFlipSLang n q ε₁ ε₂ r
  /-- Same executable-law identification for `x'`. -/
  branch_x' : ∀ r : Fin n.succ,
    lawBranch (pnfwr_positive_scale_semantic_fold choose_right accept_gap x' scale) (embed r)
      = permuteAndFlipSLang n q' ε₁ ε₂ r

/--
Once the sampler laws and the executable-mechanism identification are in place,
privacy of the extracted positive-scale no-replacement branch follows directly.
-/
theorem pnfwr_runtime_range_privacy_of_sampler_laws
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
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below usize ((8 : usize)) rem = choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap)
    (H : PNFWRSamplerLawBridge choose_right accept_gap (fun _ _ => 0) (fun _ _ => 0) lawBranch x x' scale q q' embed ε₁ ε₂ encodeGap)
    (r : Fin n.succ) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' * lawBranch (pnfwr_positive_scale x scale) (embed r)
      ≤
      lawBranch (pnfwr_positive_scale x' scale) (embed r) := by
  exact pnfwr_runtime_range_privacy_of_sampler_and_bridge
    lawBranch hlaw choose_right accept_gap x x' scale q q' embed ε₁ ε₂
    hchoose haccept
    { law_x := H.branch_x, law_x' := H.branch_x' }
    r

/--
This is the clean remaining obligation set for the OpenDP-to-SampCert bridge.

To finish the proof after upstreaming hax fixes or adding checked sampler models,
you need to instantiate:
* `lawBranch` with the output-law semantics of the extracted RustM branch,
* `choose_right` with the uniform suffix draw,
* `accept_gap` with the exact Bernoulli-negative-exponential coin,
* `branch_x` and `branch_x'` by proving the semantic fold matches
  `permuteAndFlipSLang` on the normalized score vectors.
-/

end opendp.measurements.noisy_top_k
