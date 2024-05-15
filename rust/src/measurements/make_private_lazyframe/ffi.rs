use polars::lazy::frame::LazyFrame;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::LazyFrameDomain,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util,
    },
    measures::MaxDivergence,
    metrics::SymmetricDistance,
};

use super::make_private_lazyframe;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_lazyframe(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    lazyframe: *const AnyObject,
    global_scale: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<SymmetricDistance>()).clone();
    let output_measure =
        try_!(try_as_ref!(output_measure).downcast_ref::<MaxDivergence<f64>>()).clone();

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();

    let global_scale = if let Some(param) = util::as_ref(global_scale) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };
    make_private_lazyframe(
        input_domain,
        input_metric,
        output_measure,
        lazyframe,
        global_scale,
    )
    .into_any()
    .into()
}
