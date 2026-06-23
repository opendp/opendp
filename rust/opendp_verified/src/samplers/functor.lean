import Aeneas
import Generated.OpenDP
import SampCert.SLang
import src.samplers.bytes

open Aeneas Aeneas.Std Result ControlFlow
open OpenDP SLang PMF ENNReal

namespace OpenDP.samplers

/-! ## Probabilistic Semantics

Aeneas extracts Rust programs into the `Result` monad — purely deterministic.
Randomness arises solely from `fill_bytes` calls at the hardware boundary.

`samplerDistGen A prog` assigns every `Result A` computation its SLang distribution,
for ANY type `A` (including intermediate `ControlFlow` types from loops).
`samplerDist` is the `Ok`-projection of `samplerDistGen`, giving the success distribution.

**The single randomness assumption** is `samplerDistGen_exists`: a consistent
probabilistic semantics for ALL `Result A` programs exists, characterized by five
simultaneous constraints:
  (i)    deterministic `ok` maps to `PMF.pure` (for any A)
  (ii)   sequential composition distributes as `probBind` (for any A, B)
  (iii)  `fill_bytes` draws bytes uniformly (pointwise in k) — the lone stochastic claim
  (iii.b) `fill_bytes` preserves buffer length
  (iv)   `loop body x` distributes as `probWhile` of the body's distribution

This is the only assumption about *randomness*. It is not the only `axiom` the final
theorems depend on: the Aeneas extraction models trusted Rust externals as opaque
constants (`fill_bytes` itself, and the deterministic bignum operations in
`src/externals/dashu.lean` — `from_be_bytes`, `lt`, `rem`, `sub`, …). Those are
ordinary deterministic-library specs, a separate trust category. Run
`#print axioms sample_uniform_ubig_below_pmf` to certify the complete dependency set.

**Why the randomness layer is closed**: every Aeneas-extracted program is built from
`ok`, `fail`, `>>=`, `fill_bytes` (the lone stochastic external), and `loop` (the lone
recursion mechanism). Constraints (i)–(iv) cover all cases; `fail`/`div` carry zero mass
by (ii) alone. No future sampler needs a new probabilistic constraint unless a new
stochastic external is added to the crate (a deterministic external only adds a
dashu-style value/totality spec).
-/

/-! ### The randomness axiom -/

/-- **The single randomness axiom**: a consistent probabilistic semantics for ALL
    `Result`-monad programs exists (including `ControlFlow`-typed loop intermediates),
    where `fill_bytes` draws bytes uniformly and loops distribute as `probWhile`.

    The constraints are stated jointly because they characterize a single consistent
    model, not independent claims. Together they uniquely determine `samplerDistGen` on
    every program expressible in the Aeneas extraction. Deterministic external behavior
    (dashu bignum ops) is modeled separately; see `#print axioms`. -/
axiom samplerDistGen_exists :
    ∃ (f : ∀ (A : Type), Result A → SLang A),
    /- (i) deterministic success has a point-mass distribution, for any A -/
    (∀ (A : Type) (v : A),
        f A (ok v) = PMF.pure v) ∧
    /- (ii) sequential composition distributes as SLang.probBind, for any A → B -/
    (∀ (A B : Type) (prog1 : Result A) (prog2 : A → Result B),
        f B (prog1 >>= prog2) =
        SLang.probBind (f A prog1) (fun v => f B (prog2 v))) ∧
    /- (iii) fill_bytes draws a big-endian nat uniformly on [0, 256^n) — pointwise -/
    (∀ (buffer : Slice Std.U8) (k : Nat),
        f (core.result.Result Nat error.Error)
          (do let pair ← samplers.fill_bytes buffer
              match pair.1 with
              | core.result.Result.Ok _ =>
                  ok (core.result.Result.Ok (bytes.beBytesToNat pair.2.val))
              | core.result.Result.Err e =>
                  ok (core.result.Result.Err e))
          (core.result.Result.Ok k) =
        bytes.uniformByteNatPMF buffer.length k) ∧
    /- (iii.b) fill_bytes preserves the buffer length: only outputs with the same
        length as the input have positive probability in the model. -/
    (∀ (buffer : Slice Std.U8)
       (r : core.result.Result Unit error.Error)
       (s1 : Slice Std.U8),
        0 < f ((core.result.Result Unit error.Error) × Slice Std.U8)
              (samplers.fill_bytes buffer)
              (r, s1) →
        s1.length = buffer.length) ∧
    /- (iv) Aeneas loops distribute as SampCert probWhile of the body's distribution.
        State is lifted to ControlFlow A B: the loop runs while in `cont` and exits
        with `done v`; the output probability at v is the probWhile mass at (done v). -/
    (∀ (A B : Type) (body : A → Result (ControlFlow A B)) (x : A) (v : B),
        f B (loop body x) v =
        probWhile
          (fun cf : ControlFlow A B => match cf with | cont _ => true | done _ => false)
          (fun cf : ControlFlow A B => match cf with
            | cont a => f (ControlFlow A B) (body a)
            | done _ => PMF.pure cf)
          (cont x) (done v))

