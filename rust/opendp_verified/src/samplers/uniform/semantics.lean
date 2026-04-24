import Aeneas
import Mathlib.Algebra.Order.Floor.Div
import src.externals.dashu
import src.externals.core_num_usize
import SampCert.Samplers.Uniform.Properties

open Aeneas Aeneas.Std Result
open OpenDP
open SLang PMF ENNReal Finset

namespace OpenDP.samplers.uniform

/-- Exact target PMF for uniform sampling below a positive Dashu bound. -/
noncomputable def uniformNatBelowPMF
    (upper : dashu_int.ubig.UBig)
    (h : 0 < dashu.ubigToNat upper) :
    PMF Nat :=
  SLang.UniformSample_PMF ⟨dashu.ubigToNat upper, h⟩

/-- Concrete witnesses produced by the extracted setup phase of
`sample_uniform_ubig_below`. Bundling them keeps the mathematical theorems
close to the Rust code without repeating the same extracted equalities. -/
structure UniformBelowSetup (upper : dashu_int.ubig.UBig) where
  bit_len : Usize
  byte_len : Usize
  shift : Usize
  one : dashu_int.ubig.UBig
  range : dashu_int.ubig.UBig
  remainder : dashu_int.ubig.UBig
  threshold : dashu_int.ubig.UBig
  hbit_len :
    dashu_int.ubig.UBig.Insts.Dashu_baseBitBitTest.bit_len upper = ok bit_len
  hbyte_len : core.num.Usize.div_ceil bit_len 8#usize = ok byte_len
  hshift : byte_len * 8#usize = ok shift
  hone : dashu_int.ubig.UBig.ONE = ok one
  hrange :
    dashu_int.ubig.UBig.Insts.CoreOpsBitShlUsizeUBig.shl one shift = ok range
  hrem :
    SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem range upper = ok remainder
  hthreshold :
    SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub range remainder = ok threshold

/-- The deterministic setup phase at the head of the extracted
`sample_uniform_ubig_below` function, packaged as one result. -/
noncomputable def sample_uniform_ubig_below_setup
    (upper : dashu_int.ubig.UBig) :
    Result (UniformBelowSetup upper) := do
  match hbit_len : dashu_int.ubig.UBig.Insts.Dashu_baseBitBitTest.bit_len upper with
  | ok bit_len =>
      match hbyte_len : core.num.Usize.div_ceil bit_len 8#usize with
      | ok byte_len =>
          match hone : dashu_int.ubig.UBig.ONE with
          | ok one =>
              match hshift : byte_len * 8#usize with
              | ok shift =>
                  match hrange : dashu_int.ubig.UBig.Insts.CoreOpsBitShlUsizeUBig.shl one shift with
                  | ok range =>
                      match hrem : SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem range upper with
                      | ok remainder =>
                          match hthreshold : SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub range remainder with
                          | ok threshold =>
                              pure {
                                bit_len := bit_len,
                                byte_len := byte_len,
                                shift := shift,
                                one := one,
                                range := range,
                                remainder := remainder,
                                threshold := threshold,
                                hbit_len := hbit_len,
                                hbyte_len := hbyte_len,
                                hshift := hshift,
                                hone := hone,
                                hrange := hrange,
                                hrem := hrem,
                                hthreshold := hthreshold
                              }
                          | fail e => fail e
                          | div => div
                      | fail e => fail e
                      | div => div
                  | fail e => fail e
                  | div => div
              | fail e => fail e
              | div => div
          | fail e => fail e
          | div => div
      | fail e => fail e
      | div => div
  | fail e => fail e
  | div => div

/-- For positive `upper`, the deterministic setup phase of
`sample_uniform_ubig_below` succeeds and produces a concrete
`UniformBelowSetup` witness. This is the remaining totality boundary for the
generated setup pipeline. -/
axiom sample_uniform_ubig_below_setup_exists
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper) :
    ∃ setup, sample_uniform_ubig_below_setup upper = ok setup

