import src.core.readable.notation
import src.samplers.uniform.pmf
import src.samplers.bernoulli.pmf
import src.samplers.bernoulli.exp1
import src.samplers.bernoulli.exp
import src.samplers.geometric.slow
import src.samplers.geometric.fast
import src.samplers.laplace
import src.samplers.gaussian

/-!
# Main results — the human-readable face of the verified chain

This file restates every end-to-end theorem of the ROADMAP chain in the vocabulary of the
blueprint (`blueprint/src/content.tex`), so the Lean statements and the `.tex` statements can
be read side by side. Nothing here does new probabilistic work: each `_correct` theorem is the
library theorem re-expressed through prose-facing definitions, and each `_mass` corollary
computes the pointwise mass function stated in the blueprint (`Pr[⊤] = e^{-x}`,
`P[v] = e^{-xv}(1-e^{-x})`, the discrete-Laplace mass, …).

Reading guide (notation is scoped to this file):
- `⟦ prog ⟧`   — the success law of an extracted sampler (`samplerDist`).
- `⟦ prog ⟧ₙ`  — the same law pushed to `ℕ` along `ubigToNat` (for `UBig`-valued samplers).
- `⟦ prog ⟧ℤ`  — the same law pushed to `ℤ` along `ibigToInt` (for `IBig`-valued samplers).
- `⌞ u ⌟`      — the mathematical value of an opaque dashu natural (`ubigToNat u`).
- `Bernoulli`, `BernoulliExpNeg`, `GeometricSuccesses`, `UniformBelow`, `DiscreteLaplace`,
  `DiscreteGaussian` — the reference laws under their textbook names (definitionally the
  SampCert reference samplers).
-/

open Aeneas Aeneas.Std Result
open OpenDP OpenDP.Core.Semantics OpenDP.Core.Readable
open SLang ENNReal Real

namespace OpenDP.MainResults

open OpenDP.samplers.bernoulli (RationalSetup BernoulliExpSetup bernoulliPMF
  bernoulliPMF_eq_BernoulliSamplePMF)
open OpenDP.samplers.geometric (geoTrial)

/-! ### Prose-facing notation -/

/-- `⟦ prog ⟧ₙ` — the success law of a `UBig`-valued sampler, pushed to `ℕ`. -/
scoped notation "⟦" prog "⟧ₙ" => OpenDP.samplers.uniform.samplerDist_nat prog

/-- `⟦ prog ⟧ℤ` — the success law of an `IBig`-valued sampler, pushed to `ℤ`. -/
scoped notation "⟦" prog "⟧ℤ" => OpenDP.samplers.laplace.samplerDist_int prog

/-- `⌞ u ⌟` — the mathematical value of an opaque dashu natural. -/
scoped notation "⌞" u "⌟" => dashu.ubigToNat u

/-! ### The reference laws, under their textbook names -/

/-- The uniform law on `[0, n)`. -/
noncomputable def UniformBelow (n : ℕ) (h : 0 < n) : SLang ℕ := UniformSample ⟨n, h⟩

/-- `Bernoulli(num/den)`. -/
noncomputable def Bernoulli (num den : ℕ) (hden : 0 < den) (wf : num ≤ den) : SLang Bool :=
  BernoulliSample num ⟨den, hden⟩ wf

/-- `Bernoulli(e^{-num/den})`. -/
noncomputable def BernoulliExpNeg (num den : ℕ) (hden : 0 < den) : SLang Bool :=
  BernoulliExpNegSample num ⟨den, hden⟩

/-- The number of consecutive successes of `trial` before its first failure —
the (zero-based) geometric law. -/
noncomputable def GeometricSuccesses (trial : SLang Bool) : SLang ℕ :=
  fun v => probGeometric trial (v + 1)

/-- The discrete Laplace law on `ℤ` with scale `num/den`. -/
noncomputable def DiscreteLaplace (num den : ℕ) (hn : 0 < num) (hd : 0 < den) : SLang ℤ :=
  DiscreteLaplaceSample ⟨num, hn⟩ ⟨den, hd⟩

