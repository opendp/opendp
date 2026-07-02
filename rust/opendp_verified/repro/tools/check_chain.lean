import OpenDPVerified

/-!
# Verified-chain completion guard

Machine-checks that the sampler pipeline is COMPLETE and rests only on the sanctioned trust
surface. Two properties, both of which `lake build` alone does NOT enforce:

1. **Existence**: every end-to-end theorem of the chain resolves by name (a rename/delete of
   a spec would otherwise leave the build green).
2. **Axiom footprint**: each of those theorems depends only on the sanctioned axiom set — the
   single randomness axiom, the deterministic external specs (`OpenDP.dashu.*` etc.), the
   opaque extraction constants, and the vendored Aeneas stdlib's `sorryAx`. Any new `axiom`
   (or a `sorry` smuggled in via a *new* dependency path) fails this file's elaboration.

Run via `tools/check_verified_chain.sh` (which also greps the handwritten sources for `sorry`
tokens — `sorryAx` alone cannot distinguish our sorries from the vendored stdlib's).
-/

open Lean
open Aeneas Aeneas.Std
open OpenDP OpenDP.Core.Semantics
open SLang

/-! ## Statement pinning

Each end-to-end theorem is restated verbatim and proved *by* the library theorem. If anyone
weakens, renames, or otherwise changes what a spec claims, elaboration of this file fails —
the theorem *name* surviving is not enough. These `example`s are the canonical definition of
"the chain is done". -/

section StatementPins
open OpenDP.samplers.uniform (samplerDist_nat uniformNatBelowPMF)
open OpenDP.samplers.bernoulli (RationalSetup BernoulliExpSetup bernoulliPMF)
open OpenDP.samplers.geometric (geoTrial)
open OpenDP.samplers.laplace (samplerDist_int)

/-- Stage 2: uniform on `[0, upper)`. -/
example (upper : dashu_int.ubig.UBig) (hupper : 0 < dashu.ubigToNat upper) :
    samplerDist_nat (samplers.uniform.sample_uniform_ubig_below upper) =
      ↑(uniformNatBelowPMF upper hupper) :=
  OpenDP.samplers.uniform.sample_uniform_ubig_below_pmf upper hupper

/-- Stage 3: `Bernoulli(numer/denom)`. -/
example (prob : dashu_ratio.rbig.RBig) (setup : RationalSetup prob)
    (hdenom : 0 < dashu.ubigToNat setup.denom) :
    samplerDist (samplers.bernoulli.sample_bernoulli_rational prob) =
      (bernoulliPMF setup.numer setup.denom hdenom : SLang Bool) :=
  OpenDP.samplers.bernoulli.sample_bernoulli_rational_pmf prob setup hdenom

/-- Stage 4: `Bernoulli(e^{-x})`, `x ∈ [0,1]`. -/
example (x : dashu_ratio.rbig.RBig) (setup : RationalSetup x)
    (hdenom : 0 < dashu.ubigToNat setup.denom)
    (hfrac : dashu.ubigToNat setup.numer ≤ dashu.ubigToNat setup.denom) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp1 x) =
      BernoulliExpNegSampleUnit (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, hdenom⟩ hfrac :=
  OpenDP.samplers.bernoulli.sample_bernoulli_exp1_spec x setup hdenom hfrac

/-- Stage 5: `Bernoulli(e^{-x})`, `x ≥ 0`. -/
example (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    samplerDist (samplers.bernoulli.sample_bernoulli_exp x) =
      BernoulliExpNegSample (dashu.ubigToNat setup.numer)
        ⟨dashu.ubigToNat setup.denom, setup.hdenom⟩ :=
  OpenDP.samplers.bernoulli.sample_bernoulli_exp_spec x setup

/-- Stage 6: geometric via `Bernoulli(e^{-x})` (slow). -/
example (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_slow x) =
      fun v => probGeometric (geoTrial setup.numer setup.denom setup.hdenom) (v + 1) :=
  OpenDP.samplers.geometric.sample_geometric_exp_slow_spec x setup

/-- Stage 7: geometric (fast) — the same law as stage 6. -/
example (x : dashu_ratio.rbig.RBig) (setup : BernoulliExpSetup x)
    (hpos : 0 < dashu.ubigToNat setup.numer) :
    samplerDist_nat (samplers.geometric.sample_geometric_exp_fast x) =
      fun v => probGeometric (geoTrial setup.numer setup.denom setup.hdenom) (v + 1) :=
  OpenDP.samplers.geometric.sample_geometric_exp_fast_spec x setup hpos

/-- Stage 8: discrete Laplace — the pure-DP noise mechanism. -/
example (numer denom : dashu_int.ubig.UBig)
    (hnum : 0 < dashu.ubigToNat numer) (hdenom : 0 < dashu.ubigToNat denom) :
    samplerDist_int (samplers.laplace.sample_discrete_laplace numer denom) =
      DiscreteLaplaceSample ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩ :=
  OpenDP.samplers.laplace.sample_discrete_laplace_spec numer denom hnum hdenom

/-- Stage 9: discrete Gaussian — the zCDP noise mechanism (any `mix`). -/
example (numer denom : dashu_int.ubig.UBig)
    (hnum : 0 < dashu.ubigToNat numer) (hdenom : 0 < dashu.ubigToNat denom) (mix : ℕ) :
    samplerDist_int (samplers.gaussian.sample_discrete_gaussian numer denom) =
      DiscreteGaussianSample ⟨dashu.ubigToNat numer, hnum⟩ ⟨dashu.ubigToNat denom, hdenom⟩
        mix :=
  OpenDP.samplers.gaussian.sample_discrete_gaussian_spec numer denom hnum hdenom mix

end StatementPins

/-! ## Axiom-footprint check -/

/-- The end-to-end distributional theorems of the ROADMAP chain, stages 2–9.
Double-backtick literals: elaboration FAILS here if any theorem disappears. -/
def chainTheorems : Array Name := #[
  ``OpenDP.samplers.uniform.sample_uniform_ubig_below_pmf,
  ``OpenDP.samplers.bernoulli.sample_bernoulli_rational_pmf,
  ``OpenDP.samplers.bernoulli.sample_bernoulli_exp1_spec,
  ``OpenDP.samplers.bernoulli.sample_bernoulli_exp_spec,
  ``OpenDP.samplers.geometric.sample_geometric_exp_slow_spec,
  ``OpenDP.samplers.geometric.sample_geometric_exp_fast_spec,
  ``OpenDP.samplers.laplace.sample_discrete_laplace_spec,
  ``OpenDP.samplers.gaussian.sample_discrete_gaussian_spec,
  -- The human-readable face (`src/main_results.lean`): blueprint-aligned restatements
  -- and pointwise mass functions. Existence + axiom footprint checked the same way.
  ``OpenDP.MainResults.uniform_correct, ``OpenDP.MainResults.uniform_mass,
  ``OpenDP.MainResults.bernoulli_correct, ``OpenDP.MainResults.bernoulli_mass,
  ``OpenDP.MainResults.bernoulli_exp_unit_correct, ``OpenDP.MainResults.bernoulli_exp_unit_mass,
  ``OpenDP.MainResults.bernoulli_exp_correct, ``OpenDP.MainResults.bernoulli_exp_mass,
  ``OpenDP.MainResults.geometric_slow_correct, ``OpenDP.MainResults.geometric_slow_mass,
  ``OpenDP.MainResults.geometric_fast_correct, ``OpenDP.MainResults.geometric_fast_mass,
  ``OpenDP.MainResults.geometric_fast_eq_slow,
  ``OpenDP.MainResults.discrete_laplace_correct, ``OpenDP.MainResults.discrete_laplace_mass,
  ``OpenDP.MainResults.discrete_gaussian_correct]

/-- Named axioms of the sanctioned trust surface. -/
def allowedExact : Array Name := #[
  ``propext, ``Classical.choice, ``Quot.sound, ``sorryAx,
  ``OpenDP.Core.Semantics.samplerDistGen_exists,
  ``OpenDP.core_num_usize.div_ceil_spec,
  ``OpenDP.samplers.bernoulli.div_rbig_by_ubig_exact_bernoulli_setup,
  ``OpenDP.samplers.uniform.sample_uniform_ubig_below_setup_exists]

/-- Prefix families of the sanctioned trust surface: the deterministic dashu specs and the
opaque extraction constants (types and external functions the Aeneas translation leaves
abstract). -/
def allowedPrefixes : Array Name := #[
  `OpenDP.dashu,
  `dashu_int, `dashu_ratio, `dashu_base,
  `openssl, `std.backtrace, `alloc.fmt, `core.hint, `core.num,
  `SharedLUBig, `Aeneas.Std.core]

def isAllowed (n : Name) : Bool :=
  allowedExact.contains n
  || allowedPrefixes.any (·.isPrefixOf n)
  || n.components.contains `_native  -- `native_decide` auxiliary axioms

open Elab Command in
#eval show CommandElabM Unit from do
  let mut checked := 0
  for thm in chainTheorems do
    let axs ← collectAxioms thm
    for ax in axs do
      unless isAllowed ax do
        throwError "verified-chain guard FAILED: {thm} depends on unsanctioned axiom {ax}"
    checked := checked + 1
  logInfo s!"verified-chain guard: {checked}/{chainTheorems.size} end-to-end theorems present; axiom footprints within the sanctioned trust surface."
