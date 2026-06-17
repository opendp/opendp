import Aeneas
import Generated.OpenDP
import src.externals.dashu

open Aeneas Aeneas.Std Result ControlFlow Error
open OpenDP

namespace OpenDP.utilities

/-- When `b` is mathematically zero, the generated gcd loop body stops and
returns the current accumulator. -/
theorem gcd_ubig_loop_body_eq_done
    (a b : dashu_int.ubig.UBig)
    (hzero : dashu_int.ubig.UBig.is_zero b = ok true) :
    utilities.gcd_ubig_loop.body a b = ok (ControlFlow.done a) := by
  unfold utilities.gcd_ubig_loop.body
  simp [hzero]

/-- When `b` is mathematically positive, the generated gcd loop body continues
with the Euclidean remainder state `(b, a % b)`. -/
theorem gcd_ubig_loop_body_eq_cont
    (a b r : dashu_int.ubig.UBig)
    (hzero : dashu_int.ubig.UBig.is_zero b = ok false)
    (hrem : SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem a b = ok r) :
    utilities.gcd_ubig_loop.body a b = ok (ControlFlow.cont (b, r)) := by
  unfold utilities.gcd_ubig_loop.body
  simp [hzero, hrem]

/-- Functional correctness of the extracted Euclidean gcd utility. -/
theorem gcd_ubig_loop_spec
    (a0 b0 : dashu_int.ubig.UBig) :
    utilities.gcd_ubig_loop a0 b0 ⦃ g =>
      dashu.ubigToNat g = Nat.gcd (dashu.ubigToNat a0) (dashu.ubigToNat b0) ⦄ := by
  let target := Nat.gcd (dashu.ubigToNat a0) (dashu.ubigToNat b0)
  let inv : dashu_int.ubig.UBig × dashu_int.ubig.UBig → Prop :=
    fun st => Nat.gcd (dashu.ubigToNat st.1) (dashu.ubigToNat st.2) = target
  unfold utilities.gcd_ubig_loop
  apply
    Aeneas.Std.loop.spec_decr_nat
      (measure := fun st : dashu_int.ubig.UBig × dashu_int.ubig.UBig =>
        dashu.ubigToNat st.2)
      (inv := inv)
      (post := fun g => dashu.ubigToNat g = target)
  · intro st hInv
    rcases st with ⟨a, b⟩
    rcases dashu.is_zero_exists_spec b with ⟨bz, hbz⟩
    cases bz
    · rcases dashu.rem_exists_spec a b (dashu.is_zero_false_spec b hbz) with
        ⟨r, hrem, hrnat⟩
      simp [utilities.gcd_ubig_loop.body, hbz, hrem, Aeneas.Std.WP.spec_ok]
      constructor
      · calc
          Nat.gcd (dashu.ubigToNat b) (dashu.ubigToNat r)
            = Nat.gcd (dashu.ubigToNat b) (dashu.ubigToNat a % dashu.ubigToNat b) := by
                rw [hrnat]
          _ = Nat.gcd (dashu.ubigToNat a % dashu.ubigToNat b) (dashu.ubigToNat b) := by
                rw [Nat.gcd_comm]
          _ = Nat.gcd (dashu.ubigToNat a) (dashu.ubigToNat b) := by
                rw [← Nat.gcd_rec, Nat.gcd_comm]
          _ = target := hInv
      · calc
          dashu.ubigToNat r = dashu.ubigToNat a % dashu.ubigToNat b := hrnat
          _ < dashu.ubigToNat b := Nat.mod_lt _ (dashu.is_zero_false_spec b hbz)
    · simp [utilities.gcd_ubig_loop.body, hbz, Aeneas.Std.WP.spec_ok]
      calc
        dashu.ubigToNat a = Nat.gcd (dashu.ubigToNat a) 0 := by simp
        _ = Nat.gcd (dashu.ubigToNat a) (dashu.ubigToNat b) := by
          rw [dashu.is_zero_true_spec b hbz]
        _ = target := hInv
  · exact rfl

/-- Primary mathematical result for the Rust gcd utility. -/
theorem gcd_ubig_spec
    (a b : dashu_int.ubig.UBig) :
    utilities.gcd_ubig a b ⦃ g =>
      dashu.ubigToNat g = Nat.gcd (dashu.ubigToNat a) (dashu.ubigToNat b) ⦄ := by
  simpa [utilities.gcd_ubig] using gcd_ubig_loop_spec a b

