# Verified Boolean randomized response

The Verus harness imports the production `make_randomized_response_bool`,
`BoolRR`, sampler, domains, metrics, measures, and `Measurement` source.

## Proof boundary

`Measurement::new_certified` accepts the **concrete** operator,
postprocessor, and privacy-map callback. Before calling it, the mechanism
proves `valid_one_word_components`, defined in
`core/verus/measurement.rs`. This contract requires:

- A deterministic result specification matching the actual postprocessor
  for *every* entropy word and error result.
- A callable privacy map whose every result is a successful, valid cost.
- The generic privacy-measure domination inequality for every supported
  pair of domain members at every supported metric distance, using the
  exact one-word sampling law.

`new_certified` does not inspect the mechanism or repeat the privacy
proof. Its unchecked native body is just
`Function::new_fallible`, `PrivacyMap::new_fallible`, and
`Measurement::new`. This is an **explicitly trusted Rust
callback-storage boundary**: it must install and invoke the same
callbacks that were proved valid. The preconditions are established
before this erasure takes place.
There are no mechanism-specific erasure adapters, callback witness
tokens, or additional `OneWordProgram` representation.

This first certified constructor is generic in domain, metric, privacy
measure, and output type, but currently accepts a concrete *one-word*
sampling operator. Extending certification to other sampler structures
requires their own executable sampling specifications; it must not
create a new trusted privacy claim.

## Probabilistic reasoning

The production `sample_word_then` is Verus-checked. Its result consumes
a single `sample_uniform_u64` draw and executes the actual deterministic
callback. The tiny verification-only `one_word_credit.rs` adapts an
Alerus-style linear error credit to establish a one-draw
expectation-preserving judgment. `BoolRR::release_callback` is checked
against its deterministic result model. The Boolean RR payoff inequality
and directed logarithm bound are then used to discharge the generic
constructor precondition **before erasure**.

The probabilistic interpretation of these credits depends on an
**unmechanized soundness argument for the adapted credit calculus**.
The primitive's uniform-word law and input-independent error behavior
are trusted, not properties verified of the native RNG.

## Other trusted operations

- `assemble_unchecked` and the conventional Rust `Arc<dyn Fn>`
  implementation used to store and invoke the supplied callbacks.
  Verus does not inspect their implementations.
- Numerical division/logarithm rounding, exact integer conversions,
  the floating-point bit-cast contract, positive infinity, and external
  real exponential/logarithm facts.
- Panic behavior and the exact native error representation are not
  fully modeled. Verus represents errors by `()`; the underlying RR
  payoff theorem is polymorphic over error values.

`Domain`, `DiscreteDistance`, `MaxDivergence`, metric-space checks,
and the public Boolean RR constructor use production source. Native
`CertifiedDomain`, `CertifiedMetric`, and `CertifiedMeasure` are
marker bounds; the real mathematical specifications are checked in
Verus. The native Rust type system alone is not a proof of certification.

## Negative tests

A temporary change that draws entropy but passes `Ok(0)` to the
postprocessor was rejected by Verus at the concrete
`sample_word_then` proof and failed the native distribution test
(commit `d522b57`, restored in `b146c69`).

Earlier erasure experiments showed that mutating an unchecked native
adapter could pass Verus while failing native tests. That is why
unchecked storage is now explicit, minimal, and outside the
mathematical verification boundary. It remains part of the trusted
computing base, rather than being described as verified code.

The resulting privacy proof is conditional on these stated primitives
and trusted operations, not an unconditional proof of the Rust runtime
or every OpenDP measurement.
