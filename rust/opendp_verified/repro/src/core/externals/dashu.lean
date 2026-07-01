import Aeneas
import Generated.OpenDP
import src.core.primitives.bytes

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

/-- Totality and mathematical meaning of the remainder operation used by the
extracted rejection loop body (the `owned % &ref` impl, a distinct opaque
extraction constant from `SharedLUBig...rem` above). Hypothesized on positive
divisors; the value form follows by determinism of `Result` operations. -/
axiom rem_body_exists_spec
  (x y : dashu_int.ubig.UBig) :
  0 < ubigToNat y ->
  ∃ rem,
    dashu_int.ubig.UBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem x y = ok rem ∧
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
    ubigToNat out = OpenDP.Core.Bytes.beBytesToNat buffer.val

/-- Totality and mathematical meaning of `UBig::from_be_bytes`. -/
axiom from_be_bytes_exists_spec
  (buffer : Slice Std.U8) :
  ∃ out : dashu_int.ubig.UBig,
    dashu_int.convert.UBig.from_be_bytes buffer = ok out ∧
    ubigToNat out = OpenDP.Core.Bytes.beBytesToNat buffer.val

/-- Totality of `UBig` less-than comparison. -/
axiom lt_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ∃ b : Bool,
    dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt x y = ok b

/-- Dashu UBig value 0 has ubigToNat value 0. -/
axiom zero_spec
  (zero : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.ZERO = ok zero ->
    ubigToNat zero = 0

/-- Totality and mathematical meaning of Dashu zero. -/
axiom zero_exists_spec :
  ∃ zero, dashu_int.ubig.UBig.ZERO = ok zero ∧ ubigToNat zero = 0

/-- Totality of `UBig::is_zero`. -/
axiom is_zero_exists_spec
  (x : dashu_int.ubig.UBig) :
  ∃ b, dashu_int.ubig.UBig.is_zero x = ok b

/-- Mathematical meaning of a successful `is_zero` check returning `true`. -/
axiom is_zero_true_spec
  (x : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.is_zero x = ok true ->
    ubigToNat x = 0

/-- Mathematical meaning of a successful `is_zero` check returning `false`. -/
axiom is_zero_false_spec
  (x : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.is_zero x = ok false ->
    0 < ubigToNat x

/-- Positive Dashu naturals report `false` to `is_zero`. -/
axiom is_zero_of_pos_spec
  (x : dashu_int.ubig.UBig) :
  0 < ubigToNat x ->
    dashu_int.ubig.UBig.is_zero x = ok false

/-- Mathematical meaning of Dashu addition assignment. -/
axiom add_assign_spec
  (x y z : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign x y = ok z ->
    ubigToNat z = ubigToNat x + ubigToNat y

/-- Dashu addition assignment always succeeds. -/
axiom add_assign_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ∃ z,
    dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign x y = ok z ∧
    ubigToNat z = ubigToNat x + ubigToNat y

/-- Cloning a Dashu `UBig` preserves its mathematical value. -/
axiom clone_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCloneClone.clone x = ok y ->
    ubigToNat y = ubigToNat x

/-- Cloning a Dashu `UBig` succeeds. -/
axiom clone_exists_spec
  (x : dashu_int.ubig.UBig) :
  ∃ y,
    dashu_int.ubig.UBig.Insts.CoreCloneClone.clone x = ok y ∧
    ubigToNat y = ubigToNat x

/-- Mathematical meaning of exact division by a shared positive divisor. -/
axiom div_shared_spec
  (x y q : dashu_int.ubig.UBig) :
  SharedLUBig.Insts.CoreOpsArithDivSharedRUBigUBig.div x y = ok q ->
    ubigToNat y ∣ ubigToNat x ->
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Mathematical meaning of exact division by a positive divisor. -/
axiom div_spec
  (x y q : dashu_int.ubig.UBig) :
  SharedLUBig.Insts.CoreOpsArithDivUBigUBig.div x y = ok q ->
  ubigToNat y ∣ ubigToNat x ->
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Mathematical meaning of exact division by a positive divisor for the
direct `UBig / UBig` operation emitted by Aeneas. -/
axiom div_ubig_spec
  (x y q : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div x y = ok q ->
  ubigToNat y ∣ ubigToNat x ->
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Exact shared-operand division succeeds on divisible inputs. -/
axiom div_shared_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ubigToNat y ∣ ubigToNat x ->
  ∃ q,
    SharedLUBig.Insts.CoreOpsArithDivSharedRUBigUBig.div x y = ok q ∧
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Exact division succeeds on divisible inputs. -/
axiom div_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ubigToNat y ∣ ubigToNat x ->
  ∃ q,
    SharedLUBig.Insts.CoreOpsArithDivUBigUBig.div x y = ok q ∧
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Exact direct `UBig / UBig` division succeeds on divisible inputs. -/
axiom div_ubig_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ubigToNat y ∣ ubigToNat x ->
  ∃ q,
    dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div x y = ok q ∧
    ubigToNat q = ubigToNat x / ubigToNat y

/-- Mathematical meaning of Dashu multiplication. -/
axiom mul_spec
  (x y z : dashu_int.ubig.UBig) :
  SharedLUBig.Insts.CoreOpsArithMulUBigUBig.mul x y = ok z ->
    ubigToNat z = ubigToNat x * ubigToNat y

/-- Mathematical meaning of the direct `UBig * UBig` operation emitted by
Aeneas. -/
axiom mul_ubig_spec
  (x y z : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul x y = ok z ->
    ubigToNat z = ubigToNat x * ubigToNat y

/-- Dashu multiplication succeeds. -/
axiom mul_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ∃ z,
    SharedLUBig.Insts.CoreOpsArithMulUBigUBig.mul x y = ok z ∧
    ubigToNat z = ubigToNat x * ubigToNat y

/-- Direct `UBig * UBig` multiplication succeeds. -/
axiom mul_ubig_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ∃ z,
    dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul x y = ok z ∧
    ubigToNat z = ubigToNat x * ubigToNat y

/-- Mathematical meaning of the direct `UBig + UBig` operation emitted by
Aeneas. -/
axiom add_ubig_spec
  (x y z : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add x y = ok z ->
    ubigToNat z = ubigToNat x + ubigToNat y

/-- Direct `UBig + UBig` addition succeeds. -/
axiom add_ubig_exists_spec
  (x y : dashu_int.ubig.UBig) :
  ∃ z,
    dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add x y = ok z ∧
    ubigToNat z = ubigToNat x + ubigToNat y

/-- Converting a `UBig` into an `IBig` yields a positive signed value with the
same natural magnitude. -/
axiom ibig_from_ubig_spec
  (u : dashu_int.ubig.UBig) (i : dashu_int.ibig.IBig) :
  core.convert.IntoFrom.into dashu_int.ibig.IBig.Insts.CoreConvertFromUBig u = ok i ->
    dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u)

/-- `UBig::as_ibig` yields a positive signed value with the same natural
magnitude. -/
axiom as_ibig_spec
  (u : dashu_int.ubig.UBig) (i : dashu_int.ibig.IBig) :
  dashu_int.convert.UBig.as_ibig u = ok i ->
    dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u)

/-- Converting a `UBig` into an `IBig` succeeds. -/
axiom ibig_from_ubig_exists_spec
  (u : dashu_int.ubig.UBig) :
  ∃ i,
    core.convert.IntoFrom.into dashu_int.ibig.IBig.Insts.CoreConvertFromUBig u = ok i ∧
    dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, u)

/-- `IBig::clone` preserves its signed parts. -/
axiom ibig_clone_parts_spec
  (i i' : dashu_int.ibig.IBig)
  (parts : dashu_base.sign.Sign × dashu_int.ubig.UBig) :
  dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i' ->
  dashu_int.ibig.IBig.into_parts i = ok parts ->
    dashu_int.ibig.IBig.into_parts i' = ok parts

/-- Constructing an `RBig` from positive numerator parts preserves those parts
when the denominator is positive. -/
axiom rbig_from_parts_positive_spec
  (n d : dashu_int.ubig.UBig)
  (i : dashu_int.ibig.IBig)
  (x : dashu_ratio.rbig.RBig) :
  0 < ubigToNat d ->
  dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, n) ->
  dashu_ratio.rbig.RBig.from_parts i d = ok x ->
    dashu_ratio.rbig.RBig.into_parts x = ok (i, d)

/-- Constructing an `RBig` from positive numerator parts succeeds on positive
denominators. -/
axiom rbig_from_parts_positive_exists_spec
  (n d : dashu_int.ubig.UBig)
  (i : dashu_int.ibig.IBig) :
  0 < ubigToNat d ->
  dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, n) ->
  ∃ x,
    dashu_ratio.rbig.RBig.from_parts i d = ok x ∧
    dashu_ratio.rbig.RBig.into_parts x = ok (i, d)

/-- Mathematical zero rational setup. -/
axiom rbig_zero_setup_spec :
  ∃ x : dashu_ratio.rbig.RBig,
    ∃ i : dashu_int.ibig.IBig,
      ∃ z d : dashu_int.ubig.UBig,
        dashu_ratio.rbig.RBig.ZERO = ok x ∧
        dashu_int.ubig.UBig.ZERO = ok z ∧
        dashu_int.ubig.UBig.ONE = ok d ∧
        dashu_ratio.rbig.RBig.into_parts x = ok (i, d) ∧
        dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, z)

/-- Mathematical meaning of `RBig::is_zero` on a zero rational. -/
axiom rbig_is_zero_true_spec
  (x : dashu_ratio.rbig.RBig)
  (i : dashu_int.ibig.IBig)
  (z d : dashu_int.ubig.UBig) :
  dashu_ratio.rbig.RBig.ZERO = ok x ->
  dashu_ratio.rbig.RBig.into_parts x = ok (i, d) ->
  dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, z) ->
  dashu_int.ubig.UBig.ZERO = ok z ->
  dashu_ratio.rbig.RBig.is_zero x = ok true

/-- Mathematical meaning of `RBig::is_zero` on a positive rational. -/
axiom rbig_is_zero_false_spec
  (x : dashu_ratio.rbig.RBig)
  (i : dashu_int.ibig.IBig)
  (numer d : dashu_int.ubig.UBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (i, d) ->
  dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, numer) ->
  0 < ubigToNat numer ->
  dashu_ratio.rbig.RBig.is_zero x = ok false

/-- Mathematical one rational setup. -/
axiom rbig_one_setup_spec :
  ∃ x : dashu_ratio.rbig.RBig,
    ∃ i : dashu_int.ibig.IBig,
      ∃ one : dashu_int.ubig.UBig,
        dashu_ratio.rbig.RBig.ONE = ok x ∧
        dashu_int.ubig.UBig.ONE = ok one ∧
        dashu_ratio.rbig.RBig.into_parts x = ok (i, one) ∧
        dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, one)

