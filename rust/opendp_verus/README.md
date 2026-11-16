# Boolean randomized response: verified kernel and constructor input

The public `make_randomized_response_bool` constructor now uses the
**Verus-verified** production `dyadic53::checked_numerator_from_bits`.
For every accepted IEEE-754 encoding, Verus proves that the decoded
numerator represents the exact `f64` value in the range `[0.5, 1.0]`.
The native constructor is built around `BoolRR<f64>`, and its function
callback calls `BoolRR::invoke` (one entropy draw followed by the
verified `BoolRR::release`). Its privacy callback invokes the same
operator's `BoolRR::map` on a copy of the operator.

## Verified theorem

`lemma_validated_bool_rr_privacy` connects the public constructor's
checked binary64 bits, exact integer odds, the production Boolean RR
postprocessor, and an arbitrary bounded payoff on `Result<bool, E>`.

For every valid accepted bit pattern and every input pair at the
specified discrete distance, it proves the pure-DP relational expectation
inequality under these two *explicit* premises:

1. `uniform_word_expectation_rule` describes the law of the **actual**
   `sample_uniform_u64` call, including its input-independent error
   payloads.
2. The real factor corresponding to the reported privacy cost satisfies
   `numerator <= factor * (2^53 - numerator)` for finite costs.

Verus proves the exact decoder, integer complement, prefix-event
counting, `BoolRR::release` and `BoolRR::map`, the odds implication,
and the output expectation inequality. A `p=1` operator has infinite
privacy cost for positive distances.

## Why this is not yet a theorem about the returned Measurement

We have **not** proved the formula

```text
make_randomized_response_bool(p) = Ok(m)  ==>  ValidMeasurement(m)
```

in Verus. Calling a proof about the concrete operator a proof of this
formula would be an unsound shortcut. The remaining work is precisely:

1. Prove/axiomatize at the entropy boundary that the
   `sample_uniform_u64` expectation contract represents the actual
   `fill_bytes` runtime behavior and is fresh across invocations.
2. Establish the directed-rounding specification of
   `f64::inf_div` and `f64::inf_ln`, and connect the actual returned
   `privacy_constant` to the factor used in the DP theorem.
3. Prove that the captured `BoolRR` and `privacy_cost` survive
   `Function::new_fallible`, `PrivacyMap::new`, and
   `Measurement::new` and retain their expected behavior under
   `Measurement::invoke` and `Measurement::map`.
   This should be a **generic, reusable** verified erasure theorem
   over OpenDP's existing core types, not a custom certification marker.

The metric-space check is not a substantive blocker here:
`(AtomDomain<bool>, DiscreteDistance)::check_space` returns `Ok(())`
in production. The constructor stores the callbacks unchanged; the
obstacle is checking the higher-order erasure bridge in Verus.

No separate probability-mass-function implementation, duplicate
randomizer, trusted `new_certified` marker, or new runtime crate was
introduced. Native and Verus CI exercise the shared production source.


## Constructor erasure research finding

The current pinned Verus rejects `Arc<dyn Fn(&TI) -> Fallible<TO>>`
as an unsupported multi-trait object (the `Fn` hierarchy).
A verification-only probe established that dispatch through a
**single-trait** `Arc<dyn Callable>` with an inherited Verus method
contract is supported. This probe was removed after verification:
it does **not** prove anything about production `Arc<dyn Fn>`.

A sound approach needs either a reusable, verified single-trait
callback abstraction in OpenDP core or a narrow, justified
standard-library/trait-object erasure contract. We should not
introduce a `new_certified` marker or a direct postulate about
`Measurement` validity merely to hide this obligation.
