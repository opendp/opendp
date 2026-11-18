# Verus proof of Boolean randomized response

`make_randomized_response_bool` calls `constructor::build_bool_rr`, which is
verified from the **same executable Rust source**, and packages its result
with `Measurement::new_certified(BoolRRParts(operator))`.

## Checked by Verus

- Accepted IEEE-754 bit patterns decode to the exact Bernoulli probability
  `n / 2^53`; the uniform-word comparison implements that Bernoulli event.
- If the numerical computation returns `Ok(epsilon)`, then
  `exp(epsilon) >= n / (2^53 - n)` for `n < 2^53`.
- Under the entropy contract, the concrete `BoolRR::release` obeys pure DP
  for **all bounded payoffs** on `Result<bool, E>`, including observable errors.
- `verify_built_bool_rr` executes the production builder and composes
  its numerical and privacy lemmas with the generic
  `core/verus/measurement.rs::lemma_new_certified`. It handles
  both finite epsilon and `n = 2^53` (infinite epsilon).
- A successful callback-erasure step preserves privacy **if** the callbacks
  have the same expectations and privacy map as the verified operator.

## Trusted boundary

1. `sample_uniform_u64` / `fill_bytes` has the stated uniform-word law,
   including input-independent failures.
2. The numeric primitives implement their specified exact integer cast,
   upward-rounded `inf_div` and `inf_ln`, and positive infinity. The
   usual mathematical `exp`/`ln` facts are trusted. Dashu and panic
   handling are not verified.
3. `BoolRRParts::into_parts` faithfully forwards through
   `Function::new_fallible`, `PrivacyMap::new`, and their
   `Arc<dyn Fn>` storage.

Verus does **not** verify the actual `Measurement` object's trait-object
dispatch. The final theorem is about the executable builder's result and
its callbacks **under the adapter-correspondence premise**, not an
unconditional Rust `Measurement` validity theorem. The adapter's
small implementation is audited rather than verified.
