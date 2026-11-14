//! Mechanized model validity for Boolean randomized response.
//! The exact Bernoulli sampler and the directed-rounded f64 logarithm
//! require native-to-Verus correspondence; neither is silently assumed
//! as an axiom inside this proof.
use vstd::prelude::*;
use crate::measurement::valid_measurement;
use crate::alerus_bridge::{rr_mass, lemma_rr_event_expectation};

verus! {

/// Checked executable XOR kernel, identical to the native callback.
pub fn rr_kernel(truth: bool, honest: bool) -> (output: bool)
    ensures output == (truth ^ !honest),
{
    truth ^ !honest
}

/// An entropy failure must be independent of the private Boolean.
pub open spec fn rr_fallible_kernel(
    truth: bool, sample: Result<bool, ()>,
) -> Result<bool, ()> {
    match sample {
        Ok(honest) => Ok(truth ^ !honest),
        Err(_) => Err(()),
    }
}

pub open spec fn rr_failed(sample: Result<bool, ()>) -> bool {
    match sample { Err(_) => true, Ok(_) => false }
}

pub proof fn lemma_error_independent(
    left: bool, right: bool, sample: Result<bool, ()>,
)
    ensures
        match rr_fallible_kernel(left, sample) {
            Err(_) => true,
            Ok(_) => false,
        } == rr_failed(sample),
        match rr_fallible_kernel(right, sample) {
            Err(_) => true,
            Ok(_) => false,
        } == rr_failed(sample),
{
}

pub open spec fn discrete_close(left: bool, right: bool, d: u32) -> bool {
    left == right || d > 0
}

/// The None cost represents infinite privacy loss. A finite cost represents
/// exp(epsilon), so no floating-point log or real exponential is axiomatized.
pub open spec fn rr_map(p: real, factor: real, d: u32) -> Option<real> {
    if d == 0 { Some(1real) }
    else if p == 1real { None }
    else { Some(factor) }
}

pub open spec fn rr_output_close(
    p: real, left: bool, right: bool, cost: Option<real>,
) -> bool {
    match cost {
        None => true,
        Some(factor) =>
            rr_mass(left, true, p) <= factor * rr_mass(right, true, p)
            && rr_mass(left, false, p) <= factor * rr_mass(right, false, p),
    }
}

pub open spec fn rr_valid_measurement(p: real, factor: real) -> bool {
    valid_measurement::<bool, u32, Option<real>, ()>(
        |_: bool| true,
        |left: bool, right: bool, d: u32| discrete_close(left, right, d),
        |left: bool, right: bool, cost: Option<real>|
            rr_output_close(p, left, right, cost),
        |d: u32| Ok(rr_map(p, factor, d)),
    )
}

/// For a finite privacy budget, dominate every singleton output event.
pub proof fn lemma_rr_event_domination(
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
            assert(p >= 0real);
            assert(p <= factor * p) by(nonlinear_arith)
                requires p >= 0real, factor >= 1real;
        } else {
            assert(1real - p >= 0real);
            assert(1real - p <= factor * (1real - p)) by(nonlinear_arith)
                requires 1real - p >= 0real, factor >= 1real;
        }
    } else if output == left {
        assert(rr_mass(left, output, p) == p);
        assert(rr_mass(right, output, p) == 1real - p);
    } else {
        assert(1real - p <= p);
        assert(p >= 0real);
        assert(p <= factor * p) by(nonlinear_arith)
            requires p >= 0real, factor >= 1real;
    }
}

/// Establishes a valid Measurement *model*, including zero distance,
/// finite privacy bounds, p=1 (infinite bound), and all Boolean events.
///
/// Native adapter obligations: the Alerus Bernoulli law is realized by the
/// production sampler, and exp(rounded_epsilon) >= factor >= p/(1-p).
pub proof fn lemma_make_randomized_response_bool_returns_valid(
    p: real, factor: real,
)
    requires
        1real / 2real <= p <= 1real,
        factor >= 1real,
        p < 1real ==> p <= factor * (1real - p),
    ensures rr_valid_measurement(p, factor),
{
    assert forall|left: bool, right: bool, d: u32|
        #[trigger] discrete_close(left, right, d)
        implies rr_output_close(p, left, right, rr_map(p, factor, d)) by {
        if d == 0 {
            assert(left == right);
            assert(rr_map(p, factor, d) == Some(1real));
            assert(rr_mass(left, true, p) == rr_mass(right, true, p));
            assert(rr_mass(left, false, p) == rr_mass(right, false, p));
        } else if p == 1real {
            assert(rr_map(p, factor, d) == None);
        } else {
            assert(1real / 2real <= p < 1real);
            assert(p <= factor * (1real - p));
            lemma_rr_event_domination(p, factor, left, right, true);
            lemma_rr_event_domination(p, factor, left, right, false);
        }
    }
}

/// Connect the Bernoulli post-expectation to the RR event law.
pub proof fn lemma_alerus_rr_mass(truth: bool, output: bool, p: real)
    ensures
        crate::alerus_bridge::bernoulli_expectation(
            p, |honest: bool|
                crate::alerus_bridge::rr_event_indicator(truth, output, honest),
        ) == rr_mass(truth, output, p),
{
    lemma_rr_event_expectation(truth, output, p);
}

} // verus!