/-- The discrete Gaussian law on `ℤ` with standard deviation `num/den`. -/
noncomputable def DiscreteGaussian (num den : ℕ) (hn : 0 < num) (hd : 0 < den) (mix : ℕ) :
    SLang ℤ :=
  DiscreteGaussianSample ⟨num, hn⟩ ⟨den, hd⟩ mix

/-! ### Stage 2 — uniform sampling below a bound

Blueprint: *"Rejection sampling to get an exact uniform on `[0, upper)`."* -/

theorem uniform_correct (upper : dashu_int.ubig.UBig) (hpos : 0 < ⌞upper⌟) :
    ⟦ samplers.uniform.sample_uniform_ubig_below upper ⟧ₙ = UniformBelow ⌞upper⌟ hpos := by
  rw [OpenDP.samplers.uniform.sample_uniform_ubig_below_pmf upper hpos]
  rfl

/-- *"`P[k] = 1/upper` on the support."* -/
theorem uniform_mass (upper : dashu_int.ubig.UBig) (hpos : 0 < ⌞upper⌟)
    (k : ℕ) (hk : k < ⌞upper⌟) :
    ⟦ samplers.uniform.sample_uniform_ubig_below upper ⟧ₙ k = 1 / ⌞upper⌟ := by
  rw [uniform_correct upper hpos]
  exact UniformSample_apply ⟨⌞upper⌟, hpos⟩ k hk

/-! ### Stage 3 — Bernoulli from a rational

Blueprint: *"sample uniformly from `[0, denom)`; return `⊤` iff below `numer`;
so `Pr[⊤] = numer/denom`."* -/

theorem bernoulli_correct (prob : dashu_ratio.rbig.RBig) (setup : RationalSetup prob)
    (hden : 0 < ⌞setup.denom⌟) (wf : ⌞setup.numer⌟ ≤ ⌞setup.denom⌟) :
    ⟦ samplers.bernoulli.sample_bernoulli_rational prob ⟧ =
      Bernoulli ⌞setup.numer⌟ ⌞setup.denom⌟ hden wf := by
  rw [OpenDP.samplers.bernoulli.sample_bernoulli_rational_pmf prob setup hden,
    bernoulliPMF_eq_BernoulliSamplePMF setup.numer setup.denom hden wf]
  funext b
  show (SLang.BernoulliSamplePMF _ _ _ : SLang Bool) b = _
  unfold SLang.BernoulliSamplePMF
  rw [PMF.ofFintype_apply]
  rfl

/-- *"`Pr[⊤] = numer/denom`."* -/
theorem bernoulli_mass (prob : dashu_ratio.rbig.RBig) (setup : RationalSetup prob)
    (hden : 0 < ⌞setup.denom⌟) (wf : ⌞setup.numer⌟ ≤ ⌞setup.denom⌟) :
    ⟦ samplers.bernoulli.sample_bernoulli_rational prob ⟧ true =
      (⌞setup.numer⌟ : ENNReal) / (⌞setup.denom⌟ : ENNReal) := by
  rw [bernoulli_correct prob setup hden wf]
  show BernoulliSample _ _ _ true = _
  rw [BernoulliSample_apply]
  simp only [if_true]
  rfl

/-! ### Stages 4–5 — Bernoulli(e^{-x})

Blueprint: *"`Pr[⊤] = e^{-x}`"* — stage 4 for `x = numer/denom ∈ [0,1]` (the CKS unit
construction), stage 5 for arbitrary `x ≥ 0`. -/

