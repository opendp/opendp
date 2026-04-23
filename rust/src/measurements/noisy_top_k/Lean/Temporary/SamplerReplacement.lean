
import Hax
import opendp
import ControlFlow.ExtractedStructure
import Semantic.SampCertBridge

open Std.Do
open Std.Tactic

set_option mvcgen.warning false
set_option linter.unusedVariables false

noncomputable section
open scoped Classical

/-!
# Permute-and-Flip: sampler replacement boundary (**temporary scaffolding**)

This file isolates the point where we stop relying on the hax-generated sampler
implementations and instead reason with semantic sampler replacements.

## Temporary status

The boundary here exists because the current generated Lean for the sampler layer
still contains placeholders. Once hax can extract those samplers completely,
this file should largely collapse into direct proofs about the extracted code.

## What this file does

- defines a semantic replacement for the loop step,
- defines semantic loop/branch variants,
- proves pointwise sampler replacement lifts to branch-level replacement,
- packages the remaining semantic bridge to SampCert.
-/

namespace opendp.measurements.noisy_top_k

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

open core_models.ops.control_flow
open rust_primitives.hax

/--
A semantic replacement for the extracted one-step sampler behavior.
`choose_right rem` is the semantic model of drawing a suffix offset in `[0, rem)`.
`accept_gap gap` is the semantic model of the Bernoulli-exp acceptance test.
-/
@[spec]
def pnfwr_step_semantic
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (candidates : alloc.vec.Vec usize alloc.alloc.Global)
    (left : usize) :
    RustM
      (core_models.ops.control_flow.ControlFlow
        (core_models.ops.control_flow.ControlFlow
          (core_models.result.Result usize opendp.error.Error)
          (rust_primitives.hax.Tuple2
            rust_primitives.hax.Tuple0
            rust_primitives.hax.Tuple0))
        (alloc.vec.Vec usize alloc.alloc.Global)) := do
  let rem : usize ← ((← core_models.slice.Impl.len dashu_ratio.rbig.RBig x) -? left)
  match (← choose_right rem) with
  | core_models.result.Result.Ok rightOffset => do
      let right : usize ← (left +? rightOffset)
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ←
        alloc.slice.Impl.to_vec
          (← core_models.slice.Impl.swap usize
            (← alloc.vec.Impl_1.as_slice candidates)
            left
            right)
      let candidate : usize ← candidates[left]_?
      let gap : dashu_ratio.rbig.RBig ←
        (← core_models.ops.arith.Div.div
          dashu_ratio.rbig.RBig
          dashu_ratio.rbig.RBig
          (← core_models.ops.arith.Sub.sub
            dashu_ratio.rbig.RBig
            dashu_ratio.rbig.RBig
            x_max
            (← x[candidate]_?))
          scale)
      match (← accept_gap gap) with
      | core_models.result.Result.Ok accepted => do
          if accepted then
            pure
              (core_models.ops.control_flow.ControlFlow.Break
                (core_models.ops.control_flow.ControlFlow.Break
                  (core_models.result.Result.Ok candidate)))
          else
            pure (core_models.ops.control_flow.ControlFlow.Continue candidates)
      | core_models.result.Result.Err err => do
          pure
            (core_models.ops.control_flow.ControlFlow.Break
              (core_models.ops.control_flow.ControlFlow.Break
                (core_models.result.Result.Err err)))
  | core_models.result.Result.Err err => do
      pure
        (core_models.ops.control_flow.ControlFlow.Break
          (core_models.ops.control_flow.ControlFlow.Break
            (core_models.result.Result.Err err)))

/--
If the concrete sampler calls are extensionally equal to semantic replacements,
then the entire extracted step is equal to the semantic step.
-/
theorem pnfwr_step_eq_semantic_of_sampler_eq
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (candidates : alloc.vec.Vec usize alloc.alloc.Global)
    (left : usize)
    (hchoose : ∀ rem,
      opendp.traits.samplers.uniform.sample_uniform_uint_below
        usize
        ((8 : usize))
        rem
        =
      choose_right rem)
    (haccept : ∀ gap,
      opendp.traits.samplers.cks20.sample_bernoulli_exp gap = accept_gap gap) :
    pnfwr_step x x_max scale candidates left
      =
    pnfwr_step_semantic choose_right accept_gap x x_max scale candidates left := by
  unfold pnfwr_step
  unfold pnfwr_step_semantic
  rw [hchoose]
  simp_rw [haccept]

