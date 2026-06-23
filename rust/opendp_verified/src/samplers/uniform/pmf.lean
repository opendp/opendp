import Aeneas
import Generated.OpenDP
import SampCert.SLang
import SampCert.Samplers.Uniform.Properties
import SampCert.Foundations.Until
import src.samplers.bytes
import src.samplers.functor
import src.samplers.uniform.semantics
import src.samplers.uniform.mod
import src.externals.dashu

open Aeneas Aeneas.Std Aeneas.Std.WP Result ControlFlow
open OpenDP OpenDP.bytes OpenDP.samplers
open SLang PMF ENNReal Finset Classical

namespace OpenDP.samplers.uniform

/-! ## Distributional Correctness of `sample_uniform_ubig_below`

End-to-end distributional proof:

  `samplerDist_nat (sample_uniform_ubig_below upper) = ↑(uniformNatBelowPMF upper hupper)`

where `samplerDist_nat prog = probBind (samplerDist prog) (probPure ∘ ubigToNat)`.

### Proof chain

1. `sample_uniform_ubig_below upper`
   = `bind (alloc buffer) (loop upper threshold)`    [by sample_uniform_ubig_below_eq_of_setup]
   = `loop upper threshold buffer_init`              [alloc is deterministic]

2. `samplerDist_nat (loop upper threshold buffer_init)`
   = `probBind (probUntil (uniformByteNatPMF byte_len) (· < threshold)) (· % upper)`
                                                     [by samplerDist_loop_rejection_uniform]

3. `probUntil (uniformByteNatPMF byte_len) (· < threshold)`
   = `↑(UniformSample_PMF {threshold})`             [rejection sampling = uniform, key lemma]

4. `probBind (↑(UniformSample_PMF {threshold})) (· % upper)`
   = `↑(PMF.map (· % upper) (UniformSample_PMF {threshold}))`
   = `↑(uniformNatBelowPMF upper hupper)`           [by sample_uniform_ubig_below_success_pmf_eq_uniform]
-/

/-! ### Loop theorem for the uniform rejection sampler -/

/-! #### Body distribution sub-lemmas

Three lemmas about one body iteration of `sample_uniform_ubig_below_loop`.
Proofs follow by unfolding the Aeneas-extracted body via `samplerDistGen_bind`,
then applying `samplerDistGen_fill_bytes_nat` for the stochastic step, and the
dashu specs (`from_be_bytes_spec`, `lt_true_spec`, `lt_false_spec`, `rem_spec`)
for the deterministic steps.
-/

/-- Accepted body outcomes: when the random nat k < threshold, the body outputs
    `done (Ok u)` with `ubigToNat u = k % upper`. -/
