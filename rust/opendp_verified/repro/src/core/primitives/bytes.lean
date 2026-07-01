import Aeneas
import SampCert.Samplers.Uniform.Properties

/-!
# Core primitive: byte encoding

The hardware-facing reference distribution. `fill_bytes` (the lone stochastic
external) draws a buffer of `n` uniform bytes; we view that buffer as a
big-endian natural and `uniformByteNatPMF n` is its exact law. Everything
distributional flows from here.

This file is part of `core/primitives` — the foundational Lean tools the proofs
are built from. It has no dependencies beyond Aeneas + SampCert and is pure
definitions (no proofs to maintain).
-/

open Aeneas Aeneas.Std
open SLang PMF ENNReal

namespace OpenDP.Core.Bytes

/-- The cardinality of the byte alphabet. -/
@[reducible] def byteRadix : Nat := 256

/-- Interpret a list of bytes as a big-endian natural number. -/
def beBytesToNat (bytes : List Std.U8) : Nat :=
  bytes.foldl (fun acc b => byteRadix * acc + b.val) 0

/-- Exact target PMF for `n` fresh uniform bytes, viewed through their
big-endian natural encoding: uniform on `[0, 256^n)`. -/
noncomputable def uniformByteNatPMF (n : Nat) : PMF Nat :=
  SLang.UniformSample_PMF ⟨byteRadix ^ n, pow_pos (by decide) n⟩

end OpenDP.Core.Bytes
