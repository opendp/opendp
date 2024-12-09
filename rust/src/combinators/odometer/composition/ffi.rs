use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMetric, AnyObject, AnyOdometer},
};

/// Construct an odometer that can spawn an odometer queryable.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
fn make_fully_adaptive_composition(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
) -> Fallible<AnyOdometer> {
    let compositor = super::make_fully_adaptive_composition::<_, AnyObject, _, _>(
        input_domain,
        input_metric,
        output_measure,
    )?;

    // 1.   Odometer<AnyDomain, AnyMetric, AnyMeasure, AnyMeasurement, AnyObject>
    //    -> into_any_Q() ->
    // 2.   Odometer<AnyDomain, AnyMetric, AnyMeasure, AnyObject, AnyObject>
    //    = AnyOdometer
    Ok(compositor.into_any_Q())
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_fully_adaptive_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
) -> FfiResult<*mut AnyOdometer> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();

    make_fully_adaptive_composition(input_domain, input_metric, output_measure).into()
}