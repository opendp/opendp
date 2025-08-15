#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

use std::{cell::RefCell, rc::Rc};

use crate::{
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::{Wrapper, wrap},
};

use super::{Adaptivity, Composability, CompositionMeasure};

/// Construct the DP composition of [`measurement0`, `measurement1`, ...].
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * noninteractive: all mechanisms specified up-front (but each can be interactive)
/// * compositor: all privacy parameters specified up-front (via the map)
///
/// # Arguments
/// * `measurements` - A vector of Measurements to compose. All input domains, input metrics and output measures must be equivalent.
///
/// # Generics
/// * `DI` - Input Domain
/// * `MI` - Input Metric
/// * `MO` - Output Measure
/// * `TO` - Output Type.
pub fn make_composition<DI, MI, MO, TO>(
    measurements: Vec<Measurement<DI, MI, MO, TO>>,
) -> Fallible<Measurement<DI, MI, MO, Vec<TO>>>
where
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
    TO: 'static,
    (DI, MI): MetricSpace,
{
    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement");
    }
    let input_domain = measurements[0].input_domain.clone();
    let input_metric = measurements[0].input_metric.clone();
    let output_measure = measurements[0].output_measure.clone();

    if !measurements.iter().all(|v| input_domain == v.input_domain) {
        return fallible!(DomainMismatch, "All input domains must be the same");
    }
    if !measurements.iter().all(|v| input_metric == v.input_metric) {
        return fallible!(MetricMismatch, "All input metrics must be the same");
    }
    if !measurements
        .iter()
        .all(|v| output_measure == v.output_measure)
    {
        return fallible!(MetricMismatch, "All output measures must be the same");
    }

    let functions = measurements
        .iter()
        .map(|m| m.function.clone())
        .collect::<Vec<_>>();

    let maps = measurements
        .iter()
        .map(|m| m.privacy_map.clone())
        .collect::<Vec<_>>();

    let require_sequentiality = matches!(
        output_measure.composability(Adaptivity::NonAdaptive)?,
        Composability::Sequential
    );

    Measurement::new(
        input_domain,
        input_metric,
        output_measure.clone(),
        Function::new_fallible(move |arg: &DI::Carrier| {
            let active_id = Rc::new(RefCell::new(0));

            if require_sequentiality {
                functions
                    .iter()
                    .inspect(|_| *active_id.borrow_mut() += 1)
                    .map(|f| {
                        // Wrap any spawned queryables with a check that no new queries have been asked.
                        let child_id = *active_id.borrow();

                        let wrapper = Wrapper::new_recursive_pre_hook(enclose!(active_id, move || {
                            (*active_id.borrow() == child_id)
                                .then_some(())
                                .ok_or_else(|| err!(
                                    FailedFunction,
                                    "Compositor has received a new query. To satisfy the sequentiality constraint of composition, only the most recent release from the parent compositor may be interacted with."
                                ))
                        }));

                        wrap(wrapper, || f.eval(arg))
                    })
                    .collect()
            } else {
                functions.iter().map(|f| f.eval(arg)).collect()
            }
        }),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            output_measure.compose(
                maps.iter()
                    .map(|map| map.eval(d_in))
                    .collect::<Fallible<_>>()?,
            )
        }),
    )
}

/// Construct the DP composition \[`measurement0`, `measurement1`, ...\].
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
///
/// All metrics and domains must be equivalent.
///
/// **Composition Properties**
///
/// * sequential: all measurements are applied to the same dataset
/// * basic: the composition is the linear sum of the privacy usage of each query
/// * noninteractive: all mechanisms specified up-front (but each can be interactive)
/// * compositor: all privacy parameters specified up-front (via the map)
///
/// # Arguments
/// * `measurements` - A vector of Measurements to compose.
#[deprecated(
    since = "0.14.0",
    note = "This function has been renamed, use `make_composition` instead."
)]
pub fn make_basic_composition<DI, MI, MO, TO>(
    measurements: Vec<Measurement<DI, MI, MO, TO>>,
) -> Fallible<Measurement<DI, MI, MO, Vec<TO>>>
where
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + CompositionMeasure,
    TO: 'static,
    (DI, MI): MetricSpace,
{
    make_composition(measurements)
}
