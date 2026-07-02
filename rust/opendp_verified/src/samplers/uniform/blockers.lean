import Generated.OpenDP
import SampCert.SLang
import src.core.primitives.bytes
import src.core.primitives.semantics
import src.core.externals.dashu

/-!
# Uniform loop-body distribution — a14083a6 `deref_mut` handling (RESOLVED ✓)

This file was originally a `sorry`-quarantine for two obligations that the a14083a6 `deref_mut`
codegen blocked. **Both are now fully proved** (no `sorry`); the file is kept as the home for the
`deref_mut`-stripping technique and could later be folded back into `samplers/uniform/pmf.lean`.

## The obstacle (and how it was solved)

The a14083a6 backend generates the loop body as
  `do let (s, deref_mut_back) ← lift (alloc.vec.Vec.deref_mut buffer)
      let (r, s1) ← fill_bytes s … deref_mut_back s1 …`
with `deref_mut buffer = (⟨buf.val,_⟩, fun s => ⟨s.val,_⟩)`, `lift x = ok x`. Because
`deref_mut_back` is used in BOTH loop branches, it survives as a *shared* destructuring
`let (s, deref_mut_back) := PAIR; …` between `samplerDistGen` and the `fill_bytes` bind.
`simp only` won't crack that shared let (the cracker is a default simproc), and full `simp`
cracks it but also reshapes `x * (if c then 1 else 0)` (`mul_ite`) and normalises `>>=` to
`Std.bind`, breaking the legacy tail.

**Solution — `body_eq` (reshaping lemma).** State the wrapper-free body in `>>=` form with a
single `do let p ← fill_bytes (deref buf)` binder and `p.1`/`p.2` projections (NOT a destructuring
`fun (r,s1) =>`, which would re-introduce an un-crackable matcher-let), and prove it by
`unfold; rfl` (every step is definitional). `simp_rw [body_eq]` then yields exactly the `>>=`
shape, so `samplerDistGen_bind`/`SLang.probBind` fire and the legacy body-analysis tail runs.
The post-`fill_bytes` `(·).1/(·).2` projections are cleared by a `dsimp only` after `obtain`.

Key gotchas (all in-proof now): the loop state is `alloc.vec.Vec`, which is *defeq* to `Slice`,
so the `cont` back-edge must carry a `Vec`-typed argument (`⟨p.2.val, p.2.property⟩`) for
`cont.injEq` to apply, and `tsum_eq_single`'s point is named via `(deref_mut buf).2 s` so it has
the folded `Vec` type the `∑'` ranges over.
-/

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP OpenDP.Core.Bytes OpenDP.Core.Semantics
open SLang PMF ENNReal

namespace OpenDP.samplers.uniform

