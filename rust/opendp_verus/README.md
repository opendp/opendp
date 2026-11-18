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
  **same** `BoolRR` operator it passes into `new_certified`, with the
  production function and map method items. The probability argument covers
  bounded payoffs on `Result<bool, E>`, including observable failures.
- `new_certified` erases methods of one concrete operator into the ordinary
  `Function` and `PrivacyMap` fields. Verus checks field assembly and
  correspondence of method `call_requires` / `call_ensures` with both
  *returned* erased callbacks.
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

1. Define a single `valid_measurement` predicate over the *actual*
   domain/metric/measure, probabilistic function behavior and reported
   privacy map; require it in the real `new_certified` contract, and
   preserve it for the returned Measurement.
2. Bridge the executable `BoolRR::invoke` (one `sample_uniform_u64`
   call followed by verified `BoolRR::release`) to the expectation
   functional used by the operator's DP proof. A pointwise
   `call_ensures` is insufficient to establish a **distribution**.
   That bridge must arise from a generic, narrowly trusted entropy rule
   and verified postprocessing, never an assumed mechanism privacy theorem.
3. Verify the production `AtomDomain<bool>`, `DiscreteDistance` and
   `MaxDivergence` descriptors and their actual semantics in the
   `make_randomized_response_bool` call graph.
4. Check that replacing the **function** callback with an always-truthful
   one while keeping a finite map fails at the certified constructor.

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
- `Function::new_from_operator` and `PrivacyMap::new_from_operator`
  are `external_body` **only for generic `Arc<dyn Fn>` erasure and
  dispatch**. Their contracts preserve the exact methods' possible
  results and callability. They do **not** assume privacy or a probability
  law. The operational probabilistic interpretation of the erased
  `Function` is the remaining essential missing bridge.

Do not mark `new_certified` as a privacy certification boundary until
its **actual precondition** is discharged at the production constructor,
and negative verification rejects a bad function as well as a bad map.
