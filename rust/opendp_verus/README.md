# Verus proof of Boolean randomized response

**Status: not yet an end-to-end certified Measurement.** Verification reaches
the *actual production* `Measurement::new_certified` call and proves the
Boolean RR operator's mathematical privacy bound. The constructor still
lacks the full measurement-validity precondition. A passing proof therefore
does **not** establish privacy of an arbitrary callback pair.

## Checked against production Rust

- The same `dyadic53` decoder, `BoolRR::release`, `BoolRR::invoke`,
  `BoolRR::map`, numerical log-odds builder and measurement constructor
  are compiled by native OpenDP and the Verus harness.
- `build_bool_rr_measurement` proves the MaxDivergence bound for the
  **same** `BoolRR` operator it passes into `new_certified`, with its
  verified `release_callback` and `map_callback`. The probability argument
  covers bounded payoffs on `Result<bool, E>`, including observable failures.
- The verified generic `sample_word_then` calls production entropy once,
  then runs the proved deterministic postprocessor. The narrowly trusted
  `Function::new_from_one_word_operator` contract connects the actual
  stored `Function`'s expectation to this composition, conditioned on
  `exact_word_postprocessing`, which the RR builder discharges.
- `build_bool_rr_measurement` now **ensures**
  `valid_bool_rr_measurement(&measurement)` on success: the *returned*
  `Function`'s distribution and the *returned* fallible `PrivacyMap`
  satisfy the concrete Boolean/discrete/MaxDivergence inequality, including
  zero distance and infinite privacy loss.
- `new_certified` receives the operator's deterministic postprocessor and
  its privacy-map method. Verus checks field assembly, verified
  postprocessor-to-model correspondence, and returned map
  `call_requires` / `call_ensures` preservation.
- The actual returned `PrivacyMap` is proved callable for every input
  distance and to return exactly `Ok(BoolRR::map(distance, 0.0))`.
  The returned `Function` is proved callable for every Boolean input;
  when the RR probability is one, any successful result is the true bit.

## Falsification check

A temporary commit `6424e0f1` replaced the installed privacy-map method
with a method that returned zero cost. **Verus rejected the returned
Measurement's map-correspondence assertion**, even though the incorrect
method had a truthful executable postcondition. The correct method was
restored in `02a9913`. Both native and Verus CI pass on that restoration.
This test establishes the map-wiring boundary, *not* the full DP boundary.

## Work required for full certification

1. Extend the concrete `valid_bool_rr_measurement` predicate to the
   canonical generic `valid_measurement` contract over actual
   domain/metric/measure semantics. Require it at **the real
   `Measurement::new_certified` entry**, not only as a postcondition of
   the Boolean RR builder. No arbitrary caller-selected ghost semantics.
2. Verify the production `AtomDomain<bool>`, `DiscreteDistance` and
   `MaxDivergence` descriptors and their actual semantics in the
   `make_randomized_response_bool` call graph.
3. Check that replacing the **function** postprocessor with an
   always-truthful method while keeping a finite map fails when the
   certified constructor requires the full measurement contract.

## Explicit trust boundaries

- The `sample_uniform_u64` uniform-word law and input-independent errors
  are trusted, not derived from the entropy implementation. The current
  Verus harness abstracts the native `Error` as `()`; the generic RR
  payoff theorem is quantified over error values, but the native error
  representation is not verified.
- The numeric primitives implement conservatively rounded division and
  logarithm, exact integer casts and positive infinity; mathematical
  `exp` / `ln` facts are trusted. Dashu and panic handling are outside
  the current verification.
- `Function::new_from_one_word_operator` remains a generic
  `external_body` trust boundary for preserving the sampled-word
  distribution through `Arc<dyn Fn>` erasure. Its expectation contract
  uses **only the trusted uniform-word law and the verified deterministic
  postprocessor**. It does not assume any mechanism is private.
  `PrivacyMap::new_from_operator` likewise trusts only the exact
  forwarding of the executable map callback.

Do not mark `new_certified` as a privacy certification boundary until
its **actual precondition** is discharged at the production constructor,
and negative verification rejects a bad function as well as a bad map.