/-! ### The generalized functor -/

/-- The SLang distribution of ANY `Result A` computation.
    For `A = ControlFlow α β`, gives the distribution over control-flow outcomes.
    Defined as the canonical choice from `samplerDistGen_exists`. -/
noncomputable def samplerDistGen {A : Type} (prog : Result A) : SLang A :=
  Classical.choose samplerDistGen_exists A prog

/-- The SUCCESS distribution of a `Result (Result α Error)` computation.
    `samplerDist prog v` is the probability that `prog` outputs `Ok v`.
    Sub-probabilistic: `∑ v, samplerDist prog v ≤ 1`, deficit = failure probability.
    Defined as the `Ok`-projection of `samplerDistGen`. -/
noncomputable def samplerDist {α : Type}
    (prog : Result (core.result.Result α error.Error)) : SLang α :=
  fun v => samplerDistGen prog (core.result.Result.Ok v)

/-! ### Theorems for `samplerDistGen` -/

theorem samplerDistGen_pure_ok {A : Type} (v : A) :
    samplerDistGen (ok v) = PMF.pure v :=
  (Classical.choose_spec samplerDistGen_exists).1 A v

theorem samplerDistGen_bind {A B : Type}
    (prog1 : Result A) (prog2 : A → Result B) :
    samplerDistGen (prog1 >>= prog2) =
    SLang.probBind (samplerDistGen prog1) (fun v => samplerDistGen (prog2 v)) :=
  (Classical.choose_spec samplerDistGen_exists).2.1 A B prog1 prog2

/-- **The hardware theorem**: fill_bytes draws nat k with probability uniformByteNatPMF n k.
    All distributional content flows from here. -/
theorem samplerDistGen_fill_bytes_nat (buffer : Slice Std.U8) (k : Nat) :
    samplerDistGen
      (do let pair ← samplers.fill_bytes buffer
          match pair.1 with
          | core.result.Result.Ok _ =>
              ok (core.result.Result.Ok (bytes.beBytesToNat pair.2.val))
          | core.result.Result.Err e =>
              ok (core.result.Result.Err e))
      (core.result.Result.Ok k) =
    bytes.uniformByteNatPMF buffer.length k :=
  (Classical.choose_spec samplerDistGen_exists).2.2.1 buffer k

/-- **The fill_bytes length theorem**: the probabilistic model can only assign positive
    probability to outputs where fill_bytes returns a buffer of the same length. -/
theorem samplerDistGen_fill_bytes_length_pres
    (buffer : Slice Std.U8)
    (r : core.result.Result Unit error.Error)
    (s1 : Slice Std.U8)
    (h : 0 < samplerDistGen (samplers.fill_bytes buffer) (r, s1)) :
    s1.length = buffer.length :=
  (Classical.choose_spec samplerDistGen_exists).2.2.2.1 buffer r s1 h

/-- **The loop theorem**: the distribution of an Aeneas `loop` equals `SLang.probWhile`
    of the body's distribution.  Covers every sampler loop (uniform, bernoulli, geometric)
    without a per-loop axiom. -/
theorem samplerDistGen_loop {A B : Type}
    (body : A → Result (ControlFlow A B)) (x : A) (v : B) :
    samplerDistGen (loop body x) v =
    probWhile
      (fun cf : ControlFlow A B => match cf with | cont _ => true | done _ => false)
      (fun cf : ControlFlow A B => match cf with
        | cont a => samplerDistGen (body a)
        | done _ => PMF.pure cf)
      (cont x) (done v) :=
  (Classical.choose_spec samplerDistGen_exists).2.2.2.2 A B body x v

/-! ### Derived theorems for `samplerDist` -/

