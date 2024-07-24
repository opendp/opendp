#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

use crate::{
    core::{Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::Wrapper,
};

use super::SequentialCompositionMeasure;

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
/// * `measurements` - A vector of Measurements to compose. All DI, MI, MO must be equivalent.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `TO` - Output Type.
/// * `MI` - Input Metric
/// * `MO` - Output Measure
pub fn make_composition<DI, TO, MI, MO>(
    measurements: Vec<Measurement<DI, TO, MI, MO>>,
) -> Fallible<Measurement<DI, Vec<TO>, MI, MO>>
where
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + SequentialCompositionMeasure,
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

    let wrapper = (!output_measure.concurrent()?).then(|| Wrapper::new(|_qbl| {
        fallible!(
            FailedFunction,
            "output_measure must allow concurrency to spawn queryables from a noninteractive compositor"
        )
    }));

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &DI::Carrier| {
            (functions.iter())
                .map(|f| f.eval_wrap(arg, wrapper.clone()))
                .collect()
        }),
        input_metric,
        output_measure.clone(),
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
pub fn make_basic_composition<DI, TO, MI, MO>(
    measurements: Vec<Measurement<DI, TO, MI, MO>>,
) -> Fallible<Measurement<DI, Vec<TO>, MI, MO>>
where
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + SequentialCompositionMeasure,
    (DI, MI): MetricSpace,
{
    make_composition(measurements)
}
