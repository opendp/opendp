use polars::lazy::frame::LazyFrame;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::LazyFrameDomain,
    ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
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
    param: f64,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<SymmetricDistance>()).clone();
    let output_measure =
        try_!(try_as_ref!(output_measure).downcast_ref::<MaxDivergence<f64>>()).clone();

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();
    make_private_lazyframe(input_domain, input_metric, output_measure, lazyframe, param)
        .into_any()
        .into()
}