/-- Constructing the constant positive rational `1 / 1` yields a rational with
matching positive numerator and denominator parts. -/
axiom rbig_from_parts_const_one_spec
  (u128one : Std.U128)
  (x : dashu_ratio.rbig.RBig) :
  u128one.val = 1 ->
  dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive u128one u128one = ok x ->
  ∃ i : dashu_int.ibig.IBig,
    ∃ one : dashu_int.ubig.UBig,
      dashu_int.ubig.UBig.ONE = ok one ∧
      dashu_ratio.rbig.RBig.into_parts x = ok (i, one) ∧
      dashu_int.ibig.IBig.into_parts i = ok (dashu_base.sign.Sign.Positive, one)

/-- For nonnegative rationals, `x > 1` means the positive numerator is strictly
larger than the positive denominator. -/
axiom rbig_gt_one_true_spec
  (x : dashu_ratio.rbig.RBig)
  (numerSigned : dashu_int.ibig.IBig)
  (denom numer : dashu_int.ubig.UBig)
  (oneRat : dashu_ratio.rbig.RBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) ->
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) ->
  dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok true ->
    ubigToNat denom < ubigToNat numer

/-- For nonnegative rationals, failing the `x > 1` test means the positive
numerator is at most the positive denominator. -/
axiom rbig_gt_one_false_spec
  (x : dashu_ratio.rbig.RBig)
  (numerSigned : dashu_int.ibig.IBig)
  (denom numer : dashu_int.ubig.UBig)
  (oneRat : dashu_ratio.rbig.RBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) ->
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) ->
  dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok false ->
    ubigToNat numer ≤ ubigToNat denom

