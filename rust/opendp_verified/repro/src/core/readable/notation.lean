import src.core.primitives.semantics

/-!
# Core readable: prose-facing vocabulary

`core/readable` introduces notation/macros so the final sampler proofs read as
closely as possible to the human `.tex` proofs, while delegating the real work to
`core/primitives`. This file holds the notation; tactics grow alongside the
proofs that need them.

The headline notation is the **denotation bracket** `⟦ prog ⟧` for the success
distribution of an extracted Aeneas sampler — i.e. the SLang law over the values
it returns `Ok`. A correctness statement then reads `⟦ sample_… ⟧ = <reference law>`.
-/

namespace OpenDP.Core.Readable

open OpenDP.Core.Semantics

/-- `⟦ prog ⟧` — the success distribution of an extracted sampler `prog`
(`samplerDist prog`): the probability it returns `Ok v`, as a `SLang` law. -/
scoped notation "⟦" prog "⟧" => OpenDP.Core.Semantics.samplerDist prog

end OpenDP.Core.Readable
