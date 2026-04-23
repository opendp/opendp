import Aeneas
import Generated.OpenDP
import src.samplers.uniform.semantics

open Aeneas Aeneas.Std Result
open OpenDP

namespace OpenDP.samplers.uniform

/-- Once the deterministic setup phase succeeds, the extracted function reduces
to the expected allocation step followed by the rejection loop parameterized by
the computed threshold. -/
theorem sample_uniform_ubig_below_eq_of_setup
    (upper : dashu_int.ubig.UBig)
    (setup : UniformBelowSetup upper)
    (hsetup : sample_uniform_ubig_below_setup upper = ok setup) :
    samplers.uniform.sample_uniform_ubig_below upper =
      (do
        let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 setup.byte_len
        samplers.uniform.sample_uniform_ubig_below_loop upper setup.threshold buffer) := by
  let _ := hsetup
  unfold samplers.uniform.sample_uniform_ubig_below
  simp [setup.hbit_len, setup.hbyte_len, setup.hone, setup.hshift, setup.hrange,
    setup.hrem, setup.hthreshold]

/-- The exact success law induced by `sample_uniform_ubig_below` once its
deterministic setup phase has produced the concrete rejection threshold. -/
private noncomputable def sample_uniform_ubig_below_law
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper) :
    Result (PMF Nat) := do
  let setup ← sample_uniform_ubig_below_setup upper
  let hthreshold := (sample_uniform_ubig_below_setup_spec upper hupper setup).2
  pure <|
    sample_uniform_ubig_below_success_pmf upper setup.threshold hthreshold

/-- Primary mathematical result for `sample_uniform_ubig_below`: whenever its
deterministic setup phase succeeds, the induced success distribution is
exactly uniform on `[0, upper)`. -/
theorem sample_uniform_ubig_below_law_spec
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper)
    {setup : UniformBelowSetup upper}
    (hsetup : sample_uniform_ubig_below_setup upper = ok setup) :
    sample_uniform_ubig_below_law upper hupper =
      ok (uniformNatBelowPMF upper hupper) := by
  unfold sample_uniform_ubig_below_law
  rw [hsetup]
  simp [sample_uniform_ubig_below_success_pmf_eq_uniform upper hupper setup]

/-- Conditional end-to-end specification for `sample_uniform_ubig_below`: once
the deterministic setup phase succeeds, the extracted function reduces to the
expected rejection loop instance, and the induced success distribution is
exactly uniform on `[0, upper)`. -/
theorem sample_uniform_ubig_below_spec_of_setup
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper)
    {setup : UniformBelowSetup upper}
    (hsetup : sample_uniform_ubig_below_setup upper = ok setup) :
    samplers.uniform.sample_uniform_ubig_below upper =
      (do
        let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 setup.byte_len
        samplers.uniform.sample_uniform_ubig_below_loop upper setup.threshold buffer) ∧
    sample_uniform_ubig_below_law upper hupper =
      ok (uniformNatBelowPMF upper hupper) := by
  exact ⟨
    sample_uniform_ubig_below_eq_of_setup upper setup hsetup,
    sample_uniform_ubig_below_law_spec upper hupper hsetup
  ⟩

/-- Final end-to-end specification for `sample_uniform_ubig_below`: for any
positive `upper`, the extracted function has a concrete setup witness, reduces
to the corresponding rejection loop instance, and its induced success
distribution is exactly uniform on `[0, upper)`. -/
theorem sample_uniform_ubig_below_spec
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper) :
    ∃ setup : UniformBelowSetup upper,
      sample_uniform_ubig_below_setup upper = ok setup ∧
      samplers.uniform.sample_uniform_ubig_below upper =
        (do
          let buffer ← alloc.vec.from_elem core.clone.CloneU8 0#u8 setup.byte_len
          samplers.uniform.sample_uniform_ubig_below_loop upper setup.threshold buffer) ∧
      sample_uniform_ubig_below_law upper hupper =
        ok (uniformNatBelowPMF upper hupper) := by
  rcases sample_uniform_ubig_below_setup_exists upper hupper with ⟨setup, hsetup⟩
  refine ⟨setup, hsetup, ?_, ?_⟩
  · exact sample_uniform_ubig_below_eq_of_setup upper setup hsetup
  · exact sample_uniform_ubig_below_law_spec upper hupper hsetup

end OpenDP.samplers.uniform
