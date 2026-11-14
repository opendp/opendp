//! Boolean randomized response privacy over the entire observable Result law.
//!
//! This is a *conditional* model theorem. It proves the privacy guarantee for
//! the law stated below; it does not prove that OpenDP's fallible RNG realizes
//! that law. The error atom models entropy failure with input-independent
//! probability q. Whether the actual errors have this law is a separate
//! sampler contract obligation.
use vstd::prelude::*;
use crate::measurement::valid_discrete_max_divergence_measurement;
use crate::kernel;

verus! {

/// Call the *production* kernel, imported verbatim into the verifier.
pub fn rr_kernel(truth: bool, honest: bool) -> (output: bool)
    ensures output == (truth ^ !honest),
{
    kernel::release_bool(truth, honest)
}

pub fn rr_fallible_kernel_exec(
    truth: bool, sample: Result<bool, ()>,
) -> (output: Result<bool, ()>)
    ensures output == rr_fallible_kernel(truth, sample),
{
    kernel::release_result(truth, sample)
}

pub open spec fn rr_fallible_kernel(
    truth: bool, sample: Result<bool, ()>,
) -> Result<bool, ()> {
    match sample {
        Ok(honest) => Ok(truth ^ !honest),
        Err(_) => Err(()),
    }
}

pub open spec fn rr_mass(truth: bool, output: bool, p: real) -> real {
    if truth == output { p } else { 1real - p }
}

/// Bernoulli(p) expectation, the post-expectation shape used by Alerus.
pub open spec fn bernoulli_expectation(
    p: real, post: spec_fn(bool) -> real,
) -> real {
    p * post(true) + (1real - p) * post(false)
}

/// The output law after XOR, with an input-independent entropy-error atom.
/// q is the probability of Err; success is (1-q), followed by Bernoulli(p).
pub open spec fn rr_result_mass(
    truth: bool, outcome: Result<bool, ()>, p: real, q: real,
) -> real {
    match outcome {
        Ok(value) => (1real - q) * rr_mass(truth, value, p),
        Err(_) => q,
    }
}

/// Push forward an exact Bernoulli(p) result through the *real* executable
/// release_bool function. dual_spec generates its spec from production code.
/// XOR is an involution, so the unique honest bit yielding a given released
/// output is release_bool(truth, output).
pub open spec fn rr_pushforward_mass(
    truth: bool, outcome: Result<bool, ()>, p: real, q: real,
) -> real {
    match outcome {
        Ok(value) => (1real - q)
            * (if kernel::release_bool(truth, value) { p } else { 1real - p }),
        Err(_) => q,
    }
}

/// The sampler's Bernoulli law transported through the production postprocessor
/// agrees with the canonical randomized-response law, for every observable
/// success or entropy-error outcome.
pub proof fn lemma_pushforward_matches_rr(
    truth: bool, outcome: Result<bool, ()>, p: real, q: real,
)
    ensures
        rr_pushforward_mass(truth, outcome, p, q)
            == rr_result_mass(truth, outcome, p, q),
{
    match outcome {
        Ok(value) => {
            if truth { if value {} else {} }
            else { if value {} else {} }
        }
        Err(_) => {}
    }
}

/// Derive the successful output law from the Bernoulli post-expectation.
pub proof fn lemma_success_law(truth: bool, output: bool, p: real)
    ensures
        bernoulli_expectation(
            p, |honest: bool| if (truth ^ !honest) == output { 1real } else { 0real },
        ) == rr_mass(truth, output, p),
{
    if truth { if output {} else {} }
    else { if output {} else {} }
}

/// The three model outcomes form a genuine probability mass function.
pub proof fn lemma_rr_law_normalized(truth: bool, p: real, q: real)
    requires
        0real <= p <= 1real,
        0real <= q <= 1real,
    ensures
        rr_result_mass(truth, Ok(true), p, q) >= 0real,
        rr_result_mass(truth, Ok(false), p, q) >= 0real,
        rr_result_mass(truth, Err(()), p, q) >= 0real,
        rr_result_mass(truth, Ok(true), p, q)
            + rr_result_mass(truth, Ok(false), p, q)
            + rr_result_mass(truth, Err(()), p, q) == 1real,
{
    if truth {
        assert((1real - q) * p >= 0real) by(nonlinear_arith)
            requires 0real <= p <= 1real, 0real <= q <= 1real;
        assert((1real - q) * (1real - p) >= 0real) by(nonlinear_arith)
            requires 0real <= p <= 1real, 0real <= q <= 1real;
    } else {
        assert((1real - q) * (1real - p) >= 0real) by(nonlinear_arith)
            requires 0real <= p <= 1real, 0real <= q <= 1real;
        assert((1real - q) * p >= 0real) by(nonlinear_arith)
            requires 0real <= p <= 1real, 0real <= q <= 1real;
    }
    assert(rr_result_mass(truth, Ok(true), p, q)
        + rr_result_mass(truth, Ok(false), p, q)
        + rr_result_mass(truth, Err(()), p, q) == 1real)
        by(nonlinear_arith);
}

/// The failure branch propagates *the same* error for either input when the
/// sampler's result is fixed. The sampler's result law must also be independent
/// of the private input; this lemma alone cannot establish that property.
pub proof fn lemma_error_propagates(
    truth: bool, sample: Result<bool, ()>,
)
    ensures
        match sample {
            Err(_) => rr_fallible_kernel(truth, sample) == Err(()),
            Ok(honest) => rr_fallible_kernel(truth, sample) == Ok(truth ^ !honest),
        },
{
}

/// DiscreteDistance: identical inputs have distance zero, any distinct input
/// pair is close only at positive distance.
pub open spec fn discrete_close(left: bool, right: bool, distance: u32) -> bool {
    left == right || distance > 0
}

/// Some(factor) represents exp(epsilon); None represents infinite epsilon.
/// This deliberately avoids axiomatizing floating-point ln/exp.
pub open spec fn rr_map(p: real, factor: real, distance: u32) -> Option<real> {
    if distance == 0 { Some(1real) }
    else if p == 1real { None }
    else { Some(factor) }
}

pub open spec fn rr_valid_measurement(p: real, q: real, factor: real) -> bool {
    valid_discrete_max_divergence_measurement::<bool, Result<bool, ()>, u32, ()>(
        |_: bool| true,
        |left: bool, right: bool, distance: u32| discrete_close(left, right, distance),
        |truth: bool, outcome: Result<bool, ()>| rr_pushforward_mass(truth, outcome, p, q),
        |distance: u32| Ok(rr_map(p, factor, distance)),
    )
}

/// The unscaled Bernoulli RR success law obeys pointwise domination.
pub proof fn lemma_success_domination(
    p: real, factor: real, left: bool, right: bool, output: bool,
)
    requires
        1real / 2real <= p < 1real,
        factor >= 1real,
        p <= factor * (1real - p),
    ensures
        rr_mass(left, output, p) <= factor * rr_mass(right, output, p),
{
    if left == right {
        if output == left {
            assert(p <= factor * p) by(nonlinear_arith)
                requires p >= 0real, factor >= 1real;
        } else {
            assert(1real - p <= factor * (1real - p)) by(nonlinear_arith)
                requires 1real - p >= 0real, factor >= 1real;
        }
    } else if output == left {
    } else {
        assert(1real - p <= p);
        assert(p <= factor * p) by(nonlinear_arith)
            requires p >= 0real, factor >= 1real;
    }
}

/// Lift success domination to the observable Result law, including Err.
pub proof fn lemma_result_domination(
    p: real, q: real, factor: real,
    left: bool, right: bool, outcome: Result<bool, ()>,
)
    requires
        1real / 2real <= p < 1real,
        0real <= q <= 1real,
        factor >= 1real,
        p <= factor * (1real - p),
    ensures
        rr_result_mass(left, outcome, p, q)
            <= factor * rr_result_mass(right, outcome, p, q),
{
    match outcome {
        Ok(output) => {
            lemma_success_domination(p, factor, left, right, output);
            let a = rr_mass(left, output, p);
            let b = rr_mass(right, output, p);
            assert((1real - q) * a <= factor * ((1real - q) * b))
                by(nonlinear_arith)
                requires
                    a <= factor * b,
                    1real - q >= 0real;
        }
        Err(_) => {
            assert(q <= factor * q) by(nonlinear_arith)
                requires q >= 0real, factor >= 1real;
        }
    }
}

/// The probability of *any* Boolean-or-error event in the model.
pub open spec fn rr_event_probability(
    truth: bool, p: real, q: real,
    event: spec_fn(Result<bool, ()>) -> bool,
) -> real {
    (if event(Ok(true)) { rr_result_mass(truth, Ok(true), p, q) } else { 0real })
    + (if event(Ok(false)) { rr_result_mass(truth, Ok(false), p, q) } else { 0real })
    + (if event(Err(())) { rr_result_mass(truth, Err(()), p, q) } else { 0real })
}

/// Pointwise domination implies domination for every subset of the
/// three possible observable outcomes. This makes the discrete
/// MaxDivergence interpretation explicit rather than merely asserted.
pub proof fn lemma_every_event_domination(
    p: real, q: real, factor: real,
    left: bool, right: bool,
    event: spec_fn(Result<bool, ()>) -> bool,
)
    requires
        1real / 2real <= p < 1real,
        0real <= q <= 1real,
        factor >= 1real,
        p <= factor * (1real - p),
    ensures
        rr_event_probability(left, p, q, event)
            <= factor * rr_event_probability(right, p, q, event),
{
    lemma_result_domination(p, q, factor, left, right, Ok(true));
    lemma_result_domination(p, q, factor, left, right, Ok(false));
    lemma_result_domination(p, q, factor, left, right, Err(()));

    let a = if event(Ok(true)) { rr_result_mass(left, Ok(true), p, q) } else { 0real };
    let b = if event(Ok(true)) { rr_result_mass(right, Ok(true), p, q) } else { 0real };
    let c = if event(Ok(false)) { rr_result_mass(left, Ok(false), p, q) } else { 0real };
    let d = if event(Ok(false)) { rr_result_mass(right, Ok(false), p, q) } else { 0real };
    let e = if event(Err(())) { rr_result_mass(left, Err(()), p, q) } else { 0real };
    let f = if event(Err(())) { rr_result_mass(right, Err(()), p, q) } else { 0real };

    assert(a <= factor * b);
    assert(c <= factor * d);
    assert(e <= factor * f);
    assert(a + c + e <= factor * (b + d + f)) by(nonlinear_arith)
        requires a <= factor * b, c <= factor * d, e <= factor * f;
}

/// Every successfully constructed *model* satisfies its Measurement contract.
/// Alerus/native sampling correspondence and floating-point factor bounds
/// are not yet proven; they are explicit preconditions, not hidden axioms.
pub proof fn lemma_make_randomized_response_bool_returns_valid(
    p: real, q: real, factor: real,
)
    requires
        1real / 2real <= p <= 1real,
        0real <= q <= 1real,
        factor >= 1real,
        p < 1real ==> p <= factor * (1real - p),
    ensures rr_valid_measurement(p, q, factor),
{
    assert forall|left: bool, right: bool, distance: u32|
        #[trigger] discrete_close(left, right, distance)
        implies match rr_map(p, factor, distance) {
            None => true,
            Some(cost) => forall|outcome: Result<bool, ()>|
                rr_pushforward_mass(left, outcome, p, q)
                    <= cost * rr_pushforward_mass(right, outcome, p, q),
        } by {
        if distance == 0 {
            assert(left == right);
        } else if p == 1real {
            assert(rr_map(p, factor, distance) == None);
        } else {
            assert(1real / 2real <= p < 1real);
            assert(p <= factor * (1real - p));
            assert forall|outcome: Result<bool, ()>|
                rr_pushforward_mass(left, outcome, p, q)
                    <= factor * rr_pushforward_mass(right, outcome, p, q) by {
                lemma_pushforward_matches_rr(left, outcome, p, q);
                lemma_pushforward_matches_rr(right, outcome, p, q);
                lemma_result_domination(p, q, factor, left, right, outcome);
            }
        }
    }
}

} // verus!
