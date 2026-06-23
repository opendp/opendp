import src.samplers.bernoulli.semantics
import SampCert.Samplers.Geometric.Basic
import SampCert.Samplers.Laplace.Properties

open Aeneas Aeneas.Std Result Classical
open OpenDP SLang

namespace OpenDP.samplers.geometric

/-- Positive-input setup for the slow geometric sampler. The `x = 0` case is
excluded because the Rust reference loop never terminates there. -/
structure GeometricExpSlowSetup (x : dashu_ratio.rbig.RBig)
    extends OpenDP.samplers.bernoulli.BernoulliExpSetup x where
  hpos : 0 < dashu.ubigToNat numer

/-- Canonical target for `sample_geometric_exp_slow`: the number of consecutive
successful `Bernoulli(exp(-x))` draws before the first failure. This is the
zero-based version of SampCert's geometric loop, so it is defined by shifting
`probGeometric` down by one. -/
noncomputable def geometricExpSlowTarget
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) : SLang Nat :=
  fun n =>
    SLang.probGeometric
      (OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom) (n + 1)

/-- The Bernoulli trial used by the slow geometric target is proper. -/
theorem bernoulliExpTarget_normalizes
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) :
    OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom false +
        OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom true = 1 := by
  rw [OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_false,
    OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_true]
  have hle :
      ENNReal.ofReal
        (Real.exp
          (-((((dashu.ubigToNat numer : NNReal) /
              (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ)))) ≤ 1 := by
    have hexp_le : Real.exp
        (-((((dashu.ubigToNat numer : NNReal) /
            (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ))) ≤ 1 := by
      apply (Real.exp_le_one_iff).2
      have hnonneg :
          0 ≤
            ((((dashu.ubigToNat numer : NNReal) /
                (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ)) := by
        positivity
      linarith
    simpa using ENNReal.ofReal_le_ofReal hexp_le
  simpa [add_comm, add_left_comm, add_assoc] using
    (tsub_add_cancel_of_le hle : 1 - ENNReal.ofReal
      (Real.exp
        (-((((dashu.ubigToNat numer : NNReal) /
            (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ)))) +
        ENNReal.ofReal
          (Real.exp
            (-((((dashu.ubigToNat numer : NNReal) /
                (dashu.ubigToNat denom : NNReal) : NNReal) : ℝ)))) = 1)

/-- For strictly positive `x`, the `Bernoulli(exp(-x))` success probability is
strictly below one, so the induced geometric law is proper. -/
theorem bernoulliExpTarget_apply_true_lt_one
    (setup : GeometricExpSlowSetup x) :
    OpenDP.samplers.bernoulli.bernoulliExpTarget
      setup.numer setup.denom setup.hdenom true < 1 := by
  have hratio_pos :
      0 <
        (((dashu.ubigToNat setup.numer : NNReal) /
            (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ) := by
    norm_num
    exact div_pos (by exact_mod_cast setup.hpos) (by exact_mod_cast setup.hdenom)
  have hexp_lt :
      Real.exp
        (-((((dashu.ubigToNat setup.numer : NNReal) /
            (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ))) < 1 := by
    exact (Real.exp_lt_one_iff).2 (by linarith)
  simpa [OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_true] using
    (ENNReal.ofReal_lt_one.mpr hexp_lt)

/-- Primary mathematical result for the slow geometric sampler target. For any
strictly positive `x`, the `n`th output mass is `exp(-x)^n * (1 - exp(-x))`. -/
theorem geometricExpSlowTarget_apply
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
      (OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom true) ^ n *
        (OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom false) := by
  rw [geometricExpSlowTarget]
  exact SLang.probGeometric_apply
    (OpenDP.samplers.bernoulli.bernoulliExpTarget
      setup.numer setup.denom setup.hdenom)
    (n + 1)

/-- The slow geometric target is a proper distribution on `Nat`. -/
theorem geometricExpSlowTarget_normalizes
    (setup : GeometricExpSlowSetup x) :
    (∑' n : Nat, geometricExpSlowTarget setup.numer setup.denom setup.hdenom n) = 1 := by
  have hnorm :
      OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom false +
        OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom true = 1 :=
    bernoulliExpTarget_normalizes setup.numer setup.denom setup.hdenom
  have hlt :
      OpenDP.samplers.bernoulli.bernoulliExpTarget
        setup.numer setup.denom setup.hdenom true < 1 :=
    bernoulliExpTarget_apply_true_lt_one setup
  change
    (∑' n : Nat,
      SLang.probGeometric
        (OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom) (n + 1)) = 1
  exact SLang.probGeometric_normalizes'
    (trial := OpenDP.samplers.bernoulli.bernoulliExpTarget
      setup.numer setup.denom setup.hdenom)
    hnorm hlt

/-- Structural PMF package for the slow geometric target: pointwise mass,
normalization, and the exact SampCert geometric law all line up. -/
theorem geometricExpSlowTarget_spec
    (setup : GeometricExpSlowSetup x) :
    (∀ n : Nat,
      geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
        (OpenDP.samplers.bernoulli.bernoulliExpTarget
            setup.numer setup.denom setup.hdenom true) ^ n *
          (OpenDP.samplers.bernoulli.bernoulliExpTarget
            setup.numer setup.denom setup.hdenom false)) ∧
    (∑' n : Nat, geometricExpSlowTarget setup.numer setup.denom setup.hdenom n) = 1 := by
  refine ⟨?_, geometricExpSlowTarget_normalizes setup⟩
  intro n
  exact geometricExpSlowTarget_apply setup n

/-- Top-level PMF statement for the slow geometric sampler target. For
strictly positive inputs, the output law is exactly geometric with failure
parameter `1 - exp(-x)`. -/
theorem sample_geometric_exp_slow_pmf_spec
    (setup : GeometricExpSlowSetup x) :
    ∀ n : Nat,
      geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
        (OpenDP.samplers.bernoulli.bernoulliExpTarget
            setup.numer setup.denom setup.hdenom true) ^ n *
          (OpenDP.samplers.bernoulli.bernoulliExpTarget
            setup.numer setup.denom setup.hdenom false) := by
  intro n
  exact geometricExpSlowTarget_apply setup n

/-- Closed-form SampCert PMF for the slow geometric target. -/
theorem geometricExpSlowTarget_sampcert_pmf_spec
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
      SLang.Geo
        (1 - ENNReal.ofReal
          (Real.exp
            (-((dashu.ubigToNat setup.numer : ℝ) /
              (dashu.ubigToNat setup.denom : ℝ))))) n := by
  rw [geometricExpSlowTarget_apply setup n,
    OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_true,
    OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_false]
  unfold SLang.Geo
  have hratio :
      ((((dashu.ubigToNat setup.numer : NNReal) /
          (dashu.ubigToNat setup.denom : NNReal) : NNReal) : ℝ)) =
        (dashu.ubigToNat setup.numer : ℝ) /
          (dashu.ubigToNat setup.denom : ℝ) := by
    simp
  rw [hratio]
  rw [ENNReal.sub_sub_cancel]
  · simp
  · apply ENNReal.ofReal_le_one.mpr
    apply Real.exp_le_one_iff.mpr
    have hnonneg :
        0 ≤ (dashu.ubigToNat setup.numer : ℝ) /
          (dashu.ubigToNat setup.denom : ℝ) := by
      positivity
    exact neg_nonpos.mpr hnonneg

/-- The extracted slow geometric wrapper is definitionally the zero-initialized
loop wrapper. -/
theorem sample_geometric_exp_slow_eq
    (x : dashu_ratio.rbig.RBig) :
    OpenDP.samplers.geometric.sample_geometric_exp_slow x =
      (do
        let k ← dashu_int.ubig.UBig.ZERO
        OpenDP.samplers.geometric.sample_geometric_exp_slow_loop x k) := by
  rfl

/-- One extracted slow-geometric loop step continues with the incremented
counter when the Bernoulli subcall succeeds with `true`. -/
theorem sample_geometric_exp_slow_loop_body_eq_continue
    (x r : dashu_ratio.rbig.RBig)
    (k one k1 : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one =
        ok k1) :
    samplers.geometric.sample_geometric_exp_slow_loop.body x k =
      ok (ControlFlow.cont k1) := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop.body
  simp [hclone, hbern, hone, hadd,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- Strengthened slow-geometric `true` branch: the continuation counter is the
mathematical successor. -/
theorem sample_geometric_exp_slow_loop_body_continue_counter
    (x r : dashu_ratio.rbig.RBig)
    (k one k1 : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one =
        ok k1) :
    samplers.geometric.sample_geometric_exp_slow_loop.body x k =
      ok (ControlFlow.cont k1) ∧
    dashu.ubigToNat k1 = dashu.ubigToNat k + 1 := by
  refine ⟨?_, ?_⟩
  · exact sample_geometric_exp_slow_loop_body_eq_continue
      x r k one k1 hclone hbern hone hadd
  · calc
      dashu.ubigToNat k1 = dashu.ubigToNat k + dashu.ubigToNat one :=
        dashu.add_assign_spec k one k1 hadd
      _ = dashu.ubigToNat k + 1 := by rw [dashu.one_spec one hone]

/-- One extracted slow-geometric loop step stops with the current counter when
the Bernoulli subcall succeeds with `false`. -/
theorem sample_geometric_exp_slow_loop_body_eq_done
    (x r : dashu_ratio.rbig.RBig)
    (k : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok false)) :
    samplers.geometric.sample_geometric_exp_slow_loop.body x k =
      ok (ControlFlow.done (core.result.Result.Ok k)) := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop.body
  simp [hclone, hbern,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- One extracted slow-geometric loop step propagates Bernoulli errors. -/
theorem sample_geometric_exp_slow_loop_body_eq_error
    (x r : dashu_ratio.rbig.RBig)
    (k : dashu_int.ubig.UBig)
    (err : error.Error)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Err err)) :
    samplers.geometric.sample_geometric_exp_slow_loop.body x k =
      ok (ControlFlow.done (core.result.Result.Err err)) := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop.body
  simp [hclone, hbern,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- If the loop body stops in one step, the extracted slow geometric loop
returns that same result. Mirrors `sample_bernoulli_exp_loop_eq_done`. -/
theorem sample_geometric_exp_slow_loop_eq_done
    (x : dashu_ratio.rbig.RBig)
    (k : dashu_int.ubig.UBig)
    (r : core.result.Result dashu_int.ubig.UBig error.Error)
    (hbody :
      samplers.geometric.sample_geometric_exp_slow_loop.body x k =
        ok (ControlFlow.done r)) :
    samplers.geometric.sample_geometric_exp_slow_loop x k = ok r := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop
  grind [Aeneas.Std.loop]

/-- If the loop body continues to a new counter, the loop at the original
counter equals the loop at the new counter. -/
theorem sample_geometric_exp_slow_loop_eq_cont
    (x : dashu_ratio.rbig.RBig)
    (k k1 : dashu_int.ubig.UBig)
    (hbody :
      samplers.geometric.sample_geometric_exp_slow_loop.body x k =
        ok (ControlFlow.cont k1)) :
    samplers.geometric.sample_geometric_exp_slow_loop x k =
      samplers.geometric.sample_geometric_exp_slow_loop x k1 := by
  unfold samplers.geometric.sample_geometric_exp_slow_loop
  grind [Aeneas.Std.loop]

/-- When the first Bernoulli subcall returns false the slow geometric loop
exits immediately with the current counter. -/
theorem sample_geometric_exp_slow_loop_exit
    (x r : dashu_ratio.rbig.RBig)
    (k : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok false)) :
    samplers.geometric.sample_geometric_exp_slow_loop x k =
      ok (core.result.Result.Ok k) := by
  apply sample_geometric_exp_slow_loop_eq_done
  exact sample_geometric_exp_slow_loop_body_eq_done x r k hclone hbern

/-- When the first Bernoulli subcall returns true the slow geometric loop
recurses with the incremented counter k1. -/
theorem sample_geometric_exp_slow_loop_advance
    (x r : dashu_ratio.rbig.RBig)
    (k one k1 : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one =
        ok k1) :
    samplers.geometric.sample_geometric_exp_slow_loop x k =
      samplers.geometric.sample_geometric_exp_slow_loop x k1 := by
  apply sample_geometric_exp_slow_loop_eq_cont
  exact sample_geometric_exp_slow_loop_body_eq_continue x r k one k1 hclone hbern hone hadd

/-- After a single true Bernoulli step, the natural-number counter has
increased by exactly one. Packages `loop_advance` with the counter arithmetic. -/
theorem sample_geometric_exp_slow_loop_advance_counter
    (x r : dashu_ratio.rbig.RBig)
    (k one k1 : dashu_int.ubig.UBig)
    (hclone : dashu_ratio.rbig.RBig.Insts.CoreCloneClone.clone x = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_int.ubig.UBig.ONE = ok one)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddAssignUBig.add_assign k one =
        ok k1) :
    samplers.geometric.sample_geometric_exp_slow_loop x k =
      samplers.geometric.sample_geometric_exp_slow_loop x k1 ∧
    dashu.ubigToNat k1 = dashu.ubigToNat k + 1 := by
  refine ⟨sample_geometric_exp_slow_loop_advance x r k one k1 hclone hbern hone hadd, ?_⟩
  calc
    dashu.ubigToNat k1 = dashu.ubigToNat k + dashu.ubigToNat one :=
      dashu.add_assign_spec k one k1 hadd
    _ = dashu.ubigToNat k + 1 := by rw [dashu.one_spec one hone]

/-- Structural package for the slow geometric sampler: the extracted wrapper
is the generated loop wrapper, and the target has SampCert's closed-form
geometric PMF. -/
theorem sample_geometric_exp_slow_structural_spec
    (setup : GeometricExpSlowSetup x) :
    OpenDP.samplers.geometric.sample_geometric_exp_slow x =
      (do
        let k ← dashu_int.ubig.UBig.ZERO
        OpenDP.samplers.geometric.sample_geometric_exp_slow_loop x k) ∧
    (∀ n : Nat,
      geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
        SLang.Geo
          (1 - ENNReal.ofReal
            (Real.exp
              (-((dashu.ubigToNat setup.numer : ℝ) /
                (dashu.ubigToNat setup.denom : ℝ))))) n) := by
  refine ⟨sample_geometric_exp_slow_eq x, ?_⟩
  intro n
  exact geometricExpSlowTarget_sampcert_pmf_spec setup n

/-- SLang tail target for `sample_geometric_exp_slow_loop` when started at
counter `k`. At counter k, the loop counts further successes and returns
k + (number of additional successes before first failure). This is the clean
mathematical object the extracted loop should eventually be shown to realize
operationally. -/
noncomputable def geometricExpSlowLoopTailTarget
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (k : Nat) : SLang Nat :=
  fun n =>
    if k ≤ n then
      SLang.probGeometric
        (OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom)
        (n + 1 - k)
    else 0

/-- Starting the slow geometric tail target at counter `0` recovers the
canonical slow geometric target. -/
theorem geometricExpSlowLoopTailTarget_zero_eq
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom) :
    geometricExpSlowLoopTailTarget numer denom hdenom 0 =
      geometricExpSlowTarget numer denom hdenom := by
  ext n
  simp [geometricExpSlowLoopTailTarget, geometricExpSlowTarget]

/-- Explicit PMF for the slow geometric tail target: at counter k, the mass at
output n equals `q^(n-k) * (1-q)` for n ≥ k (zero otherwise), where
q = `bernoulliExpTarget.true`. -/
theorem geometricExpSlowLoopTailTarget_pmf
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (k n : Nat) :
    geometricExpSlowLoopTailTarget numer denom hdenom k n =
      if k ≤ n then
        (OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom true) ^ (n - k) *
        (OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom false)
      else 0 := by
  simp only [geometricExpSlowLoopTailTarget, probGeometric_apply]
  by_cases hle : k ≤ n
  · simp only [hle, ↓reduceIte]
    have hpos : n + 1 - k ≠ 0 := by omega
    simp only [hpos, ↓reduceIte]
    have hexp : n + 1 - k - 1 = n - k := by omega
    rw [hexp]
  · simp [hle]

/-- The tail target satisfies the loop body recurrence: the probability of
returning n from counter k equals the probability of immediate exit times the
indicator that n = k, plus the probability of one more success times the
probability of returning n from counter k+1. -/
theorem geometricExpSlowLoopTailTarget_recurrence
    (numer denom : dashu_int.ubig.UBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (k n : Nat) :
    geometricExpSlowLoopTailTarget numer denom hdenom k n =
      OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom false *
        (if n = k then 1 else 0) +
      OpenDP.samplers.bernoulli.bernoulliExpTarget numer denom hdenom true *
        geometricExpSlowLoopTailTarget numer denom hdenom (k + 1) n := by
  simp only [geometricExpSlowLoopTailTarget_pmf]
  rcases Nat.lt_trichotomy n k with h | h | h
  · -- n < k: both sides are zero
    rw [if_neg (by omega : ¬ k ≤ n), if_neg (by omega : n ≠ k),
        if_neg (by omega : ¬ k + 1 ≤ n)]
    simp
  · -- n = k (h : n = k)
    rw [if_pos (by omega : k ≤ n), if_pos h, if_neg (by omega : ¬ k + 1 ≤ n)]
    simp [show n - k = 0 from by omega]
  · -- k < n
    have hle : k ≤ n := Nat.le_of_lt h
    have hle' : k + 1 ≤ n := h
    have hne : n ≠ k := Nat.ne_of_gt h
    rw [if_pos hle, if_neg hne, if_pos hle']
    simp only [mul_zero, zero_add]
    have hexp : n - k = n - (k + 1) + 1 := by omega
    rw [hexp, pow_succ]
    ac_rfl

/-- The slow geometric tail target is a proper probability distribution: its
mass sums to 1 for every starting counter `k`.  The proof goes by induction:
the base case (k = 0) reduces to `probGeometric_normalizes'`, and the step
(k → k+1) uses the recurrence — summing it over all `n` gives
`p.false + p.true * S = 1`, which together with the normalization of the
Bernoulli trial (`p.false + p.true = 1`) forces `S = 1` by cancellation. -/
theorem geometricExpSlowLoopTailTarget_normalizes
    (x : dashu_ratio.rbig.RBig)
    (setup : GeometricExpSlowSetup x)
    (k : Nat) :
    ∑' n : Nat, geometricExpSlowLoopTailTarget setup.numer setup.denom setup.hdenom k n = 1 := by
  -- abbreviation for the Bernoulli trial
  let p := OpenDP.samplers.bernoulli.bernoulliExpTarget setup.numer setup.denom setup.hdenom
  induction k with
  | zero =>
    simp_rw [geometricExpSlowLoopTailTarget_zero_eq, geometricExpSlowTarget]
    exact probGeometric_normalizes' p
      (bernoulliExpTarget_normalizes setup.numer setup.denom setup.hdenom)
      (bernoulliExpTarget_apply_true_lt_one setup)
  | succ k ih =>
    -- p.true > 0 (exp(-r) > 0 for all r)
    have htrue_pos : 0 < p true := by
      simp only [p, OpenDP.samplers.bernoulli.bernoulliExpTarget_apply_true]
      exact ENNReal.ofReal_pos.mpr (Real.exp_pos _)
    -- p.true ≠ ⊤ (it's < 1 < ⊤)
    have htrue_ne_top : p true ≠ ⊤ :=
      ne_of_lt (lt_trans (bernoulliExpTarget_apply_true_lt_one setup) ENNReal.one_lt_top)
    -- p.false ≠ ⊤ (p.false + p.true = 1 ≠ ⊤, so both summands are finite)
    have hnorm : p false + p true = 1 :=
      bernoulliExpTarget_normalizes setup.numer setup.denom setup.hdenom
    have hpf_ne_top : p false ≠ ⊤ :=
      (ENNReal.add_ne_top.mp (hnorm ▸ ENNReal.one_ne_top)).1
    -- Sum the recurrence: ∑ n, T(k, n) = p.false + p.true * ∑ n, T(k+1, n)
    have hkey : ∑' n, geometricExpSlowLoopTailTarget setup.numer setup.denom setup.hdenom k n =
        p false + p true * ∑' n, geometricExpSlowLoopTailTarget setup.numer setup.denom setup.hdenom (k + 1) n := by
      simp_rw [geometricExpSlowLoopTailTarget_recurrence setup.numer setup.denom setup.hdenom k]
      rw [ENNReal.tsum_add, ENNReal.tsum_mul_left, ENNReal.tsum_mul_left,
          tsum_eq_single k (by intros n hn; simp [hn])]
      simp only [↓reduceIte, mul_one]; rfl
    -- Using ih, the sum is 1: p.false + p.true * S = 1
    set S := ∑' n, geometricExpSlowLoopTailTarget setup.numer setup.denom setup.hdenom (k + 1) n
    have hsum : p false + p true * S = 1 := hkey ▸ ih
    -- p.false + p.true * S = p.false + p.true, so p.true * S = p.true (cancel p.false via tsub)
    have hpteq : p true * S = p true := by
      have heq : p false + p true * S = p false + p true := by rw [hsum, hnorm]
      calc p true * S
          = p false + p true * S - p false := (ENNReal.add_sub_cancel_left hpf_ne_top).symm
        _ = p false + p true - p false     := by rw [heq]
        _ = p true                         := ENNReal.add_sub_cancel_left hpf_ne_top
    -- p.true * S = p.true * 1, cancel p.true (which is positive and finite)
    exact (ENNReal.mul_right_inj htrue_pos.ne' htrue_ne_top).mp (hpteq.trans (mul_one _).symm)

/-- End-to-end specification for the extracted `sample_geometric_exp_slow`.
This packages the extracted wrapper equality together with the closed-form
geometric PMF. -/
theorem sample_geometric_exp_slow_end_to_end_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : GeometricExpSlowSetup x) :
    sample_geometric_exp_slow x =
      (do
        let k ← dashu_int.ubig.UBig.ZERO
        sample_geometric_exp_slow_loop x k) ∧
    geometricExpSlowTarget setup.numer setup.denom setup.hdenom =
      SLang.Geo
        (1 - ENNReal.ofReal
          (Real.exp
            (-((dashu.ubigToNat setup.numer : ℝ) /
              (dashu.ubigToNat setup.denom : ℝ))))) := by
  rcases sample_geometric_exp_slow_structural_spec setup with
    ⟨hloop, hpmf⟩
  refine ⟨hloop, ?_⟩
  funext n
  exact hpmf n

/-- On positive input, the extracted fast geometric sampler reduces to the
expected setup-and-loop structure. -/
theorem sample_geometric_exp_fast_eq_of_setup
    (setup : GeometricExpSlowSetup x) :
    OpenDP.samplers.geometric.sample_geometric_exp_fast x =
      (do
        let (numer, denom) ← dashu_ratio.rbig.RBig.into_parts x
        let u ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom
        let r ← OpenDP.samplers.uniform.sample_uniform_ubig_below u
        let cf ←
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r
        match cf with
        | core.ops.control_flow.ControlFlow.Continue val =>
          OpenDP.samplers.geometric.sample_geometric_exp_fast_loop denom numer val
        | core.ops.control_flow.ControlFlow.Break residual =>
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual) := by
  have hzero :
      dashu_ratio.rbig.RBig.is_zero x = ok false :=
    dashu.rbig_is_zero_false_spec x setup.numerSigned setup.numer setup.denom
      setup.hparts setup.hsign setup.hpos
  unfold OpenDP.samplers.geometric.sample_geometric_exp_fast
  simp [hzero, setup.hparts]
  rfl

/-- The extracted fast geometric wrapper returns zero on the zero input
branch. -/
theorem sample_geometric_exp_fast_eq_zero
    (x : dashu_ratio.rbig.RBig)
    (hzero : dashu_ratio.rbig.RBig.is_zero x = ok true) :
    OpenDP.samplers.geometric.sample_geometric_exp_fast x =
      (do
        let u ← dashu_int.ubig.UBig.ZERO
        ok (core.result.Result.Ok u)) := by
  unfold OpenDP.samplers.geometric.sample_geometric_exp_fast
  simp [hzero]

/-- One accepted fast-loop step returns `(v2 * denom + u) / numer` when the
residue Bernoulli subcall succeeds. -/
theorem sample_geometric_exp_fast_loop_eq_accept
    (denom u val1 u1 u2 u3 u4 u5 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r r2 : dashu_ratio.rbig.RBig)
    (sgn : dashu_base.sign.Sign)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_ratio.rbig.RBig.ONE = ok r2)
    (hslow :
      samplers.geometric.sample_geometric_exp_slow r2 =
        ok (core.result.Result.Ok val1))
    (hmul :
      dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom =
        ok u2)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u = ok u3)
    (hparts : dashu_int.ibig.IBig.into_parts numer = ok (sgn, u4))
    (hdiv :
      dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4 = ok u5) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      ok (core.result.Result.Ok u5) := by
  unfold samplers.geometric.sample_geometric_exp_fast_loop
  simp [hasI, hcloneI, hcloneDenom, hfrom, hbern, hone, hslow, hmul,
    hadd, hparts, hdiv,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- Strengthened accepted fast-loop step: the returned value has the expected
mathematical quotient, assuming the final exact division is valid. -/
theorem sample_geometric_exp_fast_loop_accept_value
    (denom u val1 u1 u2 u3 u4 u5 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r r2 : dashu_ratio.rbig.RBig)
    (sgn : dashu_base.sign.Sign)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_ratio.rbig.RBig.ONE = ok r2)
    (hslow :
      samplers.geometric.sample_geometric_exp_slow r2 =
        ok (core.result.Result.Ok val1))
    (hmul :
      dashu_int.ubig.UBig.Insts.CoreOpsArithMulUBigUBig.mul val1 denom =
        ok u2)
    (hadd :
      dashu_int.ubig.UBig.Insts.CoreOpsArithAddUBigUBig.add u2 u = ok u3)
    (hparts : dashu_int.ibig.IBig.into_parts numer = ok (sgn, u4))
    (hdiv :
      dashu_int.ubig.UBig.Insts.CoreOpsArithDivUBigUBig.div u3 u4 = ok u5)
    (hexact : dashu.ubigToNat u4 ∣ dashu.ubigToNat u3) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      ok (core.result.Result.Ok u5) ∧
    dashu.ubigToNat u5 =
      (dashu.ubigToNat val1 * dashu.ubigToNat denom + dashu.ubigToNat u) /
        dashu.ubigToNat u4 := by
  refine ⟨?_, ?_⟩
  · exact sample_geometric_exp_fast_loop_eq_accept
      denom u val1 u1 u2 u3 u4 u5 numer i i1 r r2 sgn hasI hcloneI
      hcloneDenom hfrom hbern hone hslow hmul hadd hparts hdiv
  · have hmulNat := dashu.mul_ubig_spec val1 denom u2 hmul
    have haddNat := dashu.add_ubig_spec u2 u u3 hadd
    have hdivNat := dashu.div_ubig_spec u3 u4 u5 hdiv hexact
    calc
      dashu.ubigToNat u5 = dashu.ubigToNat u3 / dashu.ubigToNat u4 := hdivNat
      _ =
          (dashu.ubigToNat val1 * dashu.ubigToNat denom +
              dashu.ubigToNat u) / dashu.ubigToNat u4 := by
            rw [haddNat, hmulNat]

/-- The rational constructed at the head of the fast loop is the residue
fraction `u / denom`, so the Bernoulli subcall is aligned with the
corresponding negative-exponential target. -/
theorem sample_geometric_exp_fast_loop_residue_setup
    (denom u u1 : dashu_int.ubig.UBig)
    (i i1 : dashu_int.ibig.IBig)
    (r : dashu_ratio.rbig.RBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r) :
    ∃ setup : OpenDP.samplers.bernoulli.BernoulliExpSetup r,
      setup.numerSigned = i1 ∧ setup.numer = u ∧ setup.denom = u1 := by
  have hsignI :
      dashu_int.ibig.IBig.into_parts i =
        ok (dashu_base.sign.Sign.Positive, u) :=
    dashu.as_ibig_spec u i hasI
  have hsignI1 :
      dashu_int.ibig.IBig.into_parts i1 =
        ok (dashu_base.sign.Sign.Positive, u) :=
    dashu.ibig_clone_parts_spec i i1
      (dashu_base.sign.Sign.Positive, u) hcloneI hsignI
  have hdenomClone : dashu.ubigToNat u1 = dashu.ubigToNat denom :=
    dashu.clone_spec denom u1 hcloneDenom
  have hdenom1 : 0 < dashu.ubigToNat u1 := by
    rw [hdenomClone]
    exact hdenom
  have hparts :
      dashu_ratio.rbig.RBig.into_parts r = ok (i1, u1) :=
    dashu.rbig_from_parts_positive_spec u u1 i1 r hdenom1 hsignI1 hfrom
  refine ⟨{
    numerSigned := i1
    numer := u
    denom := u1
    hparts := hparts
    hsign := hsignI1
    hdenom := hdenom1
  }, rfl, rfl, rfl⟩

/-- The Bernoulli target of the fast-loop residue subcall is the SampCert
negative-exponential Bernoulli law with numerator `u` and denominator
`denom`'s clone. -/
theorem sample_geometric_exp_fast_loop_residue_bernoulli_target
    (denom u u1 : dashu_int.ubig.UBig)
    (i i1 : dashu_int.ibig.IBig)
    (r : dashu_ratio.rbig.RBig)
    (hdenom : 0 < dashu.ubigToNat denom)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r) :
    ∃ setup : OpenDP.samplers.bernoulli.BernoulliExpSetup r,
      setup.numerSigned = i1 ∧
      setup.numer = u ∧
      setup.denom = u1 ∧
      OpenDP.samplers.bernoulli.bernoulliExpTarget
          setup.numer setup.denom setup.hdenom =
        SLang.BernoulliExpNegSample
          (dashu.ubigToNat setup.numer)
          ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ := by
  rcases sample_geometric_exp_fast_loop_residue_setup
    denom u u1 i i1 r hdenom hasI hcloneI hcloneDenom hfrom with
    ⟨setup, rfl, rfl, rfl⟩
  refine ⟨setup, rfl, rfl, rfl, ?_⟩
  rfl

/-- One rejected fast-loop step redraws a uniform residue and recurs. -/
theorem sample_geometric_exp_fast_loop_eq_reject
    (denom u val1 u1 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r : dashu_ratio.rbig.RBig)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok false))
    (huniform :
      samplers.uniform.sample_uniform_ubig_below u1 =
        ok (core.result.Result.Ok val1)) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      samplers.geometric.sample_geometric_exp_fast_loop denom numer val1 := by
  conv =>
    lhs
    unfold samplers.geometric.sample_geometric_exp_fast_loop
  simp [hasI, hcloneI, hcloneDenom, hfrom, hbern, huniform,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch]

/-- The fast loop propagates errors from the residue Bernoulli subcall. -/
theorem sample_geometric_exp_fast_loop_eq_bernoulli_error
    (denom u u1 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r : dashu_ratio.rbig.RBig)
    (err : error.Error)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Err err)) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      ok (core.result.Result.Err err) := by
  unfold samplers.geometric.sample_geometric_exp_fast_loop
  simp [hasI, hcloneI, hcloneDenom, hfrom, hbern,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- The accepted fast-loop branch propagates errors from the slow geometric
subcall. -/
theorem sample_geometric_exp_fast_loop_eq_slow_error
    (denom u u1 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r r2 : dashu_ratio.rbig.RBig)
    (err : error.Error)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok true))
    (hone : dashu_ratio.rbig.RBig.ONE = ok r2)
    (hslow :
      samplers.geometric.sample_geometric_exp_slow r2 =
        ok (core.result.Result.Err err)) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      ok (core.result.Result.Err err) := by
  unfold samplers.geometric.sample_geometric_exp_fast_loop
  simp [hasI, hcloneI, hcloneDenom, hfrom, hbern, hone, hslow,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- The rejected fast-loop branch propagates errors from uniform-redraw. -/
theorem sample_geometric_exp_fast_loop_eq_uniform_error
    (denom u u1 : dashu_int.ubig.UBig)
    (numer i i1 : dashu_int.ibig.IBig)
    (r : dashu_ratio.rbig.RBig)
    (err : error.Error)
    (hasI : dashu_int.convert.UBig.as_ibig u = ok i)
    (hcloneI : dashu_int.ibig.IBig.Insts.CoreCloneClone.clone i = ok i1)
    (hcloneDenom :
      dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom = ok u1)
    (hfrom : dashu_ratio.rbig.RBig.from_parts i1 u1 = ok r)
    (hbern :
      samplers.bernoulli.sample_bernoulli_exp r =
        ok (core.result.Result.Ok false))
    (huniform :
      samplers.uniform.sample_uniform_ubig_below u1 =
        ok (core.result.Result.Err err)) :
    samplers.geometric.sample_geometric_exp_fast_loop denom numer u =
      ok (core.result.Result.Err err) := by
  unfold samplers.geometric.sample_geometric_exp_fast_loop
  simp [hasI, hcloneI, hcloneDenom, hfrom, hbern, huniform,
    core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch,
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual]

/-- Sampling an independent fair sign bit and then projecting it away leaves
the original natural-valued distribution unchanged. -/
private theorem bind_fair_coin_project_second_eq
    (m : SLang Nat) :
    ((m >>= fun y =>
        SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl) >>= fun b =>
          Pure.pure (b, y)) >>= fun st =>
      Pure.pure st.2) = m := by
  change
    ((m.probBind fun y =>
        (SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl)).probBind fun b =>
          SLang.probPure (b, y)).probBind fun st =>
      SLang.probPure st.2) = m
  rw [SLang.bind_bind]
  simp only [SLang.bind_bind, SLang.pure_bind]
  have hcoin_const :
      ∀ y : Nat,
        (SLang.BernoulliSample 1 2 (Nat.le.step Nat.le.refl)).probBind
            (fun _ => SLang.probPure y) =
          SLang.probPure y := by
    intro y
    apply SLang.ext
    intro z
    simp only [SLang.bind_apply, SLang.pure_apply]
    rw [tsum_bool]
    by_cases hzy : z = y
    · simp [hzy]
      simpa [one_div] using (ENNReal.add_halves (1 : ENNReal))
    · simp [hzy]
  simp [hcoin_const]

/-- `Geo(exp(-1/denom)) >>= (· / numer)` equals `Geo(1 - exp(-1/denom)^numer)` by
`geo_div_geo`. Used to connect the quotient-form target to the standard PMF. -/
private theorem geometricExpFastSampCertTarget_hp_lt
    (setup : GeometricExpSlowSetup x) :
    ENNReal.ofReal (Real.exp (-(1 / (dashu.ubigToNat setup.denom : ℝ)))) < 1 := by
  apply ENNReal.ofReal_lt_one.mpr
  apply Real.exp_lt_one_iff.2
  have hden_pos : 0 < (dashu.ubigToNat setup.denom : ℝ) := by exact_mod_cast setup.hdenom
  linarith [div_pos one_pos hden_pos]

/-- The exponent power identity: `exp(-1/denom)^numer = exp(-numer/denom)`. -/
private theorem geometricExpFastSampCertTarget_pow_eq
    (setup : GeometricExpSlowSetup x) :
    (ENNReal.ofReal (Real.exp (-(1 / (dashu.ubigToNat setup.denom : ℝ))))) ^
        dashu.ubigToNat setup.numer =
      ENNReal.ofReal
        (Real.exp (-((dashu.ubigToNat setup.numer : ℝ) /
          (dashu.ubigToNat setup.denom : ℝ)))) := by
  rw [← ENNReal.ofReal_pow (Real.exp_nonneg _)]
  congr 1
  rw [← Real.exp_nat_mul]
  congr 1
  have hden_ne : (dashu.ubigToNat setup.denom : ℝ) ≠ 0 :=
    ne_of_gt (by exact_mod_cast setup.hdenom)
  field_simp

/-- Quotienting a geometric law by the positive numerator used by the fast
sampler yields the same quotient-form target that appears in SampCert's
geometric division lemma. This is the clean mathematical shape behind the
`v2 * denom + u` / `numer` step in the Rust fast path. -/
noncomputable def geometricExpFastSampCertTarget
    (numer denom : dashu_int.ubig.UBig) :
    SLang Nat :=
  SLang.Geo
    (1 - ENNReal.ofReal
      (Real.exp (-(1 / (dashu.ubigToNat denom : ℝ))))) >>= fun v =>
    Pure.pure (v / dashu.ubigToNat numer)

/-- SampCert's optimized discrete-Laplace inner loop, projected onto the
non-negative magnitude. This is the SampCert-side target for the Rust fast
geometric sampler: the loop samples an accepted residue, adds a geometric
multiple of the denominator, quotients by the numerator, and then samples an
independent sign bit that is marginalized away by projecting `st.2`. -/
noncomputable def geometricExpFastSampCertProgram
    (setup : GeometricExpSlowSetup x) : SLang Nat :=
  SLang.DiscreteLaplaceSampleLoop'
      ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
      ⟨dashu.ubigToNat setup.numer, setup.hpos⟩ >>= fun st =>
    Pure.pure st.2

/-- Rust-shaped SampCert target for the fast geometric loop: sample an
accepted residue, sample the `x = 1` slow geometric count in SampCert's
one-based representation, combine them as `u + denom * (v - 1)`, and quotient
by the numerator. -/
noncomputable def geometricExpFastRustLoopTarget
    (setup : GeometricExpSlowSetup x) : SLang Nat := do
  let denomPNat : PNat := ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
  let numerPNat : PNat := ⟨dashu.ubigToNat setup.numer, setup.hpos⟩
  let u ← SLang.DiscreteLaplaceSampleLoopIn1 denomPNat
  let v ← SLang.DiscreteLaplaceSampleLoopIn2 1 1
  let vShifted := v - 1
  let combined := u + denomPNat * vShifted
  Pure.pure (combined / numerPNat)

/-- The Rust-shaped target is exactly SampCert's optimized loop after
marginalizing away the independent sign bit. -/
theorem geometricExpFastRustLoopTarget_eq_sampcertProgram
    (setup : GeometricExpSlowSetup x) :
    geometricExpFastRustLoopTarget setup =
      geometricExpFastSampCertProgram setup := by
  unfold geometricExpFastRustLoopTarget geometricExpFastSampCertProgram
  unfold SLang.DiscreteLaplaceSampleLoop'
  let denomPNat : PNat := ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
  let numerPNat : PNat := ⟨dashu.ubigToNat setup.numer, setup.hpos⟩
  simpa [denomPNat, numerPNat] using
    (bind_fair_coin_project_second_eq
      (SLang.DiscreteLaplaceSampleLoopIn1 denomPNat >>= fun u =>
        SLang.DiscreteLaplaceSampleLoopIn2 1 1 >>= fun v =>
        let vShifted := v - 1
        let combined := u + denomPNat * vShifted
        Pure.pure (combined / numerPNat))).symm

/-- The SampCert program corresponding to the Rust fast geometric sampler has
the same closed-form geometric PMF as the direct geometric target. -/
theorem geometricExpFastSampCertProgram_spec
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpFastSampCertProgram setup n =
      SLang.Geo
        (1 - ENNReal.ofReal
          (Real.exp
            (-((dashu.ubigToNat setup.numer : ℝ) /
              (dashu.ubigToNat setup.denom : ℝ))))) n := by
  let denomPNat : PNat := ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩
  let numerPNat : PNat := ⟨dashu.ubigToNat setup.numer, setup.hpos⟩
  let p : ENNReal := ENNReal.ofReal (Real.exp (-(↑↑numerPNat / ↑↑denomPNat)))
  unfold geometricExpFastSampCertProgram
  change
    (SLang.DiscreteLaplaceSampleLoop' denomPNat numerPNat >>= fun st =>
      Pure.pure st.2) n =
    SLang.Geo (1 - p) n
  rw [← SLang.DiscreteLaplaceSampleLoop_equiv denomPNat numerPNat]
  simp only [Bind.bind, Pure.pure, SLang.bind_apply, SLang.pure_apply]
  rw [ENNReal.tsum_prod']
  rw [tsum_bool]
  simp only [SLang.DiscreteLaplaceSampleLoop_apply, SLang.Geo]
  have hsingle_false :
      (∑' b : Nat, (p ^ b * (1 - p)) * ((2 : PNat) : ENNReal)⁻¹ *
          (@ite ENNReal (n = b) (Classical.propDecidable (n = (false, b).2)) 1 0)) =
        (p ^ n * (1 - p)) * ((2 : PNat) : ENNReal)⁻¹ := by
    rw [tsum_eq_single n]
    · simp
    · intro b hb
      have hne : n ≠ b := by exact fun h => hb h.symm
      simp [hne]
  rw [hsingle_false]
  have hhalf :
      ((2 : PNat) : ENNReal)⁻¹ + ((2 : PNat) : ENNReal)⁻¹ = 1 := by
    simpa [one_div] using (ENNReal.add_halves (1 : ENNReal))
  rw [← mul_add, hhalf, mul_one]
  rw [ENNReal.sub_sub_cancel]
  · simp
  · dsimp [p]
    apply ENNReal.ofReal_le_one.mpr
    apply Real.exp_le_one_iff.mpr
    have hnonneg :
        0 ≤ (↑↑numerPNat / ↑↑denomPNat : ℝ) := by
      positivity
    exact neg_nonpos.mpr hnonneg

/-- Primary PMF for the Rust-shaped fast-loop target. This is the closest
SampCert-side statement to the Rust loop: accepted residue, one-based slow
geometric count, arithmetic combination, quotient by the numerator. -/
theorem geometricExpFastRustLoopTarget_pmf_spec
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpFastRustLoopTarget setup n =
      SLang.Geo
        (1 - ENNReal.ofReal
          (Real.exp
            (-((dashu.ubigToNat setup.numer : ℝ) /
              (dashu.ubigToNat setup.denom : ℝ))))) n := by
  rw [geometricExpFastRustLoopTarget_eq_sampcertProgram setup,
    geometricExpFastSampCertProgram_spec setup n]

/-- The slow reference target and the Rust-shaped fast-loop target are the
same distribution on positive inputs. -/
theorem geometricExpSlowTarget_eq_rustLoopTarget
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
      geometricExpFastRustLoopTarget setup n := by
  rw [geometricExpSlowTarget_sampcert_pmf_spec setup n,
    geometricExpFastRustLoopTarget_pmf_spec setup n]

/-- Quotienting `Geo(1 - exp(-1/denom))` by `numer` gives `Geo(1 - exp(-1/denom)^numer)`.
This applies `geo_div_geo` to the definition of `geometricExpFastSampCertTarget`. -/
theorem geometricExpFastQuotientTarget_spec
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpFastSampCertTarget setup.numer setup.denom n =
      SLang.Geo
        (1 - (ENNReal.ofReal
          (Real.exp (-(1 / (dashu.ubigToNat setup.denom : ℝ))))) ^
            dashu.ubigToNat setup.numer) n := by
  rw [geometricExpFastSampCertTarget]
  exact SLang.geo_div_geo n (dashu.ubigToNat setup.numer)
    (ENNReal.ofReal (Real.exp (-(1 / (dashu.ubigToNat setup.denom : ℝ)))))
    (geometricExpFastSampCertTarget_hp_lt setup) setup.hpos

/-- For all outputs, the quotient target is `Geo(1 - exp(-1/denom)^numer)`. -/
theorem geometricExpFastSampCertTarget_spec
    (setup : GeometricExpSlowSetup x) :
    ∀ n : Nat,
      geometricExpFastSampCertTarget setup.numer setup.denom n =
        SLang.Geo
          (1 - (ENNReal.ofReal
            (Real.exp (-(1 / (dashu.ubigToNat setup.denom : ℝ))))) ^
              dashu.ubigToNat setup.numer) n :=
  fun n => geometricExpFastQuotientTarget_spec setup n

/-- The quotient target and the SampCert program have the same pointwise PMF.
This is the algebraic bridge: `Geo(exp(-1/denom)) >>= (· / numer)` and
`Geo(1 - exp(-numer/denom))` define the same distribution, because
`exp(-1/denom)^numer = exp(-numer/denom)`. -/
theorem geometricExpFastQuotientTarget_eq_sampcertProgram
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpFastSampCertTarget setup.numer setup.denom n =
      geometricExpFastSampCertProgram setup n := by
  rw [geometricExpFastQuotientTarget_spec setup n,
      geometricExpFastSampCertTarget_pow_eq setup,
      geometricExpFastSampCertProgram_spec setup n]

/-- The quotient-form target has the standard Geo PMF: `Geo(1 - exp(-numer/denom))`.
Combines `geo_div_geo` with the exponential identity
`exp(-1/denom)^numer = exp(-numer/denom)`. -/
theorem geometricExpFastSampCertTarget_pmf
    (setup : GeometricExpSlowSetup x)
    (n : Nat) :
    geometricExpFastSampCertTarget setup.numer setup.denom n =
      SLang.Geo
        (1 - ENNReal.ofReal
          (Real.exp
            (-((dashu.ubigToNat setup.numer : ℝ) /
              (dashu.ubigToNat setup.denom : ℝ))))) n := by
  rw [geometricExpFastQuotientTarget_eq_sampcertProgram setup n,
      geometricExpFastSampCertProgram_spec setup n]

/-- Top-level positive-input specification for the fast geometric sampler.
This proves the extracted wrapper reaches the generated fast loop, and that
the slow reference target matches the Rust-shaped fast-loop target. The
closed-form PMF is then supplied separately by
`geometricExpFastRustLoopTarget_pmf_spec`. -/
theorem sample_geometric_exp_fast_spec
    (x : dashu_ratio.rbig.RBig)
    (setup : GeometricExpSlowSetup x) :
    OpenDP.samplers.geometric.sample_geometric_exp_fast x =
      (do
        let (numer, denom) ← dashu_ratio.rbig.RBig.into_parts x
        let u ← dashu_int.ubig.UBig.Insts.CoreCloneClone.clone denom
        let r ← OpenDP.samplers.uniform.sample_uniform_ubig_below u
        let cf ←
          core.result.Result.Insts.CoreOpsTry_traitTryTResultInfallibleE.branch r
        match cf with
        | core.ops.control_flow.ControlFlow.Continue val =>
          OpenDP.samplers.geometric.sample_geometric_exp_fast_loop denom numer val
        | core.ops.control_flow.ControlFlow.Break residual =>
          core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
            dashu_int.ubig.UBig (core.convert.FromSame error.Error) residual) ∧
    ∀ n : Nat,
      geometricExpSlowTarget setup.numer setup.denom setup.hdenom n =
        geometricExpFastRustLoopTarget setup n := by
  exact ⟨sample_geometric_exp_fast_eq_of_setup setup,
    by
      intro n
      exact geometricExpSlowTarget_eq_rustLoopTarget setup n⟩

end OpenDP.samplers.geometric
