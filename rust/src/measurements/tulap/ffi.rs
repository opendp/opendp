use std::ffi::c_double;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::AtomDomain;
use crate::err;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::metrics::AbsoluteDistance;

use super::make_tulap;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_tulap(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    epsilon: c_double,
    delta: c_double,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<AtomDomain<f64>>());
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<AbsoluteDistance<f64>>());
    make_tulap(input_domain.clone(), input_metric.clone(), epsilon, delta)
        .into_any()
        .into()
}
