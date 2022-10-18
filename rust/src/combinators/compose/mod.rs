#[cfg(feature = "ffi")]
mod ffi;

use num::Zero;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, PrivacyMap},
    domains::VectorDomain,
    error::Fallible,
    measures::{
        FixedSmoothedMaxDivergence, MaxDivergence,
        ZeroConcentratedDivergence,
    },
    traits::InfAdd,
};

/// Construct the DP composition [`measurement0`, `measurement1`, ...].
/// Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
///
/// Aside from sharing the same type, each output domain need not be equivalent.
/// This is useful when converting types to PolyDomain,
/// which can enable composition over non-homogeneous measurements.
///
/// # Arguments
/// * `measurements` - A vector of Measurements to compose. All DI, MI, MO must be equivalent.
///
/// # Generics
/// * `DI` - Input Domain.
/// * `DO` - Output Domain.
/// * `MI` - Input Metric
/// * `MO` - Output Metric
pub fn make_basic_composition<DI, DO, MI, MO>(
    measurements: Vec<&Measurement<DI, DO, MI, MO>>,
) -> Fallible<Measurement<DI, VectorDomain<DO>, MI, MO>>
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + BasicCompositionMeasure,
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
                maps.iter()
                    .map(|map| map.eval(d_in))
                    .collect::<Fallible<_>>()?,
            )
        }),
    ))
}

pub trait BasicCompositionMeasure: Measure {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance>;
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for MaxDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for FixedSmoothedMaxDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter()
            .try_fold((Q::zero(), Q::zero()), |(e1, d1), (e2, d2)| {
                Ok((e1.inf_add(e2)?, d1.inf_add(d2)?))
            })
    }
}

impl<Q: InfAdd + Zero + Clone> BasicCompositionMeasure for ZeroConcentratedDivergence<Q> {
    fn compose(&self, d_i: Vec<Self::Distance>) -> Fallible<Self::Distance> {
        d_i.iter().try_fold(Q::zero(), |sum, d_i| sum.inf_add(d_i))
    }
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use crate::core::*;
    use crate::domains::AllDomain;
    use crate::error::ExplainUnwrap;
    use crate::measurements::make_base_laplace;
    use crate::measures::MaxDivergence;
    use crate::metrics::L1Distance;

    use super::*;

    #[test]
    fn test_make_basic_composition() {
        let input_domain0 = AllDomain::<i32>::new();
        let output_domain0 = AllDomain::<f64>::new();
        let function0 = Function::new(|arg: &i32| (arg + 1) as f64);
        let input_metric0 = L1Distance::<i32>::default();
        let output_measure0 = MaxDivergence::default();
        let privacy_map0 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement0 = Measurement::new(
            input_domain0,
            output_domain0,
            function0,
            input_metric0,
            output_measure0,
            privacy_map0,
        );
        let input_domain1 = AllDomain::<i32>::new();
        let output_domain1 = AllDomain::<f64>::new();
        let function1 = Function::new(|arg: &i32| (arg - 1) as f64);
        let input_metric1 = L1Distance::<i32>::default();
        let output_measure1 = MaxDivergence::default();
        let privacy_map1 = PrivacyMap::new(|_d_in: &i32| f64::INFINITY);
        let measurement1 = Measurement::new(
            input_domain1,
            output_domain1,
            function1,
            input_metric1,
            output_measure1,
            privacy_map1,
        );
        let composition = make_basic_composition(vec![&measurement0, &measurement1]).unwrap_test();
        let arg = 99;
        let ret = composition.invoke(&arg).unwrap_test();
        assert_eq!(ret, vec![100_f64, 98_f64]);
    }

    #[test]
    fn test_make_basic_composition_2() -> Fallible<()> {
        let laplace = make_base_laplace::<AllDomain<_>>(1.0f64, None)?;
        let measurements = vec![&laplace; 2];
        let composition = make_basic_composition(measurements)?;
        let arg = 99.;
        let ret = composition.function.eval(&arg)?;

        assert_eq!(ret.len(), 2);
        println!("return: {:?}", ret);

        assert!(composition.check(&1., &2.)?);
        assert!(!composition.check(&1., &1.9999)?);
        Ok(())
    }
}
