import Aeneas
import SampCert.Samplers.Uniform.Properties

open Aeneas Aeneas.Std
open SLang PMF ENNReal

namespace OpenDP.bytes

/-- The cardinality of the byte alphabet. -/
@[reducible] def byteRadix : Nat := 256

/-- Interpret a list of bytes as a big-endian natural number. -/
def beBytesToNat (bytes : List Std.U8) : Nat :=
  bytes.foldl (fun acc b => byteRadix * acc + b.val) 0

/-- Exact target PMF for `n` fresh uniform bytes, viewed through their
big-endian natural encoding. -/
noncomputable def uniformByteNatPMF (n : Nat) : PMF Nat :=
  SLang.UniformSample_PMF ⟨byteRadix ^ n, pow_pos (by decide) n⟩

end OpenDP.bytes
