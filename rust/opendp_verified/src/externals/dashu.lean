import Aeneas
import Generated.OpenDP.FunsExternal
import src.samplers.bytes

open Aeneas Aeneas.Std Result ControlFlow Error
open OpenDP

namespace OpenDP.dashu

/-- Mathematical interpretation of Dashu naturals used in the verified crate. -/
opaque ubigToNat : dashu_int.ubig.UBig → Nat

/-- Mathematical meaning of `UBig::ONE`. -/
axiom one_spec
  (one : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.ONE = ok one ->
    ubigToNat one = 1

/-- Totality and mathematical meaning of `UBig::ONE`. -/
axiom one_exists_spec :
  ∃ one, dashu_int.ubig.UBig.ONE = ok one ∧ ubigToNat one = 1

/-- Mathematical meaning of left-shifting a `UBig`. -/
axiom shl_spec
  (x range : dashu_int.ubig.UBig)
  (shift : Usize) :
  dashu_int.ubig.UBig.Insts.CoreOpsBitShlUsizeUBig.shl x shift = ok range ->
    ubigToNat range = ubigToNat x * 2 ^ shift.val

/-- Totality and mathematical meaning of left-shifting a `UBig`. -/
axiom shl_exists_spec
  (x : dashu_int.ubig.UBig)
  (shift : Usize) :
  ∃ range,
    dashu_int.ubig.UBig.Insts.CoreOpsBitShlUsizeUBig.shl x shift = ok range ∧
    ubigToNat range = ubigToNat x * 2 ^ shift.val

/-- Mathematical meaning of Dashu remainder. -/
axiom rem_spec
  (x y rem : dashu_int.ubig.UBig) :
  SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem x y = ok rem ->
    ubigToNat rem = ubigToNat x % ubigToNat y

/-- Totality and mathematical meaning of Dashu remainder on positive divisors. -/
axiom rem_exists_spec
  (x y : dashu_int.ubig.UBig) :
  0 < ubigToNat y ->
  ∃ rem,
    SharedLUBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem x y = ok rem ∧
    ubigToNat rem = ubigToNat x % ubigToNat y

/-- Mathematical meaning of the remainder operation used by the extracted
rejection loop body. -/
axiom rem_body_spec
  (x y rem : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem x y = ok rem ->
    ubigToNat rem = ubigToNat x % ubigToNat y

/-- Mathematical meaning of Dashu subtraction. -/
axiom sub_spec
  (x y z : dashu_int.ubig.UBig) :
  SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub x y = ok z ->
    ubigToNat z = ubigToNat x - ubigToNat y

/-- Totality and mathematical meaning of Dashu subtraction when the right-hand
side is at most the left-hand side. -/
axiom sub_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ubigToNat y ≤ ubigToNat x ->
  ∃ z,
    SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub x y = ok z ∧
    ubigToNat z = ubigToNat x - ubigToNat y

/-- Mathematical meaning of Dashu `bit_len` for positive inputs. -/
axiom bit_len_cover
  (upper : dashu_int.ubig.UBig)
  (bit_len : Usize) :
  dashu_int.ubig.UBig.Insts.Dashu_baseBitBitTest.bit_len upper = ok bit_len ->
    0 < ubigToNat upper ->
    ubigToNat upper ≤ 2 ^ bit_len.val

/-- For positive inputs, the `bit_len`-driven setup arithmetic of
`sample_uniform_ubig_below` succeeds through the `* 8` step. -/
axiom bit_len_setup_exists_spec
  (upper : dashu_int.ubig.UBig) :
  0 < ubigToNat upper ->
  ∃ bit_len byte_len shift,
    dashu_int.ubig.UBig.Insts.Dashu_baseBitBitTest.bit_len upper = ok bit_len ∧
    core.num.Usize.div_ceil bit_len 8#usize = ok byte_len ∧
    byte_len * 8#usize = ok shift ∧
    ubigToNat upper ≤ 2 ^ bit_len.val

/-- Mathematical meaning of a successful strict-less-than comparison. -/
axiom lt_true_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt x y = ok true ->
    ubigToNat x < ubigToNat y

/-- Mathematical meaning of a failed strict-less-than comparison. -/
axiom lt_false_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt x y = ok false ->
    ubigToNat y ≤ ubigToNat x

/-- Mathematical meaning of `UBig::from_be_bytes`. -/
axiom from_be_bytes_spec
  (buffer : Slice Std.U8)
  (out : dashu_int.ubig.UBig) :
  dashu_int.convert.UBig.from_be_bytes buffer = ok out ->
    ubigToNat out = bytes.beBytesToNat buffer.val

end OpenDP.dashu
