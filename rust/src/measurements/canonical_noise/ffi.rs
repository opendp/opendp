use std::ffi::c_double;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::AtomDomain;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast};
use crate::measures::PrivacyCurve;
use crate::metrics::AbsoluteDistance;

use super::make_canonical_noise;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_canonical_noise(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    d_in: c_double,
    d_out: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<AtomDomain<f64>>()).clone();
    let input_metric =
        try_!(try_as_ref!(input_metric).downcast_ref::<AbsoluteDistance<f64>>()).clone();
    let d_out = try_!(try_as_ref!(d_out).downcast_ref::<PrivacyCurve>()).clone();

    make_canonical_noise(input_domain, input_metric, d_in, d_out)
        .into_any()
        .into()
}