private theorem sum_range_indicator_mod
    (upper k y : Nat)
    (c : ENNReal) :
    Finset.sum (Finset.range (upper * k))
      (fun a => if y = a % upper then c else 0) =
      if y < upper then (k : ENNReal) * c else 0 := by
  classical
  induction k with
  | zero =>
      simp
  | succ k ih =>
      rw [Nat.mul_succ, Finset.sum_range_add]
      rw [ih]
      have hblock :
          Finset.sum (Finset.range upper)
            (fun b => if y = (upper * k + b) % upper then c else 0) =
            if y < upper then c else 0 := by
        by_cases hy : y < upper
        · have hmod :
              ∀ b, b ∈ Finset.range upper →
                (upper * k + b) % upper = b := by
              intro b hb
              have hb_lt : b < upper := Finset.mem_range.mp hb
              simp [Nat.add_mod, Nat.mul_mod_right, Nat.mod_eq_of_lt hb_lt]
          rw [Finset.sum_eq_single y]
          · rw [if_pos hy]
            simp [hmod y (by simpa using hy)]
          · intro b hb hyb
            rw [if_neg]
            intro hEq
            have := hmod b hb
            rw [this] at hEq
            exact hyb hEq.symm
          · intro hyNot
            exfalso
            exact hyNot (Finset.mem_range.mpr hy)
        · have hy' : upper ≤ y := Nat.not_lt.mp hy
          rw [if_neg hy]
          apply Finset.sum_eq_zero
          intro b hb
          rw [if_neg]
          intro hEq
          have hb_lt : b < upper := Finset.mem_range.mp hb
          have hmod : (upper * k + b) % upper = b := by
            simp [Nat.add_mod, Nat.mul_mod_right, Nat.mod_eq_of_lt hb_lt]
          rw [hmod] at hEq
          have hle : upper ≤ b := by simpa [hEq] using hy' 
          exact (Nat.not_le_of_lt hb_lt) hle
      by_cases hy : y < upper
      · rw [hblock]
        calc
          (if y < upper then (k : ENNReal) * c else 0) + (if y < upper then c else 0)
            = if y < upper then ((k : ENNReal) * c + c) else 0 := by simp [hy]
          _ = if y < upper then (((k : ENNReal) + 1) * c) else 0 := by
                simp [hy, add_mul, one_mul, add_comm]
          _ = if y < upper then (((k + 1 : Nat) : ENNReal) * c) else 0 := by
                simp [Nat.cast_add, hy]
      · rw [hblock]
        simp [hy]

