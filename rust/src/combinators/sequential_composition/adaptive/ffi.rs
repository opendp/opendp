use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_adaptive_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
    d_mids: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();
    let d_in = try_as_ref!(d_in).clone();
    let d_mids = try_as_ref!(d_mids);

    fn repack_vec<T: 'static + Clone>(obj: &AnyObject) -> Fallible<Vec<AnyObject>> {
        Ok(obj
            .downcast_ref::<Vec<T>>()?
            .iter()
            .map(Clone::clone)
            .map(AnyObject::new)
            .collect())
    }

    let QO = output_measure.distance_type.clone();
    let d_mids = try_!(dispatch!(
        repack_vec,
        [(QO, [f32, f64, (f32, f32), (f64, f64)])],
        (d_mids)
    ));

    super::make_adaptive_composition::<AnyDomain, AnyMetric, AnyMeasure, AnyObject>(
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_mids,
    )
    .map(|m| m.into_any_Q().into_any_out())
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_sequential_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
    d_mids: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    opendp_combinators__make_adaptive_composition(
        input_domain,
        input_metric,
        output_measure,
        d_in,
        d_mids,
    )
}
