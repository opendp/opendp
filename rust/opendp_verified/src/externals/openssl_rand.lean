import Aeneas
import Generated.OpenDP.FunsExternal
import src.samplers.bytes

open Aeneas Aeneas.Std Result ControlFlow Error
open OpenDP
open SLang PMF ENNReal

/-- A successful `rand_bytes` call returns a big-endian natural uniformly
distributed on `[0, 256^n)`, encoded as a slice of length `n`. -/
def openssl.rand.UniformBytes (n : Nat) (buffer : Slice Std.U8) : Prop :=
  buffer.length = n ∧
  bytes.beBytesToNat buffer.val < bytes.byteRadix ^ n ∧
  bytes.uniformByteNatPMF n (bytes.beBytesToNat buffer.val) =
    (1 : ENNReal) / ((bytes.byteRadix ^ n : Nat) : ENNReal)

/-- Entropy-source failure depends only on the public requested length. -/
def openssl.rand.DataIndependentFailure (_ : Nat) : Prop := True

/-- Semantic contract for the OpenSSL randomness boundary used by the verified crate. -/
axiom openssl.rand.rand_bytes_uniform_spec
  (buffer : Slice Std.U8) :
  openssl.rand.rand_bytes buffer ⦃ r buffer' =>
    buffer'.length = buffer.length ∧
    ((r = core.result.Result.Ok () ∧
      openssl.rand.UniformBytes buffer.length buffer') ∨
     (∃ err, r = core.result.Result.Err err ∧
       openssl.rand.DataIndependentFailure buffer.length)) ⦄
