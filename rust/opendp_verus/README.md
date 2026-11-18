# Verus verification of Boolean randomized response

The public production `make_randomized_response_bool` is verified through
`build_bool_rr_measurement` and the actual `Measurement::new_certified` call. The constructor **requires** validity of
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
- The same production `make_randomized_response_bool` body runs in
  native OpenDP and the Verus harness, including probability validation,
  `AtomDomain::<bool>::new_non_nan()`, and the certified constructor.
  Native tests check that this non-null domain equals the previous
  `AtomDomain::<bool>::default()`, admits both Booleans, and that the
  public privacy map handles zero, finite, and infinite costs.
- The production `AtomDomain` representation, non-null constructor,
  `DiscreteDistance` metric-space check, and the Boolean wrapper are
  shared source. The privacy theorem applies to both Boolean inputs, so
  it does not require an assumption about general `CheckAtom` membership.

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
- `f64::to_bits` is trusted through a narrow, exact bit-cast contract
  because Verus does not yet support that Rust intrinsic directly.
- Verus uses a type-only `CheckAtom` bound. The generic production
  `Domain::member` implementation is not verified; no membership fact
  is assumed by the privacy proof, which covers every Boolean input.
  `AtomDomain::<bool>::new_non_nan()` and the relevant metric-space
  check are verified from their production source.
- The validity predicate and constructor are presently specialized to
  Boolean input, `DiscreteDistance`, and `MaxDivergence`, rather than
  generalized to every `Domain`, `Metric`, or privacy `Measure`.

The proof is conditional on these stated primitives. It is not a proof of
the native entropy source, the full Rust runtime, or arbitrary OpenDP
measurements. No caller-supplied privacy witness or certification token
is trusted.
