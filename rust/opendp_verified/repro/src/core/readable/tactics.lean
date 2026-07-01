import src.core.primitives.semantics

/-!
# Core readable: prose-facing tactics

Companion to `core/readable/notation`. Where the notation file gives the
denotation bracket `⟦·⟧`, this file packages the recurring proof *moves* into
named tactics so the sampler proofs read closer to their `.tex` prose and the
load-bearing structural manipulation lives in one audited place.
-/

open Aeneas Aeneas.Std Result
open OpenDP.Core.Semantics

namespace OpenDP.Core.Readable

open Lean Lean.Elab.Tactic Lean.Meta in
/-- Force `instantiateMVars` on the goal type without otherwise changing the term.
A preceding `rw` can leave the goal with a delayed-assigned metavariable (here, the
beta-redex from collapsing the `deref_mut` ok-bind); until it is instantiated, `simp`
sees a stuck term and reports "no progress". This realises the side effect that makes
a stray `trace_state` "fix" such goals, but cleanly and silently. -/
elab "instantiate_goal_mvars" : tactic =>
  liftMetaTactic fun mvarId => do
    -- Force resolution of `rw`'s delayed assignments in the metavar context, but return
    -- the SAME goal (a `change` to a fresh mvar would sever that delayed assignment).
    let _ ← instantiateMVars (← mvarId.getType)
    return [mvarId]

/-- **Expose a loop body's randomness.** The a14083a6 Aeneas backend binds
`alloc.vec.Vec.deref_mut buffer`, which `dsimp` turns into a pure pattern-`let`
`(s, deref_mut_back) := (buf, back_fn)` sitting under `samplerDistGen` — hiding the
`fill_bytes` draw. This tactic unfolds the named loop `body` together with the
`deref_mut`/`lift` plumbing, runs a second `dsimp` pass to zeta/iota-reduce that
back-function let (substituting `s := buf`, β-reducing `deref_mut_back s1`), and
finally expands the surfaced `fill_bytes` bind into an explicit `SLang` sum. After
it, the goal is a `∑'` over `fill_bytes` outcomes, ready for distributional work.

Usage: `expose_loop_body sample_uniform_ubig_below_loop.body`. -/
syntax (name := exposeLoopBody) "expose_loop_body " term : tactic

macro_rules
  | `(tactic| expose_loop_body $body:term) =>
    `(tactic|
        (dsimp only [$body:term, alloc.vec.Vec.deref_mut, lift, alloc.vec.Vec.deref, bind_tc_ok]
         rw [samplerDistGen_bind_ok_left]
         trace_state
         simp only [samplerDistGen_bind, SLang.probBind]))

end OpenDP.Core.Readable