/-- For nonnegative rationals, if the positive numerator is at most the
positive denominator then the `x > 1` test returns `false`. -/
axiom rbig_gt_one_false_of_le_spec
  (x : dashu_ratio.rbig.RBig)
  (numerSigned : dashu_int.ibig.IBig)
  (denom numer : dashu_int.ubig.UBig)
  (oneRat : dashu_ratio.rbig.RBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) ->
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) ->
  ubigToNat numer ≤ ubigToNat denom ->
  dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok false

/-- Subtracting `1` from a nonnegative rational with numerator at least the
denominator preserves the denominator and subtracts it from the numerator. -/
axiom rbig_sub_one_positive_spec
  (x oneRat x' : dashu_ratio.rbig.RBig)
  (numerSigned numerSigned' : dashu_int.ibig.IBig)
  (numer denom numer' one : dashu_int.ubig.UBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) ->
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) ->
  dashu_int.ubig.UBig.ONE = ok one ->
  dashu_ratio.rbig.RBig.into_parts oneRat = ok (numerSigned', one) ->
  dashu_int.ibig.IBig.into_parts numerSigned' = ok (dashu_base.sign.Sign.Positive, one) ->
  ubigToNat denom ≤ ubigToNat numer ->
  dashu_ratio.rbig.RBig.Insts.CoreOpsArithSubAssignRBig.sub_assign x oneRat = ok x' ->
  ∃ i' : dashu_int.ibig.IBig,
    SharedLUBig.Insts.CoreOpsArithSubUBigUBig.sub numer denom = ok numer' ∧
    dashu_ratio.rbig.RBig.into_parts x' = ok (i', denom) ∧
    dashu_int.ibig.IBig.into_parts i' = ok (dashu_base.sign.Sign.Positive, numer')

/-- Mathematical meaning of Dashu remainder by an 8-bit integer. -/
axiom rem_u8_spec
  (x : dashu_int.ubig.UBig) (y r : Std.U8) :
  dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem x y = ok r ->
    r.val = ubigToNat x % y.val

/-- Positive 8-bit divisors yield a successful Dashu remainder result. -/
axiom rem_u8_exists_spec
  (x : dashu_int.ubig.UBig) (y : Std.U8) :
  0 < y.val ->
  ∃ r,
    dashu_int.ubig.UBig.Insts.CoreOpsArithRemU8U8.rem x y = ok r ∧
    r.val = ubigToNat x % y.val

/-- Mathematical interpretation of the greater-than comparison for Dashu UBig. -/
axiom gt_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt y x = ok true ->
    ubigToNat y < ubigToNat x

/-- Mathematical interpretation of a failed greater-than comparison for Dashu UBig. -/
axiom gt_false_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt x y = ok false ->
    ubigToNat x ≤ ubigToNat y

/-- If the mathematical values are ordered, the greater-than comparison
returns `false`. -/
axiom gt_false_of_le_spec
  (x y : dashu_int.ubig.UBig) :
  ubigToNat x ≤ ubigToNat y ->
    dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt x y = ok false

/-- The greater-than comparison always returns `ok`, with the boolean
determined by the natural-number comparison. -/
axiom gt_spec_decide
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.gt x y =
    ok (decide (dashu.ubigToNat x > dashu.ubigToNat y))

/-- Mathematical interpretation of the less-than-or-equal-to comparison for Dashu UBig. -/
axiom le_spec
  (x y : dashu_int.ubig.UBig) :
  dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt x y = ok false ->
    ubigToNat x ≤ ubigToNat y

/-- If the positive numerator strictly exceeds the positive denominator, the
`x > 1` comparison returns `true`. (Construction direction of `rbig_gt_one_true_spec`.) -/
axiom rbig_gt_one_true_of_gt_spec
  (x : dashu_ratio.rbig.RBig)
  (numerSigned : dashu_int.ibig.IBig)
  (denom numer : dashu_int.ubig.UBig)
  (oneRat : dashu_ratio.rbig.RBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) →
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) →
  ubigToNat denom < ubigToNat numer →
  dashu_ratio.rbig.RBig.Insts.CoreCmpPartialOrdRBig.gt x oneRat = ok true

