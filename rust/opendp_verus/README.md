# Verus verification of Boolean randomized response

The public production `make_randomized_response_bool` is verified through
`build_bool_rr_measurement` and the actual `Measurement::new_certified` call.
The certified constructor is now **generic** over domain, metric, privacy
measure, and output type; it consumes semantic traits defined in
`core/measurement_semantics.rs`. Its verified precondition requires joint
privacy validity of the concrete `Function` and `PrivacyMap` objects; validity
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
  infinite-cost case. The generic certified predicate requires the map to be
  callable and for every returned value at any valid input distance to be
  successful and a valid bound. A map that always returns `Err` is rejected.
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

## Concrete one-word sampler: EPT milestone

The production `sample_word_then` now has a verified **error-credit
transport** contract. An arbitrary cost function on entropy results,
bounded in `[0, 1)`, is assigned to the exact result of
`sample_uniform_u64()`. The production `postprocess` invocation must
preserve that cost according to its verified return specification.

- `rust/opendp_verus/src/one_word_credit.rs` contains the minimal linear
  `vstd::resource` credit used in this argument. There is no general
  Alerus dependency, thin-air rule, recursion, or rejection sampler.
- `sample_uniform_u64` is the **only** function with the trusted
  probabilistic credit-transfer rule. `sample_word_then` executes and
  verifies the ordinary call followed by the ordinary postprocessor.
  Ghost arguments are supplied with `proof_with!` and erased natively.
- `BoolRR::release_callback` has a verified lemma showing that pulling
  back an arbitrary strictly bounded output payoff gives a valid input
  allocation for the generic one-word rule. The first-stage contract
  therefore applies to the actual deterministic Boolean RR callback.
- The *interpretation* of the resource judgment as an expectation theorem
  depends on the soundness of the probabilistic error-credit model.
  The adapted credit calculus has **not** itself been given a mechanized
  soundness proof here. Verus checks the conditional EPT judgment, not
  an independent operational-probability semantics of the Rust RNG.
  Arbitrary payoffs in `[0, 1]` can be scaled into `[0, 1)` when
  applying the EPT metatheorem.

The trusted erasure contract in `Function::new_from_one_word_operator`
remains separate and **has not** been discharged by this milestone.
The existing `function_expectation` equality still depends on that
contract when constructing a `Measurement`.

## Falsification

Commit `d522b57` changed only the executable
`sample_word_then` callback input from the sampled `entropy` to
`Ok(0u64)`, leaving the entropy draw in place. Verus rejected the
claim `call_ensures(postprocess, (operator, input, entropy), output)`
at `uniform_word.rs:100`, exactly where the output must be linked to
the random word. The native regression also failed. Commit
`b146c69` restored the production call and both CI jobs passed.
This is a negative test of *execution-linked* credit transport, not
a test of the `Arc<dyn Fn>` erasure claim.


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
- `Function::new_from_one_word_operator` is an `external_body` with
  an expectation-preservation postcondition. **This is substantially more
  trust than Arc dispatch:** it assumes the distribution of the erased
  executable callback is the prescribed one-word law. A native-only change
  to this adapter can invalidate privacy without invalidating Verus.
  The bounded-postcondition proof does not justify this assumption.
- `PrivacyMap::new_from_operator` separately trusts faithful
  `Arc<dyn Fn>` dispatch for the executable map. Its callback implementation
  and total-success behavior are verified before erasure.
- The production `BoolRR::invoke` duplicate was removed: the installed
  Function's actual native sampling path is `sample_word_then` followed
  by `BoolRR::release_callback`. Native sampling regression tests cover
  this erased path; they do not discharge its expectation theorem.
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
- The generic certified contract is reusable, but concrete verified semantic
  implementations currently cover the Boolean domain, `DiscreteDistance`,
  and `MaxDivergence`. The mechanism-specific RR proof is retained as an
  intermediate lemma that discharges the generic constructor contract.
- `new_certified` still accepts erased `Function` and `PrivacyMap` objects,
  rather than concrete closure arguments. The probabilistic distributional
  equality at their erasure boundary is still trusted and is not proved by
  Alerus. This is the principal outstanding certification gap.
- The native semantic traits are marker bounds while Verus provides their
  specifications. Only verified constructor call sites may rely on the
  guarantee; the Rust type system alone does not enforce proofs.

The proof is conditional on these stated primitives. It is not a proof of
the native entropy source, the full Rust runtime, or arbitrary OpenDP
measurements. No caller-supplied privacy witness or certification token
is trusted.

## Next proof boundary

For an actual end-to-end certificate, the expectation of the installed
`Function` must be **derived**, not stipulated. A sound replacement should
(a) justify the probabilistic behavior of the shared executable
`sample_word_then` using the trusted `sample_uniform_u64` rule (e.g.
Alerus-style expectation-preserving reasoning), and (b) prove or narrowly
trust that the `Arc<dyn Fn>` trampoline executes that verified program.
Importing an Alerus-style ghost model without a sound connection to the
native closure **does not** close the gap. The acceptance test is that
replacing the closure's entropy with a fixed word must cause **Verus**
verification to fail, not merely the native statistical tests.
