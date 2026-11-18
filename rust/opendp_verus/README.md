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
2. Verify directed rounding for the production `inf_div` and `inf_ln`
   operations and connect the returned binary64 privacy cost to the
   real likelihood-ratio factor in the theorem.
3. Either mechanically verify the standard-library callback-erasure
   correspondence or retain this small, explicitly trusted native adapter.

Thus we have a checked **conditional** privacy proof and a certified
constructor path, not a soundness theorem for arbitrary real-world entropy
or floating-point implementations. No `new_certified` method treats a
certificate marker as proof without the corresponding model assumptions.