/-- **Reshaping lemma (idea 4).** The a14083a6 loop body, with the `deref_mut` back-function
wrapper stripped: `lift`/`deref_mut`/`deref` reduce away and `deref_mut_back s1 ↦ s1`. Stated in
`>>=` form (defeq to the `Std.bind` form full `simp` lands on, so `simp` closes it); `rw`-ing it
into a goal then yields the `>>=` shape the legacy body analysis expects. The statement carries
no `∑'`/`*`, so the `simp` proof has no `mul_ite`/tsum reshaping to trip over. -/
lemma body_eq (upper threshold : dashu_int.ubig.UBig) (buf : alloc.vec.Vec Std.U8) :
    sample_uniform_ubig_below_loop.body upper threshold buf =
      (do
        let p ← fill_bytes (alloc.vec.Vec.deref buf)
        let cf ← core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch p.1
        match cf with
        | core.ops.control_flow.ControlFlow.Continue _ =>
          let sample ← dashu_int.convert.UBig.from_be_bytes p.2
          let b ← dashu_int.ubig.UBig.Insts.CoreCmpPartialOrdUBig.lt sample threshold
          if b = true then
            let u ← dashu_int.ubig.UBig.Insts.CoreOpsArithRemSharedRUBigUBig.rem sample upper
            ok (done (core.result.Result.Ok u))
          else ok (cont (⟨p.2.val, p.2.property⟩ : alloc.vec.Vec Std.U8))
        | core.ops.control_flow.ControlFlow.Break residual =>
          let r1 ← core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
              dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual
          ok (done r1)) := by
  -- All steps (`lift`, `deref_mut`, the `ok`-bind, the matcher) are definitional, so after
  -- unfolding the body the two sides are defeq — `rfl` closes (the `simp` route stalls only
  -- because the body's auto-generated matcher constant differs syntactically from ours).
  unfold sample_uniform_ubig_below_loop.body
  rfl

/-- Accepted body outcomes: when the random nat `k < threshold`, the body outputs
`done (Ok u)` with `ubigToNat u = k % upper`. (Proved via `body_eq`; see the file header.) -/
lemma body_done_nat_dist
    (upper threshold : dashu_int.ubig.UBig)
    (byte_len : Usize)
    (buf : alloc.vec.Vec Std.U8)
    (hlen : buf.length = byte_len.val)
    (hupper : 0 < dashu.ubigToNat upper)
    (nat_val : Nat) :
    ∑' ubig : dashu_int.ubig.UBig,
      samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf)
        (done (core.result.Result.Ok ubig)) *
      (if dashu.ubigToNat ubig = nat_val then 1 else 0) =
    ∑' k : Nat,
      uniformByteNatPMF byte_len.val k *
      (if k < dashu.ubigToNat threshold ∧ k % dashu.ubigToNat upper = nat_val
       then 1 else 0) := by
  rw [← fill_bytes_nat_bridge buf byte_len hlen
        (fun k => if k < dashu.ubigToNat threshold ∧ k % dashu.ubigToNat upper = nat_val
                  then 1 else 0)]
  -- Strip the a14083a6 `deref_mut` wrapper via `body_eq`, exposing the `fill_bytes` bind.
  simp_rw [body_eq]
  simp only [samplerDistGen_bind, SLang.probBind]
  -- Legacy body analysis (proofs_legacy/samplers/uniform/pmf.lean) — runs verbatim from here.
  simp_rw [← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  apply tsum_congr; intro pair
  simp_rw [mul_assoc]
  rw [ENNReal.tsum_mul_left]
  congr 1
  obtain ⟨r, s⟩ := pair
  -- reduce the `(r, s).1`/`.2` projections left by expanding `fill_bytes`'s pair-bind
  dsimp only
  rcases r with ⟨⟨⟩⟩ | e
  · have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Ok () : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Continue ()) := rfl
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    simp_rw [← ENNReal.tsum_mul_right]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Continue ()
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only
    obtain ⟨sample, hfbe, hsamp⟩ := dashu.from_be_bytes_exists_spec s
    obtain ⟨b, hlt⟩ := dashu.lt_exists_spec sample threshold
    simp_rw [hfbe, samplerDistGen_bind_ok_left, hlt, samplerDistGen_bind_ok_left]
    have hsv : beBytesToNat (s.val) = dashu.ubigToNat sample := hsamp.symm
    rcases b with _ | _
    · have hge : dashu.ubigToNat threshold ≤ dashu.ubigToNat sample := dashu.lt_false_spec _ _ hlt
      simp only [if_true, Bool.false_eq_true, if_false, samplerDistGen_pure_ok, PMF.pure_apply,
        reduceCtorEq, zero_mul, mul_zero, tsum_zero]
      rw [if_neg]; rw [hsv]; omega
    · have hlt' : dashu.ubigToNat sample < dashu.ubigToNat threshold := dashu.lt_true_spec _ _ hlt
      obtain ⟨u, hrem, hu⟩ := dashu.rem_body_exists_spec sample upper hupper
      simp only [if_true, one_mul]
      simp only [hrem, samplerDistGen_bind_ok_left, samplerDistGen_pure_ok, PMF.pure_apply,
                 done.injEq, core.result.Result.Ok.injEq]
      rw [tsum_eq_single u (fun a ha => by simp [ha])]
      rw [if_pos rfl, one_mul, hu, hsv]
      by_cases hnv : dashu.ubigToNat sample % dashu.ubigToNat upper = nat_val
      · rw [if_pos hnv, if_pos ⟨hlt', hnv⟩]
      · rw [if_neg hnv, if_neg (fun h => hnv h.2)]
  · have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Err e : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e)) := rfl
    have hfr := from_residual_err_ok (T := dashu_int.ubig.UBig) e
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    simp_rw [← ENNReal.tsum_mul_right]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e)
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only [hfr, samplerDistGen_bind_ok_left, samplerDistGen_pure_ok, PMF.pure_apply, if_true,
      done.injEq, reduceCtorEq, if_false, mul_zero, zero_mul, tsum_zero]

