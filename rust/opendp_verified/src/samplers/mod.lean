import src.externals.openssl_rand
import Generated.OpenDP

open Aeneas Aeneas.Std Result
open OpenDP

namespace OpenDP.samplers

/-- The verified crate's `fill_bytes` wrapper preserves the OpenSSL uniform-byte contract while
translating the foreign error type into the local error type. Successful calls produce uniform
bytes, and failures are marked as `EntropyExhausted` with data-independent semantics. -/
private theorem fill_bytes_external_spec
    (buffer : Slice Std.U8) :
    fill_bytes buffer ⦃ r buffer' =>
      buffer'.length = buffer.length ∧
      ((r = core.result.Result.Ok () ∧
        openssl.rand.UniformBytes buffer.length buffer') ∨
       (∃ err, r = core.result.Result.Err err ∧
         err.variant = error.ErrorVariant.EntropyExhausted ∧
         openssl.rand.DataIndependentFailure buffer.length)) ⦄ := by
  unfold fill_bytes
  apply Aeneas.Std.WP.spec_bind
  · exact openssl.rand.rand_bytes_uniform_spec buffer
  · intro x hx
    rcases x with ⟨r, buffer'⟩
    rcases hx with ⟨hlen, hspec⟩
    rcases hspec with hsuccess | hfailure
    · rcases hsuccess with ⟨hr, huniform⟩
      cases hr
      simp [hlen, huniform]
    · rcases hfailure with ⟨openssl_error, hr, hdata_indep⟩
      cases hr
      simp [core.fmt.rt.Argument.new_debug, core.fmt.Arguments.new]
      apply Aeneas.Std.WP.spec_bind
      · exact alloc.fmt.format_spec ()
      · intro s _
        apply Aeneas.Std.WP.spec_bind
        · exact core.hint.must_use_spec s
        · intro s1 hs1
          subst hs1
          apply Aeneas.Std.WP.spec_bind
          · exact std.backtrace.Backtrace.capture_spec
          · intro b _
            simp [hlen, hdata_indep]

/-- Auditable exact form of the byte-source contract used downstream: on
success, the returned bytes encode a uniform natural on `[0, 256^n)`. -/
theorem fill_bytes_spec
    (buffer : Slice Std.U8) :
    fill_bytes buffer ⦃ r buffer' =>
      buffer'.length = buffer.length ∧
      ((r = core.result.Result.Ok () ∧
        bytes.beBytesToNat buffer'.val < bytes.byteRadix ^ buffer.length ∧
        bytes.uniformByteNatPMF buffer.length (bytes.beBytesToNat buffer'.val) =
          (1 : ENNReal) / ((bytes.byteRadix ^ buffer.length : Nat) : ENNReal)) ∨
       (∃ err, r = core.result.Result.Err err ∧
         err.variant = error.ErrorVariant.EntropyExhausted)) ⦄ := by
  apply Aeneas.Std.WP.spec_mono' (fill_bytes_external_spec buffer)
  intro x hx
  rcases hx with ⟨hlen, hsuccess | hfailure⟩
  · rcases hsuccess with ⟨hr, _, hlt, hpmf⟩
    exact ⟨hlen, Or.inl ⟨hr, hlt, hpmf⟩⟩
  · rcases hfailure with ⟨err, hr, hvariant, _⟩
    exact ⟨hlen, Or.inr ⟨err, hr, hvariant⟩⟩

end OpenDP.samplers
