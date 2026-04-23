
import Hax
import opendp
import SampCert.DifferentialPrivacy.PermuteAndFlip.Mechanism.Selector
import SampCert.DifferentialPrivacy.PermuteAndFlip.Privacy

noncomputable section
open scoped Classical

/-!
# Permute-and-Flip: semantic bridge to SampCert

This file packages the remaining semantic obligations needed to connect an
OpenDP-facing semantic law for permute-and-flip to SampCert's executable
`permuteAndFlipSLang` theorem.

## Why this is not temporary

This file captures the *mathematical bridge* from an OpenDP-facing semantic law
and normalization to SampCert's executable `permuteAndFlipSLang` theorem. Even
after hax is fixed, some version of this bridge should remain: it is the actual
connection between OpenDP's rational-score formulation and SampCert's discrete
executable mechanism.
-/

namespace opendp.measurements.noisy_top_k
open SLang
open SLang.PermuteAndFlip

open SLang
open SLang.PermuteAndFlip

/-- Rational-score view used for the final semantic bridge. -/
abbrev QScores (n : CandidateCount) := Fin n.succ → ℚ

/-- Probability-law view of the semantic no-replacement mechanism. -/
opaque pnfwr_semantic_law
    {n : CandidateCount} :
    QScores n -> QScores n -> Fin n.succ -> ENNReal

/-- OpenDP-side rational range metric. -/
opaque qRangeDistance
    {n : CandidateCount} :
    QScores n -> QScores n -> ℕ

/-- Rational-to-SampCert normalization package. -/
structure PNFWRNormalizedInstance (n : CandidateCount) where
  q : Scores n
  q' : Scores n
  ε₁ : ℕ
  ε₂ : ℕ+

/--
Exact remaining bridge obligations after the control-flow/hax-loop proof:
- sampler semantics line up with SampCert's executable mechanism,
- the rational score normalization preserves the range metric,
- and the OpenDP semantic law agrees with the SampCert `SLang` law.
-/
structure PNFWRSampCertBridgeAssumptions
    {n : CandidateCount}
    (x x' : QScores n)
    (r : Fin n.succ) where
  norm : PNFWRNormalizedInstance n
  range_eq : qRangeDistance x x' = rangeDistance norm.q norm.q'
  law_x : pnfwr_semantic_law x x' r = permuteAndFlipSLang n norm.q norm.ε₁ norm.ε₂ r
  law_x' : pnfwr_semantic_law x' x r = permuteAndFlipSLang n norm.q' norm.ε₁ norm.ε₂ r

/--
Final privacy theorem once the remaining semantic-sampler bridge obligations are
instantiated.
-/
theorem pnfwr_semantic_range_privacy_of_bridge
    {n : CandidateCount}
    (x x' : QScores n)
    (r : Fin n.succ)
    (H : PNFWRSampCertBridgeAssumptions x x' r) :
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ qRangeDistance x x'
      * pnfwr_semantic_law x x' r
      ≤
      pnfwr_semantic_law x' x r := by
  calc
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ qRangeDistance x x'
        * pnfwr_semantic_law x x' r
      =
    (privacyBase H.norm.ε₁ H.norm.ε₂) ^ rangeDistance H.norm.q H.norm.q'
        * permuteAndFlipSLang n H.norm.q H.norm.ε₁ H.norm.ε₂ r := by
          rw [H.range_eq, H.law_x]
    _ ≤ permuteAndFlipSLang n H.norm.q' H.norm.ε₁ H.norm.ε₂ r := by
          exact permuteAndFlipSLang_range_privacy H.norm.q H.norm.q' r H.norm.ε₁ H.norm.ε₂
    _ = pnfwr_semantic_law x' x r := by
          rw [H.law_x']

end opendp.measurements.noisy_top_k
