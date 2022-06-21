#[cfg(feature = "ffi")]
pub mod ffi;

use num::Zero;

use crate::{
    core::{
        Domain, FixedSmoothedMaxDivergence, Function, MaxDivergence, Measure, Measurement, Metric,
        PrivacyMap, VectorDomain, ZeroConcentratedDivergence,
    },
    error::Fallible,
    traits::InfAdd,
};

pub trait SequentialCompositionStaticDistancesMeasure: Measure {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> SequentialCompositionStaticDistancesMeasure for MaxDivergence<Q> {
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> SequentialCompositionStaticDistancesMeasure
    for FixedSmoothedMaxDivergence<Q>
{
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter()
            .try_fold((Q::zero(), Q::zero()), |(e1, d1), (e2, d2)| {
                Ok((e1.inf_add(e2)?, d1.inf_add(d2)?))
            })
    }
}

impl<Q: InfAdd + Zero + Clone> SequentialCompositionStaticDistancesMeasure
    for ZeroConcentratedDivergence<Q>
{
    fn compose(&self, d_i: &Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

pub fn make_sequential_composition_static_distances<DI, DO, MI, MO>(
    measurements: Vec<&Measurement<DI, DO, MI, MO>>,
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, MO>>
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + SequentialCompositionStaticDistancesMeasure,
{
    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "Must have at least one measurement");
    }
    let input_domain = measurements[0].input_domain.clone();
    let output_domain = measurements[0].output_domain.clone();
    let input_metric = measurements[0].input_metric.clone();
    let output_measure = measurements[0].output_measure.clone();

    if !measurements.iter().all(|v| input_domain == v.input_domain) {
        return fallible!(DomainMismatch, "All input domains must be the same");
    }
    // if !measurements
    //     .iter()
    //     .all(|v| output_domain == v.output_domain)
    // {
    //     return fallible!(DomainMismatch, "All output domains must be the same");
    // }
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

    Ok(Measurement::new(
        input_domain,
        VectorDomain::new(output_domain),
        Function::new_fallible(move |arg: &DI::Carrier| {
            functions.iter().map(|f| f.eval(arg)).collect()
        }),
        input_metric,
        output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            output_measure.compose(
                &maps
                    .iter()
                    .map(|map| map.eval(d_in))
                    .collect::<Fallible<_>>()?,
            )
        }),
    ))
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::AllDomain;
    use crate::core::*;
    use crate::core::{L1Distance, MaxDivergence};
    use crate::error::ExplainUnwrap;
    use crate::meas::make_base_laplace;

    use super::*;

    #[test]
    fn test_make_sequential_composition_static_distances() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f64>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f64);
        let input_metric0 = L1Distance::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_relation0 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement0 = Measurement::new(
            input_domain0,
            output_domain0,
            function0,
            input_metric0,
            output_measure0,
            privacy_relation0,
        );
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_relation1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement1 = Measurement::new(
            input_domain1,
            output_domain1,
            function1,
            input_metric1,
            output_measure1,
            privacy_relation1,
        );
        let composition =
            make_sequential_composition_static_distances(vec![&measurement0, &measurement1])
                .unwrap_test();
        let arg = 99;
        let ret = composition.invoke(&arg).unwrap_test();
        assert_eq!(ret, vec![100_f64, 98_f64]);
    }

    #[test]
    fn test_make_sequential_composition_static_distances_2() -> Fallible<()> {
        let laplace = make_base_laplace::<AllDomain<_>>(1.0f64)?;
        let measurements = vec![&laplace; 2];
        let composition = make_sequential_composition_static_distances(measurements)?;
        let arg = 99.;
        let ret = composition.function.eval(&arg)?;

        assert_eq!(ret.len(), 2);

        assert!(composition.check(&1., &2.)?);
        assert!(composition.check(&1., &2.0001)?);
        assert!(!composition.check(&1., &1.999)?);
        Ok(())
    }
}