private lemma body_done_nat_dist
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
  simp only [sample_uniform_ubig_below_loop.body, alloc.vec.Vec.deref_mut, lift,
             alloc.vec.Vec.deref, bind_tc_ok, samplerDistGen_bind, SLang.probBind]
  -- Swap ∑ubig outward to ∑pair, factor out the (ubig-independent) fill_bytes weight.
  simp_rw [← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  apply tsum_congr; intro pair
  simp_rw [mul_assoc]
  rw [ENNReal.tsum_mul_left]
  congr 1
  obtain ⟨r, s⟩ := pair
  rcases r with ⟨⟩ | e
  · -- r = Ok (): branch is deterministic (→ Continue ()); collapse ∑ over control-flow.
    have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Ok () : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Continue ()) := rfl
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    -- collapse the control-flow sum (point mass at Continue ())
    simp_rw [← ENNReal.tsum_mul_right]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Continue ()
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only
    -- Reduce the deterministic from_be_bytes / lt / rem chain.
    obtain ⟨sample, hfbe, hsamp⟩ := dashu.from_be_bytes_exists_spec ⟨s.val, s.property⟩
    obtain ⟨b, hlt⟩ := dashu.lt_exists_spec sample threshold
    simp_rw [hfbe, samplerDistGen_bind_ok_left, hlt, samplerDistGen_bind_ok_left]
    have hsv : beBytesToNat (s.val) = dashu.ubigToNat sample := hsamp.symm
    rcases b with _ | _
    · -- b = false: threshold ≤ k₀, body continues (cont) — no `done (Ok _)` mass.
      have hge : dashu.ubigToNat threshold ≤ dashu.ubigToNat sample := dashu.lt_false_spec _ _ hlt
      simp only [if_true, Bool.false_eq_true, if_false, samplerDistGen_pure_ok, PMF.pure_apply,
        reduceCtorEq, zero_mul, mul_zero, tsum_zero]
      rw [if_neg]; rw [hsv]; omega
    · -- b = true: k₀ < threshold, body accepts with `done (Ok u)`, `ubigToNat u = k₀ % upper`.
      have hlt' : dashu.ubigToNat sample < dashu.ubigToNat threshold := dashu.lt_true_spec _ _ hlt
      obtain ⟨u, hrem, hu⟩ := dashu.rem_body_exists_spec sample upper hupper
      simp only [if_true, one_mul]
      simp_rw [hrem, samplerDistGen_bind_ok_left, samplerDistGen_pure_ok, PMF.pure_apply,
               done.injEq, core.result.Result.Ok.injEq]
      rw [tsum_eq_single u (fun a ha => by rw [if_neg ha, zero_mul])]
      rw [if_pos rfl, one_mul, hu, hsv]
      by_cases hnv : dashu.ubigToNat sample % dashu.ubigToNat upper = nat_val
      · rw [if_pos hnv, if_pos ⟨hlt', hnv⟩]
      · rw [if_neg hnv, if_neg (fun h => hnv h.2)]
  · -- r = Err e: branch → Break, body fails with `done (Err e)` — no `done (Ok _)` mass.
    have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
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

/-- Rejected body mass: the total probability of `cont` outcomes equals ∑ k ≥ threshold, U k. -/
private lemma body_cont_total_mass
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
  simp only [sample_uniform_ubig_below_loop.body, alloc.vec.Vec.deref_mut, lift,
             alloc.vec.Vec.deref, bind_tc_ok, samplerDistGen_bind, SLang.probBind]
  rw [ENNReal.tsum_comm]
  apply tsum_congr; intro pair
  rw [ENNReal.tsum_mul_left]
  congr 1
  obtain ⟨r, s⟩ := pair
  rcases r with ⟨⟩ | e
  · -- r = Ok (): branch → Continue; collapse control-flow sum.
    have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Ok () : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Continue ()) := rfl
    simp only [hbr, samplerDistGen_pure_ok, PMF.pure_apply]
    rw [ENNReal.tsum_comm,
        tsum_eq_single (core.ops.control_flow.ControlFlow.Continue ()
          : core.ops.control_flow.ControlFlow (core.result.Result core.convert.Infallible error.Error) Unit)
          (fun a ha => by simp [if_neg ha])]
    simp only
    obtain ⟨sample, hfbe, hsamp⟩ := dashu.from_be_bytes_exists_spec ⟨s.val, s.property⟩
    obtain ⟨b, hlt⟩ := dashu.lt_exists_spec sample threshold
    have hsv : beBytesToNat (s.val) = dashu.ubigToNat sample := hsamp.symm
    simp_rw [hfbe, samplerDistGen_bind_ok_left, hlt, samplerDistGen_bind_ok_left]
    rcases b with _ | _
    · -- b = false: body continues with `cont ⟨s.val,_⟩` — total cont mass is 1; k₀ ≥ threshold.
      have hge : dashu.ubigToNat threshold ≤ dashu.ubigToNat sample := dashu.lt_false_spec _ _ hlt
      simp only [Bool.false_eq_true, if_false, samplerDistGen_pure_ok, PMF.pure_apply, cont.injEq,
                 if_true, one_mul]
      rw [tsum_ite_eq, if_neg]; rw [hsv]; omega
    · -- b = true: body accepts with `done (Ok u)` — contributes no cont mass; k₀ < threshold.
      have hlt' : dashu.ubigToNat sample < dashu.ubigToNat threshold := dashu.lt_true_spec _ _ hlt
      simp only [if_true]
      simp_rw [samplerDistGen_bind, SLang.probBind, samplerDistGen_pure_ok, PMF.pure_apply]
      simp only [reduceCtorEq, if_false, mul_zero, tsum_zero]
      rw [hsv, if_pos hlt']
  · -- r = Err e: branch → Break, body fails — no cont mass.
    have hbr : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
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

/-- Length preservation: every `cont buf'` with positive probability satisfies
    `buf'.length = byte_len.val`. -/
private lemma body_cont_length_pres
    (upper threshold : dashu_int.ubig.UBig)
    (byte_len : Usize)
    (buf buf' : alloc.vec.Vec Std.U8)
    (hlen : buf.length = byte_len.val)
    (hpos : 0 < samplerDistGen
        (sample_uniform_ubig_below_loop.body upper threshold buf) (cont buf')) :
    buf'.length = byte_len.val := by
  -- Unfold body in place (the lift(deref_mut buf) step is deterministic).
  -- Done directly on `hpos` to use the body's native `match` matcher.
  simp only [sample_uniform_ubig_below_loop.body, alloc.vec.Vec.deref_mut, lift,
             alloc.vec.Vec.deref, bind_tc_ok] at hpos
  -- Extract witness (r, s1) from fill_bytes with positive mass
  obtain ⟨⟨r, s1⟩, hfb_pos, hrest_pos⟩ := samplerDistGen_bind_pos_exists' _ _ _ hpos
  -- fill_bytes length preservation: s1.length = (deref buf).length = buf.length = byte_len.val
  have hs1_len : s1.length = byte_len.val := by
    have h := samplerDistGen_fill_bytes_length_pres (alloc.vec.Vec.deref buf) r s1 hfb_pos
    simp only [alloc.vec.Vec.deref, alloc.vec.Vec.length, Slice.length] at h hlen ⊢
    omega
  -- Decompose: branch r >>= fun cf => match cf with ...
  obtain ⟨cf, hbranch_pos, hcont_pos⟩ := samplerDistGen_bind_pos_exists' _ _ _ hrest_pos
  rcases r with ⟨⟩ | e
  · -- r = Ok (): branch = Continue
    have hbranch_eq : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Ok () : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Continue ()) := rfl
    have hcf : cf = core.ops.control_flow.ControlFlow.Continue () :=
      samplerDistGen_ok_pos' _ cf (by rwa [hbranch_eq] at hbranch_pos)
    subst hcf
    simp only at hcont_pos
    obtain ⟨sample, hfbe, _⟩ := dashu.from_be_bytes_exists_spec _
    rw [hfbe, samplerDistGen_bind_ok_left] at hcont_pos
    obtain ⟨b, hlt⟩ := dashu.lt_exists_spec sample threshold
    rw [hlt, samplerDistGen_bind_ok_left] at hcont_pos
    rcases b with _ | _
    · -- b = false: cont case
      rw [if_neg (by decide)] at hcont_pos
      rw [samplerDistGen_pure_ok, PMF.pure_apply] at hcont_pos
      split_ifs at hcont_pos with heq
      · have hbuf' : buf' = ⟨s1.val, s1.property⟩ := ControlFlow.cont.inj heq
        rw [hbuf']; exact hs1_len
      · simp at hcont_pos
    · -- b = true: goes to done (Ok u), contradiction
      simp only [↓reduceIte] at hcont_pos
      obtain ⟨u, _, hu_pos⟩ := samplerDistGen_bind_pos_exists' _ _ _ hcont_pos
      rw [samplerDistGen_pure_ok, PMF.pure_apply] at hu_pos
      simp at hu_pos
  · -- r = Err e: branch = Break (Err e), goes to done (Err e), contradiction
    have hbranch_eq : core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch
        (core.result.Result.Err e : core.result.Result Unit error.Error) =
        ok (core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e)) := rfl
    have hcf : cf = core.ops.control_flow.ControlFlow.Break (core.result.Result.Err e) :=
      samplerDistGen_ok_pos' _ cf (by rwa [hbranch_eq] at hbranch_pos)
    subst hcf
    simp only at hcont_pos
    have hfr := from_residual_err_ok (T := dashu_int.ubig.UBig) e
    rw [hfr, samplerDistGen_bind_ok_left, samplerDistGen_pure_ok, PMF.pure_apply] at hcont_pos
    simp at hcont_pos

/-! #### probWhileCut induction

`loop_cut_step_aux` shows the buffer-loop's `(n+1)`-step truncation
(projected through `ubigToNat`) equals the nat-loop's `n`-step truncation
(summed against U, then projected through `· % upper`).

Both are zero at `n = 0`. The inductive step uses the three body lemmas.
-/

/-- Generalized cut-step induction with the control-flow guard/body abstracted as
    `cond`/`bd` variables (dodges the inline-`match` matcher-identity problem). -/
private lemma loop_cut_step_aux
    (upper threshold : dashu_int.ubig.UBig)
    (byte_len : Usize)
    (hupper : 0 < dashu.ubigToNat upper)
    (nat_val : Nat)
    (cond : ControlFlow (alloc.vec.Vec Std.U8)
              (core.result.Result dashu_int.ubig.UBig error.Error) → Bool)
    (bd : ControlFlow (alloc.vec.Vec Std.U8)
              (core.result.Result dashu_int.ubig.UBig error.Error) →
          SLang (ControlFlow (alloc.vec.Vec Std.U8)
              (core.result.Result dashu_int.ubig.UBig error.Error)))
    (hcc : ∀ b, cond (cont b) = true)
    (hcd : ∀ w, cond (done w) = false)
    (hbc : ∀ b, bd (cont b) = samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold b))
    (n : Nat) :
    ∀ buf : alloc.vec.Vec Std.U8,
    buf.length = byte_len.val →
    ∑' ubig : dashu_int.ubig.UBig,
      probWhileCut cond bd (n + 1) (cont buf) (done (core.result.Result.Ok ubig)) *
      (if dashu.ubigToNat ubig = nat_val then 1 else 0) =
    ∑' k : Nat,
      uniformByteNatPMF byte_len.val k *
      (∑' m : Nat,
        probWhileCut
          (fun v : Nat => decide (¬ (v < dashu.ubigToNat threshold)))
          (fun _ => uniformByteNatPMF byte_len.val)
          n k m *
        (if m % dashu.ubigToNat upper = nat_val then 1 else 0)) := by
  induction n with
  | zero =>
    intro buf _
    simp only [probWhileCut, probWhileFunctional, hcc buf, Bind.bind, probZero, SLang.probBind,
      if_true, mul_zero, zero_mul, tsum_zero]
  | succ n ih =>
    intro buf hbuf
    -- the depth-n nat-side total (what `ih` returns), and the per-k unfold of the nat loop
    set R := ∑' k : Nat, uniformByteNatPMF byte_len.val k *
        (∑' m : Nat, probWhileCut (fun v => decide (¬ v < dashu.ubigToNat threshold))
          (fun _ => uniformByteNatPMF byte_len.val) n k m *
          (if m % dashu.ubigToNat upper = nat_val then 1 else 0)) with hR
    have hQ : ∀ k : Nat,
        (∑' m : Nat, probWhileCut (fun v => decide (¬ v < dashu.ubigToNat threshold))
            (fun _ => uniformByteNatPMF byte_len.val) (n + 1) k m *
            (if m % dashu.ubigToNat upper = nat_val then 1 else 0)) =
        (if k < dashu.ubigToNat threshold
         then (if k % dashu.ubigToNat upper = nat_val then 1 else 0) else R) := by
      intro k
      by_cases hk : k < dashu.ubigToNat threshold
      · rw [if_pos hk]
        have hpk : probWhileCut (fun v => decide (¬ v < dashu.ubigToNat threshold))
            (fun _ => uniformByteNatPMF byte_len.val) (n + 1) k = probPure k :=
          probWhileCut_guard_false _ _ k (by simp [hk]) n
        simp_rw [hpk, SLang.pure_apply]
        rw [tsum_eq_single k (fun m hm => by rw [if_neg hm, zero_mul]), if_pos rfl, one_mul]
      · rw [if_neg hk]
        have hpk : probWhileCut (fun v => decide (¬ v < dashu.ubigToNat threshold))
            (fun _ => uniformByteNatPMF byte_len.val) (n + 1) k =
            ((uniformByteNatPMF byte_len.val : SLang Nat) >>=
              probWhileCut (fun v => decide (¬ v < dashu.ubigToNat threshold))
                (fun _ => uniformByteNatPMF byte_len.val) n) := by
          rw [probWhileCut, probWhileFunctional, if_pos (by simp [hk])]
        simp_rw [hpk, Bind.bind, SLang.bind_apply, ← ENNReal.tsum_mul_right]
        rw [ENNReal.tsum_comm]
        simp_rw [mul_assoc, ENNReal.tsum_mul_left]
        rw [hR]
    -- RHS(n+1) = (reject-mass)·R + (accept body distribution)
    simp_rw [hQ]
    -- one-step unfold of the control-flow loop at `cont buf`
    have hstep : probWhileCut cond bd (n + 1 + 1) (cont buf)
        = bd (cont buf) >>= probWhileCut cond bd (n + 1) := by
      rw [probWhileCut, probWhileFunctional, if_pos (hcc buf)]
    simp only [hstep, hbc buf, Bind.bind, SLang.bind_apply]
    simp_rw [← ENNReal.tsum_mul_right]
    rw [ENNReal.tsum_comm]
    simp_rw [mul_assoc, ENNReal.tsum_mul_left]
    rw [tsum_controlFlow]
    -- DONE fiber: point mass at `done b`, collapses to the accepted-body distribution.
    have hDONE : (∑' b : core.result.Result dashu_int.ubig.UBig error.Error,
        samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf) (done b) *
        (∑' i : dashu_int.ubig.UBig, probWhileCut cond bd (n + 1) (done b)
          (done (core.result.Result.Ok i)) * (if dashu.ubigToNat i = nat_val then 1 else 0))) =
        ∑' k : Nat, uniformByteNatPMF byte_len.val k *
          (if k < dashu.ubigToNat threshold ∧ k % dashu.ubigToNat upper = nat_val then 1 else 0) := by
      rw [← body_done_nat_dist upper threshold byte_len buf hbuf hupper nat_val]
      simp_rw [probWhileCut_done_pt cond bd hcd n, SLang.pure_apply, done.injEq,
               ← ENNReal.tsum_mul_left]
      rw [ENNReal.tsum_comm]
      apply tsum_congr; intro i
      rw [tsum_eq_single (core.result.Result.Ok i) (fun b hb => by
        rw [if_neg (fun h => hb h.symm), zero_mul, mul_zero]), if_pos rfl, one_mul]
    -- CONT fiber: each continued buffer recurses (IH), giving the reject mass times R.
    have hCONT : (∑' a : alloc.vec.Vec Std.U8,
        samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf) (cont a) *
        (∑' i : dashu_int.ubig.UBig, probWhileCut cond bd (n + 1) (cont a)
          (done (core.result.Result.Ok i)) * (if dashu.ubigToNat i = nat_val then 1 else 0))) =
        (∑' a : alloc.vec.Vec Std.U8,
          samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf) (cont a)) * R := by
      rw [← ENNReal.tsum_mul_right]
      apply tsum_congr; intro a
      by_cases ha : samplerDistGen (sample_uniform_ubig_below_loop.body upper threshold buf) (cont a) = 0
      · rw [ha, zero_mul, zero_mul]
      · have hlen_a : a.length = byte_len.val :=
          body_cont_length_pres upper threshold byte_len buf a hbuf (pos_iff_ne_zero.mpr ha)
        rw [ih a hlen_a]
    rw [hCONT, hDONE, body_cont_total_mass upper threshold byte_len buf hbuf]
    -- Nat identity: (reject mass)·R + accepted = ∑ U k · (if k<T then accepted else R).
    rw [← ENNReal.tsum_mul_right, ← ENNReal.tsum_add]
    apply tsum_congr; intro k
    by_cases hk : k < dashu.ubigToNat threshold <;> simp [hk, mul_comm]