/--
A semantic loop obtained by replacing only the sampler calls, while preserving
all extracted control flow.
-/
@[spec]
def pnfwr_loop_semantic
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (e : usize)
    (left : usize)
    (candidates : alloc.vec.Vec usize alloc.alloc.Global) :
    RustM
      (core_models.ops.control_flow.ControlFlow
        (core_models.result.Result usize opendp.error.Error)
        (alloc.vec.Vec usize alloc.alloc.Global)) := do
  if left < e then
    match (← pnfwr_step_semantic choose_right accept_gap x x_max scale candidates left) with
    | core_models.ops.control_flow.ControlFlow.Break
        (core_models.ops.control_flow.ControlFlow.Break ret) =>
        pure (core_models.ops.control_flow.ControlFlow.Break ret)
    | core_models.ops.control_flow.ControlFlow.Break
        (core_models.ops.control_flow.ControlFlow.Continue
          (rust_primitives.hax.Tuple2.mk rust_primitives.hax.Tuple0.mk candidates)) =>
        pure (core_models.ops.control_flow.ControlFlow.Continue candidates)
    | core_models.ops.control_flow.ControlFlow.Continue candidates =>
        pnfwr_loop_semantic choose_right accept_gap x x_max scale e (left + 1) candidates
  else
    pure (core_models.ops.control_flow.ControlFlow.Continue candidates)

