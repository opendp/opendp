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
/// * `MO` - Output Metric
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

    let concurrent = output_measure.concurrent();
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
    fn concurrent(&self) -> bool;
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for MaxDivergence<Q> {
    fn concurrent(&self) -> bool {
        true
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for FixedSmoothedMaxDivergence<Q> {
    fn concurrent(&self) -> bool {
        true
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter()
            .try_fold((Q::zero(), Q::zero()), |(e1, d1), (e2, d2)| {
                Ok((e1.inf_add(e2)?, d1.inf_add(d2)?))
            })
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for ZeroConcentratedDivergence<Q> {
    fn concurrent(&self) -> bool {
        true
    }
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::domains::AtomDomain;
    use crate::measurements::make_base_laplace;
    use crate::measures::MaxDivergence;
    use crate::metrics::AbsoluteDistance;

    use super::*;

    #[test]
    fn test_make_basic_composition() -> Fallible<()> {
        let input_domain0 = AtomDomain::<i32>::default();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f64);
        let input_metric0 = AbsoluteDistance::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_map0 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement0 = Measurement::new(
            input_domain0,
            function0,
            input_metric0,
            output_measure0,
            privacy_map0,
        )?;
        let input_domain1 = AtomDomain::<i32>::default();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = AbsoluteDistance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_map1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement1 = Measurement::new(
            input_domain1,
            function1,
            input_metric1,
            output_measure1,
            privacy_map1,
        )?;
        let composition = make_basic_composition(vec![measurement0, measurement1])?;
        let arg = 99;
        let ret = composition.invoke(&arg)?;
        assert_eq!(ret, vec![100_f64, 98_f64]);

        Ok(())
    }

    #[test]
    fn test_make_basic_composition_2() -> Fallible<()> {
        let input_domain = AtomDomain::default();
        let input_metric = AbsoluteDistance::default();
        let laplace = make_base_laplace(input_domain, input_metric, 1.0f64, None)?;
        let measurements = vec![laplace; 2];
        let composition = make_basic_composition(measurements)?;
        let arg = 99.;
        let ret = composition.invoke(&arg)?;

        assert_eq!(ret.len(), 2);
        println!("return: {:?}", ret);

        assert!(composition.check(&1., &2.)?);
        assert!(!composition.check(&1., &1.9999)?);
        Ok(())
    }
}
