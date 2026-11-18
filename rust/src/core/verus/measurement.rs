//! Generic semantic transfer theorem for valid measurements.
//!
//! This theorem is not a constructor contract or a model of erased Rust
//! callbacks. It can transfer an already-established privacy guarantee across
//! independently established equalities of function and map semantics. The
//! equivalence premises must never be used to assume a mechanism is private.
use vstd::prelude::*;

verus! {

pub open spec fn bounded_payoff<Z>(payoff: spec_fn(Z) -> real) -> bool {
    forall|outcome: Z| 0real <= #[trigger] payoff(outcome) <= 1real
}

/// Generic validity of the actual callable components of a measurement.
///
/// This is parameterized by the *semantic implementations* of Domain,
/// Metric, and Measure, not caller-selected predicates. On all supported
/// distances the map has an attainable, successful result. Every possible
/// result is valid and bounds the observable distribution of the function.
pub open spec fn valid_certified_parts<DI, MI, MO, TO>(
    domain: &DI,
    metric: &MI,
    measure: &MO,
    function: &crate::Function<DI::Carrier, TO>,
    privacy_map: &crate::PrivacyMap<MI, MO>,
) -> bool
    where
        DI: crate::CertifiedDomain,
        MI: crate::CertifiedMetric<DI::Carrier>,
        MO: crate::CertifiedMeasure<Result<TO, ()>>,
{
    &&& forall|input: &DI::Carrier|
        domain.contains(input) ==>
            #[trigger] crate::measurement_parts::function_call_requires(function, input)
    &&& forall|d: &MI::Distance|
        metric.valid_distance(d) ==>
            #[trigger] crate::measurement_parts::privacy_map_call_requires(privacy_map, d)
    &&& forall|d: &MI::Distance, output: Result<MO::Distance, ()>|
        metric.valid_distance(d)
            && #[trigger] crate::measurement_parts::privacy_map_call_ensures(
                privacy_map, d, output)
        ==> match output {
            Err(_) => false,
            Ok(cost) => {
                &&& measure.valid_cost(&cost)
                &&& forall|left: &DI::Carrier, right: &DI::Carrier|
                    domain.contains(left) && domain.contains(right)
                        && metric.close(left, right, d)
                    ==> measure.dominates(
                        |payoff: spec_fn(Result<TO, ()>) -> real|
                            crate::measurement_parts::function_expectation(function, left, payoff),
                        |payoff: spec_fn(Result<TO, ()>) -> real|
                            crate::measurement_parts::function_expectation(function, right, payoff),
                        &cost)
            },
        }
}

/// Entire validity obligation on *concrete* executable components,
/// before either callback is erased. This first certified instantiation
/// uses the shared one-word sampler; domain, metric, measure, and output
/// are otherwise generic. Its expectation is the proved one-word law,
/// not a field or postcondition of an already-erased Function.
pub open spec fn valid_one_word_components<DI, MI, MO, TO, Op, F, P>(
    domain: &DI,
    metric: &MI,
    measure: &MO,
    operator: Op,
    postprocess: F,
    map: P,
    model: crate::uniform_word::OneWordModel<DI::Carrier, TO>,
) -> bool
    where
        DI: crate::CertifiedDomain,
        DI::Carrier: Copy,
        MI: crate::CertifiedMetric<DI::Carrier>,
        MO: crate::CertifiedMeasure<Result<TO, ()>>,
        Op: Copy,
        F: Fn(&Op, &DI::Carrier, Result<u64, ()>) -> Result<TO, ()>,
        P: Fn(&Op, &MI::Distance) -> Result<MO::Distance, ()>,
{
    &&& crate::measurement_parts::exact_word_postprocessing(
        operator, postprocess, model)
    &&& forall|d: &MI::Distance| metric.valid_distance(d) ==>
        #[trigger] call_requires(map, (&operator, d))
    &&& forall|d: &MI::Distance, output: Result<MO::Distance, ()>|
        metric.valid_distance(d)
            && #[trigger] call_ensures(map, (&operator, d), output)
        ==> match output {
            Err(_) => false,
            Ok(cost) => {
                &&& measure.valid_cost(&cost)
                &&& forall|left: &DI::Carrier, right: &DI::Carrier|
                    domain.contains(left) && domain.contains(right)
                        && metric.close(left, right, d)
                    ==> measure.dominates(
                        |payoff: spec_fn(Result<TO, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*left, model, payoff),
                        |payoff: spec_fn(Result<TO, ()>) -> real|
                            crate::uniform_word::one_word_expectation(*right, model, payoff),
                        &cost)
            },
        }
}

/// The same generic contract about the installed OpenDP measurement.
pub open spec fn valid_certified_measurement<DI, MI, MO, TO>(
    meas: &crate::Measurement<DI, MI, MO, TO>,
) -> bool
    where
        DI: crate::CertifiedDomain,
        MI: crate::CertifiedMetric<DI::Carrier>,
        MO: crate::CertifiedMeasure<Result<TO, ()>>,
{
    valid_certified_parts(
        &meas.input_domain, &meas.input_metric, &meas.output_measure,
        &meas.function, &meas.privacy_map)
}

} // verus!