/-- Rejected body mass: the total probability of `cont` outcomes equals `∑ k ≥ threshold, U k`.
(Proved via `body_eq`; see the file header.) -/
lemma body_cont_total_mass
    (upper threshold : dashu_int.ubig.UBig)
    (byte_len : Usize)
    (buf : alloc.vec.Vec Std.U8)
    (hlen : buf.length = byte_len.val) :
    ∑' buf' : alloc.vec.Vec Std.U8,
      samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf) (cont buf') =
    ∑' k : Nat,
      uniformByteNatPMF byte_len.val k * (if k < dashu.ubigToNat threshold then 0 else 1) := by
  rw [← fill_bytes_nat_bridge buf byte_len hlen
        (fun k => if k < dashu.ubigToNat threshold then 0 else 1)]
  simp_rw [body_eq]
  simp only [samplerDistGen_bind, SLang.probBind]
  rw [ENNReal.tsum_comm]
  apply tsum_congr; intro pair
  rw [ENNReal.tsum_mul_left]
  congr 1
  obtain ⟨r, s⟩ := pair
  dsimp only
  rcases r with ⟨⟨⟩⟩ | e
  · have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Ok () : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Continue ()) := rfl
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Continue ()
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only
    obtain ⟨sample, hfbe, hsamp⟩ := dashu.from_be_bytes_exists_spec s
    obtain ⟨b, hlt⟩ := dashu.lt_exists_spec sample threshold
    have hsv : beBytesToNat (s.val) = dashu.ubigToNat sample := hsamp.symm
    simp_rw [hfbe, samplerDistGen_bind_ok_left, hlt, samplerDistGen_bind_ok_left]
    rcases b with _ | _
    · have hge : dashu.ubigToNat threshold ≤ dashu.ubigToNat sample := dashu.lt_false_spec _ _ hlt
      simp only [Bool.false_eq_true, if_false, samplerDistGen_pure_ok, PMF.pure_apply,
                 if_true, one_mul]
      -- Total mass of the point distribution at the (injective) `cont` image is 1. The point is
      -- named via the back-function `(deref_mut buf).2 s`, which has the folded `alloc.vec.Vec`
      -- type the `∑'` ranges over (a bare `⟨s.val, s.property⟩` unfolds to the raw Subtype and
      -- fails to unify with the sum's index type).
      rw [if_neg (by rw [hsv]; omega)]
      rw [tsum_eq_single ((alloc.vec.Vec.deref_mut buf).2 s)
            (fun a ha => by simp_all [alloc.vec.Vec.deref_mut])]
      exact if_pos rfl
    · have hlt' : dashu.ubigToNat sample < dashu.ubigToNat threshold := dashu.lt_true_spec _ _ hlt
      simp only [if_true]
      simp_rw [samplerDistGen_bind, SLang.probBind, samplerDistGen_pure_ok, PMF.pure_apply]
      simp only [reduceCtorEq, if_false, mul_zero, tsum_zero]
      rw [hsv, if_pos hlt']
  · have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Err e : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e)) := rfl
    have hfr := from_residual_err_ok (T := dashu_int.ubig.UBig) e
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e)
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only [if_true, hfr, samplerDistGen_bind_ok_left, samplerDistGen_pure_ok, PMF.pure_apply,
      reduceCtorEq, if_false, mul_zero, tsum_zero]

end OpenDP.samplers.uniform