private theorem uniformSamplePMF_mod_multiple_apply
    (upper k y : Nat)
    (hupper : 0 < upper)
    (hk : 0 < k) :
    PMF.map (fun x : ℕ => x % upper)
        (SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) y =
      if y < upper then (1 : ENNReal) / (upper : ENNReal) else 0 := by
  classical
  rw [PMF.map_apply]
  rw [tsum_eq_sum (s := Finset.range (upper * k))]
  · have hs :
        Finset.sum (Finset.range (upper * k))
          (fun a =>
            if y = a % upper then (SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) a else 0) =
        Finset.sum (Finset.range (upper * k))
          (fun a =>
            if y = a % upper then ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0) := by
      apply Finset.sum_congr rfl
      intro a ha
      by_cases hEq : y = a % upper
      · rw [if_pos hEq, if_pos hEq]
        have ha_lt : a < upper * k := Finset.mem_range.mp ha
        simpa [SLang.UniformSample_PMF] using
          (SLang.UniformSample_apply ⟨upper * k, Nat.mul_pos hupper hk⟩ a ha_lt)
      · rw [if_neg hEq, if_neg hEq]
    have hs' :
      Finset.sum (Finset.range (upper * k))
        (fun b => if y = b % upper then (SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) b else 0)
          =
        Finset.sum (Finset.range (upper * k))
          (fun b => if y = b % upper then ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0) := by
      simpa using hs
    have hmid :
      Finset.sum (Finset.range (upper * k))
        (fun b => if y = b % upper then ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0) =
      if y < upper then (k : ENNReal) * ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0 := by
      simpa using sum_range_indicator_mod upper k y ((1 : ENNReal) / ((upper * k : Nat) : ENNReal))
    have hbase :
      Finset.sum (Finset.range (upper * k))
        (fun b => if y = b % upper then (SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) b else 0) =
      if y < upper then (k : ENNReal) * ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0 := by
      exact hs'.trans hmid
    have hfinal :
      (if y < upper then (k : ENNReal) * ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) else 0) =
      (if y < upper then (1 : ENNReal) / (upper : ENNReal) else 0) := by
      by_cases hy : y < upper
      · rw [if_pos hy, if_pos hy]
        have hk0 : (k : ENNReal) ≠ 0 := by exact_mod_cast (Nat.ne_of_gt hk)
        calc
          (k : ENNReal) * ((1 : ENNReal) / ((upper * k : Nat) : ENNReal)) =
              (k : ENNReal) * (((upper : ENNReal) * (k : ENNReal))⁻¹) := by
                simp [Nat.cast_mul]
          _ = (((upper : ENNReal) * (k : ENNReal))⁻¹) * (k : ENNReal) := by
                rw [mul_comm]
          _ = ((upper : ENNReal)⁻¹ * (k : ENNReal)⁻¹) * (k : ENNReal) := by
                rw [ENNReal.mul_inv (by simp) (by simp)]
          _ = (upper : ENNReal)⁻¹ * ((k : ENNReal)⁻¹ * (k : ENNReal)) := by
                rw [mul_assoc]
          _ = (upper : ENNReal)⁻¹ * 1 := by
                rw [ENNReal.inv_mul_cancel hk0 (by simp)]
          _ = (1 : ENNReal) / (upper : ENNReal) := by simp
      · simp [hy]
    have hswitch :
      (∑ b ∈ range (upper * k),
        @ite ℝ≥0∞ (y = b % upper) (Classical.propDecidable (y = b % upper))
          ((SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) b) 0) =
      (∑ b ∈ range (upper * k),
        @ite ℝ≥0∞ (y = b % upper) (instDecidableEqNat y (b % upper))
          ((SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) b) 0) := by
      apply Finset.sum_congr rfl
      intro b hb
      by_cases hEq : y = b % upper <;> simp [hEq]
    exact hswitch.trans (hbase.trans hfinal)
  · intro a ha
    by_cases hEq : y = a % upper
    · have ha_ge : upper * k ≤ a := by simpa [Finset.mem_range, not_lt] using ha
      have hzero : (UniformSample ⟨upper * k, Nat.mul_pos hupper hk⟩) a = 0 := by
        exact SLang.UniformSample_apply_out ⟨upper * k, Nat.mul_pos hupper hk⟩ a ha_ge
      simpa [hEq, SLang.UniformSample_PMF] using hzero
    · simp [hEq]