/-- For a function zero on all `Err` values, the tsum over `Result` equals the tsum over `Ok`. -/
lemma tsum_result_ok_eq
    {α ε : Type}
    {f : core.result.Result α ε → ENNReal}
    (hErr : ∀ e : ε, f (core.result.Result.Err e) = 0) :
    ∑' r : core.result.Result α ε, f r = ∑' v : α, f (core.result.Result.Ok v) := by
  haveI hdecα : DecidableEq α := Classical.decEq α
  -- Decidable predicate: does r have an Ok constructor?
  let isOk : core.result.Result α ε → Prop :=
    fun r => match r with | core.result.Result.Ok _ => True | core.result.Result.Err _ => False
  haveI hOkDec : DecidablePred isOk := fun r =>
    match r with
    | core.result.Result.Ok _ => Decidable.isTrue trivial
    | core.result.Result.Err _ => Decidable.isFalse id
  apply le_antisymm
  · -- ∑' r, f r ≤ ∑' v, f (Ok v)
    rw [ENNReal.tsum_eq_iSup_sum]
    apply iSup_le; intro S
    -- Split sum: keep Ok terms (Err terms are 0)
    have hOkSum : ∑ r ∈ S, f r = ∑ r ∈ S.filter isOk, f r := by
      have := Finset.sum_filter_add_sum_filter_not S isOk f
      have hErrZero : ∑ r ∈ S.filter (fun r => ¬isOk r), f r = 0 :=
        Finset.sum_eq_zero fun r hr => by
          simp only [Finset.mem_filter, isOk] at hr
          rcases r with v | e
          · exact absurd trivial hr.2
          · exact hErr e
      rw [hErrZero, add_zero] at this; exact this.symm
    rw [hOkSum]
    -- Case split: empty vs nonempty filtered set
    rcases (S.filter isOk).eq_empty_or_nonempty with hemp | ⟨r₀, hr₀⟩
    · simp [hemp]
    -- Nonempty case: get a witness Ok v₀ to provide Nonempty α
    · simp only [Finset.mem_filter, isOk] at hr₀
      rcases r₀ with v₀ | e₀
      swap; · exact absurd hr₀.2 id
      haveI : Nonempty α := ⟨v₀⟩
      -- Ok-projection function
      let proj : core.result.Result α ε → α :=
        fun r => match r with
          | core.result.Result.Ok v => v
          | core.result.Result.Err _ => Classical.arbitrary α
      -- Rewrite as sum over α via bijection
      have hProj : ∑ r ∈ S.filter isOk, f r =
          ∑ v ∈ (S.filter isOk).image proj, f (core.result.Result.Ok v) := by
        have hinj : Set.InjOn proj ↑(S.filter isOk) := fun r₁ hr₁ r₂ hr₂ heq => by
          simp only [Finset.coe_filter, Set.mem_setOf_eq, isOk] at hr₁ hr₂
          rcases r₁ with v₁ | e₁
          · rcases r₂ with v₂ | e₂
            · simp [proj] at heq; exact congrArg _ heq
            · exact absurd hr₂.2 id
          · exact absurd hr₁.2 id
        rw [Finset.sum_image hinj]
        apply Finset.sum_congr rfl; intro r hr
        simp only [Finset.mem_filter, isOk] at hr
        rcases r with v | e
        · simp [proj]
        · exact absurd hr.2 id
      rw [hProj]; exact ENNReal.sum_le_tsum _
  · -- ∑' v, f (Ok v) ≤ ∑' r, f r
    apply ENNReal.tsum_comp_le_tsum_of_injective
    intro v₁ v₂ h; cases h; rfl

theorem samplerDist_pure_ok {α : Type} (v : α) :
    samplerDist (ok (core.result.Result.Ok v)) = PMF.pure v := by
  funext w
  simp only [samplerDist, samplerDistGen_pure_ok, PMF.pure_apply]
  congr 1; ext; constructor <;> intro h
  · exact core.result.Result.Ok.inj h
  · exact congrArg _ h

/-- Bind distributes over `samplerDist` for `Result (Result α Error)` programs. -/
theorem samplerDist_bind
    {α β : Type}
    (prog1 : Result (core.result.Result α error.Error))
    (prog2 : α → Result (core.result.Result β error.Error)) :
    samplerDist
      (do let r1 ← prog1
          match r1 with
          | core.result.Result.Ok v => prog2 v
          | core.result.Result.Err e => ok (core.result.Result.Err e)) =
    SLang.probBind (samplerDist prog1) (fun v => samplerDist (prog2 v)) := by
  funext w
  simp only [samplerDist, samplerDistGen_bind, SLang.probBind]
  -- Simplify the match in the inner samplerDistGen
  simp_rw [show ∀ r1 : core.result.Result α error.Error,
      samplerDistGen (match r1 with
        | core.result.Result.Ok v => prog2 v
        | core.result.Result.Err e => ok (core.result.Result.Err e)) (core.result.Result.Ok w) =
      match r1 with
        | core.result.Result.Ok v => samplerDistGen (prog2 v) (core.result.Result.Ok w)
        | core.result.Result.Err _ => 0 from by
    intro r1; rcases r1 with v | e
    · rfl
    · simp [samplerDistGen_pure_ok, PMF.pure_apply]]
  -- Restrict the sum to Ok terms using tsum_result_ok_eq
  apply tsum_result_ok_eq
  intro e
  simp

