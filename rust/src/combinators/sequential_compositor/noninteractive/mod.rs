#[cfg(feature = "ffi")]
mod ffi;

use num::Zero;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    interactive::wrap,
    measures::{FixedSmoothedMaxDivergence, MaxDivergence, ZeroConcentratedDivergence},
    traits::InfAdd,
};

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
pub fn make_basic_composition<DI, TO, MI, MO>(
    measurements: Vec<Measurement<DI, TO, MI, MO>>,
) -> Fallible<Measurement<DI, Vec<TO>, MI, MO>>
where
    DI: 'static + Domain,
    TO: 'static,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
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

    let concurrent = output_measure.concurrent()?;
    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &DI::Carrier| {
            let go = || functions.iter().map(|f| f.eval(arg)).collect();

            if concurrent {
                go()
            } else {
                wrap(
                    |_qbl| {
                        fallible!(
                            FailedFunction,
                            "output_measure must allow concurrency to spawn queryables from a noninteractive compositor"
                        )
                    },
                    go,
                )
            }
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

pub trait BasicCompositionMeasure: Measure {
    fn concurrent(&self) -> Fallible<bool>;
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for MaxDivergence<Q> {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for FixedSmoothedMaxDivergence<Q> {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter()
            .try_fold((Q::zero(), Q::zero()), |(e1, d1), (e2, d2)| {
                Ok((e1.inf_add(e2)?, d1.inf_add(d2)?))
            })
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for ZeroConcentratedDivergence<Q> {
    fn concurrent(&self) -> Fallible<bool> {
        Ok(true)
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

// UNIT TESTS
#[cfg(test)]
mod test;
