
import Hax
import opendp

open Std.Do
open Std.Tactic

set_option mvcgen.warning false
set_option linter.unusedVariables false

/-!
# Permute-and-Flip: extracted control-flow normalization

This file contains the non-probabilistic, non-SampCert-specific part of the
bridge from the hax-generated OpenDP extraction to a proof-friendly reference
shape.

## What is *not* temporary here?

The definitions and theorems in this file are intended to survive upstream hax
fixes. They only talk about:
- the extracted OpenDP function shape,
- the hax loop encoding (`fold_range_return`), and
- a reference loop with the same control-flow.

## What this file does

- isolates the zero-scale and positive-scale branches,
- factors the loop step into a named definition,
- proves the extracted function is definitionally equal to a structured form,
- introduces a handwritten recursive loop matching the concrete hax fold,
- unfolds the concrete `usize` hax fold one step.

This mirrors the style of the SampCert development by keeping the executable
mechanism-facing facts separate from the later privacy theorems.
-/

namespace opendp.measurements.noisy_top_k

open core_models.ops.control_flow
open rust_primitives.hax

open core_models.ops.control_flow
open rust_primitives.hax

/-- The exact empty-input error closure extracted by hax. -/
@[spec]
def pnfwr_x_is_empty :
    (rust_primitives.hax.Tuple0 -> RustM opendp.error.Error) :=
  fun _ =>
    do
      pure
        (opendp.error.Error.mk
          (variant := opendp.error.ErrorVariant.FailedFunction)
          (message :=
            core_models.option.Option.Some
              (← core_models.hint.must_use alloc.string.String
                (← alloc.fmt.format
                  (← core_models.fmt.rt.Impl_1.new_const
                    ((1 : usize))
                    (RustArray.ofVec #v["x is empty"]))))))
          (backtrace :=
            (← std.backtrace.Impl_4.capture rust_primitives.hax.Tuple0.mk)))

/-- The `scale.is_zero()` branch: deterministic argmax by index. -/
@[spec]
def pnfwr_zero_scale
    (x : RustSlice dashu_ratio.rbig.RBig) :
    RustM (core_models.result.Result usize opendp.error.Error) :=
  core_models.option.Impl.ok_or_else
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
    pnfwr_x_is_empty

/-- One loop step of `permute_and_flip_without_replacement`. -/
@[spec]
def pnfwr_step
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
  match
    (← opendp.traits.samplers.uniform.sample_uniform_uint_below
      usize
      ((8 : usize))
      (← ((← core_models.slice.Impl.len dashu_ratio.rbig.RBig x) -? left)))
  with
  | core_models.result.Result.Ok hoist341 => do
      let right : usize ← (left +? hoist341)
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ←
        alloc.slice.Impl.to_vec
          (← core_models.slice.Impl.swap usize
            (← alloc.vec.Impl_1.as_slice candidates)
            left
            right)
      let candidate : usize ← candidates[left]_?
      match
        (← opendp.traits.samplers.cks20.sample_bernoulli_exp
          (← core_models.ops.arith.Div.div
            dashu_ratio.rbig.RBig
            dashu_ratio.rbig.RBig
            (← core_models.ops.arith.Sub.sub
              dashu_ratio.rbig.RBig
              dashu_ratio.rbig.RBig
              x_max
              (← x[candidate]_?))
            scale))
      with
      | core_models.result.Result.Ok accepted => do
          if accepted then
            pure
              (core_models.ops.control_flow.ControlFlow.Break
                (core_models.ops.control_flow.ControlFlow.Break
                  (core_models.result.Result.Ok candidate)))
          else
            pure
              (core_models.ops.control_flow.ControlFlow.Continue candidates)
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

/-- Initial candidate vector `[0, 1, ..., len(x)-1]`. -/
@[spec]
def pnfwr_initial_candidates
    (x : RustSlice dashu_ratio.rbig.RBig) :
    RustM (alloc.vec.Vec usize alloc.alloc.Global) :=
  core_models.iter.traits.iterator.Iterator.collect
    (core_models.ops.range.Range usize)
    (alloc.vec.Vec usize alloc.alloc.Global)
    (core_models.ops.range.Range.mk
      (start := (0 : usize))
      (_end := (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)))

/-- The positive-scale branch isolated from the extracted definition. -/
@[spec]
def pnfwr_positive_scale
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
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ←
        pnfwr_initial_candidates x
      match
        (← rust_primitives.hax.folds.fold_range_return
          (0 : usize)
          (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
          (fun candidates _ => (do pure true : RustM Bool))
          candidates
          (pnfwr_step x x_max scale))
      with
      | core_models.ops.control_flow.ControlFlow.Break ret => do
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

/-- A structured restatement of the generated function. -/
@[spec]
def pnfwr_structured
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    RustM (core_models.result.Result usize opendp.error.Error) := do
  if (← dashu_ratio.rbig.Impl.is_zero scale) then
    pnfwr_zero_scale x
  else
    pnfwr_positive_scale x scale

/-- The hax-generated function is definitionally equal to the structured form above. -/
theorem permute_and_flip_without_replacement_eq_structured
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    permute_and_flip_without_replacement x scale = pnfwr_structured x scale := by
  unfold permute_and_flip_without_replacement
  unfold pnfwr_structured
  unfold pnfwr_zero_scale
  unfold pnfwr_positive_scale
  unfold pnfwr_initial_candidates
  unfold pnfwr_step
  unfold pnfwr_x_is_empty
  rfl

/-- A recursive reference loop matching the concrete `fold_range_return` recursion. -/
@[spec]
def pnfwr_loop_ref
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
    match (← pnfwr_step x x_max scale candidates left) with
    | core_models.ops.control_flow.ControlFlow.Break
        (core_models.ops.control_flow.ControlFlow.Break ret) =>
        pure (core_models.ops.control_flow.ControlFlow.Break ret)
    | core_models.ops.control_flow.ControlFlow.Break
        (core_models.ops.control_flow.ControlFlow.Continue
          (rust_primitives.hax.Tuple2.mk rust_primitives.hax.Tuple0.mk candidates)) =>
        pure (core_models.ops.control_flow.ControlFlow.Continue candidates)
    | core_models.ops.control_flow.ControlFlow.Continue candidates =>
        pnfwr_loop_ref x x_max scale e (left + 1) candidates
  else
    pure (core_models.ops.control_flow.ControlFlow.Continue candidates)

/-- Unfold the reference loop once. -/
theorem pnfwr_loop_ref_unfold
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (e : usize)
    (left : usize)
    (candidates : alloc.vec.Vec usize alloc.alloc.Global) :
    pnfwr_loop_ref x x_max scale e left candidates
      =
    (do
      if left < e then
        match (← pnfwr_step x x_max scale candidates left) with
        | core_models.ops.control_flow.ControlFlow.Break
            (core_models.ops.control_flow.ControlFlow.Break ret) =>
            pure (core_models.ops.control_flow.ControlFlow.Break ret)
        | core_models.ops.control_flow.ControlFlow.Break
            (core_models.ops.control_flow.ControlFlow.Continue
              (rust_primitives.hax.Tuple2.mk rust_primitives.hax.Tuple0.mk candidates)) =>
            pure (core_models.ops.control_flow.ControlFlow.Continue candidates)
        | core_models.ops.control_flow.ControlFlow.Continue candidates =>
            pnfwr_loop_ref x x_max scale e (left + 1) candidates
      else
        pure (core_models.ops.control_flow.ControlFlow.Continue candidates)) := by
  rfl

/--
Concrete unfold lemma for the underlying hax `usize` range fold with early return.
This is the actual loop semantics the backend generated for
`permute_and_flip_without_replacement`.
-/
theorem usize_fold_range_return_unfold
    {α_acc α_ret : Type}
    (s e : usize)
    (inv : α_acc -> usize -> RustM Prop)
    (init : α_acc)
    (body :
      α_acc -> usize ->
        RustM
          (core_models.ops.control_flow.ControlFlow
            (core_models.ops.control_flow.ControlFlow
              α_ret
              (rust_primitives.hax.Tuple2
                rust_primitives.hax.Tuple0
                α_acc))
            α_acc))
    (pureInv :
      {i : α_acc -> usize -> Prop //
        ∀ a b, ⦃⌜ True ⌝⦄ inv a b ⦃⇓ r => ⌜ r = (i a b) ⌝⦄}) :
    rust_primitives.USize64.fold_range_return s e inv init body pureInv
      =
    (do
      if s < e then
        match (← body init s) with
        | core_models.ops.control_flow.ControlFlow.Break
            (core_models.ops.control_flow.ControlFlow.Break res) =>
            pure (core_models.ops.control_flow.ControlFlow.Break res)
        | core_models.ops.control_flow.ControlFlow.Break
            (core_models.ops.control_flow.ControlFlow.Continue
              (rust_primitives.hax.Tuple2.mk rust_primitives.hax.Tuple0.mk res)) =>
            pure (core_models.ops.control_flow.ControlFlow.Continue res)
        | core_models.ops.control_flow.ControlFlow.Continue res =>
            rust_primitives.USize64.fold_range_return
              (s + 1) e inv res body pureInv
      else
        pure (core_models.ops.control_flow.ControlFlow.Continue init)) := by
  rfl

/-- Base case for the concrete range-fold/reference-loop bridge. -/
theorem pnfwr_fold_range_return_base
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (left e : usize)
    (candidates : alloc.vec.Vec usize alloc.alloc.Global)
    (h : ¬ left < e)
    (pureInv :
      {i : (alloc.vec.Vec usize alloc.alloc.Global) -> usize -> Prop //
        ∀ a b,
          ⦃⌜ True ⌝⦄ (do pure true : RustM Bool) ⦃⇓ r => ⌜ r = (i a b) ⌝⦄}) :
    rust_primitives.USize64.fold_range_return
      left
      e
      (fun candidates _ => (do pure true : RustM Bool))
      candidates
      (pnfwr_step x x_max scale)
      pureInv
      =
    pnfwr_loop_ref x x_max scale e left candidates := by
  rw [usize_fold_range_return_unfold]
  rw [pnfwr_loop_ref_unfold]
  simp [h]

/-- Induction step for the concrete range-fold/reference-loop bridge. -/
theorem pnfwr_fold_range_return_step
    (x : RustSlice dashu_ratio.rbig.RBig)
    (x_max : dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (left e : usize)
    (h : left < e)
    (pureInv :
      {i : (alloc.vec.Vec usize alloc.alloc.Global) -> usize -> Prop //
        ∀ a b,
          ⦃⌜ True ⌝⦄ (do pure true : RustM Bool) ⦃⇓ r => ⌜ r = (i a b) ⌝⦄})
    (IH : ∀ candidates',
      rust_primitives.USize64.fold_range_return
        (left + 1)
        e
        (fun candidates _ => (do pure true : RustM Bool))
        candidates'
        (pnfwr_step x x_max scale)
        pureInv
        =
      pnfwr_loop_ref x x_max scale e (left + 1) candidates')
    (candidates : alloc.vec.Vec usize alloc.alloc.Global) :
    rust_primitives.USize64.fold_range_return
      left
      e
      (fun candidates _ => (do pure true : RustM Bool))
      candidates
      (pnfwr_step x x_max scale)
      pureInv
      =
    pnfwr_loop_ref x x_max scale e left candidates := by
  rw [usize_fold_range_return_unfold]
  rw [pnfwr_loop_ref_unfold]
  simp [h, IH]

/--
The positive-scale branch rewritten so that the remaining proof obligation is
exactly the concrete range-fold/reference-loop bridge.
-/
@[spec]
def pnfwr_positive_scale_ref
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
      let candidates : alloc.vec.Vec usize alloc.alloc.Global ←
        pnfwr_initial_candidates x
      match
        (← pnfwr_loop_ref
          x
          x_max
          scale
          (← core_models.slice.Impl.len dashu_ratio.rbig.RBig x)
          (0 : usize)
          candidates)
      with
      | core_models.ops.control_flow.ControlFlow.Break ret => do
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

end opendp.measurements.noisy_top_k
