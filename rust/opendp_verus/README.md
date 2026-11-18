# Verified Boolean randomized response

The production `make_randomized_response_bool` constructor now uses
`Measurement::new_certified` with a private `CertifiedBoolRR` adapter.
This follows the pattern established by
`Transformation::new_certified` on `verified-stable-lazyframe`.

## What Verus proves

- The production binary64 bit decoder validates the probability interval and
  derives the exact dyadic Bernoulli numerator.
- The production integer sampler and `BoolRR::release` implement the expected
  Boolean response for each entropy word, preserving every error payload.
- Under the uniform-word expectation contract and an exact odds bound,
  `lemma_bool_rr_dp` proves pure-DP expectation domination for every bounded
  payoff on `Result<bool, E>`.
- `lemma_rr_operator_valid_measurement` lifts the operator theorem to the
  measurement-level relational expectation specification.
- `core/verus/measurement.rs::lemma_new_certified` transfers that guarantee
  from the concrete operator to erased callbacks, **provided** the callback
  expectation and privacy map realize the verified operator's semantics.
- `lemma_bool_rr_new_certified` specializes this erasure theorem to Boolean
  randomized response.
- The production `upper_log_odds_u53` helper uses the same `f64::inf_div`
  and `f64::inf_ln` operations as before. Verus checks their composition
  and proves that a successful returned epsilon satisfies
  `exp(epsilon) >= numerator / (2^53 - numerator)` from small trusted
  numerical primitive contracts.
- `lemma_certified_rr_with_computed_epsilon` connects that exact
  likelihood-ratio bound to the certified measurement theorem.

The proof does **not** model a second implementation of the sampler or require
the Alerus library. The existing `Arc<dyn Fn>` representation is unchanged.

## Native certification boundary

`CertifiedMeasurementParts` is a private-to-the-crate adapter interface.
The only Boolean RR implementation is for the private `CertifiedBoolRR`
wrapper, which is created only after the public constructor has checked its
input bits and computed its privacy cost. It installs the same `BoolRR`
state in both callbacks via `Function::new_fallible` and `PrivacyMap::new`.

Verus currently cannot verify OpenDP's `Arc<dyn Fn>` dispatch itself.
The adapter's equality-of-expectations and privacy-map correspondence are
therefore **trusted obligations**, just as in
`Transformation::new_certified`. A trait implementation alone does not
supply a proof of validity for arbitrary callbacks.

## Remaining work for fully checked constructor validity

1. Connect the uniform-word expectation contract to the actual fallible
   `sample_uniform_u64` and `fill_bytes` entropy source.
2. Prove the underlying numerical primitive contracts, if a smaller
   trusted computing base is ever required. Currently we **trust only**
   successful-result upward-rounding contracts for `InfDiv<f64>` and
   `InfLn<f64>`, the exact cast for integers up to `2^53`, and
   the real-analysis facts `exp(ln(x)) = x` for positive `x`
   and monotonicity of `exp`. The numeric composition theorem itself
   is verified; no Dashu implementation or panic machinery is verified.
3. Either mechanically verify the standard-library callback-erasure
   correspondence or retain this small, explicitly trusted native adapter.

Thus we have a checked **conditional** privacy proof and a certified
constructor path. The floating-point composition is verified under the
explicit trusted arithmetic contracts; the actual implementation of those
contracts and the probabilistic entropy source remain outside Verus. No `new_certified` method treats a
certificate marker as proof without the corresponding model assumptions.
