use crate::{
    core::{Domain, Function, Metric, StabilityMap, Transformation, Measurement, Measure, PrivacyMap},
    domains::ProductDomain,
    error::{Fallible, ExplainUnwrap},
    traits::TotalOrd,
};

#[cfg(feature = "ffi")]
mod ffi;

/// Construct the parallel execution of [`transformation0`, `transformation1`, ...]. Returns a Transformation.
/// 
/// # Arguments
/// * `transformations` - A list of transformations to apply, one to each element.
pub fn make_partition_map_trans<DI, DO, MI, MO>(
    transformations: Vec<&Transformation<DI, DO, MI, MO>>,
) -> Fallible<Transformation<ProductDomain<DI>, ProductDomain<DO>, MI, MO>>
where
    MO::Distance: TotalOrd,
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Metric,
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

        input_metric,
        output_metric,
        StabilityMap::new_fallible(move |d_in: &MI::Distance| {
            maps.iter()
                .map(|map| map.eval(d_in))
                .reduce(|l, r| l?.total_max(r?))
                .unwrap_assert("there is at least one transformation")
        }),
    ))
}

/// Construct the parallel composition of [`measurement0`, `measurement1`, ...]. Returns a Measurement.
/// 
/// # Arguments
/// * `measuerements` - A list of measuerements to apply, one to each element.
pub fn make_partition_map_meas<DI, DO, MI, MO>(
    measurements: Vec<&Measurement<DI, DO, MI, MO>>,
) -> Fallible<Measurement<ProductDomain<DI>, ProductDomain<DO>, MI, MO>>
where
    MO::Distance: TotalOrd,
    DI: 'static + Domain,
    DO: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + Measure,
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
        input_metric,
        output_measure,
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            maps.iter()
                .map(|map| map.eval(d_in))
                .reduce(|l, r| l?.total_max(r?))
                .unwrap_assert("there is at least one measurement")
        }),
    ))
}