/-- Semantic reduction for the positive-input branch of
`div_rbig_by_ubig_exact`. Once the external Dashu calls are fixed, the
extracted utility returns the rational with the expected reduced numerator and
scaled denominator parts. -/
theorem div_rbig_by_ubig_exact_eq_of_positive_steps
    (numer denom k numer' k' g nRed kRed denom' : dashu_int.ubig.UBig)
    (i : dashu_int.ibig.IBig)
    (x : dashu_ratio.rbig.RBig)
    (hk : 0 < dashu.ubigToNat k)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hnumer : 0 < dashu.ubigToNat numer)
    (hcloneNumer :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone numer = ok numer')
    (hcloneK :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone k = ok k')
    (hgcd : utilities.gcd_ubig numer' k' = ok g)
    (hdivNumer :
      SharedLUBig.Insts.CoreOpsArithDivSharedRUBigUBig.div numer g = ok nRed)
    (hdivK :
      SharedLUBig.Insts.CoreOpsArithDivUBigUBig.div k g = ok kRed)
    (hinto :
      core.convert.IntoFrom.into dashu_int.ibig.IBig.Insts.CoreConvertFromUBig nRed = ok i)
    (hmul :
      SharedLUBig.Insts.CoreOpsArithMulUBigUBig.mul denom kRed = ok denom')
    (hparts :
      dashu_ratio.rbig.RBig.from_parts i denom' = ok x) :
    utilities.div_rbig_by_ubig_exact numer denom k = ok x ∧
    dashu_ratio.rbig.RBig.into_parts x = ok (i, denom') ∧
    dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, nRed) ∧
    dashu.ubigToNat g = Nat.gcd (dashu.ubigToNat numer) (dashu.ubigToNat k) ∧
    dashu.ubigToNat nRed = dashu.ubigToNat numer / dashu.ubigToNat g ∧
    dashu.ubigToNat kRed = dashu.ubigToNat k / dashu.ubigToNat g ∧
    dashu.ubigToNat denom' =
      dashu.ubigToNat denom * (dashu.ubigToNat k / dashu.ubigToNat g) := by
  have hkZero : dashu_int.ubig.UBig.is_zero k = ok false :=
    dashu.is_zero_of_pos_spec k hk
  have hnumerZero : dashu_int.ubig.UBig.is_zero numer = ok false :=
    dashu.is_zero_of_pos_spec numer hnumer
  have hcloneNumerNat : dashu.ubigToNat numer' = dashu.ubigToNat numer :=
    dashu.clone_spec numer numer' hcloneNumer
  have hcloneKNat : dashu.ubigToNat k' = dashu.ubigToNat k :=
    dashu.clone_spec k k' hcloneK
  have hgcdNat' : dashu.ubigToNat g = Nat.gcd (dashu.ubigToNat numer') (dashu.ubigToNat k') := by
    have := gcd_ubig_spec numer' k'
    rw [hgcd] at this
    simpa [Aeneas.Std.WP.spec_ok] using this
  have hgcdNat : dashu.ubigToNat g = Nat.gcd (dashu.ubigToNat numer) (dashu.ubigToNat k) := by
    rw [hgcdNat', hcloneNumerNat, hcloneKNat]
  have hgcdDvdNumer : dashu.ubigToNat g ∣ dashu.ubigToNat numer := by
    rw [hgcdNat]
    exact Nat.gcd_dvd_left _ _
  have hgcdDvdK : dashu.ubigToNat g ∣ dashu.ubigToNat k := by
    rw [hgcdNat]
    exact Nat.gcd_dvd_right _ _
  have hdivNumerNat :
      dashu.ubigToNat nRed = dashu.ubigToNat numer / dashu.ubigToNat g :=
    dashu.div_shared_spec numer g nRed hdivNumer hgcdDvdNumer
  have hdivKNat :
      dashu.ubigToNat kRed = dashu.ubigToNat k / dashu.ubigToNat g :=
    dashu.div_spec k g kRed hdivK hgcdDvdK
  have hgcdPos : 0 < dashu.ubigToNat g := by
    rw [hgcdNat]
    exact Nat.gcd_pos_of_pos_right _ hk
  have hintoParts :
      dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, nRed) :=
    dashu.ibig_from_ubig_spec nRed i hinto
  have hkRedPos : 0 < dashu.ubigToNat kRed := by
    rw [hdivKNat]
    exact Nat.div_pos (Nat.le_of_dvd hk hgcdDvdK) hgcdPos
  have hmulNat :
      dashu.ubigToNat denom' = dashu.ubigToNat denom * dashu.ubigToNat kRed :=
    dashu.mul_spec denom kRed denom' hmul
  have hpartsInto :
      dashu_ratio.rbig.RBig.into_parts x = ok (i, denom') := by
    apply dashu.rbig_from_parts_positive_spec nRed denom' i x
    · rw [hmulNat]
      exact Nat.mul_pos hdenom hkRedPos
    · exact hintoParts
    · exact hparts
  refine ⟨?_, hpartsInto, hintoParts, hgcdNat, hdivNumerNat, hdivKNat, ?_⟩
  · unfold utilities.div_rbig_by_ubig_exact
    simp [core.convert.IntoFrom.into, hkZero, hnumerZero, hcloneNumer, hcloneK, hgcd,
      hdivNumer, hdivK, hmul]
    rw [show dashu_int.ibig.IBig.Insts.CoreConvertFromUBig.from nRed = ok i from hinto]
    simpa [hparts]
  · calc
      dashu.ubigToNat denom' = dashu.ubigToNat denom * dashu.ubigToNat kRed := hmulNat
      _ = dashu.ubigToNat denom * (dashu.ubigToNat k / dashu.ubigToNat g) := by
            rw [hdivKNat]

end OpenDP.utilities
