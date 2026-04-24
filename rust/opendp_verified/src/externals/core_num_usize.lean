import Aeneas
import Mathlib.Algebra.Order.Floor.Div
import Generated.OpenDP.FunsExternal

open Aeneas Aeneas.Std Result ControlFlow Error
open OpenDP

namespace OpenDP.core_num_usize

/-- Mathematical meaning of `usize::div_ceil` on naturals. -/
axiom div_ceil_spec
  (a b q : Usize) :
  core.num.Usize.div_ceil a b = ok q ->
    q.val = a.val ⌈/⌉ b.val

end OpenDP.core_num_usize
