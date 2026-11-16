# Boolean randomized response: one executable kernel, one expectation proof

This verification experiment deliberately avoids a generic probabilistic
weakest-preexpectation library and a separate discrete PMF model.

## Mechanized so far

- The harness includes the **exact production source** for
  `dyadic53::success`, `dyadic53::from_word`, and `BoolRR::release`.
- `traits/samplers/bernoulli/verus.rs` gives one expectation-preservation
  contract for the uniform-word entropy primitive (including arbitrary
  error payloads) and proves the exact prefix probability.
- `measurements/randomized_response/verus.rs` composes that premise
  through the production method and proves the finite pure-DP inequality
  for *all bounded payoffs*, both neighboring Boolean inputs, and arbitrary
  error payloads. The p=1 case has infinite privacy cost.

The expectation functional passed to the proof **must denote the actual
execution of `sample_uniform_u64`**. This correspondence is still a
probabilistic trust obligation, not a Verus-established fact.

## Still required for a complete executable Measurement guarantee

1. Establish the expectation-preservation rule for OpenDP's actual fallible
   `fill_bytes` / `sample_uniform_u64` implementation, including freshness
   and input-independent errors.
2. Prove the input `f64` bit representation and the directed-rounded
   `inf_div` / `inf_ln` computation yield a conservative privacy factor.
3. Verify that the actual `Function`, `PrivacyMap`, and `Measurement::new`
   erasure preserve the certified operator and privacy map.

No `new_certified`, unchecked distribution bridge, or alternate executable
randomized-response implementation has been added.
