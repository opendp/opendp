# Verus proof of Boolean randomized response

> **Work in progress on `verified-rr-measurement-contract`: not an
> end-to-end certified Measurement constructor.** The current production
> `make_randomized_response_bool` constructs the actual `Function` and
> `PrivacyMap` and passes both directly to `Measurement::new_certified`.
> However, `core/mod.rs` is not yet imported into the Verus harness,
> `new_certified` has no mechanically enforced validity precondition, and
> the real production constructor's call is not verified. The existing
> `verify_built_bool_rr` theorem remains conditional on an assumed
> adapter correspondence. **Do not describe the resulting Measurement as
> machine-certified until that assumption is discharged at the actual call
> site.**

## Required acceptance criteria

1. Specify `valid_measurement` in terms of the actual domain, metric,
   measure, function and privacy-map fields. The specification must cover
   observable errors and conservative finite/infinite privacy accounting.
2. Verify the production `Measurement::new_certified` body with that
   validity predicate as a precondition and preserved on successful return.
   Merely verifying an identically named transfer lemma is insufficient.
3. Verify the actual `make_randomized_response_bool` call to that
   constructor. This must discharge the precondition for the real
   `Function::new_fallible` and `PrivacyMap::new` callbacks, not free
   ghost callbacks constrained by `adapter_realizes`.
4. If `Arc<dyn Fn>` erasure and dispatch remain trusted, specify their
   preservation of actual callback semantics *once*, without trusting the
   privacy guarantee or mapping itself.
5. Demonstrate failing verification when the callback always returns the
   truthful bit while its privacy map remains finite, or when a privacy
   map understates its actual cost. Include those checks in the CI review
   process. Ordinary Rust compilation cannot enforce Verus preconditions.

## Current partial proof

`make_randomized_response_bool` calls `constructor::build_bool_rr`, which is
verified from the **same executable Rust source**, then creates actual
callback objects and passes them to `Measurement::new_certified`.
The latter step is not yet verified.

## Checked by Verus

- Accepted IEEE-754 bit patterns decode to the exact Bernoulli probability
  `n / 2^53`; the uniform-word comparison implements that Bernoulli event.
- If the numerical computation returns `Ok(epsilon)`, then
  `exp(epsilon) >= n / (2^53 - n)` for `n < 2^53`.
- Under the entropy contract, the concrete `BoolRR::release` obeys pure DP
  for **all bounded payoffs** on `Result<bool, E>`, including observable errors.
- The two-outcome domination inequality is proved once, separately
  from RR's entropy and adjacency specifications.
- `verify_built_bool_rr` executes the production builder and composes
  its numerical and privacy lemmas with the generic
  `core/verus/measurement.rs::lemma_new_certified`. It handles both finite
  epsilon and `n = 2^53` (infinite epsilon). Its callback conclusion is
  conditional on `adapter_realizes` for **that returned operator only**;
  no universal adapter assumption is required.
- A successful callback-erasure step preserves privacy **if** the callbacks
  have the same expectations and privacy map as the verified operator.

## Trusted boundary

1. `sample_uniform_u64` / `fill_bytes` has the stated uniform-word law,
   including input-independent failures.
2. The numeric primitives implement their specified exact integer cast,
   upward-rounded `inf_div` and `inf_ln`, and positive infinity. The
   mathematical `exp`/`ln` facts are trusted. The exact binary64 real
   interpretation remains defined but opaque to ordinary proof callers;
   a proved lemma supplies bitwise congruence. Dashu and panic handling
   are not verified.
3. `BoolRRParts::into_parts` faithfully forwards through
   `Function::new_fallible` and `PrivacyMap::new` and their
   `Arc<dyn Fn>` storage. The map contract now trusts **bit-for-bit
   forwarding of `BoolRR::map`'s f64 output**, rather than assuming the
   callback already computes the correct abstract privacy factor.

Verus does **not** verify the actual `Measurement` object's trait-object
dispatch. The final theorem is about the executable builder's result and
its callbacks **under the adapter-correspondence premise**, not an
unconditional Rust `Measurement` validity theorem. The adapter's
small implementation is audited rather than verified.