/-- `probUntil`'s internally-built rejection condition `decide (¬ decide (v < T) = true)`
    coincides with the clean `decide (¬ v < T)` used by `loop_cut_step_aux`/`nat_cond`. -/
private lemma until_cond_simp (T : Nat) :
    (fun v : Nat => decide (¬ (decide (v < T) = true))) = (fun v : Nat => decide (¬ v < T)) := by
  funext v; simp only [decide_eq_true_eq]

/-- The uniform byte rejection loop distributes as `SLang.probUntil`.

    Proof strategy:
    1. Rewrite LHS via `samplerDistGen_loop`: buffer loop → `probWhile` over ControlFlow.
    2. Both LHS and RHS unfold to `iSup n, cut_n` for monotone cut sequences.
    3. `loop_cut_step_aux` shows `LHS_cut (n+1) = RHS_cut n` for all n and all buffers of
       the right length.
    4. Taking the `iSup` (shifted by 1) gives LHS = RHS. -/
theorem samplerDist_loop_rejection_uniform
    (upper threshold : dashu_int.ubig.UBig)
    (byte_len : Usize)
    (buffer : alloc.vec.Vec Std.U8)
    (hbuf : buffer.length = byte_len.val)
    (hupper : 0 < dashu.ubigToNat upper)
    (_hthreshold : 0 < dashu.ubigToNat threshold) :
    probBind
      (samplerDist (sample_uniform_ubig_below_loop upper threshold buffer))
      (fun ubig => probPure (dashu.ubigToNat ubig)) =
    probBind
      (probUntil (uniformByteNatPMF byte_len.val) (· < dashu.ubigToNat threshold))
      (fun nat_val => probPure (nat_val % dashu.ubigToNat upper)) := by
  set T := dashu.ubigToNat threshold
  set U_nat := dashu.ubigToNat upper
  set U := uniformByteNatPMF byte_len.val
  let body := sample_uniform_ubig_below_loop.body upper threshold
  let cond_cf : ControlFlow (alloc.vec.Vec Std.U8)
        (core.result.Result dashu_int.ubig.UBig error.Error) → Bool :=
    fun cf => match cf with | cont _ => true | done _ => false
  let body_cf : ControlFlow (alloc.vec.Vec Std.U8)
        (core.result.Result dashu_int.ubig.UBig error.Error) →
        SLang (ControlFlow (alloc.vec.Vec Std.U8)
          (core.result.Result dashu_int.ubig.UBig error.Error)) :=
    fun cf => match cf with
      | cont b => samplerDistGen (body b)
      | done _ => PMF.pure cf
  let nat_cond : Nat → Bool := fun v => decide (¬ v < T)
  let nat_body : Nat → SLang Nat := fun _ => U
  funext nat_val
  simp only [SLang.bind_apply, SLang.pure_apply]
  -- Normalize indicator order: pure_apply gives (if nat_val = x) but loop_cut_step_aux uses (if x = nat_val)
  simp_rw [eq_comm (a := nat_val)]
  -- Step 1: Rewrite LHS samplerDist via samplerDistGen_loop
  conv_lhs =>
    arg 1; ext ubig
    rw [show samplerDist (sample_uniform_ubig_below_loop upper threshold buffer) ubig =
        probWhile cond_cf body_cf (cont buffer) (done (core.result.Result.Ok ubig)) from by
      simp only [samplerDist, sample_uniform_ubig_below_loop, samplerDistGen_loop]
      congr 1 <;> (funext cf; cases cf <;> rfl)]
  -- Step 2: probWhile = ⨆ n, probWhileCut n
  simp only [probWhile]
  -- Step 3: Exchange ∑' ubig and ⨆ n on LHS
  simp_rw [ENNReal.iSup_mul]
  rw [tsum_iSup_commute _ (fun ubig => (probWhileCut_monotonic cond_cf body_cf (cont buffer)
      (done (core.result.Result.Ok ubig))).mul_const (zero_le _))]
  -- Step 4: Shift ⨆ by 1 (probWhileCut 0 = 0)
  rw [show (⨆ n, ∑' ubig : dashu_int.ubig.UBig,
        probWhileCut cond_cf body_cf n (cont buffer)
          (done (core.result.Result.Ok ubig)) *
        (if dashu.ubigToNat ubig = nat_val then 1 else 0)) =
      ⨆ n, ∑' ubig : dashu_int.ubig.UBig,
        probWhileCut cond_cf body_cf (n + 1) (cont buffer)
          (done (core.result.Result.Ok ubig)) *
        (if dashu.ubigToNat ubig = nat_val then 1 else 0) from by
    apply le_antisymm
    · apply iSup_le; intro n; apply le_iSup_of_le n
      apply ENNReal.tsum_le_tsum; intro ubig
      exact mul_le_mul_left (probWhileCut_monotonic cond_cf body_cf (cont buffer)
        (done (core.result.Result.Ok ubig)) (Nat.le_succ n)) _
    · apply iSup_le; intro n
      exact le_iSup_of_le (n + 1) (le_refl _)]
  -- Step 5: Apply loop_cut_step_aux to equate n-th cuts
  rw [show ⨆ n, ∑' ubig : dashu_int.ubig.UBig,
        probWhileCut cond_cf body_cf (n + 1) (cont buffer)
          (done (core.result.Result.Ok ubig)) *
        (if dashu.ubigToNat ubig = nat_val then 1 else 0) =
      ⨆ n, ∑' k : Nat, U k *
        (∑' m : Nat,
          probWhileCut nat_cond nat_body n k m *
          (if m % U_nat = nat_val then 1 else 0)) from by
    congr 1; ext n
    exact loop_cut_step_aux upper threshold byte_len hupper nat_val _ _
        (fun _ => rfl) (fun _ => rfl) (fun _ => rfl) n buffer hbuf]
  -- Step 6: Transform RHS into the same ⨆ n form
  -- 6a: Unfold probUntil on RHS (under ∑' nat')
  simp_rw [show ∀ nat' : Nat,
      probUntil U (· < T) nat' * (if nat' % U_nat = nat_val then 1 else 0) =
      (∑' k : Nat, U k * probWhile nat_cond nat_body k nat') *
      (if nat' % U_nat = nat_val then 1 else 0) from fun nat' => by
    congr 1
    simp only [probUntil, Bind.bind, SLang.bind_apply, until_cond_simp]
    rfl]
  -- 6b: Bring ind inside inner tsum
  simp_rw [← ENNReal.tsum_mul_right]
  -- 6c: Swap ∑' nat' and ∑' k
  rw [ENNReal.tsum_comm]
  -- 6d: Regroup multiplication
  simp_rw [mul_assoc (U _)]
  -- 6e: Factor U k out of inner tsum
  simp_rw [ENNReal.tsum_mul_left]
  -- 6f: probWhile = ⨆ n, probWhileCut n
  simp_rw [probWhile]
  -- 6g: (⨆ n, ...) * ind = ⨆ n, ... * ind
  simp_rw [ENNReal.iSup_mul]
  -- 6h: Inner interchange ∑' nat', ⨆ n = ⨆ n, ∑' nat'
  conv_rhs =>
    arg 1; ext k
    rw [tsum_iSup_commute _ (fun nat' =>
      (probWhileCut_monotonic nat_cond nat_body k nat').mul_const (zero_le _))]
  -- 6i: Outer interchange ∑' k, U k * ⨆ n = ⨆ n, ∑' k, U k *
  rw [show ∑' k : Nat, U k * ⨆ n, ∑' nat' : Nat,
          probWhileCut nat_cond nat_body n k nat' *
          (if nat' % U_nat = nat_val then 1 else 0) =
      ⨆ n, ∑' k : Nat, U k * ∑' nat' : Nat,
          probWhileCut nat_cond nat_body n k nat' *
          (if nat' % U_nat = nat_val then 1 else 0) from by
    simp_rw [mul_iSup]
    exact tsum_iSup_commute
      (fun k n => U k * ∑' nat' : Nat, probWhileCut nat_cond nat_body n k nat' *
                  (if nat' % U_nat = nat_val then 1 else 0))
      (fun k n₁ n₂ hn =>
        mul_le_mul_of_nonneg_left
          (ENNReal.tsum_le_tsum (fun nat' =>
            mul_le_mul_left (probWhileCut_monotonic nat_cond nat_body k nat' hn) _))
          (zero_le _))]

/-! ### nat-distribution of a UBig-valued program -/

/-- The nat-valued SLang distribution of a `Result`-monad program that outputs `UBig`. -/
noncomputable def samplerDist_nat
    (prog : Result (core.result.Result dashu_int.ubig.UBig error.Error)) :
    SLang Nat :=
  SLang.probBind (samplerDist prog) (fun ubig => SLang.probPure (dashu.ubigToNat ubig))

/-! ### Auxiliary: threshold ≤ 256^byte_len -/

theorem threshold_le_byte_range
    (upper : dashu_int.ubig.UBig)
    (setup : UniformBelowSetup upper) :
    dashu.ubigToNat setup.threshold ≤ bytes.byteRadix ^ setup.byte_len.val := by
  rw [← setup_range_eq_byte_range upper setup]
  rw [dashu.sub_spec setup.range setup.remainder setup.threshold setup.hthreshold]
  exact Nat.sub_le _ _

/-! ### Auxiliary: buffer allocation always succeeds -/

theorem from_elem_u8_zero_ok (n : Usize) :
    ∃ v : alloc.vec.Vec Std.U8,
      alloc.vec.from_elem core.clone.CloneU8 0#u8 n = ok v ∧ v.length = n.val := by
  have hclone : core.clone.CloneU8.clone 0#u8 = ok 0#u8 := by simp
  obtain ⟨v, hok, hP⟩ := spec_imp_exists (alloc.vec.from_elem_spec core.clone.CloneU8 0#u8 n hclone)
  exact ⟨v, hok, hP.2⟩

/-! ### Main theorem -/

/-- **End-to-end distributional correctness of `sample_uniform_ubig_below`**:
    the Rust implementation samples from the uniform distribution on `[0, upper)`.

    The only axiom used is `samplerDist_loop_rejection_uniform` (hardware trust). -/
theorem sample_uniform_ubig_below_pmf
    (upper : dashu_int.ubig.UBig)
    (hupper : 0 < dashu.ubigToNat upper) :
    samplerDist_nat (samplers.uniform.sample_uniform_ubig_below upper) =
    ↑(uniformNatBelowPMF upper hupper) := by
  obtain ⟨setup, hsetup⟩ := sample_uniform_ubig_below_setup_exists upper hupper
  have hthreshold := threshold_pos upper hupper setup
  have hle := threshold_le_byte_range upper setup
  -- Step 1: Decompose into deterministic alloc + loop
  have hdecomp := sample_uniform_ubig_below_eq_of_setup upper setup hsetup
  obtain ⟨buffer_init, hbuf_ok, hbuf_len⟩ := from_elem_u8_zero_ok setup.byte_len
  -- Reduce to just the loop
  have hprog_eq : samplers.uniform.sample_uniform_ubig_below upper =
      samplers.uniform.sample_uniform_ubig_below_loop upper setup.threshold buffer_init := by
    rw [hdecomp, hbuf_ok]; rfl
  -- Step 2: Apply loop axiom
  unfold samplerDist_nat
  rw [hprog_eq]
  have hloop := samplerDist_loop_rejection_uniform
    upper setup.threshold setup.byte_len buffer_init hbuf_len hupper hthreshold
  -- Step 3: probUntil(uniformByteNat) = UniformSample_PMF
  rw [probUntil_uniformByteNat_eq_uniform setup.byte_len.val
      (dashu.ubigToNat setup.threshold) hthreshold hle] at hloop
  rw [hloop]
  -- Step 4: Use hsucc to rewrite the RHS
  have hsucc : sample_uniform_ubig_below_success_pmf upper setup.threshold hthreshold =
               uniformNatBelowPMF upper hupper :=
    sample_uniform_ubig_below_success_pmf_eq_uniform upper hupper setup
  rw [← hsucc]
  simp only [sample_uniform_ubig_below_success_pmf]
  -- Goal: probBind ↑(UniformSample_PMF ⟨threshold,_⟩) (probPure ∘ (% upper))
  --     = ↑(PMF.map (% upper) (UniformSample_PMF ⟨threshold,_⟩))
  funext b
  simp only [SLang.probBind, SLang.probPure, PMF.map_apply, mul_ite, mul_one, mul_zero]
  congr 1; ext a; split_ifs <;> rfl

end OpenDP.samplers.uniform