/-- `Bernoulli(e^{-num/den})` for `num/den ∈ [0,1]` (SampCert's unit sampler). -/
noncomputable def BernoulliExpNegUnit (num den : ℕ) (hden : 0 < den) (wf : num ≤ den) :
    SLang Bool :=
  BernoulliExpNegSampleUnit num ⟨den, hden⟩ wf

theorem bernoulli_exp_unit_correct (x : dashu_ratio.rbig.RBig) (setup : RationalSetup x)
    (hden : 0 < ⌞setup.denom⌟) (wf : ⌞setup.numer⌟ ≤ ⌞setup.denom⌟) :
    ⟦ samplers.bernoulli.sample_bernoulli_exp1 x ⟧ =
      BernoulliExpNegUnit ⌞setup.numer⌟ ⌞setup.denom⌟ hden wf :=
  OpenDP.samplers.bernoulli.sample_bernoulli_exp1_spec x setup hden wf

/-- *"`Pr[⊤] = e^{-x}` for `x ∈ [0,1]`"*, as a real-number formula. -/
theorem bernoulli_exp_unit_mass (x : dashu_ratio.rbig.RBig) (setup : RationalSetup x)
    (hden : 0 < ⌞setup.denom⌟) (wf : ⌞setup.numer⌟ ≤ ⌞setup.denom⌟) :
    ⟦ samplers.bernoulli.sample_bernoulli_exp1 x ⟧ true =
      ENNReal.ofReal (Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ)))) := by
  rw [bernoulli_exp_unit_correct x setup hden wf]
  show BernoulliExpNegSampleUnit _ _ _ true = _
  rw [BernoulliExpNegSampleUnit_apply_true _ _ _ _ rfl]
  congr 1
  rw [ENNReal.toReal_div]
  congr 1 <;> simp

theorem bernoulli_exp_correct (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    ⟦ samplers.bernoulli.sample_bernoulli_exp x ⟧ =
      BernoulliExpNeg ⌞setup.numer⌟ ⌞setup.denom⌟ setup.hdenom :=
  OpenDP.samplers.bernoulli.sample_bernoulli_exp_spec x setup

/-- *"`Pr[⊤] = e^{-x}`"*, as a real-number formula. -/
theorem bernoulli_exp_mass (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    ⟦ samplers.bernoulli.sample_bernoulli_exp x ⟧ true =
      ENNReal.ofReal (Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ)))) := by
  rw [bernoulli_exp_correct x setup]
  show BernoulliExpNegSample _ _ true = _
  rw [BernoulliExpNegSample_apply_true]
  all_goals congr 1 <;> (push_cast; ring)

/-! ### Stages 6–7 — the geometric samplers

Blueprint: *"count consecutive successes of `Bernoulli(e^{-x})`"*; the fast variant
*"realises the same geometric law as stage 6"*. -/

theorem geometric_slow_correct (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    ⟦ samplers.geometric.sample_geometric_exp_slow x ⟧ₙ =
      GeometricSuccesses (BernoulliExpNeg ⌞setup.numer⌟ ⌞setup.denom⌟ setup.hdenom) :=
  OpenDP.samplers.geometric.sample_geometric_exp_slow_spec x setup

theorem geometric_fast_correct (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x)
    (hpos : 0 < ⌞setup.numer⌟) :
    ⟦ samplers.geometric.sample_geometric_exp_fast x ⟧ₙ =
      GeometricSuccesses (BernoulliExpNeg ⌞setup.numer⌟ ⌞setup.denom⌟ setup.hdenom) :=
  OpenDP.samplers.geometric.sample_geometric_exp_fast_spec x setup hpos

/-- The two geometric implementations sample the *same* distribution. -/
theorem geometric_fast_eq_slow (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x)
    (hpos : 0 < ⌞setup.numer⌟) :
    ⟦ samplers.geometric.sample_geometric_exp_fast x ⟧ₙ =
      ⟦ samplers.geometric.sample_geometric_exp_slow x ⟧ₙ := by
  rw [geometric_fast_correct x setup hpos, geometric_slow_correct x setup]

/-- *"`P[v] = e^{-xv} · (1 - e^{-x})`"*, as a real-number formula. -/
theorem geometric_slow_mass (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x)
    (v : ℕ) :
    ⟦ samplers.geometric.sample_geometric_exp_slow x ⟧ₙ v =
      ENNReal.ofReal
        (Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ) * v)) *
          (1 - Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ))))) := by
  rw [geometric_slow_correct x setup]
  show probGeometric _ (v + 1) = _
  rw [probGeometric_apply, if_neg (Nat.succ_ne_zero v), Nat.add_sub_cancel]
  show BernoulliExpNegSample _ _ true ^ v * BernoulliExpNegSample _ _ false = _
  rw [BernoulliExpNegSample_apply_true, BernoulliExpNegSample_apply_false]
  rw [show Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ) * v)) =
      Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ))) ^ v from by
    rw [← Real.exp_nat_mul]; congr 1; ring]
  rw [ENNReal.ofReal_mul (by positivity), ENNReal.ofReal_pow (Real.exp_nonneg _),
    ENNReal.ofReal_sub 1 (Real.exp_nonneg _), ENNReal.ofReal_one]
  all_goals congr 2

