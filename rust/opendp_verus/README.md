# Verus proof of Boolean randomized response

**Status: incomplete measurement certification.** This proof checks the
production Boolean RR operator and its privacy bound, and now verifies the
production call through `Measurement::new_certified`. It **does not yet**
enforce the full measurement-validity precondition at that call. A successful
Verus run here must not be described as a certified OpenDP Measurement.

## Verified executable path

- `dyadic53.rs` validates IEEE-754 probability bits and implements an exact
  Bernoulli event on a uniform 64-bit word.
- `BoolRR::release` and `BoolRR::invoke` are compiled from the same source
  by OpenDP and Verus. The latter calls the production `sample_uniform_u64`.
- `upper_log_odds_u53` and `build_bool_rr` construct a conservative privacy
  cost using the explicitly trusted numerical primitives.
- `build_bool_rr_measurement` constructs the **actual** operator and calls
  `Measurement::new_certified`. Its proof invokes
  `lemma_built_bool_rr_valid` on that exact operator before erasure, not on a
  separate rebuilt copy. The lemma proves MaxDivergence for bounded payoffs
  on `Result<bool, E>`, including observable failures.
- `core/measurement.rs` and `core/measurement_parts.rs` are shared
  production source. Verus verifies the ordinary constructor's common
  `construct` body and the `new_certified` assembly. The latter accepts
  fallible function and privacy-map closures and erases them internally.

## What is not yet proved

1. The **full validity predicate** must be attached as a required
   precondition to the real `Measurement::new_certified`. It must refer to the
   actual input domain, metric, output measure, function distribution, and
   returned privacy map, including errors and infinite privacy loss.
2. `Function::new_fallible` and `PrivacyMap::new_fallible` must have narrow,
   generic, auditable contracts preserving the **actual** callbacks through
   `Arc<dyn Fn>` erasure. Currently their allocation is external to Verus
   and has no proved behavioral preservation postcondition.
3. The proof of Boolean RR's operator distribution and numeric cost must
   discharge the constructor's full predicate **for the installed callbacks**
   rather than for separately supplied ghost functions. No
   `adapter_realizes` assumption or invented proof marker is acceptable.
4. The outer `make_randomized_response_bool` must be included in verification
   to check `AtomDomain<bool>`, `DiscreteDistance`, and `MaxDivergence`
   descriptors and validation against their production semantics.
5. Negative verification tests must reject an always-truthful callback with
   finite privacy cost and a map that understates privacy loss.

## Explicit trust boundary

- The law of `sample_uniform_u64` (uniform word on successful draws,
  input-independent errors) is trusted. `uniform_word.rs` defines a
  canonical expectation functional and an external proof of its law. Verus
  does **not** itself prove that this functional is the operational
  probabilistic semantics of the native RNG call; that is part of the
  trusted entropy primitive.
- Native numerical primitives must satisfy their specified exact cast,
  conservative division/logarithm, and infinity contracts. Mathematical
  exponential/logarithm facts are trusted. Dashu and panic handling remain
  outside verification.
- `Arc<dyn Fn>` allocation and dynamic dispatch remain external, and
  preserving the behavioral semantics through them is **not yet established**.

Neither the trusted primitives nor a correct field-assembly proof substitute
for verification of a complete OpenDP Measurement.
