# Verified Boolean randomized response

The production `make_randomized_response_bool` constructor first invokes the
**same executable `constructor::build_bool_rr` verified in the Verus harness**, then
calls `Measurement::new_certified` with the private `CertifiedBoolRR` adapter.
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
- **`constructor::verify_built_bool_rr<E>` now executes `build_bool_rr`** and
  proves its successful output satisfies the measurement validity predicate
  for the operator's corresponding callbacks, assuming the uniform entropy
  contract and the single audited `adapter_realizes` contract. Its proof
  covers both finite cost and the `p = 1` infinite-cost branch.
- The same builder also verifies rejection of invalid probability encodings
  and propagation of numerical errors without claiming success on `Err`.

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

## Verified boundary and trusted computing base

The constructor-level theorem is conditional on three narrow boundaries:

1. The actual `sample_uniform_u64` / `fill_bytes` implementation satisfies
   `uniform_word_expectation_rule`, including input-independent failures.
2. The native numerical primitives implement their small trusted contracts:
   exact casts through `2^53`, upward-rounded successful `InfDiv<f64>` and
   `InfLn<f64>`, positive IEEE infinity, and standard `exp`/`ln` real-analysis
   properties. Dashu and panic handling are deliberately not verified.
3. `CertifiedBoolRR::into_parts` and OpenDP's `Arc<dyn Fn>` erasure satisfy
   `adapter_realizes` for the actual operator produced by the builder.

The **public Rust `Measurement` struct and `new_certified` implementation
are not parsed by Verus**. Consequently there is still no literal verifier
theorem over the returned concrete Rust `Measurement` value. Instead,
`verify_built_bool_rr` checks the production operator constructor and proves
validity of the erased callbacks **relative to** the audited adapter contract.
This is an end-to-end conditional guarantee under the chosen trust model,
not a proof of Rust's dynamic callback dispatch or of the entropy source.

No unconditional certification is inferred merely from implementing the
`CertifiedMeasurementParts` trait.
