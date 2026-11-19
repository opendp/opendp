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

`sample_uniform_u64` is the **single native entropy entry point
and trusted in-band credit-transfer rule**. Its native body obtains
eight random bytes from `fill_bytes`, interprets them as a `u64`,
and propagates errors. Under Verus it accepts an erased
`Tracked<ErrorCredit>` and a `Ghost<WordCost>`, returning a tracked
credit whose value is the selected cost of the actual word or error.
There is no separate, externally specified sampling wrapper.

`sample_word_then` calls that same primitive in both configurations
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
transfer. That law now requires zero error mass for a zero payoff
and bounds error payoffs by the total error probability. The
**verified** `lemma_uniform_word_normalization` derives
`E[0] = 0` and `E[1] = 1` instead of adding those as trusted axioms.

The **verified** `lemma_true_event_probability` proves the exact
`Ok(true)` mass in the production Boolean RR release model:
`success_mass * p` when the private input is true and
`success_mass * (1 - p)` otherwise. The indicator-credit test now
uses this explicit quantity as its precondition.

These lemmas constrain the prefix-event payoffs needed by Boolean RR.
They do **not** establish all probability-measure laws (such as
linearity for arbitrary word payoffs) for the uninterpreted
`word_expectation`, nor do they connect it to the native entropy
provider. The `ErrorCredit` resource is tracked linearly. Following the
[upstream Alerus sampling rule](https://github.com/ahuoguo/alerus/blob/main/alerus/rand_primitives.rs),
the post-draw credit allocation is only required to be
**nonnegative**, not strictly below 1. This matters for 0/1
indicator payoffs: an allocation of 1 on a selected outcome is how
Alerus bounds that event's probability. The input credit is still
constrained by the resource algebra to be strictly below 1.
A post-draw credit of 1 is contradictory *in the probabilistic
error-credit semantics*; it does **not** establish that the event is
impossible as an ordinary Rust safety property.

The harness's `probe_rr_true_event` verifies an effect-aware
call with exactly such an indicator allocation. This exercises the
real production RR sampler, but does not prove a distributional
postcondition for the plain measurement `Fn`.

Alerus identifies ordinary `vstd::proph::Prophecy` variables
as unsound in combination with Eris-style credits; the proof
harness must not introduce them. `ProphecyGhost` is permitted by
upstream Alerus. Its Clippy ban cannot currently be run against this
Verus-only harness using ordinary Cargo, so **this restriction is not
automatically enforced** here. The
[AlerusBelt soundness argument](https://arxiv.org/abs/2607.12282)
is **not imported as a mechanized proof** of this adapted OpenDP
credit calculus, and its connection to runtime callback
expectations remains unproved.

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
closure's expectation to the proved one-word release expectation
**for bounded payoffs** (values between 0 and 1), which are exactly the
test functions quantified by `MaxDivergence::dominates`. The previous
assumption covered arbitrary real-valued payoffs unnecessarily.
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

A [direct falsification of the installed closure](https://github.com/opendp/opendp/actions/runs/44425623408)
replaced `move |truth| run_bool_rr(&operator, truth)` with
`move |truth| Ok(*truth)`, leaving the effect-verified sampler
unchanged. **Verus still verified all 66 obligations** because the
constructor's assumption directly supplies the expectation law.
The native `test_bool_rr_native_sampling_law_regression` rejected
the mutation (15 tests passed, 1 failed); the correct closure was
restored in [`0560dca`](https://github.com/opendp/opendp/commit/0560dca3dddc2f6590e52461d423fb4dec8613a3).
This is evidence of a genuine soundness gap, not proof that the
native regression test establishes privacy.

Closing the gap without a trusted assumption requires an operational
probabilistic semantics for invoking an ordinary Rust `Fn` that
accounts for the actual entropy effect, and a sound theorem connecting
that semantics to the in-band credit judgment. Ordinary pointwise
Verus `Fn` contracts, or another unproved callback wrapper, cannot
derive a probability distribution. The remaining explicit assumption
is therefore retained and auditable rather than presented as verified.

AlerusBelt's mechanized semantic model does not include the
full production Rust concurrency semantics. Here the claim is about
a **single mechanism invocation**; importing AlerusBelt's result
directly for a concurrently invoked `Arc<dyn Fn>` would require
additional justification. Upstream also excludes ordinary prophecy
variables, as noted above.

The verified result is **conditional** on these boundaries, not a
fully mechanized theorem about every native runtime effect.
