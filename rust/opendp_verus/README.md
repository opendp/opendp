# Verus verification of Boolean randomized response

The production `build_bool_rr_measurement` is verified through the actual
`Measurement::new_certified` call. The constructor **requires** validity of
the concrete `Function` and `PrivacyMap` objects it receives; validity
is not assumed when they are assembled into a `Measurement`.

## Verified

- Production probability decoding, Bernoulli word postprocessing, and
  conservative log-odds construction (subject to the numerical contracts below).
- A single `sample_uniform_u64` draw followed by the executable
  `BoolRR::release_callback`. Its result model is checked against the
  callback's actual Verus postcondition for *every* word and error result.
- The distribution of the erased `Function`, expressed as expectations of
  arbitrary bounded payoffs on `Result<bool, ()>`. This uses the trusted
  one-word entropy law and a generic `Arc<dyn Fn>` erasure contract.
- The actual erased `PrivacyMap` forwards `BoolRR::map_callback`, and
  its returned epsilon is finite or positive infinity (never NaN or
  negative infinity). A zero input distance reports positive zero.
- The reported cost bounds the returned function's distribution under the
  Boolean discrete metric and `MaxDivergence`, including errors and the
  infinite-cost case.
- The *input* contract of production `Measurement::new_certified` requires
  this privacy validity of the actual callback objects. The constructor's
  verified body preserves them in the returned measurement.
- `DiscreteDistance` and `MaxDivergence` types are the same production
  source in native OpenDP and the Verus harness. Their mathematical
  discrete adjacency and multiplicative-factor specifications live beside
  their type definitions. The constructor no longer accepts another
  metric or measure merely because its distance type matches.
- The guarantee applies to any compatible Boolean input domain: the
  theorem is proved for *both* possible Boolean inputs, so restricting
  membership cannot invalidate it.

## Falsification

Commit `07bfc4e` temporarily replaced the already-proved sampled-word
`Function` with an always-truthful `Function` at the final constructor
call. Verus rejected the **precondition at `new_certified`**. The correct
function was restored in `521d161`. This confirms that validity cannot
be borrowed from an unrelated callback.

An earlier negative test (`6424e0f`) similarly rejected the installation
of a zero-cost privacy map that disagreed with the operator's cost.

## Explicit trust and remaining work

- `sample_uniform_u64` is trusted to provide a uniform word on success,
  with errors independent of the private Boolean. The actual RNG backend,
  its failure behavior, and entropy quality are not mechanized.
- The generic `Function::new_from_one_word_operator` and
  `PrivacyMap::new_from_operator` `external_body` contracts trust
  faithful `Arc<dyn Fn>` dispatch and one-word expectation preservation.
  **Neither assumes differential privacy.**
- Numerical upward rounding for division and logarithm, exact integer
  casts and positive infinity are trusted primitives. Mathematical
  exponential/logarithm facts are external. Panic behavior is not proved.
- Verus currently models `Error` as `()`. The general mathematical
  randomized-response theorem is polymorphic in error values, but
  faithful modeling of OpenDP's native error representation is not proved.
- The verifier imports the production shared RR builder and constructor,
  but not the thin public `make_randomized_response_bool` wrapper and
  its actual `AtomDomain<bool>` default/metric-space implementation.
- The validity predicate and constructor are presently specialized to
  Boolean input, `DiscreteDistance`, and `MaxDivergence`, rather than
  generalized to every `Domain`, `Metric`, or privacy `Measure`.

The proof is conditional on these stated primitives. It is not a proof of
the native entropy source, the full Rust runtime, or arbitrary OpenDP
measurements. No caller-supplied privacy witness or certification token
is trusted.