theorem samplerDist_fill_bytes_nat (buffer : Slice Std.U8) :
    let n := buffer.length
    samplerDist
      (do let pair ← samplers.fill_bytes buffer
          match pair.1 with
          | core.result.Result.Ok _ =>
              ok (core.result.Result.Ok (bytes.beBytesToNat pair.2.val))
          | core.result.Result.Err e =>
              ok (core.result.Result.Err e)) =
    bytes.uniformByteNatPMF n := by
  funext k
  exact samplerDistGen_fill_bytes_nat buffer k

/-- **Master fill_bytes→nat bridge**: any nat-indexed weight `g` summed against the
    body's first `fill_bytes` draw (Ok-branch keyed by `beBytesToNat`) collapses to the
    uniform byte distribution weighted by `g`.  Both body lemmas in `uniform/pmf.lean`
    instantiate this.

    Routes all `fill_bytes` randomness through axiom (iii) (`samplerDistGen_fill_bytes_nat`),
    the only handle on the raw pair distribution. -/
theorem fill_bytes_nat_bridge
    (buf : alloc.vec.Vec Std.U8)
    (byte_len : Usize)
    (hlen : buf.length = byte_len.val)
    (g : Nat → ENNReal) :
    ∑' pair : (core.result.Result Unit error.Error) × Slice Std.U8,
      samplerDistGen (samplers.fill_bytes (alloc.vec.Vec.deref buf)) pair *
      (match pair.1 with
       | core.result.Result.Ok _ => g (bytes.beBytesToNat pair.2.val)
       | core.result.Result.Err _ => 0) =
    ∑' k : Nat, bytes.uniformByteNatPMF byte_len.val k * g k := by
  have hlen' : (alloc.vec.Vec.deref buf).length = byte_len.val := by
    simpa [alloc.vec.Vec.deref, alloc.vec.Vec.length, Slice.length] using hlen
  -- Rewrite the uniform weight via the fill_bytes axiom and expand the wrapper bind, in place
  -- (so the inner `match` is the axiom's matcher; we only reduce it per-constructor later).
  simp_rw [← hlen', ← samplerDistGen_fill_bytes_nat (alloc.vec.Vec.deref buf),
           samplerDistGen_bind, SLang.probBind]
  -- Bring `g k` inside, then swap the order of summation to put `pair` outermost.
  simp_rw [← ENNReal.tsum_mul_right]
  rw [ENNReal.tsum_comm]
  -- Per pair, collapse the inner sum over `k`.
  apply tsum_congr; intro pair
  obtain ⟨r, s⟩ := pair
  rcases r with _ | e
  · -- Ok branch: the wrapper outputs `Ok (beBytesToNat s.val)`, point mass at that nat.
    simp only [samplerDistGen_pure_ok, PMF.pure_apply, core.result.Result.Ok.injEq]
    rw [tsum_eq_single (bytes.beBytesToNat s.val) (fun k hk => by rw [if_neg hk]; ring)]
    rw [if_pos rfl]; ring
  · -- Err branch: zero mass.
    simp only [samplerDistGen_pure_ok, PMF.pure_apply]
    simp

/-! ### Derived lemmas -/

/-- A `Result.ok v` prefix disappears under `samplerDist`. -/
theorem samplerDist_bind_ok_left
    {α β : Type}
    (v : α)
    (prog2 : α → Result (core.result.Result β error.Error)) :
    samplerDist ((ok v) >>= prog2) = samplerDist (prog2 v) := by
  simp [Aeneas.Std.bind]

/-- A `Result.ok v` prefix disappears under `samplerDistGen`. -/
theorem samplerDistGen_bind_ok_left
    {A B : Type}
    (v : A)
    (prog2 : A → Result B) :
    samplerDistGen ((ok v) >>= prog2) = samplerDistGen (prog2 v) := by
  simp [Aeneas.Std.bind]

/-! ### Generic loop helpers (reusable across all sampler loop proofs) -/

/-- A `probWhileCut` started from a state where the guard is `false`
    is a point mass at that state, regardless of depth. -/
theorem probWhileCut_guard_false {γ : Type} (cond : γ → Bool) (bd : γ → SLang γ)
    (a : γ) (hc : cond a = false) (j : Nat) :
    probWhileCut cond bd (j + 1) a = probPure a := by
  simp only [probWhileCut, probWhileFunctional, hc, Bool.false_eq_true, if_false]
  rfl

/-- `probWhileCut` from a `done` state (guard always `false` there) is a point mass. -/
theorem probWhileCut_done_pt {α β : Type} (cond : ControlFlow α β → Bool)
    (bd : ControlFlow α β → SLang (ControlFlow α β))
    (hc : ∀ w : β, cond (done w) = false) (j : Nat) (w : β) :
    probWhileCut cond bd (j + 1) (done w) = probPure (done w) :=
  probWhileCut_guard_false cond bd (done w) (hc w) j

/-- Split a `tsum` over `ControlFlow` into its `cont` and `done` fibers. -/
theorem tsum_controlFlow {α β : Type} (F : ControlFlow α β → ENNReal) :
    ∑' cf : ControlFlow α β, F cf =
    (∑' a : α, F (cont a)) + (∑' b : β, F (done b)) := by
  let e : (α ⊕ β) ≃ ControlFlow α β :=
    { toFun := Sum.elim cont done
      invFun := fun cf => match cf with | cont a => Sum.inl a | done b => Sum.inr b
      left_inv := fun s => by cases s <;> rfl
      right_inv := fun cf => by cases cf <;> rfl }
  rw [← Equiv.tsum_eq e F,
      Summable.tsum_sum (f := fun s => F (e s)) ENNReal.summable ENNReal.summable]
  rfl

/-! ### Positivity helpers for samplerDistGen -/

theorem tsum_pos_exists_of_pos' {α : Type*} {f : α → ENNReal}
    (h : 0 < ∑' i, f i) : ∃ i, 0 < f i := by
  by_contra hc
  push_neg at hc
  have hz : ∀ i, f i = 0 := fun i => le_antisymm (hc i) (zero_le _)
  simp_rw [hz, tsum_zero] at h
  exact lt_irrefl _ h

theorem pos_of_mul_pos_left' {a b : ENNReal} (h : 0 < a * b) : 0 < a := by
  by_contra h'
  push_neg at h'
  rw [le_antisymm h' (zero_le _), zero_mul] at h
  exact lt_irrefl 0 h

theorem pos_of_mul_pos_right' {a b : ENNReal} (h : 0 < a * b) : 0 < b := by
  by_contra h'
  push_neg at h'
  rw [le_antisymm h' (zero_le _), mul_zero] at h
  exact lt_irrefl 0 h

theorem samplerDistGen_ok_pos' {A : Type} (v : A) (x : A)
    (h : 0 < samplerDistGen (ok v) x) : x = v := by
  rw [samplerDistGen_pure_ok, PMF.pure_apply] at h
  split_ifs at h with hxv
  · exact hxv
  · exact absurd h (lt_irrefl 0)

theorem samplerDistGen_bind_pos_exists' {A B : Type}
    (prog : Result A) (f : A → Result B) (x : B)
    (h : 0 < samplerDistGen (prog >>= f) x) :
    ∃ a : A, 0 < samplerDistGen prog a ∧ 0 < samplerDistGen (f a) x := by
  rw [samplerDistGen_bind, SLang.probBind] at h
  obtain ⟨a, ha⟩ := tsum_pos_exists_of_pos' h
  exact ⟨a, pos_of_mul_pos_left' ha, pos_of_mul_pos_right' ha⟩

/-! ### Aeneas from_residual helper -/

/-- `from_residual (Err e) = ok (Err e)` for any result type `T`. -/
theorem from_residual_err_ok {T : Type} (e : error.Error) :
    core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual
        T (core.convert.FromSame error.Error)
        (core.result.Result.Err e) = ok (core.result.Result.Err e) := by
  simp [core.result.Result.Insts.CoreOpsTry_traitFromResidualResultInfallibleE.from_residual,
    core.convert.FromSame.from_, Aeneas.Std.bind]

/-! ### Monotone convergence: iSup/tsum interchange -/

/-- Monotone convergence: iSup and tsum commute when the summand is monotone in the index. -/
theorem tsum_iSup_commute {α : Type*} (g : α → ℕ → ENNReal) (hmono : ∀ x, Monotone (g x)) :
    (∑' x, ⨆ n, g x n) = ⨆ n, ∑' x, g x n := by
  rw [ENNReal.tsum_eq_iSup_sum]
  simp_rw [ENNReal.finsetSum_iSup_of_monotone hmono]
  rw [iSup_comm]
  simp_rw [← ENNReal.tsum_eq_iSup_sum]

end OpenDP.samplers
