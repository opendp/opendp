use crate::{
    core::FfiResult,
    ffi::any::{AnyDomain, AnyMeasure, AnyMetric, AnyObject, AnyOdometer},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_fully_adaptive_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
) -> FfiResult<*mut AnyOdometer> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();
    let d_in = try_as_ref!(d_in).clone();
    super::make_fully_adaptive_composition(input_domain, input_metric, output_measure, d_in).into()
}