/-- The fast sampler has the same mass function. -/
theorem geometric_fast_mass (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x)
    (hpos : 0 < ⌞setup.numer⌟) (v : ℕ) :
    ⟦ samplers.geometric.sample_geometric_exp_fast x ⟧ₙ v =
      ENNReal.ofReal
        (Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ) * v)) *
          (1 - Real.exp (-((⌞setup.numer⌟ : ℝ) / (⌞setup.denom⌟ : ℝ))))) := by
  rw [geometric_fast_eq_slow x setup hpos]
  exact geometric_slow_mass x setup v

/-! ### Stage 8 — discrete Laplace (the pure-DP noise mechanism)

Blueprint: *"sign × geometric magnitude with the `(negative, 0)` outcome rejected"*, with mass
`P[z] = (e^{d/n} - 1)/(e^{d/n} + 1) · e^{-|z|·d/n}` for scale `n/d`. -/

theorem discrete_laplace_correct (numer denom : dashu_int.ubig.UBig)
    (hn : 0 < ⌞numer⌟) (hd : 0 < ⌞denom⌟) :
    ⟦ samplers.laplace.sample_discrete_laplace numer denom ⟧ℤ =
      DiscreteLaplace ⌞numer⌟ ⌞denom⌟ hn hd :=
  OpenDP.samplers.laplace.sample_discrete_laplace_spec numer denom hn hd

/-- *"`P[out = z] = (e^{denom/numer} − 1)/(e^{denom/numer} + 1) · e^{−|z|·denom/numer}`"*. -/
theorem discrete_laplace_mass (numer denom : dashu_int.ubig.UBig)
    (hn : 0 < ⌞numer⌟) (hd : 0 < ⌞denom⌟) (z : ℤ) :
    ⟦ samplers.laplace.sample_discrete_laplace numer denom ⟧ℤ z =
      ENNReal.ofReal
        ((Real.exp ((⌞denom⌟ : ℝ) / (⌞numer⌟ : ℝ)) - 1) /
            (Real.exp ((⌞denom⌟ : ℝ) / (⌞numer⌟ : ℝ)) + 1) *
          Real.exp (-(|(z : ℝ)| * (⌞denom⌟ : ℝ) / (⌞numer⌟ : ℝ)))) := by
  rw [discrete_laplace_correct numer denom hn hd]
  show DiscreteLaplaceSample _ _ z = _
  rw [DiscreteLaplaceSample_apply]
  congr 2
  · congr 2 <;>
    · congr 1
      rw [one_div_div]
      push_cast
      rfl
  · rw [div_div_eq_mul_div]
    push_cast
    rfl

/-! ### Stage 9 — discrete Gaussian (the zCDP noise mechanism, the final target)

Blueprint: *"rejection sampling from the verified discrete-Laplace proposal with the CKS
acceptance test"*. The result holds for every value of SampCert's `mix` parameter (the
implementation-selection knob), as it must. The pointwise mass function
`P[z] ∝ e^{-z²·denom²/(2·numer²)}` lives in SampCert's analytic layer, which does not build
on the pinned toolchain — the distribution-level identification below is the full statement
of correctness against the reference sampler. -/

theorem discrete_gaussian_correct (numer denom : dashu_int.ubig.UBig)
    (hn : 0 < ⌞numer⌟) (hd : 0 < ⌞denom⌟) (mix : ℕ) :
    ⟦ samplers.gaussian.sample_discrete_gaussian numer denom ⟧ℤ =
      DiscreteGaussian ⌞numer⌟ ⌞denom⌟ hn hd mix :=
  OpenDP.samplers.gaussian.sample_discrete_gaussian_spec numer denom hn hd mix

end OpenDP.MainResults
