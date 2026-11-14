//! Boolean randomized response privacy over the entire observable Result law.
//!
//! This is a *conditional* model theorem. It proves the privacy guarantee for
//! the law stated below; it does not prove that OpenDP's fallible RNG realizes
//! that law. The error atom models entropy failure with input-independent
//! probability q. Whether the actual errors have this law is a separate
//! sampler contract obligation.
use vstd::prelude::*;
use crate::measurement::valid_measurement;

verus! {

/// Same executable deterministic kernel as the native RR callback.
pub fn rr_kernel(truth: bool, honest: bool) -> (output: bool)
    ensures output == (truth ^ !honest),
{
    truth ^ !honest
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

/// A pointwise pure-DP relation on the full result law.
pub open spec fn max_divergence_bound<Y>(
    left: spec_fn(Y) -> real,
    right: spec_fn(Y) -> real,
    bound: Option<real>,
) -> bool {
    match bound {
        None => true,
        Some(factor) => forall|outcome: Y|
            #[trigger] left(outcome) <= factor * right(outcome),
    }
}

pub open spec fn rr_valid_measurement(p: real, q: real, factor: real) -> bool {
    valid_measurement::<bool, Result<bool, ()>, u32, Option<real>, ()>(
        |_: bool| true,
        |left: bool, right: bool, distance: u32| discrete_close(left, right, distance),
        |truth: bool, outcome: Result<bool, ()>| rr_result_mass(truth, outcome, p, q),
        |left: spec_fn(Result<bool, ()>) -> real,
         right: spec_fn(Result<bool, ()>) -> real,
         bound: Option<real>| max_divergence_bound(left, right, bound),
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
        implies max_divergence_bound(
            |outcome: Result<bool, ()>| rr_result_mass(left, outcome, p, q),
            |outcome: Result<bool, ()>| rr_result_mass(right, outcome, p, q),
            rr_map(p, factor, distance),
        ) by {
        if distance == 0 {
            assert(left == right);
        } else if p == 1real {
            assert(rr_map(p, factor, distance) == None);
        } else {
            assert(1real / 2real <= p < 1real);
            assert(p <= factor * (1real - p));
            assert forall|outcome: Result<bool, ()>|
                rr_result_mass(left, outcome, p, q)
                    <= factor * rr_result_mass(right, outcome, p, q) by {
                lemma_result_domination(p, q, factor, left, right, outcome);
            }
        }
    }
}

} // verus!