private theorem uniformSamplePMF_mod_multiple
    (upper k : Nat)
    (hupper : 0 < upper)
    (hk : 0 < k) :
    PMF.map (fun x : ℕ => x % upper)
        (SLang.UniformSample_PMF ⟨upper * k, Nat.mul_pos hupper hk⟩) =
      SLang.UniformSample_PMF ⟨upper, hupper⟩ := by
  ext y
  rw [uniformSamplePMF_mod_multiple_apply upper k y hupper hk]
  symm
  simpa [SLang.UniformSample_PMF] using (SLang.UniformSample_apply' ⟨upper, hupper⟩ y)

/-- Internal denotational model for successful outcomes of
`sample_uniform_ubig_below`. It mirrors the rejection shape of the Rust
implementation once the concrete rejection threshold has been identified. -/
noncomputable def sample_uniform_ubig_below_success_pmf
    (upper : dashu_int.ubig.UBig)
    (threshold : dashu_int.ubig.UBig)
    (hthreshold : 0 < dashu.ubigToNat threshold) :
    PMF ℕ :=
  PMF.map (fun x : ℕ => x % dashu.ubigToNat upper)
    (SLang.UniformSample_PMF ⟨dashu.ubigToNat threshold, hthreshold⟩)

/-- The concrete `bit_len` / `div_ceil` / `* 8` computation in
`sample_uniform_ubig_below` yields a shift large enough that `2^shift`
covers any positive `upper`. -/
private theorem shift_covers_upper
    (upper : dashu_int.ubig.UBig)
    (setup : UniformBelowSetup upper)
    (hupper : 0 < dashu.ubigToNat upper)
    :
    dashu.ubigToNat upper ≤ 2 ^ setup.shift.val := by
  have hbit_cover :
      dashu.ubigToNat upper ≤ 2 ^ setup.bit_len.val := by
    exact dashu.bit_len_cover upper setup.bit_len setup.hbit_len hupper
  have hbyte_len_nat : setup.byte_len.val = setup.bit_len.val ⌈/⌉ (8 : Nat) := by
    simpa using core_num_usize.div_ceil_spec setup.bit_len 8#usize setup.byte_len setup.hbyte_len
  have hshift_nat : setup.shift.val = setup.byte_len.val * 8 := by
    have hmul := Aeneas.Std.UScalar.mul_equiv setup.byte_len 8#usize
    have hshift' : Aeneas.Std.UScalar.mul setup.byte_len 8#usize = ok setup.shift := by
      simpa using setup.hshift
    rw [hshift'] at hmul
    simpa using hmul.2.1
  have hbit_le_shift : setup.bit_len.val ≤ setup.shift.val := by
    rw [hshift_nat, hbyte_len_nat]
    simpa [nsmul_eq_mul, mul_comm] using
      (le_smul_ceilDiv (a := (8 : Nat)) (b := setup.bit_len.val) (by decide))
  exact le_trans hbit_cover (Nat.pow_le_pow_right (by decide) hbit_le_shift)

/-- The concrete `range` computed by the extracted setup phase is exactly the
byte-sized bound `256 ^ byte_len`. -/
theorem setup_range_eq_byte_range
    (upper : dashu_int.ubig.UBig)
    (setup : UniformBelowSetup upper) :
    dashu.ubigToNat setup.range = bytes.byteRadix ^ setup.byte_len.val := by
  have hone_nat : dashu.ubigToNat setup.one = 1 := dashu.one_spec setup.one setup.hone
  have hshift_nat : setup.shift.val = setup.byte_len.val * 8 := by
    have hmul := Aeneas.Std.UScalar.mul_equiv setup.byte_len 8#usize
    have hshift' : Aeneas.Std.UScalar.mul setup.byte_len 8#usize = ok setup.shift := by
      simpa using setup.hshift
    rw [hshift'] at hmul
    simpa using hmul.2.1
  rw [dashu.shl_spec setup.one setup.range setup.shift setup.hrange, hone_nat, hshift_nat]
  simp
  calc
    2 ^ (setup.byte_len.val * 8) = (2 ^ 8) ^ setup.byte_len.val := by
      rw [Nat.mul_comm, Nat.pow_mul]
    _ = bytes.byteRadix ^ setup.byte_len.val := by
      norm_num [OpenDP.bytes.byteRadix]

/-- The concrete rejection threshold computed by `sample_uniform_ubig_below`
is a positive multiple of `upper`. -/
private theorem threshold_eq_mul_div
    (upper : dashu_int.ubig.UBig)
    (range remainder threshold : dashu_int.ubig.UBig)
    (hrem :
      SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem range upper = ok remainder)
    (hthreshold :
      SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub range remainder = ok threshold) :
    dashu.ubigToNat threshold =
      dashu.ubigToNat upper * (dashu.ubigToNat range / dashu.ubigToNat upper) := by
  rw [dashu.sub_spec range remainder threshold hthreshold]
  rw [dashu.rem_spec range upper remainder hrem]
  have hmod_le : dashu.ubigToNat range % dashu.ubigToNat upper ≤ dashu.ubigToNat range := by
    exact Nat.mod_le _ _
  apply (Nat.sub_eq_iff_eq_add hmod_le).2
  simpa [Nat.add_comm] using
    (Nat.mod_add_div (dashu.ubigToNat range) (dashu.ubigToNat upper)).symm

/-- The concrete rejection threshold computed by `sample_uniform_ubig_below`
is positive whenever `upper` is positive. -/
theorem threshold_pos
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper)
    (setup : UniformBelowSetup upper) :
    0 < dashu.ubigToNat setup.threshold := by
  have hcover :
      dashu.ubigToNat upper ≤ 2 ^ setup.shift.val := by
    exact shift_covers_upper upper setup hupper
  have hone_nat : dashu.ubigToNat setup.one = 1 := dashu.one_spec setup.one setup.hone
  have hrange_nat : dashu.ubigToNat setup.range = 2 ^ setup.shift.val := by
    rw [dashu.shl_spec setup.one setup.range setup.shift setup.hrange, hone_nat]
    simp
  have hupper_le_range : dashu.ubigToNat upper ≤ dashu.ubigToNat setup.range := by
    simpa [hrange_nat]
  rw [threshold_eq_mul_div upper setup.range setup.remainder setup.threshold setup.hrem setup.hthreshold]
  have hrange_pos : 0 < dashu.ubigToNat setup.range := by
    rw [hrange_nat]
    exact pow_pos (by decide) setup.shift.val
  have hquot_pos : 0 < dashu.ubigToNat setup.range / dashu.ubigToNat upper := by
    exact Nat.div_pos hupper_le_range hupper
  exact Nat.mul_pos hupper hquot_pos

/-- Primary deterministic facts computed by the extracted setup phase of
`sample_uniform_ubig_below`: the byte-sampling range is exactly `256^byte_len`,
and the rejection threshold is strictly positive for positive `upper`. -/
theorem sample_uniform_ubig_below_setup_spec
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper)
    (setup : UniformBelowSetup upper) :
    dashu.ubigToNat setup.range = bytes.byteRadix ^ setup.byte_len.val ∧
    0 < dashu.ubigToNat setup.threshold := by
  exact ⟨setup_range_eq_byte_range upper setup, threshold_pos upper hupper setup⟩

/-- The exact uniform success law for the concrete rejection threshold computed
by `sample_uniform_ubig_below`. -/
theorem sample_uniform_ubig_below_success_pmf_eq_uniform
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper)
    (setup : UniformBelowSetup upper) :
    sample_uniform_ubig_below_success_pmf upper setup.threshold
        (threshold_pos upper hupper setup) =
      uniformNatBelowPMF upper hupper := by
  let k : Nat := dashu.ubigToNat setup.range / dashu.ubigToNat upper
  have hk : 0 < k := by
    dsimp [k]
    have hcover :
        dashu.ubigToNat upper ≤ 2 ^ setup.shift.val := by
      exact shift_covers_upper upper setup hupper
    have hone_nat : dashu.ubigToNat setup.one = 1 := dashu.one_spec setup.one setup.hone
    have hrange_nat : dashu.ubigToNat setup.range = 2 ^ setup.shift.val := by
      rw [dashu.shl_spec setup.one setup.range setup.shift setup.hrange, hone_nat]
      simp
    have hupper_le_range : dashu.ubigToNat upper ≤ dashu.ubigToNat setup.range := by
      simpa [hrange_nat]
    exact Nat.div_pos hupper_le_range hupper
  have hthreshold_nat :
      dashu.ubigToNat setup.threshold = dashu.ubigToNat upper * k := by
    simpa [k] using
      threshold_eq_mul_div upper setup.range setup.remainder setup.threshold setup.hrem setup.hthreshold
  calc
    sample_uniform_ubig_below_success_pmf upper setup.threshold
        (threshold_pos upper hupper setup)
      =
        PMF.map (fun x : ℕ => x % dashu.ubigToNat upper)
          (SLang.UniformSample_PMF ⟨dashu.ubigToNat upper * k, Nat.mul_pos hupper hk⟩) := by
            unfold sample_uniform_ubig_below_success_pmf
            simp [hthreshold_nat, k]
    _ = SLang.UniformSample_PMF ⟨dashu.ubigToNat upper, hupper⟩ := by
          simpa [k] using uniformSamplePMF_mod_multiple
            (dashu.ubigToNat upper) k hupper hk
    _ = uniformNatBelowPMF upper hupper := by
          unfold uniformNatBelowPMF
          rfl

end OpenDP.samplers.uniform
