import Hax
import opendp
import ControlFlow.ExtractedStructure

noncomputable section
open scoped Classical

/-!
# Permute-and-Flip: extracted-function branch reduction

This file contains the stable facts that transport the structured control-flow
normalization back to the *actual extracted* function
`permute_and_flip_without_replacement`.

## Why this is not temporary

These theorems only use:
- the extracted OpenDP function,
- the structured wrapper from `ControlFlow/ExtractedStructure`, and
- the extracted zero-test branch split.

They should remain useful even after hax's sampler extraction and runtime
semantics are completed.

## What this file does

- reduces `pnfwr_structured` to the positive-scale branch when `is_zero` is
  definitionally false,
- reduces `pnfwr_structured` to the zero-scale branch when `is_zero` is
  definitionally true,
- transports those reductions back to the actual extracted function, and
- lifts the reduction through any law interpreter that respects computation
  equality.
-/

namespace opendp.measurements.noisy_top_k

open core_models.ops.control_flow
open rust_primitives.hax

/--
A branch-law interpreter for computations returning `Result usize Error`.
This alias is stable and useful outside the temporary hax workaround layer.
-/
abbrev BranchLaw :=
  RustM (core_models.result.Result usize opendp.error.Error) -> usize -> ENNReal

/-- Equality of computations transports to equality of interpreted laws. -/
def LawRespectsEq (lawBranch : BranchLaw) : Prop :=
  ∀ {m₁ m₂ : RustM (core_models.result.Result usize opendp.error.Error)},
    m₁ = m₂ -> ∀ r : usize, lawBranch m₁ r = lawBranch m₂ r

/--
If the extracted `is_zero` test is definitionally `false`, the structured wrapper
reduces to the positive-scale branch.
-/
theorem pnfwr_structured_eq_positive_scale_of_is_zero_false
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool)) :
    pnfwr_structured x scale = pnfwr_positive_scale x scale := by
  unfold pnfwr_structured
  rw [hzero]
  rfl

/--
If the extracted `is_zero` test is definitionally `true`, the structured wrapper
reduces to the deterministic argmax branch.
-/
theorem pnfwr_structured_eq_zero_scale_of_is_zero_true
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure true : RustM Bool)) :
    pnfwr_structured x scale = pnfwr_zero_scale x := by
  unfold pnfwr_structured
  rw [hzero]
  rfl

/--
The actual extracted function reduces to the positive-scale branch whenever the
extracted zero-test reduces to `false`.
-/
theorem permute_and_flip_without_replacement_eq_positive_scale_of_is_zero_false
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool)) :
    permute_and_flip_without_replacement x scale = pnfwr_positive_scale x scale := by
  rw [permute_and_flip_without_replacement_eq_structured]
  exact pnfwr_structured_eq_positive_scale_of_is_zero_false x scale hzero

/--
Law-level transport of the previous computation equality.
-/
theorem pnfwr_law_eq_positive_scale_of_is_zero_false
    (lawBranch : BranchLaw)
    (hlaw : LawRespectsEq lawBranch)
    (x : RustSlice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.Impl.is_zero scale = (do pure false : RustM Bool)) :
    ∀ r : usize,
      lawBranch (permute_and_flip_without_replacement x scale) r
        =
      lawBranch (pnfwr_positive_scale x scale) r := by
  intro r
  apply hlaw
  exact permute_and_flip_without_replacement_eq_positive_scale_of_is_zero_false x scale hzero


end opendp.measurements.noisy_top_k
