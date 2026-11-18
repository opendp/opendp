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

## Typed program and remaining erasure boundary

The Boolean RR constructor now builds the actual `OneWordProgram`
operator/postprocessor pair as an ordinary Rust struct. Its production
`run` method is verified to propagate the one-word error credit by
calling the verified `sample_word_then`. The generated native closure
in `Function::erase_one_word_program` is only
`move |arg| program.run(arg)`. No duplicate sampler or callback
implementation is used.

**The erasure itself remains trusted.** Its `external_body` contract
now records only the one-word *operational provenance* of the installed
`Function`, rather than postulating its complete expectation law.
`function_expectation` is defined from that provenance and the
primitive word expectation, with opaque semantics for ordinary Functions.
This removes a separate probabilistic assumption from the erasure
contract, but Verus does **not** prove that the native `Arc<dyn Fn>`
executes the recorded program. Altering only the closure inside this
external body can therefore still falsify the privacy claim without a
Verus failure. The soundness metatheorem for the error-credit model
is also not mechanized.

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

Commit `ef28151` retained the entropy draw in the new
`OneWordProgram::run` but discarded its result and returned
postprocessing of `Ok(0)`. Verus rejected the **postcondition of
the typed program itself** (`uniform_word.rs:150`), while the
native distribution regression failed. Commit `bb3b770` restored
the proper return path. This independently tests the newly introduced
typed program boundary.

Commit `7b15f19` altered **only** the native closure inside
`Function::erase_one_word_program` to use fixed-word postprocessing.
Verus **still passed** (70 verified, 0 errors); the native sampling
regression failed. Commit `8f58d1f` restored the closure.
This is the decisive negative test for the current unresolved trust
boundary: the external operational-provenance assertion is not
checked against the runtime `Arc` erasure code.


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
- `Function::erase_one_word_program` is an `external_body`
  whose remaining trusted postcondition identifies the installed
  **deterministic one-word program**, not its complete probability law.
  `function_expectation` now derives the one-word law from this
  provenance and `uniform_word_expectation`. Verus verifies construction
  of the typed program and its error-credit EPT, but **not** the
  correspondence between the `Arc<dyn Fn>` trampoline and the named
  `program.run`. The native-only mutation `7b15f19` shows that
  this remaining trust can invalidate privacy without a Verus error.
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

The concrete `OneWordProgram::run` satisfies the one-word
error-credit EPT and mutations of both sampling layers are rejected
by Verus. The erased Function's expectation is now a *definition*
from one-word operational provenance rather than a postulated numeric
equality. However, the native `Arc<dyn Fn>` forwarding of the named
program remains a trusted external-body assertion. Mutation `7b15f19`
confirmed Verus cannot detect a dishonest implementation of that
forwarding.

To eliminate this last trusted callback correspondence, either verify
an actual `Arc`/closure dispatch representation or represent one-word
sampling explicitly in the runtime `Function` and verify its invocation
there. Both options should be evaluated against the cost of expanding
the production `Function` abstraction. Separately, a soundness
argument for the restricted error-credit logic is still necessary.
Until then this remains a **conditional**, not unconditional,
end-to-end privacy proof.
