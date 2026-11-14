# Randomized response: verified model, unfinished runtime bridge

The standalone Verus crate proves Boolean randomized response privacy for a
*specified* probability law, including an input-independent entropy-error
outcome and every singleton observable `Result<bool, ()>`. It does **not**
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
