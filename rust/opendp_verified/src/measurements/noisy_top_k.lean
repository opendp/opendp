import Generated.OpenDP
import src.samplers.mod
import SampCert.DifferentialPrivacy.PermuteAndFlip.Mechanism.Selector
import SampCert.DifferentialPrivacy.PermuteAndFlip.Privacy
import SampCert.DifferentialPrivacy.PermuteAndFlip.Peeling

noncomputable section

open scoped Classical
open Aeneas Aeneas.Std Result
open OpenDP

namespace OpenDP.measurements.noisy_top_k

theorem sample_uniform_usize_below_eq
    (limit : Std.Usize) :
    samplers.uniform.sample_uniform_usize_below limit =
      samplers.uniform.sample_uniform_usize_below limit := by
  rfl

theorem permute_and_flip_true_eq
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    permute_and_flip x scale true =
      permute_and_flip_with_replacement x scale := by
  rfl

theorem permute_and_flip_false_eq
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig) :
    permute_and_flip x scale false =
      permute_and_flip_without_replacement x scale := by
  rfl

theorem permute_and_flip_with_replacement_zero
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (index : Std.Usize)
    (hmax : max_index x = ok (some index))
    (hzero : dashu_ratio.rbig.RBig.is_zero scale = ok true) :
    permute_and_flip_with_replacement x scale =
      ok (core.result.Result.Ok index) := by
  unfold permute_and_flip_with_replacement
  rw [hmax, hzero]
  rfl

theorem permute_and_flip_without_replacement_zero
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (index : Std.Usize)
    (hmax : max_index x = ok (some index))
    (hzero : dashu_ratio.rbig.RBig.is_zero scale = ok true) :
    permute_and_flip_without_replacement x scale =
      ok (core.result.Result.Ok index) := by
  unfold permute_and_flip_without_replacement
  rw [hmax, hzero]
  rfl

theorem permute_and_flip_zero_true
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (index : Std.Usize)
    (hmax : max_index x = ok (some index))
    (hzero : dashu_ratio.rbig.RBig.is_zero scale = ok true) :
    permute_and_flip x scale true =
      ok (core.result.Result.Ok index) := by
  rw [permute_and_flip_true_eq]
  exact permute_and_flip_with_replacement_zero x scale index hmax hzero

theorem permute_and_flip_zero_false
    (x : Slice dashu_ratio.rbig.RBig)
    (scale : dashu_ratio.rbig.RBig)
    (index : Std.Usize)
    (hmax : max_index x = ok (some index))
    (hzero : dashu_ratio.rbig.RBig.is_zero scale = ok true) :
    permute_and_flip x scale false =
      ok (core.result.Result.Ok index) := by
  rw [permute_and_flip_false_eq]
  exact permute_and_flip_without_replacement_zero x scale index hmax hzero

namespace SampCert

open SLang.PermuteAndFlip

theorem subsetPermuteAndFlipSLang_canonicalOrder_range_private
    {n : CandidateCount}
    (ε₁ : ℕ)
    (ε₂ : ℕ+) :
    RangePrivate
      (privacyBase ε₁ ε₂)
      1
      (subsetPermuteAndFlipSLang (canonicalOrder n) (by simp [canonicalOrder]) ε₁ ε₂) := by
  simpa [canonicalOrder] using
    subsetPermuteAndFlip_range_private (canonicalOrder n) (by simp [canonicalOrder]) ε₁ ε₂

theorem subsetPermuteAndFlipSLang_canonicalOrder_range_privacy
    {n : CandidateCount}
    (q q' : Scores n)
    (r : Fin n.succ)
    (ε₁ : ℕ)
    (ε₂ : ℕ+) :
    (privacyBase ε₁ ε₂) ^ rangeDistance q q' *
      subsetPermuteAndFlipSLang (canonicalOrder n) (by simp [canonicalOrder]) ε₁ ε₂ q r
      ≤
      subsetPermuteAndFlipSLang (canonicalOrder n) (by simp [canonicalOrder]) ε₁ ε₂ q' r := by
  simpa [one_mul] using
    subsetPermuteAndFlipSLang_canonicalOrder_range_private (n := n) ε₁ ε₂ q q' r

end SampCert
end OpenDP.measurements.noisy_top_k