/--
The positive-scale branch with semantic samplers substituted in place of the
extracted sampler implementations.
-/
@[spec]
def pnfwr_positive_scale_semantic
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    RustM (core_models.result.Result usize opendp.error.Error) := do
  match
    (← core_models.option.Impl.ok_or_else
      usize
      opendp.error.Error
      (rust_primitives.hax.Tuple0 -> RustM opendp.error.Error)
      (← core_models.iter.traits.iterator.Iterator.max_by_key
        (core_models.ops.range.Range usize)
        dashu_ratio.rbig.RBig
        (usize -> RustM dashu_ratio.rbig.RBig)
        (core_models.ops.range.Range.mk
          (start := (0 : usize))
          (_end := (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)))
        (fun i => do x[i]_? : RustM dashu_ratio.rbig.RBig))
      pnfwr_x_is_empty)
  with
  | core_models.result.Result.Ok x_max_index => do
      let x_max : dashu_ratio.rbig.RBig ← x[x_max_index]_?
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ← pnfwr_initial_candidates x
      match
        (← pnfwr_loop_semantic choose_right accept_gap x x_max scale
          (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
          (0 : usize)
          candidates)
      with
      | core_models.ops.control_flow.ControlFlow.Break ret =>
          pure ret
      | core_models.ops.control_flow.ControlFlow.Continue candidates => do
          rust_primitives.hax.never_to_any
            (← core_models.panicking.panic_fmt
              (← core_models.fmt.rt.Impl_1.new_v1
                ((1 : usize))
                ((0 : usize))
                (RustArray.ofVec
                  #v["internal error: entered unreachable code: at least one x[candidate] is equal to x_max"])
                (RustArray.ofVec #v[])))
  | core_models.result.Result.Err err => do
      pure (core_models.result.Result.Err err)

/--
This theorem packages the current proof boundary cleanly:
if you can identify the extracted loop with the semantic loop, then the whole
positive-scale branch is identified as well.
-/
theorem pnfwr_positive_scale_ref_eq_semantic_of_loop_eq
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hloop : ∀ x_max candidates,
      pnfwr_loop_ref x x_max scale
        (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
        (0 : usize)
        candidates
      =
      pnfwr_loop_semantic choose_right accept_gap x x_max scale
        (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
        (0 : usize)
        candidates) :
    pnfwr_positive_scale_ref x scale
      =
    pnfwr_positive_scale_semantic choose_right accept_gap x scale := by
  unfold pnfwr_positive_scale_ref
  unfold pnfwr_positive_scale_semantic
  simp
  apply congrArg
  funext x_max_index
  apply congrArg
  funext x_max
  apply congrArg
  funext candidates
  simpa using hloop x_max candidates

/-!
## Temporary composition layer

The declarations below sharpen the previous sampler-replacement boundary into a
small set of semantic obligations whose discharge would immediately yield the
SampCert privacy theorem. They are temporary and should disappear once the
extracted sampler/runtime semantics are complete.
-/

open core_models.ops.control_flow
open rust_primitives.hax
open SLang
open SLang.PermuteAndFlip

/--
A semantic replacement of the *actual extracted positive-scale branch*, using the
same hax `fold_range_return` loop skeleton but semantic sampler calls.

This is a better bridge target than the earlier recursive helper because it stays
as close as possible to the generated code while still cutting off the extracted
sampler internals.
-/
@[spec]
def pnfwr_positive_scale_semantic_fold
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    RustM (core_models.result.Result usize opendp.error.Error) := do
  match
    (← core_models.option.Impl.ok_or_else
      usize
      opendp.error.Error
      (rust_primitives.hax.Tuple0 -> RustM opendp.error.Error)
      (← core_models.iter.traits.iterator.Iterator.max_by_key
        (core_models.ops.range.Range usize)
        dashu_ratio.rbig.RBig
        (usize -> RustM dashu_ratio.rbig.RBig)
        (core_models.ops.range.Range.mk
          (start := (0 : usize))
          (_end := (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)))
        (fun i => do x[i]_? : RustM dashu_ratio.rbig.RBig))
      pnfwr_x_is_empty)
  with
  | core_models.result.Result.Ok x_max_index => do
      let x_max : dashu_ratio.rbig.RBig ← x[x_max_index]_?
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ← pnfwr_initial_candidates x
      match
        (← rust_primitives.hax.folds.fold_range_return
          (0 : usize)
          (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
          (fun candidates _ => (do pure true : RustM Bool))
          candidates
          (pnfwr_step_semantic choose_right accept_gap x x_max scale))
      with
      | core_models.ops.control_flow.ControlFlow.Break ret =>
          pure ret
      | core_models.ops.control_flow.ControlFlow.Continue candidates => do
          rust_primitives.hax.never_to_any
            (← core_models.panicking.panic_fmt
              (← core_models.fmt.rt.Impl_1.new_v1
                ((1 : usize))
                ((0 : usize))
                (RustArray.ofVec
                  #v["internal error: entered unreachable code: at least one x[candidate] is equal to x_max"])
                (RustArray.ofVec #v[])))
  | core_models.result.Result.Err err => do
      pure (core_models.result.Result.Err err)

/--
Pointwise equality of the concrete extracted step with a semantic step lifts
*directly* to equality of the whole extracted positive-scale branch, because both
sides are the same `fold_range_return` skeleton with only the body replaced.
-/
theorem pnfwr_positive_scale_eq_semantic_fold_of_step_eq
    (choose_right : usize -> RustM (core_models.result.Result usize opendp.error.Error))
    (accept_gap : dashu_ratio.rbig.RBig -> RustM (core_models.result.Result Bool opendp.error.Error))
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hstep : ∀ x_max left candidates,
      pnfwr_step x x_max scale candidates left
        =
      pnfwr_step_semantic choose_right accept_gap x x_max scale candidates left) :
    pnfwr_positive_scale x scale
      =
    pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale := by
  unfold pnfwr_positive_scale
  unfold pnfwr_positive_scale_semantic_fold
  simp_rw [hstep]

/--
The most useful concrete composition so far: if the two sampler calls in the
extracted step are extensionally equal to semantic replacements, then the whole
extracted positive-scale branch is equal to the semantic-fold branch.
-/
theorem pnfwr_positive_scale_eq_semantic_fold_of_sampler_eq
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
    pnfwr_positive_scale x scale
      =
    pnfwr_positive_scale_semantic_fold choose_right accept_gap x scale := by
  apply pnfwr_positive_scale_eq_semantic_fold_of_step_eq
  intro x_max left candidates
  simpa using
    pnfwr_step_eq_semantic_of_sampler_eq
      choose_right accept_gap x x_max scale candidates left hchoose haccept

/--
A sharper final bridge package: everything up to the extracted OpenDP branch is
now reduced to sampler replacement plus a semantic law that matches SampCert.
-/
structure PNFWRConcreteToSampCertAssumptions
    {n : CandidateCount}
    (x x' : QScores n)
    (r : Fin n.succ) where
  norm : PNFWRNormalizedInstance n
  range_eq : qRangeDistance x x' = rangeDistance norm.q norm.q'
  law_x : pnfwr_semantic_law x x' r = permuteAndFlipSLang n norm.q norm.ε₁ norm.ε₂ r
  law_x' : pnfwr_semantic_law x' x r = permuteAndFlipSLang n norm.q' norm.ε₁ norm.ε₂ r

/--
Once the semantic no-replacement law is identified with SampCert's executable
permute-and-flip law after normalization, privacy follows immediately from the
SampCert theorem.
-/
theorem pnfwr_range_privacy_of_concrete_bridge
    {n : CandidateCount}
    (x x' : QScores n)
    (r : Fin n.succ)
    (H : PNFWRConcreteToSampCertAssumptions x x' r) :
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ qRangeDistance x x'
      * pnfwr_semantic_law x x' r
      ≤
      pnfwr_semantic_law x' x r := by
  calc
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ qRangeDistance x x'
        * pnfwr_semantic_law x x' r
      =
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ rangeDistance H.norm.q H.norm.q'
        * permuteAndFlipSLang n H.norm.q H.norm.ε₁ H.norm.ε₂ r := by
          rw [H.range_eq, H.law_x]
    _ ≤ permuteAndFlipSLang n H.norm.q' H.norm.ε₁ H.norm.ε₂ r := by
          exact permuteAndFlipSLang_range_privacy H.norm.q H.norm.q' r H.norm.ε₁ H.norm.ε₂
    _ = pnfwr_semantic_law x' x r := by
          rw [H.law_x']

/--
This is the exact remaining semantic gap to discharge after sampler replacement:
- identify `choose_right` with the uniform suffix draw used by the sampled permutation,
- identify `accept_gap` with SampCert's exact Bernoulli-negative-exponential coin,
- prove the impossible-fallthrough branch cannot occur.

At that point, `pnfwr_semantic_law` can be instantiated by the law of
`pnfwr_positive_scale_semantic_fold`, and the theorem above becomes the final
privacy theorem for the extracted OpenDP implementation.
-/

end opendp.measurements.noisy_top_k
