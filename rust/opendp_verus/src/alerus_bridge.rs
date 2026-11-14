//! Alerus's Bernoulli expectation rule specialized to the RR XOR kernel.
//! Alerus establishes the expectation-preservation rule for its rational
//! Bernoulli sampler. The *native* sampler correspondence remains an explicit
//! adapter obligation; it is not established by this module.
use vstd::prelude::*;

verus! {

pub open spec fn bernoulli_expectation(
    p: real,
    post: spec_fn(bool) -> real,
) -> real {
    p * post(true) + (1real - p) * post(false)
}

pub open spec fn rr_mass(truth: bool, output: bool, p: real) -> real {
    if truth == output { p } else { 1real - p }
}

pub open spec fn rr_event_indicator(
    truth: bool, output: bool, honest: bool,
) -> real {
    if (truth ^ !honest) == output { 1real } else { 0real }
}

/// The exact output law of the deterministic pushforward of Bernoulli(p).
pub proof fn lemma_rr_event_expectation(truth: bool, output: bool, p: real)
    ensures
        bernoulli_expectation(
            p, |honest: bool| rr_event_indicator(truth, output, honest),
        ) == rr_mass(truth, output, p),
{
    if truth {
        if output {
        } else {
        }
    } else {
        if output {
        } else {
        }
    }
}

} // verus!