/-- Constructing the unit rational `1/1` via `from_parts_const` returns the same
value as `RBig::ONE`: if `ONE = ok oneRat` and `lift 1#u32 = ok i`, then
`from_parts_const Positive i i = ok oneRat`. -/
axiom rbig_from_parts_const_eq_one
  (i : Std.U128)
  (oneRat : dashu_ratio.rbig.RBig) :
  dashu_ratio.rbig.RBig.ONE = ok oneRat →
  Aeneas.Std.lift (UScalar.cast UScalarTy.U128 1#u32) = ok i →
  dashu_ratio.rbig.RBig.from_parts_const dashu_base.sign.Sign.Positive i i = ok oneRat

/-- Subtracting `oneRat` from a nonneg rational whose numerator ≥ denominator always succeeds. -/
axiom rbig_sub_assign_one_exists
  (x oneRat : dashu_ratio.rbig.RBig)
  (numerSigned : dashu_int.ibig.IBig)
  (denom numer : dashu_int.ubig.UBig) :
  dashu_ratio.rbig.RBig.into_parts x = ok (numerSigned, denom) →
  dashu_int.ibig.IBig.into_parts numerSigned = ok (dashu_base.sign.Sign.Positive, numer) →
  ubigToNat denom ≤ ubigToNat numer →
  ∃ xMinusOne : dashu_ratio.rbig.RBig,
    dashu_ratio.rbig.RBig.Insts.CoreOpsArithSubAssignRBig.sub_assign x oneRat = ok xMinusOne

end OpenDP.dashu
