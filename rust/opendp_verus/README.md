# OpenDP Verus proofs

This crate contains Verus proof companions for OpenDP Rust implementations. It
is a separate Cargo workspace because `vstd` is compiled by the pinned Verus
toolchain, not by OpenDP's normal Rust build.

`src/row_by_row.rs` proves stability 1 for the semantic kernel of
`make_row_by_row_fallible`. The proof is parameterized by an edit matching:

- arbitrary, substitution cost 2: `SymmetricDistance`
- ordered, substitution cost 2: `InsertDeleteDistance`
- full arbitrary matching, substitution cost 1: `ChangeOneDistance`
- full positional matching, substitution cost 1: `HammingDistance`

The proof conditions on both fallible invocations succeeding. This matches
OpenDP's `Function` contract: an error has no output carrier on which the output
metric could be evaluated.

`alerus/randomized_response_bool.rs` uses Alerus error credits to verify the
expectation-preservation rule for the executable rational-sampler mechanism.
It verifies the valid `RBig` decomposition and bounds used by OpenDP's sampler,
then proves the pure-DP inequality for every event in the Boolean output space.
The constructor-level theorem covers the `d_in = 0`, finite
`ln(p / (1 - p))`, and `p = 1` infinite-epsilon privacy-map branches. OpenDP
rejects the unsupported constant-time setting, so every constructed Boolean
randomized-response measurement follows the rational-sampler path.
`lemma_make_randomized_response_bool_returns_valid` combines these results into
a single theorem: whenever the proof-facing constructor succeeds, its result
satisfies the Boolean `DiscreteDistance` to `MaxDivergence` valid-measurement
predicate. The executable proof uses the production `truth ^ !honest` kernel,
and the fallible invocation model additionally proves that observing an entropy
error is independent of the private Boolean for a fixed sampler result.

Run `tools/verify_verus.sh` for the standalone Verus crate and
`tools/verify_alerus.sh` for the Alerus proofs.
