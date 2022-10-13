use num::Zero;

use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation, Measurement, Measure, PrivacyMap},
    domains::ProductDomain,
    error::{Fallible, ExplainUnwrap},
    traits::{TotalOrd, InfMul, ExactIntCast}, metrics::{ProductMetric, IntDistance}, measures::{MaxDivergence, ZeroConcentratedDivergence, FixedSmoothedMaxDivergence, SmoothedMaxDivergence, SMDCurve},
};

#[cfg(feature = "ffi")]
mod ffi;

/// Construct the parallel execution of [`transformation0`, `transformation1`, ...]. Returns a Transformation.
/// 
/// # Arguments
/// * `transformations` - A list of transformations to apply, one to each element.
pub fn make_parallel_transformation<DI, DO, MI, MO>(
    transformations: Vec<&Transformation<DI, DO, MI, MO>>,
) -> Fallible<Transformation<ProductDomain<DI>, ProductDomain<DO>, ProductMetric<MI>, ProductMetric<MO>>>
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Metric,
    MO::Distance: TotalOrd
{
    if transformations.is_empty() {
        return fallible!(MakeTransformation, "must pass at least one transformation");
    }
    let input_metric = transformations[0].input_metric.clone();
    if transformations.iter().any(|meas| meas.input_metric != input_metric) {
        return fallible!(MakeTransformation, "all transformations must have the same input metric")
    }
    let output_metric = transformations[0].output_metric.clone();
    if transformations.iter().any(|meas| meas.output_metric != output_metric) {
        return fallible!(MakeTransformation, "all transformations must have the same output metric")
    }

    let functions = transformations
        .iter()
        .map(|trans| trans.function.clone())
        .collect::<Vec<_>>();
    let maps = transformations
        .iter()
        .map(|trans| trans.stability_map.clone())
        .collect::<Vec<_>>();

    Ok(Transformation::new(
        ProductDomain::new(
            transformations
                .iter()
                .map(|trans| trans.input_domain.clone())
                .collect(),
        ),
        ProductDomain::new(
            transformations
                .iter()
                .map(|trans| trans.output_domain.clone())
                .collect(),
        ),
        Function::new_fallible(move |arg: &Vec<DI::Carrier>| {
            functions
                .iter()
                .zip(arg)
                .map(|(func, part)| func.eval(part))
                .collect()
        }),

        ProductMetric::new(input_metric),
        ProductMetric::new(output_metric),
        StabilityMap::new_fallible(move |(k, r): &(MI::Distance, IntDistance)| {
            let k = maps.iter()
                .map(|map| map.eval(k))
                .reduce(|l, r| l?.total_max(r?))
                .unwrap_assert("there is at least one transformation")?;
            Ok((k, *r))
        }),
    ))
}

pub trait ParallelCompositionMeasure: Measure {
    fn compose(&self, d_i: Vec<Self::Distance>, partition_limit: IntDistance) -> Fallible<Self::Distance>;
}

fn compose_scalars<Q>(mut d_mids: Vec<Q>, partition_limit: IntDistance) -> Fallible<Q>
    where Q: Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul {
    let seed = d_mids.pop().unwrap_or_else(Q::zero);

    let d_max = (d_mids.into_iter())
        .try_fold(seed, |max, d_i| max.total_max(d_i))?;

    Q::exact_int_cast(partition_limit)?.inf_mul(&d_max)
}

impl<Q> ParallelCompositionMeasure for MaxDivergence<Q>
        where Q: Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul {
    fn compose(&self, d_mids: Vec<Q>, partition_limit: IntDistance) -> Fallible<Self::Distance> {
        compose_scalars(d_mids, partition_limit)
    }
}

impl<Q> ParallelCompositionMeasure for FixedSmoothedMaxDivergence<Q>
        where Q: Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul {
    fn compose(&self, d_mids: Vec<Self::Distance>, partition_limit: IntDistance) -> Fallible<Self::Distance> {
        let (epsilons, deltas) = d_mids.into_iter().unzip();
        Ok((
            compose_scalars(epsilons, partition_limit)?, 
            compose_scalars(deltas, partition_limit)?
        ))
    }
}

impl<Q> ParallelCompositionMeasure for SmoothedMaxDivergence<Q>
        where Q: 'static + Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul {
    fn compose(&self, d_mids: Vec<Self::Distance>, partition_limit: IntDistance) -> Fallible<Self::Distance> {
        Ok(SMDCurve::new(move |delta| {
            let epsilons = d_mids.iter()
                .map(|curve_i| curve_i.epsilon(delta))
                .collect::<Fallible<Vec<_>>>()?;
            compose_scalars(epsilons, partition_limit)
        }))
    }
}

impl<Q> ParallelCompositionMeasure for ZeroConcentratedDivergence<Q>
        where Q: Zero + Clone + TotalOrd + ExactIntCast<IntDistance> + InfMul {
    fn compose(&self, d_mids: Vec<Q>, partition_limit: IntDistance) -> Fallible<Self::Distance> {
        compose_scalars(d_mids, partition_limit)
    }
}

/// Construct the parallel composition of [`measurement0`, `measurement1`, ...]. Returns a Measurement.
/// 
/// # Arguments
/// * `measurements` - A list of measurements to apply, one to each element.
pub fn make_parallel_composition<DI, DO, MI, MO>(
    measurements: Vec<&Measurement<DI, DO, MI, MO>>,
) -> Fallible<Measurement<ProductDomain<DI>, ProductDomain<DO>, ProductMetric<MI>, MO>>
where
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + ParallelCompositionMeasure,
{
    if measurements.is_empty() {
        return fallible!(MakeMeasurement, "must pass at least one measurement");
    }
    let input_metric = measurements[0].input_metric.clone();
    if measurements.iter().any(|meas| meas.input_metric != input_metric) {
        return fallible!(MakeMeasurement, "all measurements must have the same input metric")
    }
    let output_measure = measurements[0].output_measure.clone();
    if measurements.iter().any(|meas| meas.output_measure != output_measure) {
        return fallible!(MakeMeasurement, "all measurements must have the same output measure")
    }

    let functions = measurements
        .iter()
        .map(|meas| meas.function.clone())
        .collect::<Vec<_>>();
    let maps = measurements
        .iter()
        .map(|meas| meas.privacy_map.clone())
        .collect::<Vec<_>>();

    Ok(Measurement::new(
        ProductDomain::new(
            measurements
                .iter()
                .map(|meas| meas.input_domain.clone())
                .collect(),
        ),
        ProductDomain::new(
            measurements
                .iter()
                .map(|meas| meas.output_domain.clone())
                .collect(),
        ),
        Function::new_fallible(move |arg: &Vec<DI::Carrier>| {
            functions
                .iter()
                .zip(arg)
                .map(|(func, part)| func.eval(part))
                .collect()
        }),
        ProductMetric::new(input_metric),
        output_measure.clone(),
        PrivacyMap::new_fallible(move |(k, r): &(MI::Distance, IntDistance)| {
            let d_i = (maps.iter())
                .map(|map| map.eval(k))
                .collect::<Fallible<Vec<_>>>()?;
            
            output_measure.compose(d_i, *r)
        }),
    ))
}