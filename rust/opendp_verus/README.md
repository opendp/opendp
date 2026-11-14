# Randomized response: verified model, unfinished runtime bridge

The Verus Cargo harness imports the actual production kernel and in-tree
proof modules. The harness itself contains no proof implementation. Verus
proves the executable `kernel::release_bool` and `kernel::release_result`
functions, including exact error propagation, while the remaining
probabilistic semantics are still conditional. The in-tree model proof proves Boolean randomized response privacy for a
*specified* probability law, including an input-independent entropy-error
outcome and every singleton observable `Result<bool, ()>`. The proof also
checks that these three probabilities sum to one, and that the privacy
inequality holds for all events (all subsets of the three outcomes). It does **not**
verify that the production sampler realizes that law.

## Explicit remaining obligations

1. **Sampler law:** for any fixed non-private rational probability `p`,
   `sample_bernoulli_rational` must have an input-independent error
   probability `q`, conditional on success it produces Bernoulli(p).
   The proof models only one error atom; a production-level proof must also
   account for the full error payload distribution.
2. **Randomness foundation:** Alerus's Bernoulli expectation-preservation
   theorem relies on a trusted `rand_ubig` contract; the current
   `alerus_bridge` *did not* use that theorem, so it has been removed.
   We need a sound bridge from a uniform source to the actual fallible
   OpenDP sampler, not a second manually specified sampler model.
3. **Numeric bound:** native `f64` privacy constant must satisfy
   `exp(privacy_constant) >= p/(1-p)` for all accepted `p < 1`.
   The Verus theorem instead assumes a real factor satisfying this bound.
4. **Constructor correspondence:** the native closure and dynamic
   `Measurement` wrapping must realize the kernel and privacy map analyzed
   by Verus. The premature `new_certified` marker was removed until this
   obligation is actually discharged.

The `make_randomized_response_bool` constructor retains `constant_time=true`
through its original sampler path, which is **not** within this proof scope.
For `constant_time=false`, it uses the rational sampler.

CI separately runs the standalone Verus verifier and native Rust tests.


## Source integration

- `rust/src/measurements/randomized_response/kernel.rs`: executed in production
  and verified directly using attribute-based Verus annotations.
- `rust/src/measurements/randomized_response/verus.rs`: operator proof using
  the production kernel's verified contract.
- `rust/src/core/verus/measurement.rs`: generic discrete-law Measurement spec.
- `rust/opendp_verus`: minimal Cargo verification harness only; no duplicate
  executable implementation or duplicate proof definitions.

The main Measurement theorem is now stated over `rr_pushforward_mass`:
the output distribution obtained by applying the *production* `release_bool`
function (with Verus-generated dual spec) to an assumed Bernoulli sampler.
The earlier `rr_result_mass` formula is only a derived lemma, not an
independently assumed mechanism law. This still requires verifying that
OpenDP's sampler satisfies the Bernoulli distributional contract.

## Fixed-width Bernoulli specialization

The Boolean RR path no longer allocates an arbitrary-precision rational or
uses rejection sampling. For all valid p in [0.5, 1], a binary64 value is an
integer numerator over 2^53. A single uniform 64-bit entropy sample is shifted
by 11 bits and compared with the numerator. The native integer functions in
`traits/samplers/bernoulli/dyadic53.rs` are imported and checked directly by
Verus. Uniform entropy, the float/bit correspondence and numeric privacy-map
rounding are still outstanding parts of full-stack verification.

The actual sampler now delegates all deterministic error propagation and
Bernoulli postprocessing to `dyadic53::from_word`, which is a shared
Verus-checked executable function. The proof calls `from_word` and
`release_result` together, verifying the complete deterministic path from
an entropy word (or an error) to the returned randomized-response bit.

The adjacent `traits/samplers/bernoulli/verus.rs` proof now reduces the
production Bernoulli success condition to a prefix event on a uniform 64-bit
word, and derives its exact mass `numerator / 2^53` from the ideal uniform-u64
CDF. This is a deterministic reduction with a transparent entropy premise;
it is not yet a machine-checked correctness proof of `fill_bytes`.
