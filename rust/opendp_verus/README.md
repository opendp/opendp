# Verified Boolean randomized response

The Verus harness imports the production OpenDP `Measurement`, Boolean
randomized-response constructor, sampler, domain, metric, and measure.

## Certified constructor

`Measurement::new_certified` is independent of randomized response and
sampling algorithms. It takes exactly five runtime arguments:

```rust
new_certified(input_domain, input_metric, output_measure, func, map)
```

`func` and `map` are ordinary concrete `Fn` callbacks. Its
`valid_concrete_components` precondition requires domain membership
to imply that `func` is callable; each supported distance to be
accepted by `map`; every map result to be a successful, valid privacy
cost; and the *actual callback's* output expectations to satisfy the
semantic measure's domination inequality on every neighboring pair.

The only native work in `new_certified` is:

```rust
Self::new(
    input_domain, input_metric, output_measure,
    Function::new_fallible(func),
    PrivacyMap::new_fallible(map),
)
```

This unchecked storage operation is in the trusted computing base.
No mechanism-specific operator, entropy word, postprocessor, privacy
map factory, or semantic witness is an argument of `new_certified`.

## Boolean RR verification

The RR builder constructs its own closures:

```rust
let func = move |truth: &bool| run_bool_rr(&operator, truth);
let map = move |distance: &u32| operator.map_callback(distance);
```

The production `sample_word_then` and deterministic
`BoolRR::release_callback` are verified separately. One-word
expectation preservation uses a small Alerus-style linear error-credit
resource in `one_word_credit.rs`; the randomized-response payoff
inequality and the reported floating-point cost are proved using the
real operator. The RR-specific proof then establishes the generic
`valid_concrete_components` precondition *before* calling
`new_certified`.

## Explicitly trusted callback correspondence

**This part is not mechanized.** Ordinary Verus `Fn` contracts
describe possible return values, not the distribution of a randomized
closure. `sample_word_then`'s EPT contract needs ghost error credits
which a normal `Fn(&bool)` invocation cannot provide. The native
`run_bool_rr` forwarding method is therefore a one-line
`external_body`:

```rust
sample_word_then(operator, truth, BoolRR::release_callback)
```

The private `trust_bool_rr_closure_forwarding` proof function is also
`external_body`. It asserts only the execution correspondence of the
two literal closures above: the `func` law is the already-proved
one-word postprocessing law, and `map` forwards the actual operator
map. It **does not assert differential privacy**, but it *does* assert
a substantive probabilistic correspondence, and it would be unsound
if invoked on different callbacks. It must be audited together with
its single callsite. Merely passing Verus is not proof of that
correspondence.

This is distinct from the generic constructor's ordinary `Arc<dyn
Fn>` erasure trust. Future work could formally integrate EPTs for
concrete `Fn` closures to remove the former assumption.

Other trusted pieces are the RNG's uniform-word/input-independent
error law, the unmechanized soundness interpretation of the adapted
error-credit calculus, numerical division/logarithm rounding and
selected arithmetic identities, plus the native panic/error model.
Verus uses `()` for the native error value.

## Regression and falsification

Replacing the sampled entropy with `Ok(0)` *inside*
`sample_word_then` was rejected by Verus and failed native tests
(commit `d522b57`, restored in `b146c69`).

Earlier experiments showed that modifying an unchecked callback
adapter could pass Verus while breaking the native distribution. The
current callback-forwarding code is likewise an explicit trusted
boundary; its behavior must be reviewed and tested separately.

The resulting certificate is conditional on these documented
primitives and trusted Rust forwarding, not a mechanized theorem
about the entire native runtime.
