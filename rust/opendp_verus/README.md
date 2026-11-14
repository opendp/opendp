# Verified Boolean randomized response (experimental)

This branch contains only the new Verus proof companion and native
randomized-response Measurement integration. It deliberately leaves out the
old Lean/SampCert tree from the earlier alerus branch.

- measurement.rs: generic fallible-map Measurement validity predicate.
- alerus_bridge.rs: expectation calculation for the Bernoulli-XOR kernel.
- randomized_response_bool.rs: executable Boolean kernel, input-independent
  error channel, complete Boolean event domination and constructor model.
- The native constructor goes through an operator-specific certified bridge.

The following **remain trusted**, and must be closed before the result is
claimed as a fully mechanized production privacy proof:
1. Correspondence from Alerus's Bernoulli expectation-preservation rule to
   OpenDP's production rational Bernoulli sampler.
2. Correspondence of the exact rational input probability and the numeric
   directed-rounding privacy-map constant to the real model, in particular
   exp(epsilon) >= p/(1-p).
3. An input-independent entropy failure channel.

The proof uses factor = exp(epsilon) symbolically and represents infinity
separately; it does not incorrectly treat floating-point log as exact.
The Boolean constructor rejects constant_time=true for the experimental
verified path. Categorical randomized response is left unchanged.
