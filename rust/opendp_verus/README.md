# Verified Boolean randomized response

The Verus harness imports the **production** OpenDP measurement,
randomized-response constructor, sampler, domains, metric, and measure.

## Certification boundary

`Measurement::new_certified(input_domain, input_metric, output_measure, func, map)`
requires `valid_concrete_components` before storing ordinary Rust
`Fn` closures. Its contract covers input membership, callability, the
privacy map's totality and valid costs, and domination of the output
expectations for every neighboring pair. This constructor does not
supply a mechanism-specific operator or a probabilistic witness.

Its unchecked body just passes the closures to `Function::new_fallible`
and `PrivacyMap::new_fallible` and invokes the ordinary constructor.
Callback storage, `Arc<dyn Fn>` erasure, and their correspondence
contracts are part of the trusted boundary.

## Sampling and in-band credit verification

The native `sample_uniform_u64()` obtains eight random bytes via
`fill_bytes` and interprets them as a `u64`, propagating errors.
It has no probabilistic Verus postcondition of its own.

`sample_uniform_u64_with_credit` is the **single trusted in-band
credit-transfer rule** for this entropy draw. In both configurations
the wrapper invokes `sample_uniform_u64()`. In Verus it also accepts
an erased `Tracked<ErrorCredit>` and a `Ghost<WordCost>`, and returns
a tracked credit whose value is the selected cost of the actual word
or error. The native wrapper has no extra runtime arguments and
returns the original `Result`.

`sample_word_then` uses that same wrapper in both configurations
and applies the **production postprocessor** to the entropy result.
Verus proves that the output credit equals the postprocessor's
output allocation. Its in-band credit arguments and returned token
exist only in the Verus signature; the native API retains its three
original arguments. This replaces, rather than derives by a mode
conversion, the previous ghost-output EPT interface.

`BoolRR::release_callback` is verified against the concrete
`BoolRR::release`. The RR payoff inequality and the reported
floating-point privacy cost are proved using the actual operator,
including the rounding arithmetic contracts. The verifier checks
`valid_concrete_components` before the call to `new_certified`.

The separate external lemma `lemma_uniform_word_expectation_rule`
constrains the *mathematical law* of the entropy word, including the
input-independent error case. It is not a second executable credit
transfer. The custom `ErrorCredit` resource is tracked linearly,
but a full semantic soundness proof relating the adapted credit
calculus to distributional expectations is **not mechanized**.

## Effect-aware Fn experiment

`run_bool_rr_with_effect` is now defined next to the production
Boolean RR constructor and verifies the actual
`sample_word_then`/`BoolRR::release_callback` sampling pipeline
with in-band `Tracked<ErrorCredit>`. The native
`run_bool_rr` callback delegates to that same named function;
only the in-band credit arguments and result are erased.

`src/effect_fn_probe.rs` verifies an ordinary effect-aware
`Fn` closure that calls this **production** function. No
mechanism-specific sampler or proof model is copied into the harness.

A prior version with the same in-band contract was
[falsified successfully](https://github.com/opendp/opendp/actions/runs/44420407979):
consuming the RNG word but returning `Ok(*truth)` failed the
closure's postcondition (68 verified, 1 error).
The **production** `run_bool_rr_with_effect` was also
[falsified](https://github.com/opendp/opendp/actions/runs/44424090708):
after performing the verified draw, returning `Ok(*truth)`
instead failed at the production function's output-credit
postcondition (65 verified, 1 failed). The mutation was restored.

Earlier attempts to put `with Tracked(...)` on a closure, or
to convert `proof_with!` ghost outputs into executable `Tracked`
results, failed in the pinned Verus frontend. This motivated
using an in-band contract throughout the sampler rather than
wrappers between incompatible proof modes.

## Remaining trust / not yet end-to-end

The production constructor still installs:

```rust
let func = move |truth: &bool| run_bool_rr(&operator, truth);
let map = move |distance: &u32| operator.map_callback(distance);
```

The map closure's executable return relation is verified. However,
the ordinary `Fn(&bool)` has no linear-credit arguments, and its
`callback_expectation` is uninterpreted by Verus. The RR constructor
still contains one **explicit `assume`** equating that actual
closure's expectation to the proved one-word release expectation.
The `run_bool_rr` native forwarding function remains
`external_body`, although it now delegates directly to the
effect-verified production implementation. The generic
`new_certified` interface is unchanged.

Thus the **effect-aware closure** is verified, but erasing that
effect-aware interface into the ordinary `Fn` used by the
measurement is not. A sound EPT-to-`callback_expectation`
interpretation and the final callback forwarding/erasure theorem
are still missing; moving the `assume` elsewhere would not solve
this.

Other trusted components include the entropy distribution and
input-independent error law, the credit calculus's semantic
interpretation, floating-point division/logarithm and selected
arithmetic identities, plus native error/panic behavior. The
Verus harness represents the native error type as `()`.

The verified result is **conditional** on these boundaries, not a
fully mechanized theorem about every native runtime effect.
